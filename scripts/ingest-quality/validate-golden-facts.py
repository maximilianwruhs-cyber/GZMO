#!/usr/bin/env python3
"""Phase A — validate golden must_recall_facts before they gate recall/judge.

Three stages per fact:
  1. heuristics  — reject fragments, path-inference artifacts, too-short claims
  2. corpus      — fact must appear (normalized substring) in the archive file
  3. llm         — Prime verifies the claim against the archive (same rules as
                   the ingest verify gate), with quote evidence

Writes reports/golden-fact-audit.json. Exit 1 with --fail-on-invalid when any
must_recall_fact is invalid.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

HARNESS = Path(__file__).parent.resolve()
PROJECT_ROOT = HARNESS.parent.parent
sys.path.insert(0, str(HARNESS))

from eval_llm import (  # noqa: E402
    VERIFY_RULES,
    chat_verdicts,
    judge_seed,
    evidence_in_source,
    extract_snippet,
    normalize_quote,
    parse_engine_local,
)

PATH_INFERENCE_MARKER = "(inferred from path"
# Truncated extraction artifacts — sentence fragments, not standalone claims.
FRAGMENT_PREFIXES = (
    "primary agent defined in",
    "used for",
    "tool für",
    "library used for",
    "integrated into",
)


def load_yaml(path: Path) -> dict:
    import yaml  # type: ignore

    return yaml.safe_load(path.read_text(encoding="utf-8")) or {}


def build_path_index(report_path: Path) -> dict[str, str]:
    if not report_path.is_file():
        return {}
    try:
        data = json.loads(report_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return {}
    out: dict[str, str] = {}
    for entry in data.get("files", []):
        name = entry.get("file_name")
        fp = entry.get("file_path")
        if name and fp:
            out[name] = fp
    return out


def read_archive(file_path: str, cache: dict[str, str]) -> str:
    if file_path in cache:
        return cache[file_path]
    p = Path(file_path)
    text = ""
    if p.is_file():
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            text = ""
    cache[file_path] = text
    return text


def heuristic_status(fact: str, min_chars: int) -> tuple[bool, str]:
    stripped = fact.strip()
    low = stripped.lower()
    if len(stripped) < min_chars:
        return False, "too_short"
    if PATH_INFERENCE_MARKER in low:
        return False, "path_inference"
    if low.endswith(".md") or low.endswith(".html"):
        return False, "filename"
    for prefix in FRAGMENT_PREFIXES:
        if low.startswith(prefix):
            return False, "fragment"
    return True, "ok"


def collect_facts(files: dict) -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []
    for file_name, cfg in files.items():
        for probe in cfg.get("probes", []) or []:
            query = probe.get("query")
            for fact in probe.get("must_recall_facts", []) or []:
                fact_text = fact["text"] if isinstance(fact, dict) else fact
                items.append(
                    {
                        "file": file_name,
                        "probe_query": query,
                        "fact": fact_text,
                    }
                )
    return items


def llm_validate_group(
    archive: str,
    group: list[dict[str, Any]],
    url: str,
    model: str,
    timeout_s: int,
    min_confidence: float,
    batch_size: int,
) -> None:
    """Annotate each item with llm_valid / llm_confidence / evidence in place."""
    system = (
        "You are a strict fact-checker validating a golden evaluation set.\n"
        "Judge whether each CLAIM is supported by the SOURCE document only.\n"
        "Do NOT use the query or outside knowledge.\n\n" + VERIFY_RULES
    )
    for start in range(0, len(group), batch_size):
        batch = group[start : start + batch_size]
        listing = "\n".join(
            f'C{i}: claim="{it["fact"]}" | query="{(it.get("probe_query") or "")[:120]}"'
            for i, it in enumerate(batch)
        )
        snippet = extract_snippet(archive, batch[0].get("probe_query", ""), batch[0]["fact"], 6000)
        user = f"SOURCE:\n---\n{snippet}\n---\n\nCLAIMS:\n{listing}"
        try:
            verdicts = chat_verdicts(url, model, system, user, 0.1, timeout_s, seed=judge_seed())
        except Exception as e:  # noqa: BLE001 — record, do not abort the run
            for it in batch:
                it["llm_valid"] = False
                it["llm_confidence"] = 0.0
                it["evidence"] = ""
                it["llm_error"] = str(e)
            continue
        vmap = {int(v.get("index", -1)): v for v in verdicts}
        for i, it in enumerate(batch):
            v = vmap.get(i, {})
            supported = bool(v.get("supported"))
            confidence = float(v.get("confidence", 0.0))
            evidence = (v.get("evidence") or "").strip()
            ok = (
                supported
                and confidence >= min_confidence
                and len(evidence) >= 12
                and evidence_in_source(evidence, archive)
            )
            it["llm_valid"] = ok
            it["llm_confidence"] = confidence
            it["evidence"] = evidence


def main() -> None:
    parser = argparse.ArgumentParser(description="Validate golden must_recall_facts")
    parser.add_argument("--expected", type=Path, default=HARNESS / "expected.yaml")
    parser.add_argument("--ingest-report", type=Path, default=HARNESS / "report.json")
    parser.add_argument("--llm", action="store_true", help="Run Prime LLM stage 3")
    parser.add_argument("--max-facts", type=int, default=0, help="Cap facts (0=all)")
    parser.add_argument("--batch-size", type=int, default=6)
    parser.add_argument("--min-confidence", type=float, default=0.85)
    parser.add_argument("--fail-on-invalid", action="store_true")
    parser.add_argument(
        "--min-chars",
        type=int,
        default=8,
        help="Minimum claim length for the heuristic stage (gate). 20 for strict audit.",
    )
    parser.add_argument(
        "--invalid-source",
        choices=["heuristic", "corpus", "llm"],
        default="heuristic",
        help="Strictest stage that must pass for a fact to count as valid. "
        "heuristic=gate (remove artifacts); corpus/llm=stricter informational audit.",
    )
    parser.add_argument("--sample", type=int, default=10)
    args = parser.parse_args()

    if not args.expected.is_file():
        print(f"Error: expected.yaml not found: {args.expected}", file=sys.stderr)
        sys.exit(2)

    files = load_yaml(args.expected).get("files", {})
    path_index = build_path_index(args.ingest_report)
    facts = collect_facts(files)
    if args.max_facts > 0:
        facts = facts[: args.max_facts]

    archive_cache: dict[str, str] = {}
    for it in facts:
        ok, reason = heuristic_status(it["fact"], args.min_chars)
        it["heuristic_ok"] = ok
        it["heuristic_reason"] = reason
        fp = path_index.get(it["file"])
        archive = read_archive(fp, archive_cache) if fp else ""
        it["has_archive"] = bool(archive.strip())
        if not it["has_archive"]:
            it["corpus_substr"] = False
        else:
            it["corpus_substr"] = normalize_quote(it["fact"]) in normalize_quote(archive)

    if args.llm:
        url, model = parse_engine_local(PROJECT_ROOT)
        print(f"validation_endpoint={url} model={model}")
        timeout_s = int(os.environ.get("JUDGE_TIMEOUT", "120"))
        by_file: dict[str, list[dict[str, Any]]] = {}
        for it in facts:
            # Only spend LLM budget on facts that cleared heuristics + have a source.
            if it["heuristic_ok"] and it["has_archive"]:
                by_file.setdefault(it["file"], []).append(it)
            else:
                it["llm_valid"] = False
                it["llm_confidence"] = 0.0
                it["evidence"] = ""
        for file_name, group in by_file.items():
            fp = path_index.get(file_name, "")
            archive = read_archive(fp, archive_cache)
            llm_validate_group(
                archive, group, url, model, timeout_s, args.min_confidence, args.batch_size
            )

    def is_valid(it: dict[str, Any]) -> bool:
        if not it["heuristic_ok"]:
            return False
        if args.invalid_source in ("corpus", "llm") and not it["corpus_substr"]:
            return False
        if args.invalid_source == "llm" and not it.get("llm_valid", False):
            return False
        return True

    for it in facts:
        it["status"] = "valid" if is_valid(it) else "invalid"

    valid_n = sum(1 for it in facts if it["status"] == "valid")
    invalid = [it for it in facts if it["status"] == "invalid"]

    audit = {
        "summary": {
            "total_facts": len(facts),
            "valid": valid_n,
            "invalid": len(invalid),
            "invalid_source": args.invalid_source,
            "min_chars": args.min_chars,
            "corpus_grounded": sum(1 for it in facts if it["corpus_substr"]),
            "llm": args.llm,
        },
        "facts": [
            {
                "file": it["file"],
                "probe_query": it.get("probe_query"),
                "fact": it["fact"],
                "status": it["status"],
                "heuristic_reason": it["heuristic_reason"],
                "has_archive": it["has_archive"],
                "corpus_substr": it["corpus_substr"],
                "llm_valid": it.get("llm_valid"),
                "llm_confidence": it.get("llm_confidence"),
                "evidence": it.get("evidence", ""),
            }
            for it in facts
        ],
    }

    reports_dir = HARNESS / "reports"
    reports_dir.mkdir(parents=True, exist_ok=True)
    out_path = reports_dir / "golden-fact-audit.json"
    out_path.write_text(json.dumps(audit, indent=2), encoding="utf-8")

    print(
        f"golden_fact_audit: {valid_n}/{len(facts)} valid "
        f"(invalid_source={args.invalid_source}, llm={args.llm})"
    )
    print(f"Saved {out_path}")

    if invalid and args.sample:
        print("\n--- Invalid golden facts (sample) ---")
        for it in invalid[: args.sample]:
            why = it["heuristic_reason"] if not it["heuristic_ok"] else (
                "no_archive" if not it["has_archive"] else
                "not_in_corpus" if not it["corpus_substr"] else "llm_unsupported"
            )
            print(f"  [{why}] {it['file'][:50]}")
            print(f"    fact: {it['fact'][:70]}")

    if args.fail_on_invalid and invalid:
        print(f"\nFAIL: {len(invalid)} invalid golden fact(s)", file=sys.stderr)
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
