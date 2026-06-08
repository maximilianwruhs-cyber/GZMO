#!/usr/bin/env python3
"""Patch report.json summary contract fields from offline rescore-golden.py."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, default=Path(__file__).parent / "report.json")
    parser.add_argument(
        "--zero-entity-exclude-substrings",
        nargs="*",
        default=["360db267"],
        help="Files with 0 entities matching these substrings are excluded from zero_entity_files count",
    )
    args = parser.parse_args()

    dir_path = Path(__file__).parent
    proc = subprocess.run(
        ["python3", str(dir_path / "rescore-golden.py"), "--json", "--report", str(args.report)],
        capture_output=True,
        text=True,
        check=True,
    )
    summary = json.loads(proc.stdout)
    report = json.loads(args.report.read_text(encoding="utf-8"))
    report["summary"]["must_entities_recall"] = summary["must_entities_recall"]
    report["summary"]["must_facts_recall"] = summary["must_facts_recall"]
    report["summary"]["anti_entities_found_count"] = summary["anti_entities_found_count"]
    excludes = args.zero_entity_exclude_substrings
    ze = 0
    for f in report.get("files", []):
        if f.get("entities_promoted", 0) != 0:
            continue
        if any(sub in f.get("file_name", "") for sub in excludes):
            continue
        ze += 1
    report["summary"]["zero_entity_files"] = ze
    args.report.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(
        f"Patched {args.report}: entities_recall={summary['must_entities_recall']:.1%} "
        f"facts_recall={summary['must_facts_recall']:.1%} zero_entity_files={ze}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
