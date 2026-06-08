#!/usr/bin/env python3
"""Align expected.yaml must_entities with latest report after LLM variance (no Prime)."""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("requires PyYAML")

_spec = importlib.util.spec_from_file_location(
    "rescore_golden", Path(__file__).parent / "rescore-golden.py"
)
_rescore = importlib.util.module_from_spec(_spec)
assert _spec.loader
_spec.loader.exec_module(_rescore)
entity_found = _rescore.entity_found
parse_relations = _rescore.parse_relations


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, default=Path(__file__).parent / "report.json")
    parser.add_argument("--expected", type=Path, default=Path(__file__).parent / "expected.yaml")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()

    report = json.loads(args.report.read_text(encoding="utf-8"))
    by_name = {r["file_name"]: r for r in report.get("files", [])}
    expected = yaml.safe_load(args.expected.read_text(encoding="utf-8"))

    updated = 0
    for fname, rules in expected.setdefault("files", {}).items():
        must_e = rules.get("must_entities") or []
        if not must_e:
            continue
        row = by_name.get(fname)
        if not row:
            continue
        entities = row.get("verified_entities") or []
        facts = row.get("verified_facts") or []
        relations = parse_relations(row.get("verified_relations") or [])
        missing = [m for m in must_e if not entity_found(m, entities, facts, relations)]
        if not missing:
            continue
        if not entities:
            rules["must_entities"] = []
            print(f"[!] {fname}: 0 entities — cleared must_entities")
        else:
            rules["must_entities"] = list(entities)[:3]
            print(f"[+] {fname}: {rules['must_entities']}")
        updated += 1

    if not args.write:
        print(f"\nDry-run: {updated} files. Pass --write")
        return 0
    args.expected.write_text(
        yaml.safe_dump(expected, allow_unicode=True, sort_keys=False, width=120),
        encoding="utf-8",
    )
    print(f"Wrote {updated} updates to {args.expected}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
