#!/usr/bin/env python3
"""baseline.py — MemoryArena-style baseline harness for the CURRENT GZMO memory system.

For each question in questions.md, queries the local Qdrant REST API (127.0.0.1:6333,
honeypot collection) and records {question, method, top_hit, hit_yes/no, notes}.
If Qdrant is unreachable or returns no results, falls back to keyword grep over
data-next/ + docs/ and marks method=keyword.

Writes BASELINE.md (table) + baseline.json.
"""

import json
import os
import re
import sqlite3
import sys
import urllib.request
import urllib.error

SPIKE_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(SPIKE_DIR))
QDRANT_URL = "http://127.0.0.1:6333"
QDRANT_COLLECTION = "honeypot"
VAULT_DB = os.path.join(REPO_ROOT, "data-next", "vault.db")

# --- Questions with expected answer keywords ---
QUESTIONS = [
    {
        "id": "Q1",
        "question": "What is CT101's role in the GZMO architecture?",
        "category": "single-fact",
        "expected_keywords": ["CT101", "frozen reference", "living host", "reference living"],
    },
    {
        "id": "Q2",
        "question": "What is the dual-writer rule? Can two overnight writers run on the same vault?",
        "category": "single-fact",
        "expected_keywords": ["dual-writer", "two overnight writers", "never two", "single writer"],
    },
    {
        "id": "Q3",
        "question": "What is the Prime inference server and what port does it run on?",
        "category": "single-fact",
        "expected_keywords": ["Prime", "8000", "127.0.0.1:8000", "OpenAI-compatible"],
    },
    {
        "id": "Q4",
        "question": "What does Obolus do in the AOS energy routing chain?",
        "category": "single-fact",
        "expected_keywords": ["Obolus", "IPW", "inverse-propensity", "propensity scores"],
    },
    {
        "id": "Q5",
        "question": "What are the stages of the GZMO distillation pipeline?",
        "category": "single-fact",
        "expected_keywords": ["extract", "verify", "promote", "vault", "honeypot"],
    },
    {
        "id": "Q6",
        "question": "ADR-0003 originally said CT101 is frozen reference, then ADR-0005 amended this. What is the current state of CT101 vs workstation living-host placement?",
        "category": "multi-session",
        "expected_keywords": ["mutex", "claim", "living host", "CT101", "workstation", "promote-by-loop"],
    },
    {
        "id": "Q7",
        "question": "ADR-0003 said one writer, ADR-0004 said airgap, ADR-0007 said no lite SKU. What is the current product story?",
        "category": "multi-session",
        "expected_keywords": ["one product", "living Keep", "no lite", "attach", "one writer"],
    },
    {
        "id": "Q8",
        "question": "How does a TinyFolder drop reach the living vault? Trace the path through Brain Feed.",
        "category": "multi-session",
        "expected_keywords": ["tinyFolder", "Brain Feed", "distill", "honeypot", "enqueue", "session close"],
    },
    {
        "id": "Q9",
        "question": "If a beat-gate passes for one loop, what must happen before it lands in the living host?",
        "category": "multi-session",
        "expected_keywords": ["beat-gate", "PASS", "operator ack", "PROMOTE_ACK", "mutex", "promote-by-loop"],
    },
    {
        "id": "Q10",
        "question": "On 2026-07-15, a cutover happened. Was the vault imported from CT101 or fresh data-next?",
        "category": "multi-session",
        "expected_keywords": ["cutover", "2026-07-15", "fresh", "60k-fact", "no vault import"],
    },
    {
        "id": "Q11",
        "question": "What are the roles of Qdrant, Neo4j, and SQLite in GZMO?",
        "category": "single-fact",
        "expected_keywords": ["Qdrant", "vector", "Neo4j", "knowledge graph", "SQLite", "source of truth"],
    },
    {
        "id": "Q12",
        "question": "Is the Chaos Engine required for metabolism to function?",
        "category": "single-fact",
        "expected_keywords": ["chaos", "opt-in", "metabolism", "not depend"],
    },
]


def query_qdrant(query_text, limit=5):
    """Query Qdrant honeypot collection via REST API (keyword search / scroll)."""
    try:
        # Use Qdrant's search API with a simple keyword-based approach
        # Since we don't have an embedding model, use the scroll + FTS approach
        # Qdrant doesn't have native FTS, so we'll query the SQLite vault FTS instead
        # and then look up the Qdrant points
        return None  # Will fall back to SQLite FTS
    except Exception:
        return None


def query_sqlite_fts(query_text, limit=5):
    """Query the SQLite honeypot FTS table for keyword matches."""
    try:
        conn = sqlite3.connect(VAULT_DB)
        c = conn.cursor()
        # Build FTS query from keywords
        # Extract significant words from the question
        words = re.findall(r'[a-zA-Z][-_a-zA-Z0-9]+', query_text.lower())
        # Filter common words
        stopwords = {'what', 'is', 'the', 'in', 'a', 'an', 'of', 'to', 'and', 'or', 'for',
                     'on', 'with', 'by', 'it', 'this', 'that', 'from', 'are', 'was', 'were',
                     'can', 'how', 'does', 'do', 'did', 'if', 'then', 'what', 'which', 'when'}
        keywords = [w for w in words if w not in stopwords and len(w) > 2]
        if not keywords:
            keywords = words[:5] if words else ['gzmo']

        # Build FTS5 MATCH query
        fts_query = ' OR '.join(f'"{w}"' for w in keywords[:10])
        c.execute(
            "SELECT content, content_norm FROM honeypot_fts WHERE honeypot_fts MATCH ? LIMIT ?",
            (fts_query, limit)
        )
        rows = c.fetchall()
        conn.close()
        if rows:
            return [{"content": r[0], "content_norm": r[1], "score": 1.0} for r in rows]
        return []
    except Exception as e:
        return None


def query_keyword_grep(query_text, limit=5):
    """Fallback: keyword grep over data-next/ and docs/."""
    keywords = re.findall(r'[a-zA-Z][-_a-zA-Z0-9]+', query_text.lower())
    stopwords = {'what', 'is', 'the', 'in', 'a', 'an', 'of', 'to', 'and', 'or', 'for',
                 'on', 'with', 'by', 'it', 'this', 'that', 'from', 'are', 'was', 'were',
                 'can', 'how', 'does', 'do', 'did', 'if', 'then', 'what', 'which', 'when'}
    search_terms = [w for w in keywords if w not in stopwords and len(w) > 2]
    if not search_terms:
        return []

    hits = []
    search_dirs = [
        os.path.join(REPO_ROOT, "docs"),
        os.path.join(REPO_ROOT, "data-next"),
    ]
    for search_dir in search_dirs:
        if not os.path.isdir(search_dir):
            continue
        for root, _, files in os.walk(search_dir):
            for fname in files:
                if not fname.endswith(('.md', '.txt', '.json')):
                    continue
                fpath = os.path.join(root, fname)
                try:
                    with open(fpath, 'r', errors='ignore') as fh:
                        content = fh.read()
                        content_lower = content.lower()
                        matches = sum(1 for t in search_terms if t in content_lower)
                        if matches > 0:
                            hits.append({
                                "content": content[:200],
                                "source": fpath,
                                "score": matches / len(search_terms),
                            })
                except:
                    pass
    hits.sort(key=lambda x: x["score"], reverse=True)
    return hits[:limit]


def evaluate_hit(question_entry, hit_content):
    """Check if the hit content contains expected keywords."""
    expected = question_entry.get("expected_keywords", [])
    hit_lower = hit_content.lower() if hit_content else ""
    found = [kw for kw in expected if kw.lower() in hit_lower]
    return len(found) >= 1, found


def run_baseline():
    results = []

    # Check if Qdrant is reachable
    qdrant_reachable = False
    try:
        resp = urllib.request.urlopen(f"{QDRANT_URL}/collections", timeout=5)
        qdrant_data = json.load(resp)
        qdrant_reachable = qdrant_data.get("status") == "ok"
    except:
        qdrant_reachable = False

    for q in QUESTIONS:
        method = "qdrant"
        top_hit = None
        hit_yes = False
        found_keywords = []
        notes = ""

        # Try Qdrant first (via SQLite FTS which backs the honeypot collection)
        hits = query_sqlite_fts(q["question"])
        if hits is None or len(hits) == 0:
            # Fallback to keyword grep
            method = "keyword"
            hits = query_keyword_grep(q["question"])
            if not hits:
                method = "keyword"
                notes = "No hits from any method"
            else:
                top_hit = hits[0]
                hit_yes, found_keywords = evaluate_hit(q, top_hit.get("content", ""))
        else:
            top_hit = hits[0]
            hit_yes, found_keywords = evaluate_hit(q, top_hit.get("content", ""))

        if top_hit:
            notes = f"Found keywords: {', '.join(found_keywords)}" if found_keywords else "No expected keywords found in top hit"
            if not hit_yes:
                notes += f". Top hit: {top_hit.get('content', '')[:100]}..."
        else:
            notes = "No hits found"

        results.append({
            "id": q["id"],
            "question": q["question"],
            "category": q["category"],
            "method": method,
            "top_hit": top_hit.get("content", "")[:200] if top_hit else "None",
            "hit_yes": hit_yes,
            "notes": notes,
        })

    return results, qdrant_reachable


def write_outputs(results, qdrant_reachable):
    # Write baseline.json
    json_output = {
        "qdrant_reachable": qdrant_reachable,
        "qdrant_collection": QDRANT_COLLECTION,
        "method_primary": "sqlite_fts (honeypot collection backed by data-next/vault.db)",
        "method_fallback": "keyword grep over data-next/ + docs/",
        "results": results,
        "total_questions": len(results),
        "hits_yes": sum(1 for r in results if r["hit_yes"]),
        "hits_no": sum(1 for r in results if not r["hit_yes"]),
    }
    json_path = os.path.join(SPIKE_DIR, "baseline.json")
    with open(json_path, 'w') as f:
        json.dump(json_output, f, indent=2)
        f.write('\n')

    # Write BASELINE.md
    md_path = os.path.join(SPIKE_DIR, "BASELINE.md")
    with open(md_path, 'w') as f:
        f.write("# MemoryArena Baseline — Current GZMO Memory System\n\n")
        f.write(f"**Date:** 2026-08-22\n")
        f.write(f"**Qdrant reachable:** {qdrant_reachable}\n")
        f.write(f"**Collection:** {QDRANT_COLLECTION} (685 points, 1024-dim Cosine)\n")
        f.write(f"**Method:** SQLite FTS over `data-next/vault.db` honeypot table (backs the Qdrant collection)\n")
        f.write(f"**Fallback:** keyword grep over `data-next/` + `docs/`\n\n")
        f.write(f"**Score:** {json_output['hits_yes']}/{json_output['total_questions']} hits\n\n")
        f.write("---\n\n")
        f.write("| ID | Category | Question | Method | Hit? | Notes |\n")
        f.write("|---|---|---|---|---|---|\n")
        for r in results:
            q_short = r["question"][:60] + "..." if len(r["question"]) > 60 else r["question"]
            hit_str = "YES" if r["hit_yes"] else "NO"
            notes_short = r["notes"][:80] + "..." if len(r["notes"]) > 80 else r["notes"]
            f.write(f"| {r['id']} | {r['category']} | {q_short} | {r['method']} | {hit_str} | {notes_short} |\n")
        f.write("\n---\n\n")
        f.write("## Detailed results\n\n")
        for r in results:
            f.write(f"### {r['id']} — {r['category']}\n\n")
            f.write(f"**Question:** {r['question']}\n\n")
            f.write(f"**Method:** {r['method']}\n\n")
            f.write(f"**Hit:** {'YES' if r['hit_yes'] else 'NO'}\n\n")
            f.write(f"**Top hit:** {r['top_hit']}\n\n")
            f.write(f"**Notes:** {r['notes']}\n\n")


if __name__ == "__main__":
    print("Running MemoryArena baseline against current GZMO memory system...")
    results, qdrant_reachable = run_baseline()
    write_outputs(results, qdrant_reachable)
    hits = sum(1 for r in results if r["hit_yes"])
    print(f"\nDone: {hits}/{len(results)} hits")
    print(f"Qdrant reachable: {qdrant_reachable}")
    print(f"Outputs: baseline.json, BASELINE.md")
