#!/usr/bin/env python3
"""Build curated Markdown from skill_arxiv.sh OAI-PMH harvest cache for gzmo ingest."""
from __future__ import annotations

import argparse
import json
from collections import OrderedDict
from datetime import datetime, timezone
from pathlib import Path


def arxiv_id_from_identifier(identifier: str) -> str:
    prefix = "oai:arXiv.org:"
    if identifier.startswith(prefix):
        return identifier[len(prefix) :]
    return identifier.rsplit(":", 1)[-1]


def load_deduped_records(meta_path: Path) -> list[dict]:
    by_id: OrderedDict[str, dict] = OrderedDict()
    if not meta_path.is_file():
        return []
    for line in meta_path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        row = json.loads(line)
        ident = row.get("identifier", "")
        if not ident:
            continue
        by_id[ident] = row
    return list(by_id.values())


def build_markdown(
    records: list[dict],
    *,
    set_label: str,
    from_date: str | None,
    source_path: Path,
) -> str:
    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    lines = [
        f"# arXiv OAI-PMH harvest — {set_label}",
        "",
        f"Source: `{source_path}` (Tier-2 `skill_arxiv.sh harvest`).",
        f"Generated: {ts} UTC.",
        "",
        "## Harvest summary",
        "",
        f"- Set: `{set_label}` (OAI-PMH setSpec derived via `oai_set_spec`)",
        f"- Records (deduplicated): {len(records)}",
    ]
    if from_date:
        lines.append(f"- From date: `{from_date}`")
    lines.extend(
        [
            "- Promotion path: curated markdown → `gzmo ingest` → vault/honeypot",
            "",
            "## Preprint index",
            "",
        ]
    )

    for row in records:
        ident = row.get("identifier", "")
        aid = arxiv_id_from_identifier(ident)
        title = (row.get("title") or "").strip().replace("\n", " ")
        datestamp = row.get("datestamp", "")
        harvested_at = row.get("harvested_at", "")
        lines.append(f"### arXiv:{aid}")
        lines.append(f"- Title: {title}")
        lines.append(f"- OAI identifier: `{ident}`")
        lines.append(f"- Datestamp: {datestamp}")
        if harvested_at:
            lines.append(f"- Harvested at: {harvested_at}")
        lines.append(f"- Abs URL: https://arxiv.org/abs/{aid}")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--meta",
        type=Path,
        default=Path("data/arxiv-cache/metadata.jsonl"),
        help="Harvest metadata JSONL (default: data/arxiv-cache/metadata.jsonl)",
    )
    parser.add_argument(
        "--out",
        type=Path,
        default=Path.home()
        / "Schreibtisch/knowledge/curated/thema_004-arxiv-harvest-csAI.md",
        help="Output curated markdown path (part suffix when batching)",
    )
    parser.add_argument("--set", dest="set_label", default="cs.AI", help="Category label")
    parser.add_argument("--from-date", default="2026-06-01", help="Harvest from filter (doc only)")
    parser.add_argument("--max-records", type=int, default=0, help="Cap records (0 = all)")
    parser.add_argument(
        "--batch-size",
        type=int,
        default=300,
        help="Records per curated file (0 = single file)",
    )
    args = parser.parse_args()

    records = load_deduped_records(args.meta)
    if args.max_records > 0:
        records = records[: args.max_records]

    if not records:
        print(f"No records in {args.meta}")
        return 1

    batch_size = args.batch_size if args.batch_size > 0 else len(records)
    written: list[Path] = []
    for i in range(0, len(records), batch_size):
        batch = records[i : i + batch_size]
        part = (i // batch_size) + 1
        if len(records) > batch_size:
            out_path = args.out.with_name(
                f"{args.out.stem}-part{part:02d}{args.out.suffix}"
            )
        else:
            out_path = args.out
        out_path.parent.mkdir(parents=True, exist_ok=True)
        md = build_markdown(
            batch,
            set_label=args.set_label,
            from_date=args.from_date,
            source_path=args.meta.resolve(),
        )
        # Annotate batch in summary when split
        if len(records) > batch_size:
            header = (
                f"- Batch: {part} of {(len(records) + batch_size - 1) // batch_size}"
                f" (records {i + 1}–{i + len(batch)} of {len(records)})\n"
            )
            md = md.replace(
                f"- Records (deduplicated): {len(batch)}",
                f"- Records (deduplicated): {len(batch)}\n{header.rstrip()}",
                1,
            )
        out_path.write_text(md, encoding="utf-8")
        written.append(out_path)
        print(f"Wrote {len(batch)} records → {out_path} ({len(md)} chars)")

    print(f"Total: {len(records)} records in {len(written)} file(s)")
    for p in written:
        print(p)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
