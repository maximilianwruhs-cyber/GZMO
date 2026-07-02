#!/usr/bin/env python3
"""
Google Scholar Labs Authentication Setup

Performs one-time manual login to Google and saves the authenticated
browser state (cookies, session tokens) to a JSON file for reuse.

This avoids triggering Google's bot detection and 2FA on every query.

Usage:
    python auth_setup.py [--auth-dir AUTH_DIR]

After running, authenticate manually in the browser window, then press
ENTER in the terminal to save the session state.
"""

import argparse
import os
import sys
from pathlib import Path

try:
    from playwright.sync_api import sync_playwright
except ImportError:
    print("Error: playwright not installed. Run: pip install playwright")
    print("Then: playwright install chromium")
    sys.exit(1)


def get_default_auth_dir() -> Path:
    """Get the default auth directory relative to the script location."""
    script_dir = Path(__file__).parent.resolve()
    # Try to find the GZMO project root
    for parent in script_dir.parents:
        if (parent / "gzmo.toml").exists():
            return parent / "playwright" / ".auth"
    # Fallback to script directory
    return script_dir / ".auth"


def setup_auth(auth_dir: Path, headless: bool = False) -> Path:
    """
    Run a headed browser for manual Google authentication.

    Args:
        auth_dir: Directory to save the auth state
        headless: If True, run headless (not recommended for auth setup)

    Returns:
        Path to the saved auth state file
    """
    auth_dir.mkdir(parents=True, exist_ok=True)
    state_file = auth_dir / "google_state.json"

    print(f"Starting browser for Google authentication...")
    print(f"Auth state will be saved to: {state_file}")
    print("\nPlease:")
    print("  1. Log in to your Google account in the browser window")
    print("  2. Navigate to https://scholar.google.com to verify access")
    print("  3. Return here and press ENTER to save the session state")
    print()

    with sync_playwright() as p:
        # Launch browser (headed for manual interaction)
        browser = p.chromium.launch(headless=headless)
        context = browser.new_context()
        page = context.new_page()

        # Navigate to Google Scholar Labs
        print("Opening Google Scholar Labs...")
        page.goto("https://scholar.google.com/scholar_labs/search?hl=en&q=test")

        # Wait for user to complete login
        input("Press ENTER after you've logged in and verified access... ")

        # Save the storage state
        context.storage_state(path=str(state_file))
        print(f"\n✓ Auth state saved to: {state_file}")

        browser.close()

    return state_file


def verify_auth(auth_file: Path) -> bool:
    """Verify that the auth state file exists and is valid JSON."""
    if not auth_file.exists():
        return False

    try:
        import json
        with open(auth_file, 'r') as f:
            data = json.load(f)
        # Basic validation - should have cookies and origins
        return 'cookies' in data and 'origins' in data
    except (json.JSONDecodeError, KeyError):
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Setup Google authentication for Scholar Labs automation"
    )
    parser.add_argument(
        "--auth-dir",
        type=Path,
        default=None,
        help="Directory to save auth state (default: project_root/playwright/.auth)"
    )
    parser.add_argument(
        "--headless",
        action="store_true",
        help="Run in headless mode (not recommended for setup)"
    )
    parser.add_argument(
        "--verify-only",
        action="store_true",
        help="Only verify existing auth state, don't run setup"
    )

    args = parser.parse_args()

    auth_dir = args.auth_dir or get_default_auth_dir()
    state_file = auth_dir / "google_state.json"

    if args.verify_only:
        if verify_auth(state_file):
            print(f"✓ Auth state valid: {state_file}")
            sys.exit(0)
        else:
            print(f"✗ Auth state missing or invalid: {state_file}")
            sys.exit(1)

    # Check if auth already exists
    if state_file.exists():
        response = input(f"Auth state already exists at {state_file}. Overwrite? [y/N] ")
        if response.lower() not in ('y', 'yes'):
            print("Cancelled.")
            sys.exit(0)

    try:
        setup_auth(auth_dir, headless=args.headless)

        # Verify the saved state
        if verify_auth(state_file):
            print("\n✓ Auth setup complete and verified.")
            print(f"\nYou can now run queries with:")
            print(f"  python query.py --question 'your research question'")
        else:
            print("\n⚠ Warning: Auth state saved but verification failed.")
            sys.exit(1)

    except KeyboardInterrupt:
        print("\n\nCancelled by user.")
        sys.exit(130)
    except Exception as e:
        print(f"\n✗ Error during setup: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
