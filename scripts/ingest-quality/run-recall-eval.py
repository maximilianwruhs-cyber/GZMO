#!/usr/bin/env python3
import json
import os
import urllib.request
import sys
import argparse
import re
from pathlib import Path
from datetime import datetime

EMBED_URL = os.environ.get("EMBED_URL", "http://192.168.31.110:8081/v1/embeddings")
EMBED_MODEL = os.environ.get("EMBED_MODEL", "Qwen3-Embedding-0.6B-Q8_0.gguf")
QDRANT_BASE = os.environ.get("QDRANT_URL", "http://192.168.31.202:6333").rstrip("/")
QDRANT_COLLECTION = os.environ.get("QDRANT_COLLECTION", "honeypot")
QDRANT_URL = f"{QDRANT_BASE}/collections/{QDRANT_COLLECTION}/points/search"

def embed_text(text: str) -> list[float]:
    req_body = {
        "model": EMBED_MODEL,
        "input": text
    }
    req = urllib.request.Request(
        EMBED_URL,
        data=json.dumps(req_body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        res = json.loads(resp.read().decode())
        return res["data"][0]["embedding"]

def search_gzmo_rrf(query: str, limit: int = 5) -> list[dict]:
    """Recall via gzmo RRF (honeypot SQLite + embedder).

    Returns full hit items: {content, source_file, fact_id, score}.
    """
    import subprocess
    repo = Path(__file__).resolve().parents[2]
    # Use repo-local binary (ignore CARGO_TARGET_DIR sandbox redirects).
    bin_path = repo / "target" / "release" / "gzmo"
    if not bin_path.exists():
        bin_path = repo / "target" / "debug" / "gzmo"
    if not bin_path.exists():
        raise FileNotFoundError(
            f"gzmo binary not found under {repo}/target — run: cargo build --release -p gzmo-cli"
        )
    cmd = [
        str(bin_path),
        "memory",
        "search",
        query,
        "--limit",
        str(limit),
        "--json",
        "--no-scratch",
    ]
    proc = subprocess.run(
        cmd,
        cwd=str(repo),
        capture_output=True,
        text=True,
        timeout=120,
        env={**os.environ, "GZMO_SESSION_ID": "recall-eval-rrf"},
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr or proc.stdout or "gzmo memory search failed")
    data = json.loads(proc.stdout)
    items = data.get("items") or []
    hits = [
        {
            "content": item.get("content", ""),
            "source_file": item.get("source_file"),
            "fact_id": item.get("fact_id"),
            "score": item.get("score", 0.0),
            "evidence_text": item.get("evidence_text"),
        }
        for item in items
        if item.get("content")
    ]
    if hits:
        return hits
    # Fallback: parse formatted text lines (no source_file available).
    for line in data.get("text", "").splitlines():
        if line.startswith("- [") and ") " in line:
            hits.append({"content": line.split(") ", 1)[-1].strip(), "source_file": None, "score": 0.0})
    return hits[:limit]


def search_qdrant(vector: list[float]) -> list[dict]:
    req_body = {
        "vector": vector,
        "limit": 5,
        "with_payload": True
    }
    req = urllib.request.Request(
        QDRANT_URL,
        data=json.dumps(req_body).encode(),
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        res = json.loads(resp.read().decode())
        return res["result"]

STRICT_MIN_CHARS = 8


def hit_source_matches(hit: dict, file_name: str) -> bool:
    sf = hit.get("source_file")
    if not sf or not file_name:
        return False
    leaf = file_name.split("/")[-1].lower()
    return sf.lower().endswith(leaf) or leaf.endswith(sf.split("/")[-1].lower())


def is_match(
    fact: str,
    concat_text: str,
    mode: str,
    hits: list | None = None,
    file_name: str = "",
    require_source: bool = False,
) -> bool:
    concat_text_lower = concat_text.lower()
    if mode == "token":
        fact_tokens = [w for w in re.findall(r'\w+', fact.lower()) if w]
        if not fact_tokens:
            return fact.lower() in concat_text_lower
        found = sum(1 for tok in fact_tokens if tok in concat_text_lower)
        return (found / len(fact_tokens)) >= 0.70
    if mode == "strict":
        # Claim must appear within a SINGLE hit (no cross-hit substring), and be
        # long enough to be a claim rather than a keyword. With require_source,
        # that hit must also be cited from the probe's own file.
        if len(fact.strip()) < STRICT_MIN_CHARS:
            return False
        norm_fact = " ".join(fact.lower().split())
        for h in hits or []:
            text_to_check = h.get("evidence_text") or h.get("text", "")
            if norm_fact in " ".join((text_to_check or "").lower().split()):
                if not require_source or hit_source_matches(h, file_name):
                    return True
        return False
    # mode == "normalized"
    norm_fact = " ".join(fact.lower().split())
    norm_concat = " ".join(concat_text_lower.split())
    return norm_fact in norm_concat


def load_valid_facts(audit_path: Path) -> set[tuple[str, str]] | None:
    """Return {(file_name, fact_text)} judged valid by the golden audit, or None."""
    if not audit_path.is_file():
        return None
    try:
        data = json.loads(audit_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None
    return {
        (f.get("file"), f.get("fact"))
        for f in data.get("facts", [])
        if f.get("status") == "valid"
    }

def main():
    harness_dir = Path(__file__).parent.resolve()
    yaml_path = harness_dir / "expected.yaml"
    
    parser = argparse.ArgumentParser(description="Recall@5 evaluation")
    parser.add_argument("--batch", choices=["1", "2", "all"], default="all", help="Subset of files to evaluate")
    parser.add_argument(
        "--match",
        choices=["normalized", "token", "strict"],
        default="normalized",
        help="Fact matching mode. strict=per-hit substring, audit-valid facts only.",
    )
    parser.add_argument(
        "--require-source-match",
        action="store_true",
        help="strict mode: only count a hit whose source_file is cited from the probe's file",
    )
    parser.add_argument(
        "--backend",
        choices=["qdrant", "gzmo"],
        default="gzmo",
        help="Retrieval backend: gzmo RRF (honeypot) or Qdrant vector-only",
    )
    parser.add_argument(
        "--track",
        choices=["auto", "rrf", "rrf_strict", "golden", "qdrant"],
        default="auto",
        help="Metric track for recall-metrics.json: rrf=algorithm, rrf_strict=audit-valid only, golden=probe-aligned, qdrant=vector baseline",
    )
    args = parser.parse_args()

    track = args.track
    if track == "auto":
        if args.backend == "qdrant":
            track = "qdrant"
        elif args.match == "strict":
            track = "rrf_strict"
        else:
            track = os.environ.get("GZMO_RECALL_TRACK", "rrf")

    valid_facts = None
    if args.match == "strict":
        valid_facts = load_valid_facts(harness_dir / "reports" / "golden-fact-audit.json")
        if valid_facts is None:
            print(
                "Warning: --match strict but golden-fact-audit.json missing; "
                "run validate-golden-facts.py first. Counting all facts.",
                file=sys.stderr,
            )
    
    if not yaml_path.exists():
        print(f"Error: expected.yaml not found at {yaml_path}")
        sys.exit(1)
        
    try:
        import yaml
        expected_data = yaml.safe_load(yaml_path.read_text(encoding="utf-8"))
    except Exception as e:
        print(f"Error reading/parsing expected.yaml: {e}")
        sys.exit(1)
        
    files = expected_data.get("files", {})
    
    batch1_files = set()
    core_golden_path = harness_dir / "core-golden-files.txt"
    if core_golden_path.exists():
        for line in core_golden_path.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line and not line.startswith("#"):
                batch1_files.add(line)
                
    batch2_files = {
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterROADMAPmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterdocsCONCEPTmd.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMGZMO_soul_merged_newArtifactsArchitektur_und_Implementierung_autonomer_Sy.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterdocsSCIENTIFIC_FOUNDATIONSmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterdocsVISUAL_IDENTITYmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsops_monitoring_agentmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentssystem_hygiene_agentmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsqa_testing_agentmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsrag_db_agentmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsobservability_watchermd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsdashboard_curator_agentmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsstrategy_analystmd.md",
        "wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentstoken_agentmd.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesObolus_Extension__Kon.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolusNotesProjekt_Obulus__Die_Evolution_der_Digitalen_Glhtml.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesGitHub_-_maximilianwr.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesGitHub_-_microsoft_vs.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesQuelltext_code_3html.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesQuelltext_code_1html.md",
        "wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesWebview_API___Visual_.md"
    }
    
    total_facts = 0
    recalled_facts_count = 0
    lost_facts_list = []
    
    eval_files = []
    
    backend_label = (
        "gzmo RRF (honeypot)" if args.backend == "gzmo" else f"Qdrant: {QDRANT_COLLECTION}"
    )
    print(f"=== Running Recall@5 Evaluation — backend: {backend_label} ===")
    print(f"Filter: batch={args.batch} | Match mode={args.match} | track={track}")
    
    for filename, file_cfg in files.items():
        if args.batch == "1" and filename not in batch1_files:
            continue
        if args.batch == "2" and filename not in batch2_files:
            continue
            
        probes = file_cfg.get("probes", [])
        if not probes:
            continue
            
        file_eval_probes = []
        for probe in probes:
            query = probe.get("query")
            must_recall = probe.get("must_recall_facts", [])
            
            if not query:
                continue
                
            try:
                if args.backend == "gzmo":
                    gzmo_hits = search_gzmo_rrf(query, limit=5)
                    hits = []
                    concat_text = ""
                    for i, gh in enumerate(gzmo_hits):
                        text = gh.get("content", "")
                        ev_text = gh.get("evidence_text")
                        concat_text += " " + (ev_text or text)
                        hits.append({
                            "point_id": gh.get("fact_id"),
                            "score": gh.get("score", 0.0),
                            "text": text,
                            "evidence_text": ev_text,
                            "source_file": gh.get("source_file"),
                            "rank": i + 1,
                        })
                else:
                    vec = embed_text(query)
                    results = search_qdrant(vec)
                    hits = []
                    concat_text = ""
                    for i, r in enumerate(results):
                        point_id = r.get("id")
                        score = r.get("score", 0.0)
                        payload = r.get("payload", {})
                        text = payload.get("content", payload.get("text", ""))
                        concat_text += " " + text
                        hits.append({
                            "point_id": point_id,
                            "score": score,
                            "text": text,
                            "rank": i + 1,
                        })
            except Exception as e:
                print(f"  [ERROR] Retrieval failed for query '{query}': {e}")
                continue
                
            recalled = []
            lost = []
            
            for fact in must_recall:
                fact_text = fact["text"] if isinstance(fact, dict) else fact
                if valid_facts is not None and (filename, fact_text) not in valid_facts:
                    # strict track: skip facts the golden audit marked invalid
                    continue
                total_facts += 1
                if is_match(
                    fact_text, concat_text, args.match, hits,
                    file_name=filename, require_source=args.require_source_match,
                ):
                    recalled_facts_count += 1
                    recalled.append(fact_text)
                else:
                    lost.append(fact_text)
                    # Find best snippet in hits for logging
                    best_snippet = "N/A"
                    best_score = 0.0
                    for h in hits:
                        if h["score"] > best_score:
                            best_score = h["score"]
                            best_snippet = h["text"][:100] + "..."
                    lost_facts_list.append({
                        "file": filename,
                        "query": query,
                        "fact": fact,
                        "best_snippet": best_snippet
                    })
                    
            file_eval_probes.append({
                "query": query,
                "recalled_facts": recalled,
                "lost_facts": lost,
                "hits": hits
            })
            
        if file_eval_probes:
            eval_files.append({
                "file_name": filename,
                "probes": file_eval_probes
            })
            
    recall_at_5 = recalled_facts_count / total_facts if total_facts > 0 else 1.0
    print(f"\nEvaluation complete. Recall@5: {recall_at_5:.4f} ({recalled_facts_count}/{total_facts} facts)")
    
    # Save the recall baseline report
    date_str = datetime.now().strftime("%Y%m%d")
    report_out_dir = harness_dir / "reports"
    report_out_dir.mkdir(parents=True, exist_ok=True)
    baseline_report_path = report_out_dir / f"recall5-baseline-{date_str}.json"
    
    output_data = {
        "summary": {
            "mem_score": {
                "recall_at_5": recall_at_5
            }
        },
        "retrieval_evaluation": {
            "files": eval_files
        }
    }
    
    metrics_entry = {
        "track": track,
        "backend": args.backend,
        "match": args.match,
        "batch": args.batch,
        "recall_at_5": recall_at_5,
        "recalled": recalled_facts_count,
        "total": total_facts,
        "timestamp": datetime.now().isoformat(timespec="seconds"),
    }

    try:
        baseline_report_path.write_text(json.dumps(output_data, indent=2), encoding="utf-8")
        print(f"Saved recall baseline report to {baseline_report_path}")
        if args.backend == "gzmo":
            gzmo_report_path = report_out_dir / "gzmo_report.json"
            gzmo_report_path.write_text(json.dumps(output_data, indent=2), encoding="utf-8")
            print(f"Saved gzmo report to {gzmo_report_path}")
        elif args.backend == "qdrant":
            qdrant_report_path = report_out_dir / "qdrant_report.json"
            qdrant_report_path.write_text(json.dumps(output_data, indent=2), encoding="utf-8")
            print(f"Saved qdrant report to {qdrant_report_path}")

        metrics_path = report_out_dir / "recall-metrics.json"
        metrics_doc = {"runs": [], "latest": {}}
        if metrics_path.exists():
            try:
                metrics_doc = json.loads(metrics_path.read_text(encoding="utf-8"))
            except Exception:
                pass
        metrics_doc.setdefault("runs", []).append(metrics_entry)
        metrics_doc.setdefault("latest", {})[track] = metrics_entry
        metrics_path.write_text(json.dumps(metrics_doc, indent=2), encoding="utf-8")
        print(f"Recorded recall track '{track}' in {metrics_path}")
    except Exception as e:
        print(f"Error saving baseline report: {e}")
        
    # Also attempt to patch scripts/ingest-quality/report.json if it exists and is not empty
    report_json_path = harness_dir / "report.json"
    if report_json_path.exists():
        try:
            content = report_json_path.read_text(encoding="utf-8")
            if content.strip():
                report_data = json.loads(content)
                if "summary" not in report_data:
                    report_data["summary"] = {}
                if "mem_score" not in report_data["summary"]:
                    report_data["summary"]["mem_score"] = {}
                ms = report_data["summary"]["mem_score"]
                ms["recall_at_5"] = recall_at_5
                if track == "rrf":
                    ms["recall_at_5_rrf"] = recall_at_5
                elif track == "rrf_strict":
                    ms["recall_at_5_rrf_strict"] = recall_at_5
                elif track == "golden":
                    ms["recall_at_5_golden"] = recall_at_5
                elif track == "qdrant":
                    ms["recall_at_5_qdrant"] = recall_at_5
                report_data["retrieval_evaluation"] = output_data["retrieval_evaluation"]
                report_json_path.write_text(json.dumps(report_data, indent=2), encoding="utf-8")
                print("Merged Recall@5 results into scripts/ingest-quality/report.json")
        except Exception as e:
            print(f"Non-fatal error updating report.json: {e}")
            
    # Print the top 10 lost facts
    if lost_facts_list:
        print("\n=== Top Lost Facts ===")
        for idx, lf in enumerate(lost_facts_list[:10]):
            print(f"[{idx+1}] File: {lf['file']}")
            print(f"    Query: \"{lf['query']}\"")
            print(f"    Lost Fact: \"{lf['fact']}\"")
            print(f"    Best snippet: \"{lf['best_snippet']}\"")
            
if __name__ == "__main__":
    main()

