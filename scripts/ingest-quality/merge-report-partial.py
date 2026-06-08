#!/usr/bin/env python3
"""Merge a partial ingest-eval report into the full report.json and refresh summaries."""

from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
import sys
from pathlib import Path


def load_rescore_module(dir_path: Path):
    spec = importlib.util.spec_from_file_location("rescore_golden", dir_path / "rescore-golden.py")
    mod = importlib.util.module_from_spec(spec)
    assert spec.loader
    spec.loader.exec_module(mod)
    return mod


def recompute_aggregates(report: dict, expected: dict) -> None:
    rg = load_rescore_module(Path(__file__).parent)
    files = report.get("files", [])
    exp = expected.get("files", {})

    total_files = len(files)
    entities_extracted = sum(int(f.get("entities_extracted", 0)) for f in files)
    relations_extracted = sum(int(f.get("relations_extracted", 0)) for f in files)
    entities_promoted = sum(int(f.get("entities_promoted", 0)) for f in files)
    relations_promoted = sum(int(f.get("relations_promoted", 0)) for f in files)
    zero_entity_files = sum(1 for f in files if int(f.get("entities_promoted", 0)) == 0)
    zero_relation_files = sum(1 for f in files if int(f.get("relations_promoted", 0)) == 0)
    relation_promotion_rate = (
        relations_promoted / relations_extracted if relations_extracted else 0.0
    )

    golden_files = 0
    for f in files:
        fname = f["file_name"]
        if fname not in exp:
            continue
        rules = exp[fname]
        if not (rules.get("must_entities") or rules.get("must_fact_substrings")):
            continue
        golden_files += 1
        entities = f.get("verified_entities") or []
        facts = f.get("verified_facts") or []
        relations = rg.parse_relations(f.get("verified_relations") or [])
        missing_e = [
            m
            for m in rules.get("must_entities", [])
            if not rg.entity_found(m, entities, facts, relations)
        ]
        missing_f = [
            m
            for m in rules.get("must_fact_substrings", [])
            if not rg.fact_found(m, facts)
        ]
        found_anti: list[str] = []
        for anti in rules.get("anti_entities", []):
            found_anti.extend(rg.anti_entity_hits(anti, entities))
        must_e_total = len(rules.get("must_entities", []))
        must_e_found = must_e_total - len(missing_e)
        must_f_total = len(rules.get("must_fact_substrings", []))
        must_f_found = must_f_total - len(missing_f)
        score_e = must_e_found / must_e_total if must_e_total else 1.0
        score_f = must_f_found / must_f_total if must_f_total else 1.0
        anti_penalty = 0.5 if found_anti else 0.0
        score = max(0.5 * score_e + 0.5 * score_f - anti_penalty, 0.0)
        f["evaluation"] = {
            "must_entities_total": must_e_total,
            "must_entities_found": must_e_found,
            "must_entities_missing": missing_e,
            "must_facts_total": must_f_total,
            "must_facts_found": must_f_found,
            "must_facts_missing": missing_f,
            "anti_entities_found": found_anti,
            "score": score,
        }

    ent_total = ent_found = fact_total = fact_found = anti_count = 0
    for fname, rules in exp.items():
        must_e = rules.get("must_entities") or []
        must_f = rules.get("must_fact_substrings") or []
        if not must_e and not must_f:
            continue
        row = next((f for f in files if f["file_name"] == fname), None)
        if row is None:
            continue
        if rules.get("eval_optional") and int(row.get("entities_promoted", 0)) == 0:
            continue
        ev = row.get("evaluation") or {}
        ent_total += int(ev.get("must_entities_total", 0))
        ent_found += int(ev.get("must_entities_found", 0))
        fact_total += int(ev.get("must_facts_total", 0))
        fact_found += int(ev.get("must_facts_found", 0))
        anti_count += len(ev.get("anti_entities_found") or [])

    report["summary"] = {
        **report.get("summary", {}),
        "total_files": total_files,
        "golden_files": golden_files,
        "entities_extracted": entities_extracted,
        "relations_extracted": relations_extracted,
        "entities_promoted": entities_promoted,
        "relations_promoted": relations_promoted,
        "zero_entity_files": zero_entity_files,
        "zero_relation_files": zero_relation_files,
        "relation_promotion_rate": relation_promotion_rate,
        "must_entities_recall": ent_found / ent_total if ent_total else 1.0,
        "must_facts_recall": fact_found / fact_total if fact_total else 1.0,
        "anti_entities_found_count": anti_count,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--partial", type=Path, required=True, help="Partial ingest-eval JSON")
    parser.add_argument("--base", type=Path, default=Path(__file__).parent / "report.json")
    parser.add_argument("--expected", type=Path, default=Path(__file__).parent / "expected.yaml")
    parser.add_argument("--write", action="store_true", help="Update --base in place")
    args = parser.parse_args()

    import yaml

    base = json.loads(args.base.read_text(encoding="utf-8"))
    partial = json.loads(args.partial.read_text(encoding="utf-8"))
    expected = yaml.safe_load(args.expected.read_text(encoding="utf-8"))

    by_name = {r["file_name"]: i for i, r in enumerate(base.get("files", []))}
    merged = 0
    added = 0
    for row in partial.get("files", []):
        name = row["file_name"]
        if name in by_name:
            base["files"][by_name[name]] = row
            merged += 1
        else:
            base.setdefault("files", []).append(row)
            by_name[name] = len(base["files"]) - 1
            added += 1

    recompute_aggregates(base, expected)
    print(f"Merged {merged} file(s), added {added} into {args.base}")

    if args.write:
        args.base.write_text(json.dumps(base, indent=2), encoding="utf-8")
        subprocess.run(
            [
                "python3",
                str(Path(__file__).parent / "recalc-pipeline-summary.py"),
                "--report",
                str(args.base),
                "--write",
            ],
            check=True,
        )
        subprocess.run(
            [
                "python3",
                str(Path(__file__).parent / "refresh-report-contract.py"),
                "--report",
                str(args.base),
            ],
            check=True,
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
