#!/usr/bin/env python3
"""Convert Obsidian [[wikilinks]] to OKF CommonMark across the LLM-Wiki tree."""
from __future__ import annotations

import argparse
import re
from pathlib import Path

WIKILINK = re.compile(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]")


def to_commonmark(match: re.Match[str]) -> str:
    target = match.group(1).strip()
    alias = (match.group(2) or target).strip()
    slug = target.lower().replace(" ", "-").replace("_", "-")
    if "/" not in slug and not slug.endswith(".md"):
        slug = f"entities/{slug}.md"
    if not slug.startswith("/"):
        slug = f"/{slug}"
    return f"[{alias}]({slug})"


def migrate_file(path: Path, dry_run: bool) -> bool:
    text = path.read_text(encoding="utf-8")
    if "[[" not in text:
        return False
    updated = WIKILINK.sub(to_commonmark, text)
    if updated == text:
        return False
    if not dry_run:
        path.write_text(updated, encoding="utf-8")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--wiki",
        type=Path,
        default=Path(__file__).resolve().parents[1] / "wiki",
        help="Wiki root (default: survey_GZMO/wiki)",
    )
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    changed = 0
    scanned = 0
    for path in sorted(args.wiki.rglob("*.md")):
        scanned += 1
        if migrate_file(path, args.dry_run):
            changed += 1
            if changed <= 5 or args.dry_run:
                print(path)
    print(f"{'would update' if args.dry_run else 'updated'} {changed}/{scanned} markdown files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
