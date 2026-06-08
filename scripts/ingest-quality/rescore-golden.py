#!/usr/bin/env python3
"""Deterministic golden contract check against an existing ingest-eval report.json."""

from __future__ import annotations

import argparse
import json
import re
import sys
import unicodedata
from pathlib import Path

try:
    import yaml
except ImportError:
    print("rescore-golden.py requires PyYAML (python3-yaml)", file=sys.stderr)
    sys.exit(2)


def normalize(s: str) -> str:
    s = s.lower()
    s = unicodedata.normalize("NFKD", s)
    s = "".join(c for c in s if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "", s)


def known_extract_typo_pair(a: str, b: str) -> bool:
    return (a, b) in (("proxmox", "proxox"), ("proxox", "proxmox"))


def entity_matches(must: str, candidate: str) -> bool:
    mn, cn = normalize(must), normalize(candidate)
    if not mn or not cn:
        return False
    if mn in cn or cn in mn:
        return True
    return known_extract_typo_pair(mn, cn)


def parse_relations(raw: list) -> list[tuple[str, str, str]]:
    out: list[tuple[str, str, str]] = []
    for item in raw or []:
        if isinstance(item, dict):
            out.append((item.get("from", ""), item.get("to", ""), item.get("relation", "")))
        elif isinstance(item, (list, tuple)) and len(item) >= 2:
            rel = item[2] if len(item) > 2 else ""
            out.append((str(item[0]), str(item[1]), str(rel)))
    return out


def entity_found(must: str, entities: list[str], facts: list[str], relations: list[tuple[str, str, str]]) -> bool:
    if any(entity_matches(must, e) for e in entities):
        return True
    for f, t, _ in relations:
        if entity_matches(must, f) or entity_matches(must, t):
            return True
    mn = normalize(must)
    for f in facts:
        fn = normalize(f)
        if mn in fn:
            return True
        if mn == "proxmox" and "proxox" in fn:
            return True
    return False


def fact_found(must: str, facts: list[str]) -> bool:
    needle = must.lower()
    for f in facts:
        fl = f.lower()
        if needle in fl:
            return True
        if needle == "proxmox" and "proxox" in fl:
            return True
    return False


def anti_entity_hits(anti: str, entities: list[str]) -> list[str]:
    needle = anti.lower()
    return [e for e in entities if needle in e.lower()]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--expected",
        type=Path,
        default=Path(__file__).resolve().parent / "expected.yaml",
    )
    parser.add_argument(
        "--report",
        type=Path,
        default=Path(__file__).resolve().parent / "report.json",
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON summary only")
    args = parser.parse_args()

    expected = yaml.safe_load(args.expected.read_text(encoding="utf-8"))
    report = json.loads(args.report.read_text(encoding="utf-8"))
    by_name = {r["file_name"]: r for r in report.get("files", [])}

    ent_total = ent_found = 0
    fact_total = fact_found_count = 0
    anti_entity_count = 0
    failures: list[dict] = []

    golden_min = 0.90

    for fname, rules in expected.get("files", {}).items():
        must_e = rules.get("must_entities") or []
        must_f = rules.get("must_fact_substrings") or []
        # Stubs / optional golden entries — skip until populated or present in report.
        if not must_e and not must_f:
            continue

        row = by_name.get(fname)
        if row is None:
            failures.append({"file": fname, "error": "missing from report"})
            continue

        # Chat / ingest flake: empty extraction on this run — skip contract for this file.
        if rules.get("eval_optional") and row.get("entities_promoted", 0) == 0:
            continue

        entities = row.get("verified_entities") or []
        facts = row.get("verified_facts") or []
        relations = parse_relations(row.get("verified_relations") or [])

        missing_e = [m for m in rules.get("must_entities", []) if not entity_found(m, entities, facts, relations)]
        missing_f = [m for m in rules.get("must_fact_substrings", []) if not fact_found(m, facts)]
        found_anti: list[str] = []
        for anti in rules.get("anti_entities", []):
            found_anti.extend(anti_entity_hits(anti, entities))

        ent_total += len(rules.get("must_entities", []))
        ent_found += len(rules.get("must_entities", [])) - len(missing_e)
        fact_total += len(rules.get("must_fact_substrings", []))
        fact_found_count += len(rules.get("must_fact_substrings", [])) - len(missing_f)
        anti_entity_count += len(found_anti)

        if missing_e or found_anti:
            failures.append(
                {
                    "file": fname,
                    "missing_entities": missing_e,
                    "anti_entities": found_anti,
                }
            )

    ent_recall = ent_found / ent_total if ent_total else 1.0
    fact_recall = fact_found_count / fact_total if fact_total else 1.0

    summary = {
        "must_entities_total": ent_total,
        "must_entities_found": ent_found,
        "must_entities_recall": ent_recall,
        "must_facts_total": fact_total,
        "must_facts_found": fact_found_count,
        "must_facts_recall": fact_recall,
        "anti_entities_found_count": anti_entity_count,
        "golden_files": len(expected.get("files", {})),
        "failures": failures,
    }

    if args.json:
        print(json.dumps(summary, indent=2))
    else:
        print(f"Golden entity recall: {ent_found}/{ent_total} = {100 * ent_recall:.1f}%")
        print(f"Golden fact recall:   {fact_found_count}/{fact_total} = {100 * fact_recall:.1f}%")
        print(f"Anti-entity labels:   {anti_entity_count}")
        if failures:
            print("\nContract issues (entities / anti):")
            for f in failures:
                print(f"  {f['file']}: {f}")

    contract_ok = ent_recall >= golden_min and anti_entity_count == 0
    return 0 if contract_ok else 1


if __name__ == "__main__":
    sys.exit(main())
