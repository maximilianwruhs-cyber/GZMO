#!/usr/bin/env python3
"""
Google Scholar Labs Follow-up Query Handler

Handles multi-turn conversations within a Scholar Labs session.
Types follow-up questions into the chat interface and retrieves updated results.

Usage:
    python followup.py --session-file session_20240115_143022.json --question "Filter to human studies only"
"""

import argparse
import json
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


def load_session(session_file: Path) -> dict:
    """Load a saved session state."""
    with open(session_file, 'r', encoding='utf-8') as f:
        return json.load(f)


def save_session(session_file: Path, session_data: dict) -> None:
    """Save updated session state."""
    session_file.write_text(
        json.dumps(session_data, indent=2, ensure_ascii=False),
        encoding='utf-8'
    )


def send_followup(
    page,
    question: str,
    timeout: int = 30000
) -> bool:
    """
    Type a follow-up question into the Scholar Labs chat interface.

    Args:
        page: Playwright page object
        question: Follow-up question text
        timeout: Timeout in milliseconds

    Returns:
        True if successful, False otherwise
    """
    # Try multiple selectors for the chat input
    input_selectors = [
        "textarea[placeholder*='follow']",
        "textarea[placeholder*='ask']",
        "input[placeholder*='follow']",
        "input[placeholder*='ask']",
        "[data-testid='chat-input']",
        ".scholar-labs-chat-input",
        "//textarea",  # XPath fallback - any textarea
        "//input[@type='text']",  # Any text input
    ]

    input_elem = None
    for selector in input_selectors:
        try:
            input_elem = page.wait_for_selector(selector, timeout=5000)
            if input_elem:
                print(f"Found chat input: {selector}")
                break
        except PlaywrightTimeout:
            continue

    if not input_elem:
        print("Error: Could not find chat input element")
        return False

    # Type the follow-up question
    input_elem.fill(question)
    time.sleep(0.5)  # Brief pause

    # Try to find and click send button, or press Enter
    send_selectors = [
        "button[type='submit']",
        "button[aria-label*='send']",
        "[data-testid='send-button']",
        ".scholar-labs-send",
    ]

    send_clicked = False
    for selector in send_selectors:
        try:
            send_btn = page.wait_for_selector(selector, timeout=2000)
            if send_btn:
                send_btn.click()
                send_clicked = True
                print("Clicked send button")
                break
        except PlaywrightTimeout:
            continue

    if not send_clicked:
        # Press Enter as fallback
        input_elem.press("Enter")
        print("Pressed Enter to submit")

    # Wait for response
    time.sleep(2)  # Initial wait for response to start

    # Wait for results to update
    try:
        page.wait_for_selector(
            ".scholar-labs-result-container, [data-testid='scholar-labs-result']",
            timeout=timeout
        )
        print("✓ Response received")
        return True
    except PlaywrightTimeout:
        print("Warning: Timeout waiting for response update")
        return False


def run_followup(
    session_file: Path,
    question: str,
    auth_path: Path,
    timeout: int = 30000,
) -> dict:
    """
    Run a follow-up query in an existing Scholar Labs session.

    Args:
        session_file: Path to the JSON session file from previous query
        question: Follow-up question
        auth_path: Path to Playwright auth state
        timeout: Timeout in milliseconds

    Returns:
        Updated results dictionary
    """
    if not session_file.exists():
        raise FileNotFoundError(f"Session file not found: {session_file}")

    if not auth_path.exists():
        raise FileNotFoundError(
            f"Auth state not found: {auth_path}. Run: python auth_setup.py"
        )

    # Load previous session
    session = load_session(session_file)
    original_query = session.get("query", "")

    print(f"Loading session: {session_file}")
    print(f"Original query: {original_query[:80]}...")
    print(f"Follow-up: {question}")

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(storage_state=str(auth_path))
        page = context.new_page()

        try:
            # Navigate to Scholar Labs with original query
            # This should restore the conversation context
            from urllib.parse import quote_plus
            encoded_query = quote_plus(original_query)
            url = f"https://scholar.google.com/scholar_labs/search?hl=en&q={encoded_query}"

            page.goto(url, wait_until="domcontentloaded", timeout=60000)

            # Wait for initial results
            page.wait_for_selector(
                ".scholar-labs-result-container, [data-testid='scholar-labs-result']",
                timeout=30000
            )

            # Send follow-up
            success = send_followup(page, question, timeout)

            if not success:
                browser.close()
                raise RuntimeError("Failed to send follow-up question")

            # Give results time to update
            time.sleep(3)

            # Get updated content
            html_content = page.content()

            browser.close()

            # Parse updated results
            parsed_results = parse_scholar_labs_html(html_content, original_query)

            # Update session data
            timestamp = datetime.utcnow().isoformat() + "Z"
            session["followups"] = session.get("followups", [])
            session["followups"].append({
                "timestamp": timestamp,
                "question": question,
                "result_count": len(parsed_results),
            })
            session["last_updated"] = timestamp
            session["results"] = parsed_results
            session["result_count"] = len(parsed_results)

            # Save updated session
            save_session(session_file, session)

            return {
                "session_file": str(session_file),
                "original_query": original_query,
                "followup": question,
                "timestamp": timestamp,
                "result_count": len(parsed_results),
                "results": parsed_results,
            }

        except Exception as e:
            browser.close()
            raise


def main():
    parser = argparse.ArgumentParser(
        description="Send follow-up query to Google Scholar Labs"
    )
    parser.add_argument(
        "--session-file", "-s",
        type=Path,
        required=True,
        help="Path to session JSON file from previous query"
    )
    parser.add_argument(
        "--question", "-q",
        required=True,
        help="Follow-up question"
    )
    parser.add_argument(
        "--auth-path",
        type=Path,
        default=None,
        help="Path to auth state JSON"
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
        help="Output JSON file"
    )
    parser.add_argument(
        "--rate-sleep",
        type=float,
        default=3.0,
        help="Seconds to sleep after followup for rate limiting"
    )

    args = parser.parse_args()

    # Determine auth path
    if args.auth_path is None:
        from query import get_default_auth_path
        args.auth_path = get_default_auth_path()

    try:
        result = run_followup(
            session_file=args.session_file,
            question=args.question,
            auth_path=args.auth_path,
            timeout=args.timeout,
        )

        # Rate limiting
        if args.rate_sleep > 0:
            time.sleep(args.rate_sleep)

        # Output
        json_output = json.dumps(result, indent=2, ensure_ascii=False)

        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(json_output)
            print(f"\n✓ Results saved to: {args.output}")
        else:
            print("\n" + "=" * 60)
            print(json_output)

        print(f"\n✓ Follow-up complete. Found {result['result_count']} results.")
        print(f"Session updated: {args.session_file}")
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
