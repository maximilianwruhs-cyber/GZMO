#!/usr/bin/env python3
"""Build curated Markdown from verified Scholar Labs results for gzmo ingest.

This script takes the output of skill_scholar.sh verify (or ingest-query)
and converts it into structured markdown ready for `gzmo ingest`.

Usage:
    python build-scholar-harvest-curated.py \
        --input verified_results.json \
        --out ~/Schreibtisch/knowledge/curated/thema_008-scholar-harvest-batch01.md \
        [--batch-size 50]
"""
from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def sanitize_filename(text: str, max_len: int = 50) -> str:
    """Create a safe filename from text."""
    # Remove non-alphanumeric characters
    safe = re.sub(r'[^\w\s-]', '', text)
    # Replace spaces with hyphens
    safe = re.sub(r'\s+', '-', safe.strip())
    # Limit length
    return safe[:max_len].lower()


def load_verified_results(input_path: Path) -> dict[str, Any]:
    """Load verified results JSON."""
    with open(input_path, 'r', encoding='utf-8') as f:
        return json.load(f)


def build_paper_section(paper: dict, index: int) -> list[str]:
    """Build markdown section for a single paper."""
    title = paper.get("title", "Untitled")
    authors = paper.get("authors", [])
    journal = paper.get("journal", "Unknown")
    year = paper.get("year", "Unknown")
    doi = paper.get("doi", "")
    url = paper.get("url", "")
    citation_count = paper.get("citation_count")
    summary = paper.get("contextual_summary", "")
    findings = paper.get("key_findings", [])

    # Verification info
    verification = paper.get("verification", {})
    verif_status = verification.get("status", "unknown")
    verif_conf = verification.get("max_confidence", 0.0)
    verif_threshold = verification.get("threshold", 0.85)

    # Find OA PDF from verification sources
    oa_pdf = None
    oa_url = None
    for source in verification.get("sources", []):
        if source.get("source") == "unpaywall":
            oa_pdf = source.get("oa_pdf_url")
            oa_url = source.get("oa_url")
        elif source.get("source") == "semantic_scholar":
            if not oa_pdf:
                oa_pdf = source.get("open_access_pdf", {}).get("url") if isinstance(source.get("open_access_pdf"), dict) else None
        elif source.get("source") == "openalex":
            if not oa_url:
                oa_open_access = source.get("open_access", {})
                if isinstance(oa_open_access, dict):
                    oa_url = oa_open_access.get("oa_url")

    lines = [
        f"### {index}. {title}",
        "",
        f"**Authors:** {', '.join(authors) if authors else 'Unknown'}",
        f"**Journal:** {journal} ({year})",
    ]

    if doi:
        lines.append(f"**DOI:** [{doi}]({doi})")

    if url:
        lines.append(f"**URL:** [{url[:60]}...]({url})" if len(url) > 60 else f"**URL:** [{url}]({url})")

    # Verification badge
    status_emoji = {
        "verified": "✓",
        "tentative": "~",
        "unverified": "?",
        "failed": "✗"
    }.get(verif_status, "?")

    lines.append(f"**Verification:** {status_emoji} {verif_status} (confidence: {verif_conf:.2f} / threshold: {verif_threshold})")

    if citation_count:
        lines.append(f"**Citations:** {citation_count}")

    # OA links
    if oa_pdf:
        lines.append(f"**Open Access PDF:** [{oa_pdf[:50]}...]({oa_pdf})" if len(oa_pdf) > 50 else f"**Open Access PDF:** [{oa_pdf}]({oa_pdf})")
    elif oa_url:
        lines.append(f"**Open Access:** [{oa_url[:50]}...]({oa_url})" if len(oa_url) > 50 else f"**Open Access:** [{oa_url}]({oa_url})")

    lines.append("")

    # AI-generated summary
    if summary:
        lines.append("**AI Summary (Scholar Labs):**")
        lines.append(f"> {summary}")
        lines.append("")

    # Key findings
    if findings:
        lines.append("**Key Findings:**")
        for finding in findings:
            lines.append(f"- {finding}")
        lines.append("")

    # Verification sources (condensed)
    sources = verification.get("sources", [])
    if sources:
        source_names = [s.get("source", "unknown") for s in sources if not s.get("error")]
        if source_names:
            lines.append(f"**Verified against:** {', '.join(source_names)}")
            lines.append("")

    lines.append("---")
    lines.append("")

    return lines


def build_markdown(
    data: dict[str, Any],
    *,
    source_path: Path,
    query_info: str = "",
) -> str:
    """Build full curated markdown from verified results."""
    ts = datetime.now(timezone.utc)
    ts_iso = ts.strftime("%Y-%m-%dT%H:%M:%SZ")
    ts_date = ts.strftime("%Y-%m-%d")

    query = data.get("query", "Unknown query")
    results = data.get("results", [])
    verified_at = data.get("verified_at", ts_iso)
    threshold = data.get("threshold", 0.85)

    # Stats
    stats = data.get("verification_stats", {})
    verified_count = stats.get("verified", 0)
    tentative_count = stats.get("tentative", 0)
    unverified_count = stats.get("unverified", 0)
    failed_count = stats.get("failed", 0)

    # Extract query fingerprint for title
    query_slug = sanitize_filename(query, 30)

    lines = [
        "---",
        f"title: thema_008-scholar-harvest-{query_slug}-{ts_date}",
        f"created: {ts_iso}",
        "source: google_scholar_labs",
        f"original_query: {query}",
        f"verification_threshold: {threshold}",
        f"verified_at: {verified_at}",
        f"source_file: {source_path}",
        "---",
        "",
        f"# Google Scholar Labs Harvest — {query_slug}",
        "",
        f"**Original Query:** {query}",
        "",
        f"**Harvested:** {ts_date}",
        "",
        f"**Verified:** {verified_at}",
        "",
        f"**Threshold:** {threshold}",
        "",
        "## Verification Summary",
        "",
        f"- **Total papers:** {len(results)}",
        f"- ✓ Verified: {verified_count}",
        f"- ~ Tentative: {tentative_count}",
        f"- ? Unverified: {unverified_count}",
        f"- ✗ Failed: {failed_count}",
        "",
        "---",
        "",
        "## Papers",
        "",
    ]

    for i, paper in enumerate(results, 1):
        lines.extend(build_paper_section(paper, i))

    # Footer
    lines.extend([
        "",
        "## Notes",
        "",
        "This harvest was generated via the thema_008 Google Scholar Labs integration.",
        "Papers have been cross-referenced with OpenAlex, Crossref, Semantic Scholar,",
        "and Unpaywall for verification and open-access link resolution.",
        "",
        f"Source file: `{source_path}`",
        "",
        "---",
        "",
        "*Generated by build-scholar-harvest-curated.py*",
        "",
    ])

    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input", "-i",
        type=Path,
        required=True,
        help="Input verified results JSON file (from skill_scholar.sh verify)"
    )
    parser.add_argument(
        "--out", "-o",
        type=Path,
        default=Path.home() / "Schreibtisch/knowledge/curated/thema_008-scholar-harvest-batch.md",
        help="Output curated markdown path (part suffix when batching)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=50,
        help="Papers per curated file (0 = single file, default: 50)",
    )
    parser.add_argument(
        "--max-records",
        type=int,
        default=0,
        help="Cap records to process (0 = all)",
    )
    args = parser.parse_args()

    if not args.input.is_file():
        print(f"ERROR: Input file not found: {args.input}")
        return 1

    data = load_verified_results(args.input)
    results = data.get("results", [])

    if not results:
        print(f"No verified papers in {args.input}")
        return 1

    if args.max_records > 0:
        results = results[:args.max_records]
        data["results"] = results

    batch_size = args.batch_size if args.batch_size > 0 else len(results)
    written: list[Path] = []

    for i in range(0, len(results), batch_size):
        batch = results[i:i + batch_size]
        batch_data = {**data, "results": batch}

        part = (i // batch_size) + 1
        if len(results) > batch_size:
            out_path = args.out.with_name(
                f"{args.out.stem}-part{part:02d}{args.out.suffix}"
            )
        else:
            out_path = args.out

        out_path.parent.mkdir(parents=True, exist_ok=True)

        md = build_markdown(
            batch_data,
            source_path=args.input.resolve(),
        )

        # Add batch info when splitting
        if len(results) > batch_size:
            batch_info = f"\n**Batch:** {part} of {(len(results) + batch_size - 1) // batch_size} (papers {i + 1}–{i + len(batch)} of {len(results)})\n"
            md = md.replace("## Papers", f"## Papers{batch_info}", 1)

        out_path.write_text(md, encoding="utf-8")
        written.append(out_path)
        print(f"Wrote {len(batch)} papers → {out_path} ({len(md)} chars)")

    print(f"\nTotal: {len(results)} papers in {len(written)} file(s)")
    for p in written:
        print(p)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
