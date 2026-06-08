#!/usr/bin/env python3
"""List golden must_fact_substrings missing from report.json (no Prime)."""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

try:
    import yaml
except ImportError:
    print("report-missing-facts.py requires PyYAML", file=sys.stderr)
    sys.exit(2)


def load_rescore(dir_path: Path):
    spec = importlib.util.spec_from_file_location("rescore_golden", dir_path / "rescore-golden.py")
    mod = importlib.util.module_from_spec(spec)
    assert spec.loader
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, default=Path(__file__).parent / "report.json")
    parser.add_argument("--expected", type=Path, default=Path(__file__).parent / "expected.yaml")
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Write markdown report (default: reports/missing-facts-YYYYMMDD.md)",
    )
    parser.add_argument("--top", type=int, default=25, help="Max files to list in detail")
    args = parser.parse_args()

    dir_path = Path(__file__).parent
    rg = load_rescore(dir_path)
    expected = yaml.safe_load(args.expected.read_text(encoding="utf-8"))
    report = json.loads(args.report.read_text(encoding="utf-8"))
    by_name = {r["file_name"]: r for r in report.get("files", [])}

    rows: list[dict] = []
    total_facts = found_facts = 0

    for fname, rules in expected.get("files", {}).items():
        must_f = rules.get("must_fact_substrings") or []
        if not must_f:
            continue
        row = by_name.get(fname)
        if row is None:
            rows.append(
                {
                    "file": fname,
                    "missing_count": len(must_f),
                    "missing": must_f,
                    "note": "not in report",
                }
            )
            total_facts += len(must_f)
            continue
        if rules.get("eval_optional") and int(row.get("entities_promoted", 0)) == 0:
            continue
        facts = row.get("verified_facts") or []
        missing = [m for m in must_f if not rg.fact_found(m, facts)]
        total_facts += len(must_f)
        found_facts += len(must_f) - len(missing)
        if missing:
            rows.append({"file": fname, "missing_count": len(missing), "missing": missing})

    rows.sort(key=lambda r: r["missing_count"], reverse=True)
    recall = found_facts / total_facts if total_facts else 1.0

    out_path = args.out or (dir_path / "reports" / f"missing-facts-{datetime.now(timezone.utc).strftime('%Y%m%d')}.md")
    out_path.parent.mkdir(parents=True, exist_ok=True)

    lines = [
        "# Golden missing facts report",
        "",
        f"**Report:** `{args.report}`  ",
        f"**Generated:** {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}  ",
        f"**Fact recall:** {found_facts}/{total_facts} = {100 * recall:.1f}%  ",
        f"**Files with gaps:** {len(rows)}  ",
        "",
        "## Top files (fix YAML or re-eval with patch-report-file)",
        "",
    ]
    for r in rows[: args.top]:
        lines.append(f"### `{r['file']}` ({r['missing_count']} missing)")
        if r.get("note"):
            lines.append(f"- {r['note']}")
        for m in r["missing"][:12]:
            lines.append(f"- `{m[:120]}{'…' if len(m) > 120 else ''}`")
        if len(r["missing"]) > 12:
            lines.append(f"- … +{len(r['missing']) - 12} more")
        lines.append("")

    lines.append("## Suggested actions")
    lines.append("")
    lines.append("1. `python3 scripts/ingest-quality/sharpen-golden-facts.py --write` (offline anchors)")
    lines.append("2. `python3 scripts/ingest-quality/fill-golden-from-report.py --write` (align to promoted facts)")
    lines.append("3. `python3 scripts/ingest-quality/patch-report-file.py <path>` (Prime, one file)")
    lines.append("4. `scripts/ingest-quality/replay-wave-core.sh` (Prime, 15 core files)")
    lines.append("")

    out_path.write_text("\n".join(lines), encoding="utf-8")
    json_path = out_path.with_suffix(".json")
    json_path.write_text(
        json.dumps(
            {
                "fact_recall": recall,
                "total_facts": total_facts,
                "found_facts": found_facts,
                "files_with_gaps": len(rows),
                "top": rows[: args.top],
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    print(f"Fact recall: {found_facts}/{total_facts} = {100 * recall:.1f}%")
    print(f"Files with missing facts: {len(rows)}")
    print(f"Wrote {out_path}")
    print(f"Wrote {json_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
