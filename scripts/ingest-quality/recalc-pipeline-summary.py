#!/usr/bin/env python3
"""Recompute pipeline summary with relation-promotion waivers (no Prime).

Reads gate-config.yaml exclude patterns + expected.yaml waive_relation_promotion.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    print("recalc-pipeline-summary.py requires PyYAML", file=sys.stderr)
    sys.exit(2)


def load_yaml(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8")) or {}


def waived(file_name: str, expected_files: dict, patterns: list[str]) -> bool:
    rules = expected_files.get(file_name) or {}
    if rules.get("waive_relation_promotion"):
        return True
    return any(p in file_name for p in patterns)


def recompute(report: dict, expected: dict, patterns: list[str]) -> dict:
    files = report.get("files", [])
    exp_files = expected.get("files", {})

    ext_all = prom_all = 0
    ext = prom = 0
    zero_rel = 0
    waived_names: list[str] = []

    for row in files:
        name = row["file_name"]
        e = int(row.get("relations_extracted", 0))
        p = int(row.get("relations_promoted", 0))
        ext_all += e
        prom_all += p
        if waived(name, exp_files, patterns):
            waived_names.append(name)
            continue
        ext += e
        prom += p
        if p == 0:
            zero_rel += 1

    rate_all = prom_all / ext_all if ext_all else 0.0
    rate = prom / ext if ext else rate_all

    summary = dict(report.get("summary", {}))
    summary["relation_promotion_rate_all"] = rate_all
    summary["relation_promotion_rate"] = rate
    summary["relation_promotion_waived_files"] = len(waived_names)
    summary["relations_extracted_scoped"] = ext
    summary["relations_promoted_scoped"] = prom
    summary["zero_relation_files"] = zero_rel
    # Keep entity totals global (waivers are relation-only)
    return summary, waived_names


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, default=Path(__file__).parent / "report.json")
    parser.add_argument("--expected", type=Path, default=Path(__file__).parent / "expected.yaml")
    parser.add_argument("--config", type=Path, default=Path(__file__).parent / "gate-config.yaml")
    parser.add_argument("--write", action="store_true", help="Update report.json summary in place")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    if not args.report.exists():
        print(f"Missing report: {args.report}", file=sys.stderr)
        return 2

    cfg = load_yaml(args.config)
    patterns = list(cfg.get("pipeline", {}).get("relation_promotion_exclude_patterns", []))
    expected = load_yaml(args.expected) if args.expected.exists() else {"files": {}}

    report = json.loads(args.report.read_text(encoding="utf-8"))
    summary, waived_names = recompute(report, expected, patterns)

    out = {
        "relation_promotion_rate": summary["relation_promotion_rate"],
        "relation_promotion_rate_all": summary["relation_promotion_rate_all"],
        "waived_files": len(waived_names),
        "zero_relation_files": summary["zero_relation_files"],
    }

    if args.json:
        print(json.dumps(out, indent=2))
    else:
        print(
            f"Relation prom (scoped): {100 * summary['relation_promotion_rate']:.1f}% "
            f"({summary['relations_promoted_scoped']}/{summary['relations_extracted_scoped']})"
        )
        print(
            f"Relation prom (all):    {100 * summary['relation_promotion_rate_all']:.1f}% "
            f"(waived {len(waived_names)} files)"
        )
        print(f"Zero-relation files (scoped): {summary['zero_relation_files']}")

    if args.write:
        report["summary"] = {**report.get("summary", {}), **summary}
        args.report.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(f"Updated {args.report}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
