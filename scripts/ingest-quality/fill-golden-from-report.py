#!/usr/bin/env python3
"""Populate empty expected.yaml stubs from an ingest-eval report.json (no Prime)."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("fill-golden-from-report.py requires PyYAML", file=sys.stderr)
    sys.exit(2)


def fact_snippet(fact: str, max_len: int = 40) -> str:
    """Short in-text anchor (no ellipsis — breaks substring match)."""
    s = re.sub(r"\s+", " ", fact.strip())
    s = re.sub(r"\$[^$]*\$", " ", s).strip()
    if len(s) <= max_len:
        return s
    return s[:max_len].rsplit(" ", 1)[0]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, default=Path(__file__).parent / "report.json")
    parser.add_argument("--expected", type=Path, default=Path(__file__).parent / "expected.yaml")
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--max-entities", type=int, default=3)
    parser.add_argument("--max-facts", type=int, default=2)
    args = parser.parse_args()

    report = json.loads(args.report.read_text(encoding="utf-8"))
    by_name = {r["file_name"]: r for r in report.get("files", [])}
    expected = yaml.safe_load(args.expected.read_text(encoding="utf-8"))
    files = expected.setdefault("files", {})

    updated = 0
    for fname, rules in files.items():
        if rules.get("must_entities") or rules.get("must_fact_substrings"):
            continue
        row = by_name.get(fname)
        if not row:
            continue
        ents = list(row.get("verified_entities") or [])[: args.max_entities]
        facts = [
            fact_snippet(f)
            for f in (row.get("verified_facts") or [])[: args.max_facts]
            if f and len(f.strip()) >= 12
        ]
        if not ents:
            continue
        rules["must_entities"] = ents
        if facts:
            rules["must_fact_substrings"] = facts
        # Remove TODO comments by replacing block
        updated += 1
        print(f"[+] {fname}: {ents} | facts={len(facts)}")

    if not args.write:
        print(f"\nDry-run: would update {updated} files. Pass --write to apply.")
        return 0

    args.expected.write_text(
        yaml.safe_dump(expected, allow_unicode=True, sort_keys=False, width=120),
        encoding="utf-8",
    )
    print(f"\nWrote {updated} entries to {args.expected}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
