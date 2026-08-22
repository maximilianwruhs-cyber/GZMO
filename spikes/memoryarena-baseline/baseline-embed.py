#!/usr/bin/env python3
"""baseline-embed.py — MemoryArena embed-path baseline via the REAL router on VM200.

Re-runs the SAME 12 questions from questions.md through the production embed path:
  1. Embed each query via router http://192.168.31.110:8081/v1/embeddings (gzmo-embed, 1024-dim)
  2. Vector search in local Qdrant 127.0.0.1:6333 collection `honeypot` (685 points)
  3. Evaluate hit using the SAME expected_keywords as baseline.py
  4. Record {question, top_hit, hit, notes}

Writes BASELINE-EMBED.md + baseline-embed.json.
Does NOT modify original baseline.py / BASELINE.md.
"""

import json
import os
import sys
import urllib.request
import urllib.error

SPIKE_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(SPIKE_DIR))

EMBED_URL = "http://192.168.31.110:8081/v1/embeddings"
EMBED_MODEL = "gzmo-embed"
QDRANT_URL = "http://127.0.0.1:6333"
QDRANT_COLLECTION = "honeypot"

# Same 12 questions with expected keywords (UNCHANGED from baseline.py)
QUESTIONS = [
    {"id": "Q1", "question": "What is CT101's role in the GZMO architecture?",
     "category": "single-fact",
     "expected_keywords": ["CT101", "frozen reference", "living host", "reference living"]},
    {"id": "Q2", "question": "What is the dual-writer rule? Can two overnight writers run on the same vault?",
     "category": "single-fact",
     "expected_keywords": ["dual-writer", "two overnight writers", "never two", "single writer"]},
    {"id": "Q3", "question": "What is the Prime inference server and what port does it run on?",
     "category": "single-fact",
     "expected_keywords": ["Prime", "8000", "127.0.0.1:8000", "OpenAI-compatible"]},
    {"id": "Q4", "question": "What does Obolus do in the AOS energy routing chain?",
     "category": "single-fact",
     "expected_keywords": ["Obolus", "IPW", "inverse-propensity", "propensity scores"]},
    {"id": "Q5", "question": "What are the stages of the GZMO distillation pipeline?",
     "category": "single-fact",
     "expected_keywords": ["extract", "verify", "promote", "vault", "honeypot"]},
    {"id": "Q6", "question": "ADR-0003 originally said CT101 is frozen reference, then ADR-0005 amended this. What is the current state of CT101 vs workstation living-host placement?",
     "category": "multi-session",
     "expected_keywords": ["mutex", "claim", "living host", "CT101", "workstation", "promote-by-loop"]},
    {"id": "Q7", "question": "ADR-0003 said one writer, ADR-0004 said airgap, ADR-0007 said no lite SKU. What is the current product story?",
     "category": "multi-session",
     "expected_keywords": ["one product", "living Keep", "no lite", "attach", "one writer"]},
    {"id": "Q8", "question": "How does a TinyFolder drop reach the living vault? Trace the path through Brain Feed.",
     "category": "multi-session",
     "expected_keywords": ["tinyFolder", "Brain Feed", "distill", "honeypot", "enqueue", "session close"]},
    {"id": "Q9", "question": "If a beat-gate passes for one loop, what must happen before it lands in the living host?",
     "category": "multi-session",
     "expected_keywords": ["beat-gate", "PASS", "operator ack", "PROMOTE_ACK", "mutex", "promote-by-loop"]},
    {"id": "Q10", "question": "On 2026-07-15, a cutover happened. Was the vault imported from CT101 or fresh data-next?",
     "category": "multi-session",
     "expected_keywords": ["cutover", "2026-07-15", "fresh", "60k-fact", "no vault import"]},
    {"id": "Q11", "question": "What are the roles of Qdrant, Neo4j, and SQLite in GZMO?",
     "category": "single-fact",
     "expected_keywords": ["Qdrant", "vector", "Neo4j", "knowledge graph", "SQLite", "source of truth"]},
    {"id": "Q12", "question": "Is the Chaos Engine required for metabolism to function?",
     "category": "single-fact",
     "expected_keywords": ["chaos", "opt-in", "metabolism", "not depend"]},
]


def embed_query(text):
    """Embed text via the real router on VM200."""
    payload = json.dumps({"model": EMBED_MODEL, "input": text}).encode("utf-8")
    req = urllib.request.Request(EMBED_URL, data=payload, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        data = json.loads(resp.read())
    return data["data"][0]["embedding"]


def query_qdrant_vector(vector, limit=5):
    """Search Qdrant collection `honeypot` by vector similarity."""
    payload = json.dumps({
        "vector": vector,
        "limit": limit,
        "with_payload": True,
        "with_vectors": False
    }).encode("utf-8")
    url = f"{QDRANT_URL}/collections/{QDRANT_COLLECTION}/points/search"
    req = urllib.request.Request(url, data=payload, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=15) as resp:
        data = json.loads(resp.read())
    return data["result"]


def evaluate_hit(question_entry, hit_content):
    """Same evaluation as baseline.py: check if any expected keyword appears in the hit."""
    expected = question_entry.get("expected_keywords", [])
    hit_lower = hit_content.lower() if hit_content else ""
    found = [kw for kw in expected if kw.lower() in hit_lower]
    return len(found) >= 1, found


def run_embed_baseline():
    # Check Qdrant reachable
    qdrant_reachable = True
    try:
        req = urllib.request.Request(f"{QDRANT_URL}/collections/{QDRANT_COLLECTION}", headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=5) as resp:
            info = json.loads(resp.read())["result"]
            points = info["points_count"]
        print(f"Qdrant reachable. Collection: {QDRANT_COLLECTION} ({points} points)")
    except Exception as e:
        print(f"Qdrant NOT reachable: {e}")
        qdrant_reachable = False

    # Check embed router
    embed_reachable = True
    try:
        test_vec = embed_query("test")
        print(f"Embed router reachable. Dim: {len(test_vec)}")
    except Exception as e:
        print(f"Embed router NOT reachable: {e}")
        embed_reachable = False

    results = []
    for q in QUESTIONS:
        qid = q["id"]
        question = q["question"]
        print(f"\n--- {qid} ---")
        print(f"Q: {question[:80]}")

        if not embed_reachable or not qdrant_reachable:
            results.append({
                "id": qid, "question": question, "category": q["category"],
                "expected_keywords": q["expected_keywords"],
                "method": "embed-failed",
                "top_hit": "", "hit_yes": False,
                "notes": f"embed_reachable={embed_reachable} qdrant_reachable={qdrant_reachable}"
            })
            continue

        # Embed the question via the real router
        try:
            vector = embed_query(question)
        except Exception as e:
            results.append({
                "id": qid, "question": question, "category": q["category"],
                "expected_keywords": q["expected_keywords"],
                "method": "embed-error",
                "top_hit": "", "hit_yes": False,
                "notes": f"embed error: {e}"
            })
            continue

        # Vector search in Qdrant
        try:
            hits = query_qdrant_vector(vector, limit=5)
        except Exception as e:
            results.append({
                "id": qid, "question": question, "category": q["category"],
                "expected_keywords": q["expected_keywords"],
                "method": "qdrant-error",
                "top_hit": "", "hit_yes": False,
                "notes": f"qdrant search error: {e}"
            })
            continue

        if not hits:
            results.append({
                "id": qid, "question": question, "category": q["category"],
                "expected_keywords": q["expected_keywords"],
                "method": "embed",
                "top_hit": "", "hit_yes": False,
                "notes": "No results returned"
            })
            continue

        # Get top hit payload
        top = hits[0]
        payload = top.get("payload", {})
        # The payload contains the honeypot content
        content = ""
        for key in ["content", "text", "body", "chunk"]:
            if key in payload:
                content = str(payload[key])
                break
        if not content:
            content = json.dumps(payload)[:2000]

        # Also get score
        score = top.get("score", 0)

        # Truncate for display
        content_display = content[:500]

        # Evaluate hit using same keywords
        hit, found_kws = evaluate_hit(q, content)
        hit_kw_str = ", ".join(found_kws) if found_kws else "none"
        print(f"  top hit (score={score:.4f}): {content_display[:80]}")
        print(f"  hit={hit} keywords_found={hit_kw_str}")

        # Also check top-5 for any hit (broader evaluation)
        top5_hits = []
        for h in hits[:5]:
            h_content = ""
            h_payload = h.get("payload", {})
            for key in ["content", "text", "body", "chunk"]:
                if key in h_payload:
                    h_content = str(h_payload[key])
                    break
            if not h_content:
                h_content = json.dumps(h_payload)[:2000]
            h_hit, h_found = evaluate_hit(q, h_content)
            if h_hit:
                top5_hits.append({"score": h.get("score", 0), "found": h_found, "content": h_content[:200]})

        results.append({
            "id": qid,
            "question": question,
            "category": q["category"],
            "expected_keywords": q["expected_keywords"],
            "method": "embed",
            "top_hit": content_display,
            "top_score": score,
            "hit_yes": hit,
            "found_keywords": found_kws,
            "top5_any_hit": len(top5_hits) > 0,
            "top5_hits": top5_hits[:2],
            "notes": f"Found keywords: {hit_kw_str}; score={score:.4f}; top5_any_hit={len(top5_hits) > 0}"
        })

    return results, qdrant_reachable, embed_reachable


def write_outputs(results, qdrant_reachable, embed_reachable):
    hits = sum(1 for r in results if r.get("hit_yes"))
    top5_hits = sum(1 for r in results if r.get("top5_any_hit"))

    # JSON
    output = {
        "date": "2026-08-22",
        "method": "embed-path (router http://192.168.31.110:8081/v1/embeddings → Qdrant 127.0.0.1:6333 honeypot)",
        "embed_model": "gzmo-embed (Qwen3-Embedding-0.6B-Q8_0.gguf, 1024-dim)",
        "qdrant_collection": "honeypot",
        "qdrant_points": 685,
        "qdrant_reachable": qdrant_reachable,
        "embed_reachable": embed_reachable,
        "score_top1": f"{hits}/12",
        "score_top5": f"{top5_hits}/12",
        "results": results
    }
    with open(os.path.join(SPIKE_DIR, "baseline-embed.json"), "w") as f:
        json.dump(output, f, indent=2)

    # Markdown
    md = []
    md.append("# MemoryArena Embed-Path Baseline — REAL Router + Qdrant\n")
    md.append(f"**Date:** 2026-08-22\n")
    md.append(f"**Method:** Embed query via router `http://192.168.31.110:8081/v1/embeddings` (gzmo-embed, Qwen3-Embedding-0.6B-Q8_0, 1024-dim) → vector search in Qdrant `127.0.0.1:6333` collection `honeypot` (685 points)\n")
    md.append(f"**Qdrant reachable:** {qdrant_reachable}\n")
    md.append(f"**Embed router reachable:** {embed_reachable}\n")
    md.append(f"**Score (top-1):** {hits}/12 hits\n")
    md.append(f"**Score (top-5):** {top5_hits}/12 hits\n")
    md.append(f"**Comparison:** keyword-only baseline was 3/12\n\n")
    md.append("---\n\n")
    md.append("| ID | Category | Question | Method | Hit? (top-1) | Hit? (top-5) | Score | Top Hit | Keywords Found |\n")
    md.append("|---|---|---|---|---|---|---|---|---|\n")
    for r in results:
        top_hit = r.get("top_hit", "")[:80].replace("|", "\\|").replace("\n", " ")
        found = ", ".join(r.get("found_keywords", [])) or "none"
        md.append(f"| {r['id']} | {r['category']} | {r['question'][:60]} | {r['method']} | {'YES' if r.get('hit_yes') else 'NO'} | {'YES' if r.get('top5_any_hit') else 'NO'} | {r.get('top_score', 0):.4f} | {top_hit} | {found} |\n")
    md.append("\n---\n\n## Detailed results\n\n")
    for r in results:
        md.append(f"### {r['id']} — {r['category']}\n\n")
        md.append(f"**Question:** {r['question']}\n\n")
        md.append(f"**Method:** {r['method']}\n\n")
        md.append(f"**Hit (top-1):** {'YES' if r.get('hit_yes') else 'NO'}\n\n")
        md.append(f"**Hit (top-5):** {'YES' if r.get('top5_any_hit') else 'NO'}\n\n")
        md.append(f"**Top hit (score={r.get('top_score', 0):.4f}):** {r.get('top_hit', '')[:300]}\n\n")
        md.append(f"**Keywords found:** {', '.join(r.get('found_keywords', [])) or 'none'}\n\n")
        md.append(f"**Notes:** {r.get('notes', '')}\n\n")
        if r.get("top5_hits"):
            for i, h in enumerate(r["top5_hits"]):
                md.append(f"**Top-5 hit {i+1} (score={h['score']:.4f}):** {h['content'][:200]}\n\n")

    # Comparison paragraph
    md.append("\n---\n\n## Comparison: keyword-only vs embed-path\n\n")
    md.append(f"**Keyword-only baseline (BASELINE.md):** 3/12 hits (Q4 Obolus, Q8 TinyFolder/distill, Q12 chaos).\n\n")
    md.append(f"**Embed-path baseline (this file):** {hits}/12 top-1 hits, {top5_hits}/12 top-5 hits.\n\n")
    # Which questions flipped?
    baseline_hits = {"Q4", "Q8", "Q12"}
    embed_hits = {r["id"] for r in results if r.get("hit_yes")}
    embed_top5 = {r["id"] for r in results if r.get("top5_any_hit")}
    gained = embed_hits - baseline_hits
    lost = baseline_hits - embed_hits
    md.append(f"**Questions gained (keyword→embed):** {sorted(gained) if gained else 'none'}\n\n")
    md.append(f"**Questions lost (keyword→embed):** {sorted(lost) if lost else 'none'}\n\n")
    md.append(f"**Questions in top-5 but not top-1:** {sorted(embed_top5 - embed_hits) if (embed_top5 - embed_hits) else 'none'}\n\n")

    md.append("### Interpretation\n\n")
    md.append("The embed-path uses the REAL production embedding model (Qwen3-Embedding-0.6B) to embed each ")
    md.append("question and search Qdrant by cosine similarity, rather than the keyword-matching approach ")
    md.append("of the original baseline. This tests whether the semantic embed can surface the correct ")
    md.append("honeypot entries that keyword search missed.\n\n")
    md.append("If the embed-path scores similarly or worse than keyword-only, the system's weakness is in ")
    md.append("**retrieval ranking** — the correct content may not be in the top hit even with semantic search, ")
    md.append("suggesting the honeypot collection lacks the right document chunks or the embedding model ")
    md.append("doesn't discriminate well for these architectural questions.\n\n")
    md.append("If the embed-path scores better, the weakness was in the **keyword matching** of the original ")
    md.append("baseline — semantic similarity surfaces relevant chunks that exact keyword matching missed. ")
    md.append("This would imply the MemoryLake HOLD should focus on improving fact-coverage (what gets ")
    md.append("ingested into the honeypot) rather than retrieval ranking.\n")

    with open(os.path.join(SPIKE_DIR, "BASELINE-EMBED.md"), "w") as f:
        f.write("".join(md))

    print(f"\n=== Results: {hits}/12 top-1, {top5_hits}/12 top-5 (keyword baseline: 3/12) ===")


if __name__ == "__main__":
    results, qdrant_ok, embed_ok = run_embed_baseline()
    write_outputs(results, qdrant_ok, embed_ok)
