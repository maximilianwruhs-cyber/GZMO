#!/usr/bin/env python3
"""
Google Scholar Labs Query Engine

Performs queries against Google Scholar Labs using Playwright with
persisted authentication state.

Usage:
    python query.py --question "How do microplastics affect gut microbiota?"
    python query.py --question "..." --output results.json
    python query.py --question "..." --hl de  # German UI
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Optional

try:
    from playwright.sync_api import sync_playwright, TimeoutError as PlaywrightTimeout
except ImportError:
    print("Error: playwright not installed. Run: pip install playwright")
    sys.exit(1)

from parse import parse_scholar_labs_html


def get_project_root() -> Path:
    """Find the GZMO project root."""
    script_dir = Path(__file__).parent.resolve()
    for parent in script_dir.parents:
        if (parent / "gzmo.toml").exists():
            return parent
    return script_dir.parent


def get_default_auth_path() -> Path:
    """Get the default auth state file path."""
    return get_project_root() / "playwright" / ".auth" / "google_state.json"


def get_default_cache_dir() -> Path:
    """Get the default cache directory."""
    return get_project_root() / "data" / "scholar-cache"


def query_scholar_labs(
    question: str,
    auth_path: Path,
    cache_dir: Path,
    hl: str = "en",
    timeout: int = 30000,
    save_raw: bool = True,
) -> dict:
    """
    Query Google Scholar Labs and return parsed results.

    Args:
        question: The research question to query
        auth_path: Path to the Playwright auth state JSON
        cache_dir: Directory to cache raw HTML responses
        hl: Interface language code ('en' or 'de')
        timeout: Timeout in milliseconds for waiting for results
        save_raw: Whether to save raw HTML to cache

    Returns:
        Dictionary with query metadata and parsed results
    """
    if not auth_path.exists():
        raise FileNotFoundError(
            f"Auth state not found at {auth_path}. "
            f"Run: python auth_setup.py"
        )

    cache_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.utcnow().isoformat() + "Z"

    # URL encode the question
    from urllib.parse import quote_plus
    encoded_question = quote_plus(question)

    url = f"https://scholar.google.com/scholar_labs/search?hl={hl}&q={encoded_question}"

    print(f"Querying: {question[:80]}...")
    print(f"URL: {url[:100]}...")

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(storage_state=str(auth_path))
        page = context.new_page()

        try:
            # Navigate to Scholar Labs
            page.goto(url, wait_until="domcontentloaded", timeout=60000)

            # Wait for AI results container with retry logic
            selectors = [
                ".scholar-labs-result-container",
                "[data-testid='scholar-labs-result']",
                ".gs_ai_results",
                "//div[contains(@class, 'result')]",  # XPath fallback
            ]

            result_selector = None
            for selector in selectors:
                try:
                    page.wait_for_selector(selector, timeout=timeout // len(selectors))
                    result_selector = selector
                    print(f"✓ Results loaded (selector: {selector})")
                    break
                except PlaywrightTimeout:
                    continue

            if not result_selector:
                # Check for login redirect
                current_url = page.url
                if "accounts.google.com" in current_url:
                    raise RuntimeError(
                        "Session expired or invalid. Google redirected to login page. "
                        f"Run: python auth_setup.py"
                    )
                raise RuntimeError(
                    f"Timeout waiting for results. Current URL: {current_url}"
                )

            # Give a moment for dynamic content to settle
            time.sleep(2)

            # Get page content
            html_content = page.content()

            # Save raw HTML if requested
            if save_raw:
                raw_file = cache_dir / "raw" / f"query_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.html"
                raw_file.parent.mkdir(parents=True, exist_ok=True)
                with open(raw_file, 'w', encoding='utf-8') as f:
                    f.write(html_content)
                print(f"✓ Raw HTML saved: {raw_file}")

            browser.close()

            # Parse the HTML
            parsed_results = parse_scholar_labs_html(html_content, question)

            result = {
                "query": question,
                "timestamp": timestamp,
                "url": url,
                "hl": hl,
                "result_count": len(parsed_results),
                "results": parsed_results,
            }

            # Save query to cache log
            cache_file = cache_dir / "queries.jsonl"
            with open(cache_file, 'a', encoding='utf-8') as f:
                f.write(json.dumps({
                    "timestamp": timestamp,
                    "query": question,
                    "result_count": len(parsed_results),
                }) + "\n")

            return result

        except Exception as e:
            browser.close()
            raise


def main():
    parser = argparse.ArgumentParser(
        description="Query Google Scholar Labs"
    )
    parser.add_argument(
        "--question", "-q",
        required=True,
        help="Research question to query"
    )
    parser.add_argument(
        "--auth-path",
        type=Path,
        default=None,
        help="Path to auth state JSON (default: project_root/playwright/.auth/google_state.json)"
    )
    parser.add_argument(
        "--cache-dir",
        type=Path,
        default=None,
        help="Cache directory (default: project_root/data/scholar-cache)"
    )
    parser.add_argument(
        "--hl",
        choices=["en", "de"],
        default="en",
        help="Interface language (default: en for stable parsing)"
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=30000,
        help="Timeout in milliseconds (default: 30000)"
    )
    parser.add_argument(
        "--output", "-o",
        type=Path,
        help="Output JSON file (default: stdout)"
    )
    parser.add_argument(
        "--no-cache",
        action="store_true",
        help="Don't save raw HTML to cache"
    )
    parser.add_argument(
        "--rate-sleep",
        type=float,
        default=3.0,
        help="Seconds to sleep after query for rate limiting (default: 3.0)"
    )

    args = parser.parse_args()

    auth_path = args.auth_path or get_default_auth_path()
    cache_dir = args.cache_dir or get_default_cache_dir()

    try:
        result = query_scholar_labs(
            question=args.question,
            auth_path=auth_path,
            cache_dir=cache_dir,
            hl=args.hl,
            timeout=args.timeout,
            save_raw=not args.no_cache,
        )

        # Rate limiting
        if args.rate_sleep > 0:
            time.sleep(args.rate_sleep)

        # Output results
        json_output = json.dumps(result, indent=2, ensure_ascii=False)

        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(json_output)
            print(f"\n✓ Results saved to: {args.output}")
        else:
            print("\n" + "=" * 60)
            print(json_output)

        print(f"\n✓ Query complete. Found {result['result_count']} results.")
        sys.exit(0)

    except FileNotFoundError as e:
        print(f"\n✗ {e}")
        sys.exit(1)
    except RuntimeError as e:
        print(f"\n✗ {e}")
        sys.exit(2)
    except Exception as e:
        print(f"\n✗ Unexpected error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(3)


if __name__ == "__main__":
    main()
