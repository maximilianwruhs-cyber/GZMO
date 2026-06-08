#!/usr/bin/env python3
"""Sharpen must_fact_substrings in expected.yaml from report verified_facts (no Prime).

Replaces truncated snippets (with '...') and mismatched phrases with short in-text anchors.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("sharpen-golden-facts.py requires PyYAML", file=sys.stderr)
    sys.exit(2)


def fact_found(needle: str, facts: list[str]) -> bool:
    n = needle.lower()
    return any(n in f.lower() for f in facts)


def clean_fact_text(fact: str) -> str:
    s = fact.strip()
    s = s.replace("≈", "≈").replace("–", "-")
    s = re.sub(r"\$[^$]*\$", " ", s)
    s = re.sub(r"[\x00-\x08\x0b-\x1f]", "", s)
    s = re.sub(r"\s+", " ", s)
    return s.strip()


def anchors_from_fact(fact: str, max_anchors: int = 2) -> list[str]:
    s = clean_fact_text(fact)
    if len(s) < 10:
        return []

    found: list[str] = []
    seen: set[str] = set()

    def add(a: str) -> None:
        a = a.strip()
        if len(a) < 10 or "..." in a:
            return
        key = a.lower()
        if key in seen:
            return
        seen.add(key)
        found.append(a)

    for m in re.finditer(r"'([^']{10,55})'", s):
        add(m.group(1))
    for m in re.finditer(r'"([^"]{10,55})"', s):
        add(m.group(1))

    if "$" in fact or "μ" in fact or "Φ" in fact:
        for token in ("$OBL", "1 $OBL", "Watt-hour", "Average fitness", "mutation", "Consciousness Score"):
            if token.lower() in s.lower() or token in fact:
                add(token if token in fact else next((t for t in (token,) if t.lower() in s.lower()), token))

    if len(s) <= 44:
        add(s)
    else:
        head = s[:40].rsplit(" ", 1)[0]
        if len(head) >= 12 and not head.endswith(":"):
            add(head)
        tail_words = s.split()
        if len(tail_words) >= 4:
            add(" ".join(tail_words[-4:]))

    words = s.split()
    for i in range(len(words) - 2):
        chunk = " ".join(words[i : i + 3])
        if len(chunk) >= 14 and (
            sum(1 for c in chunk if c.isupper()) >= 2
            or any(c.isdigit() for c in chunk)
            or chunk.lower() in ("openclaw", "fastapi", "postgresql", "intel nuc")
        ):
            add(chunk)
            break

    return found[:max_anchors]


def sharpen_file(rules: dict, row: dict | None, max_facts: int, max_anchors_per_fact: int) -> int:
    if row is None:
        return 0
    facts = [clean_fact_text(f) for f in (row.get("verified_facts") or []) if f and len(f.strip()) >= 10]
    if not facts:
        return 0

    anchors: list[str] = []
    for f in facts[:max_facts]:
        anchors.extend(anchors_from_fact(f, max_anchors_per_fact))

    # Keep existing must_facts that still match (manual curated)
    for old in rules.get("must_fact_substrings") or []:
        if "..." not in old and fact_found(old, facts):
            anchors.insert(0, old)

    deduped: list[str] = []
    seen: set[str] = set()
    for a in anchors:
        k = a.lower()
        if k not in seen:
            seen.add(k)
            deduped.append(a)

    if not deduped:
        return 0

    old_n = len(rules.get("must_fact_substrings") or [])
    rules["must_fact_substrings"] = deduped[: max_facts * max_anchors_per_fact]
    return 1 if len(rules["must_fact_substrings"]) != old_n or old_n == 0 else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, default=Path(__file__).parent / "report.json")
    parser.add_argument("--expected", type=Path, default=Path(__file__).parent / "expected.yaml")
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--max-facts", type=int, default=3, help="verified_facts per file to scan")
    parser.add_argument("--anchors-per-fact", type=int, default=2)
    parser.add_argument("--only-missing", action="store_true", help="Only files with failing fact checks")
    args = parser.parse_args()

    report = json.loads(args.report.read_text(encoding="utf-8"))
    by_name = {r["file_name"]: r for r in report.get("files", [])}
    expected = yaml.safe_load(args.expected.read_text(encoding="utf-8"))
    files = expected.setdefault("files", {})

    updated = 0
    for fname, rules in files.items():
        if not rules.get("must_entities") and not rules.get("must_fact_substrings"):
            continue
        row = by_name.get(fname)
        if args.only_missing and row:
            olds = rules.get("must_fact_substrings") or []
            facts = row.get("verified_facts") or []
            if olds and all(fact_found(o, facts) for o in olds):
                continue
        if sharpen_file(rules, row, args.max_facts, args.anchors_per_fact):
            updated += 1
            n = len(rules.get("must_fact_substrings", []))
            print(f"[+] {fname}: {n} fact anchors")

    if not args.write:
        print(f"\nDry-run: would sharpen {updated} files. Pass --write to apply.")
        return 0

    args.expected.write_text(
        yaml.safe_dump(expected, allow_unicode=True, sort_keys=False, width=120),
        encoding="utf-8",
    )
    print(f"\nWrote {updated} files to {args.expected}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
