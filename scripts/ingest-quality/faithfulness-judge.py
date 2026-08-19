#!/usr/bin/env python3
"""M4 retrieval faithfulness judge.

Grounding modes (RAGAS-style: claim must be entailed by the grounding source):
  context — claim vs the concatenated top-5 retrieval hits (GATE metric)
  corpus  — claim vs the archive source document (stricter, informational)
  both    — supported only if context AND corpus agree
  either  — diagnostic, supported if either agrees

proxy mode keeps the fast substring self-consistency check (claim in hits).

Writes reports/faithfulness-judge-YYYYMMDD.json (+ -latest.json).
"""

from __future__ import annotations

import argparse
import json
import os
import sqlite3
import sys
from datetime import datetime
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
    normalize_ws,
    parse_engine_local,
)


def load_yaml(path: Path) -> dict:
    try:
        import yaml  # type: ignore

        return yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except Exception:
        return {}


def build_path_index(ingest_report: dict | None) -> dict[str, dict]:
    if not ingest_report:
        return {}
    out: dict[str, dict] = {}
    for entry in ingest_report.get("files", []):
        name = entry.get("file_name")
        if name:
            out[name] = entry
    return out


def read_archive(file_path: str, cache: dict[str, str]) -> str:
    if file_path in cache:
        return cache[file_path]
    text = ""
    p = Path(file_path)
    if p.is_file():
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            text = ""
    cache[file_path] = text
    return text


def load_honeypot_source(vault_db: Path, file_name: str, max_chars: int) -> str:
    if not vault_db.is_file():
        return ""
    suffix = file_name.split("/")[-1]
    try:
        conn = sqlite3.connect(str(vault_db))
        rows = conn.execute(
            """
            SELECT content FROM honeypot
            WHERE is_latest = 1 AND source_file IS NOT NULL
              AND (source_file = ? OR source_file LIKE '%' || ?)
            ORDER BY promoted_at DESC LIMIT 40
            """,
            (file_name, suffix),
        ).fetchall()
        conn.close()
    except sqlite3.Error:
        return ""
    parts, seen, total = [], set(), 0
    for (content,) in rows:
        c = (content or "").strip()
        if not c or c in seen:
            continue
        seen.add(c)
        parts.append(c)
        total += len(c)
        if total >= max_chars:
            break
    return "\n".join(parts)


def corpus_source_for(
    file_name: str,
    path_index: dict[str, dict],
    vault_db: Path,
    archive_cache: dict[str, str],
) -> str:
    entry = path_index.get(file_name) or {}
    fp = entry.get("file_path")
    if fp:
        text = read_archive(fp, archive_cache)
        if text.strip():
            return text
    # Fallback to promoted honeypot content when the raw archive is unavailable.
    return load_honeypot_source(vault_db, file_name, 14000)


def fact_text_of(fact: Any) -> str:
    return fact["text"] if isinstance(fact, dict) else fact


def fact_in_hits(fact: str, hits: list) -> bool:
    nf = normalize_ws(fact).lower()
    if not nf:
        return False
    blob = normalize_ws(" ".join((h.get("evidence_text") or h.get("text", "")) for h in hits)).lower()
    return nf in blob


def format_exemplars(exemplars_cfg: dict) -> str:
    lines = []
    for ex in exemplars_cfg.get("exemplars", [])[:6]:
        lines.append(
            f"Claim: {ex.get('claim')}\n"
            f"Source: {ex.get('source_snippet')}\n"
            f"Verdict: supported={ex.get('supported')} confidence={ex.get('confidence')}\n"
            f"Evidence: {ex.get('evidence', '')}\n"
        )
    return "\n".join(lines)


SYSTEM_TEMPLATE = (
    "You are a strict fact-checker for retrieval memory evaluation.\n"
    "Judge whether each CLAIM is supported by the SOURCE only.\n"
    "Do NOT use the query, outside knowledge, or anything beyond the SOURCE.\n\n"
    + VERIFY_RULES
    + "\n\nEXEMPLARS:\n{exemplars}\n"
)


def judge_group(
    source: str,
    items: list[dict[str, Any]],
    field: str,
    url: str,
    model: str,
    min_confidence: float,
    min_evidence: int,
    batch_size: int,
    exemplars_text: str,
    timeout_s: int,
) -> None:
    """Annotate each item with f"{field}_ok"/_conf/_ev verdicts against `source`."""
    ok_key, conf_key, ev_key = f"{field}_ok", f"{field}_conf", f"{field}_ev"
    if not source.strip():
        for it in items:
            it[ok_key] = False
            it[conf_key] = 0.0
            it[ev_key] = ""
            it.setdefault("notes", []).append(f"{field}:no_source")
        return
    system = SYSTEM_TEMPLATE.format(exemplars=exemplars_text)
    for start in range(0, len(items), batch_size):
        batch = items[start : start + batch_size]
        listing = "\n".join(
            f'C{i}: claim="{it["fact"]}" | query="{(it.get("query") or "")[:120]}"'
            for i, it in enumerate(batch)
        )
        user = f"SOURCE:\n---\n{source}\n---\n\nCLAIMS:\n{listing}"
        try:
            verdicts = chat_verdicts(url, model, system, user, 0.1, timeout_s, seed=judge_seed())
        except Exception as e:  # noqa: BLE001 — record, keep going
            for it in batch:
                it[ok_key] = False
                it[conf_key] = 0.0
                it[ev_key] = ""
                it.setdefault("notes", []).append(f"{field}:error:{e}")
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
                and len(evidence) >= min_evidence
                and evidence_in_source(evidence, source)
            )
            it[ok_key] = ok
            it[conf_key] = confidence
            it[ev_key] = evidence


def collect_work(eval_files: list, valid_facts: set | None) -> list[dict[str, Any]]:
    work: list[dict[str, Any]] = []
    for file_data in eval_files:
        file_name = file_data.get("file_name", "")
        for probe in file_data.get("probes", []):
            query = probe.get("query")
            hits = probe.get("hits", [])
            for fact in probe.get("recalled_facts", []):
                ft = fact_text_of(fact)
                item = {
                    "file": file_name,
                    "query": query,
                    "fact": ft,
                    "hits": hits,
                    "notes": [],
                }
                if valid_facts is not None and (file_name, ft) not in valid_facts:
                    item["invalid_claim"] = True
                work.append(item)
    return work


def run_proxy(work: list[dict[str, Any]]) -> tuple[float, int, int, list]:
    total = supported = 0
    failures = []
    for it in work:
        total += 1
        if fact_in_hits(it["fact"], it["hits"]):
            supported += 1
        else:
            failures.append(
                {"file": it["file"], "query": it["query"], "fact": it["fact"], "reason": "not_in_top5_hits"}
            )
    score = supported / total if total else 1.0
    return score, supported, total, failures


def run_llm(
    work: list[dict[str, Any]],
    grounding: str,
    path_index: dict,
    vault_db: Path,
    url: str,
    model: str,
    min_confidence: float,
    min_evidence: int,
    batch_size: int,
    exemplars_text: str,
    timeout_s: int,
    archive_cache: dict[str, str],
) -> None:
    need_context = grounding in ("context", "both", "either")
    need_corpus = grounding in ("corpus", "both", "either")

    active = [it for it in work if not it.get("invalid_claim")]
    for it in work:
        if it.get("invalid_claim"):
            it.setdefault("notes", []).append("skipped_invalid_claim")

    if need_context:
        # group by probe — the hits are the per-probe context source
        groups: dict[tuple, list] = {}
        for it in active:
            groups.setdefault((it["file"], it["query"]), []).append(it)
        for (file_name, query), items in groups.items():
            source_parts = []
            for h in items[0].get("hits", []):
                part = f"HIT_CONTENT: {h.get('text', '')}"
                if h.get("evidence_text"):
                    part += f"\nEVIDENCE_SPAN: {h.get('evidence_text')}"
                source_parts.append(part)
            source = "\n\n".join(source_parts).strip()
            judge_group(
                source, items, "context", url, model, min_confidence,
                min_evidence, batch_size, exemplars_text, timeout_s,
            )

    if need_corpus:
        groups = {}
        for it in active:
            groups.setdefault((it["file"], it["query"]), []).append(it)
        for (file_name, query), items in groups.items():
            full = corpus_source_for(file_name, path_index, vault_db, archive_cache)
            source = extract_snippet(full, query or "", items[0]["fact"], 6000)
            judge_group(
                source, items, "corpus", url, model, min_confidence,
                min_evidence, batch_size, exemplars_text, timeout_s,
            )


def score_breakdown(work: list[dict[str, Any]], grounding: str) -> dict[str, Any]:
    judged = [it for it in work if not it.get("invalid_claim")]
    n = len(judged)

    def frac(key: str) -> float | None:
        if n == 0:
            return 1.0
        return sum(1 for it in judged if it.get(key)) / n

    ctx = frac("context_ok") if grounding in ("context", "both", "either") else None
    corp = frac("corpus_ok") if grounding in ("corpus", "both", "either") else None

    if grounding == "context":
        combined_key = "context_ok"
    elif grounding == "corpus":
        combined_key = "corpus_ok"
    elif grounding == "both":
        for it in judged:
            it["judge_ok"] = bool(it.get("context_ok")) and bool(it.get("corpus_ok"))
        combined_key = "judge_ok"
    else:  # either
        for it in judged:
            it["judge_ok"] = bool(it.get("context_ok")) or bool(it.get("corpus_ok"))
        combined_key = "judge_ok"

    combined = frac(combined_key)
    supported = sum(1 for it in judged if it.get(combined_key)) if n else 0
    # Gate metric is always the context grounding when available, else combined.
    gate = ctx if ctx is not None else combined
    return {
        "faithfulness_context": ctx,
        "faithfulness_corpus": corp,
        "faithfulness_judge": gate,
        "faithfulness_combined": combined,
        "supported": supported,
        "total": n,
        "invalid_claims": sum(1 for it in work if it.get("invalid_claim")),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="M4 retrieval faithfulness judge")
    parser.add_argument("--recall-report", type=Path, default=HARNESS / "reports" / "gzmo_report.json")
    parser.add_argument("--ingest-report", type=Path, default=HARNESS / "report.json")
    parser.add_argument("--mode", choices=["proxy", "llm"], default="proxy")
    parser.add_argument("--grounding", choices=["context", "corpus", "both", "either"], default="context")
    parser.add_argument("--max-facts", type=int, default=0, help="Cap checks (0=all)")
    parser.add_argument("--batch-size", type=int, default=8)
    parser.add_argument("--min-confidence", type=float, default=0.85)
    parser.add_argument("--min-evidence", type=int, default=12)
    parser.add_argument("--gate", action="store_true", help="Exit 1 if below gate min")
    parser.add_argument("--gate-min", type=float, default=0.90)
    parser.add_argument("--sample", type=int, default=5)
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--merge-mem-score", action="store_true")
    args = parser.parse_args()

    if not args.recall_report.is_file():
        print(f"Error: recall report not found: {args.recall_report}", file=sys.stderr)
        sys.exit(1)

    recall_data = json.loads(args.recall_report.read_text(encoding="utf-8"))
    eval_files = recall_data.get("retrieval_evaluation", {}).get("files", [])

    ingest_data = None
    if args.ingest_report.is_file():
        ingest_data = json.loads(args.ingest_report.read_text(encoding="utf-8"))
    path_index = build_path_index(ingest_data)

    audit_path = HARNESS / "reports" / "golden-fact-audit.json"
    valid_facts: set | None = None
    if audit_path.is_file():
        try:
            audit = json.loads(audit_path.read_text(encoding="utf-8"))
            valid_facts = {
                (f.get("file"), f.get("fact"))
                for f in audit.get("facts", [])
                if f.get("status") == "valid"
            }
        except (json.JSONDecodeError, OSError):
            valid_facts = None

    exemplars_text = format_exemplars(load_yaml(HARNESS / "faithfulness_exemplars.yaml"))
    vault_db = PROJECT_ROOT / "data" / "vault.db"
    archive_cache: dict[str, str] = {}

    work = collect_work(eval_files, valid_facts)
    if args.max_facts > 0:
        work = work[: args.max_facts]

    if args.mode == "proxy":
        score, supported, total, failures = run_proxy(work)
        breakdown = {
            "faithfulness_context": None,
            "faithfulness_corpus": None,
            "faithfulness_judge": score,
            "faithfulness_combined": score,
            "supported": supported,
            "total": total,
            "invalid_claims": 0,
        }
        mode_label = "proxy"
    else:
        url, model = parse_engine_local(PROJECT_ROOT)
        print(f"judge_endpoint={url} model={model} grounding={args.grounding}")
        run_llm(
            work, args.grounding, path_index, vault_db, url, model,
            args.min_confidence, args.min_evidence, args.batch_size,
            exemplars_text, timeout_s=int(os.environ.get("JUDGE_TIMEOUT", "120")),
            archive_cache=archive_cache,
        )
        breakdown = score_breakdown(work, args.grounding)
        score = breakdown["faithfulness_judge"]
        mode_label = "llm"
        failures = [
            {
                "file": it["file"], "query": it["query"], "fact": it["fact"],
                "reason": "skipped_invalid_claim" if it.get("invalid_claim") else "unsupported",
                "context_ok": it.get("context_ok"),
                "corpus_ok": it.get("corpus_ok"),
            }
            for it in work
            if not it.get("invalid_claim")
            and not (it.get("judge_ok") if args.grounding in ("both", "either") else
                     it.get(f"{args.grounding}_ok"))
        ]

    def fmt(v):
        return f"{v:.4f}" if isinstance(v, (int, float)) else "n/a"

    print(
        f"faithfulness_{mode_label}: judge(gate)={fmt(breakdown['faithfulness_judge'])} "
        f"context={fmt(breakdown['faithfulness_context'])} "
        f"corpus={fmt(breakdown['faithfulness_corpus'])} "
        f"({breakdown['supported']}/{breakdown['total']} supported, "
        f"{breakdown['invalid_claims']} invalid claims skipped)"
    )

    if args.sample and failures:
        print("\n--- Faithfulness failures (sample) ---")
        for item in failures[: args.sample]:
            print(f"  [{item.get('reason')}] ctx={item.get('context_ok')} corp={item.get('corpus_ok')} {item.get('file', '')[:46]}")
            print(f"    query: {(item.get('query') or '')[:68]}")
            print(f"    fact:  {(item.get('fact') or '')[:68]}")

    out_doc = {
        "summary": {
            **breakdown,
            "mode": mode_label,
            "grounding": args.grounding,
            "min_confidence": args.min_confidence,
            "recall_report": str(args.recall_report),
        },
        "failures": failures[:50],
        "results": [
            {k: it.get(k) for k in (
                "file", "query", "fact", "invalid_claim",
                "context_ok", "context_conf", "context_ev",
                "corpus_ok", "corpus_conf", "corpus_ev", "notes",
            )}
            for it in work[:200]
        ],
    }

    if args.write_report or args.mode == "llm":
        reports_dir = HARNESS / "reports"
        reports_dir.mkdir(parents=True, exist_ok=True)
        date_str = datetime.now().strftime("%Y%m%d")
        out_path = reports_dir / f"faithfulness-judge-{date_str}.json"
        out_path.write_text(json.dumps(out_doc, indent=2), encoding="utf-8")
        (reports_dir / "faithfulness-judge-latest.json").write_text(
            json.dumps(out_doc, indent=2), encoding="utf-8"
        )
        print(f"Saved {out_path}")

    if args.merge_mem_score and args.ingest_report.is_file():
        try:
            report_data = json.loads(args.ingest_report.read_text(encoding="utf-8"))
            ms = report_data.setdefault("summary", {}).setdefault("mem_score", {})
            ms["faithfulness_judge"] = breakdown["faithfulness_judge"]
            if breakdown["faithfulness_context"] is not None:
                ms["faithfulness_context"] = breakdown["faithfulness_context"]
            if breakdown["faithfulness_corpus"] is not None:
                ms["faithfulness_corpus"] = breakdown["faithfulness_corpus"]
            args.ingest_report.write_text(json.dumps(report_data, indent=2), encoding="utf-8")
            print(f"Merged faithfulness into {args.ingest_report}")
        except Exception as e:  # noqa: BLE001
            print(f"Non-fatal merge error: {e}", file=sys.stderr)

    if args.gate and (score is None or score < args.gate_min):
        print(f"GATE FAIL: faithfulness {fmt(score)} < {args.gate_min}", file=sys.stderr)
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
