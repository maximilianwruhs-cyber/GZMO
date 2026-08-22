#!/usr/bin/env python3
"""recall_bench.py — memoryarena-12q: current (Qdrant+FTS5 RRF) vs pgvector hybrid RRF.

Arms:
  R1 current:  Qdrant honeypot top-20 + SQLite FTS5 honeypot_fts top-20 → RRF(k=60) → top-10
  R2 pgvector: vector <=> top-20 + tsvector/ts_rank top-20 → RRF(k=60) → top-10

Ground truth: reuse baseline-embed expected_keywords; resolve to honeypot fact ids
from the /tmp vault copy (is_latest=1 preferred). Hit@10 = any GT id in top-10.

Stdlib only (urllib, sqlite3, json). Postgres via docker exec psql.
"""

from __future__ import annotations

import json
import math
import os
import sqlite3
import subprocess
import sys
import time
import urllib.error
import urllib.request

SPIKE_DIR = os.path.dirname(os.path.abspath(__file__))
SPIKE_VAULT = "/tmp/vault-spike.db"
CONTAINER = "gzmo-pgvector-spike"

EMBED_URL = "http://192.168.31.110:8081/v1/embeddings"
EMBED_MODEL = "gzmo-embed"  # Qwen3-Embedding-0.6B via VM200 router
QDRANT_URL = "http://127.0.0.1:6333"
QDRANT_COLLECTION = "honeypot"
RRF_K = 60.0
TOP_ARM = 20
TOP_FINAL = 10

# Same 12 questions + expected_keywords as spikes/memoryarena-baseline/baseline-embed.py
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


def embed_query(text: str) -> list[float]:
    payload = json.dumps({"model": EMBED_MODEL, "input": text}).encode("utf-8")
    req = urllib.request.Request(
        EMBED_URL, data=payload, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        data = json.loads(resp.read())
    return data["data"][0]["embedding"]


def rrf_fuse(rank_lists: list[list[str]], k: float = RRF_K) -> list[str]:
    scores: dict[str, float] = {}
    for lst in rank_lists:
        for idx, doc_id in enumerate(lst):
            rank = idx + 1
            scores[doc_id] = scores.get(doc_id, 0.0) + 1.0 / (k + rank)
    return [i for i, _ in sorted(scores.items(), key=lambda kv: (-kv[1], kv[0]))]


def fts_match_query(query: str) -> str:
    words = [
        f'"{w.replace(chr(34), "")}"'
        for w in query.split()
        if len(w) >= 2
    ]
    return " OR ".join(words) if words else ""


def qdrant_search(vector: list[float], limit: int = TOP_ARM) -> list[str]:
    payload = json.dumps(
        {
            "vector": vector,
            "limit": limit,
            "with_payload": False,
            "with_vectors": False,
        }
    ).encode("utf-8")
    url = f"{QDRANT_URL}/collections/{QDRANT_COLLECTION}/points/search"
    req = urllib.request.Request(
        url, data=payload, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=15) as resp:
        data = json.loads(resp.read())
    ids = []
    for hit in data.get("result", []):
        hid = hit.get("id")
        if hid is not None:
            ids.append(str(hid))
    return ids


def sqlite_fts_search(conn: sqlite3.Connection, query: str, limit: int = TOP_ARM) -> list[str]:
    match_q = fts_match_query(query)
    if not match_q:
        return []
    sql = (
        "SELECT h.id FROM honeypot h "
        "JOIN honeypot_fts fts ON h.rowid = fts.rowid "
        "WHERE honeypot_fts MATCH ?1 AND h.is_latest = 1 "
        "ORDER BY rank LIMIT ?2"
    )
    try:
        rows = conn.execute(sql, (match_q, limit)).fetchall()
    except sqlite3.OperationalError:
        # Broad fallback already is OR; empty on parse errors
        return []
    return [r[0] for r in rows]


class PgSession:
    """Persistent `docker exec -i psql` so wall-ms measures query time, not container spawn."""

    def __init__(self) -> None:
        self.proc = subprocess.Popen(
            [
                "docker",
                "exec",
                "-i",
                CONTAINER,
                "psql",
                "-U",
                "postgres",
                "-v",
                "ON_ERROR_STOP=1",
                "-t",
                "-A",
                "-q",
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        # Warm handshake
        self.query("SELECT 1;")

    def query(self, sql: str) -> list[str]:
        if self.proc.stdin is None or self.proc.stdout is None:
            raise RuntimeError("psql session pipes closed")
        marker = f"__END_{time.time_ns()}__"
        # One statement + sentinel SELECT so we can read until marker without relying on EOF.
        payload = sql.rstrip().rstrip(";") + ";\n" + f"SELECT '{marker}';\n"
        self.proc.stdin.write(payload)
        self.proc.stdin.flush()
        lines: list[str] = []
        while True:
            line = self.proc.stdout.readline()
            if line == "":
                err = self.proc.stderr.read() if self.proc.stderr else ""
                raise RuntimeError(f"psql EOF unexpectedly: {err}")
            line = line.rstrip("\n")
            if line == marker:
                break
            if line != "":
                lines.append(line)
        return lines

    def close(self) -> None:
        try:
            if self.proc.stdin:
                self.proc.stdin.write("\\q\n")
                self.proc.stdin.flush()
                self.proc.stdin.close()
        except Exception:
            pass
        try:
            self.proc.wait(timeout=5)
        except Exception:
            self.proc.kill()


def pg_hybrid_search(
    pg: PgSession, question: str, vector: list[float], limit: int = TOP_ARM
) -> tuple[list[str], list[str], float]:
    """One SQL round-trip: vector top-N + tsvector top-N.

    Returns (vec_ids, ts_ids, server_ms) where server_ms is measured inside Postgres
    via clock_timestamp() — host→5432 TCP times out on CT101 (docker-proxy), so
    client wall includes docker-exec pipe floor and is reported separately.
    """
    lit = "[" + ",".join(f"{x:.8g}" for x in vector) + "]"
    q = question.replace("'", "''")
    sql = f"""
WITH t0 AS (SELECT clock_timestamp() AS t),
params AS (
  SELECT '{lit}'::vector AS qv,
         plainto_tsquery('english', '{q}') AS tsq
),
vec AS (
  SELECT id,
         ROW_NUMBER() OVER (ORDER BY embedding <=> (SELECT qv FROM params)) AS rnk
  FROM honeypot
  WHERE is_latest = 1 AND embedding IS NOT NULL
  ORDER BY embedding <=> (SELECT qv FROM params)
  LIMIT {int(limit)}
),
ts AS (
  SELECT id,
         ROW_NUMBER() OVER (
           ORDER BY ts_rank(content_norm, (SELECT tsq FROM params)) DESC
         ) AS rnk
  FROM honeypot
  WHERE is_latest = 1 AND content_norm @@ (SELECT tsq FROM params)
  ORDER BY ts_rank(content_norm, (SELECT tsq FROM params)) DESC
  LIMIT {int(limit)}
),
hits AS (
  SELECT 'v'::text AS arm, id, rnk FROM vec
  UNION ALL
  SELECT 't'::text AS arm, id, rnk FROM ts
)
SELECT arm, id,
       (EXTRACT(EPOCH FROM (clock_timestamp() - (SELECT t FROM t0))) * 1000)::float8
         AS server_ms
FROM hits
ORDER BY arm, rnk
"""
    rows = pg.query(sql)
    vec_ids: list[str] = []
    ts_ids: list[str] = []
    server_ms = 0.0
    for row in rows:
        parts = row.split("|")
        if len(parts) < 3:
            continue
        arm, doc_id, sms = parts[0], parts[1], parts[2]
        try:
            server_ms = float(sms)
        except ValueError:
            pass
        if arm == "v":
            vec_ids.append(doc_id)
        elif arm == "t":
            ts_ids.append(doc_id)
    if not ts_ids:
        tokens = [t.strip(".,?!\"'()[]") for t in question.split() if len(t) >= 4][:6]
        if tokens:
            clauses = " OR ".join(
                f"content ILIKE '%{t.replace(chr(39), chr(39) + chr(39))}%'" for t in tokens
            )
            t0 = time.perf_counter()
            ts_ids = pg.query(
                f"SELECT id FROM honeypot WHERE is_latest = 1 AND ({clauses}) "
                f"LIMIT {int(limit)}"
            )
            # fallback path: approximate server time from second round-trip wall
            server_ms = max(server_ms, (time.perf_counter() - t0) * 1000.0)
    return vec_ids, ts_ids, server_ms


def resolve_ground_truth(conn: sqlite3.Connection, q: dict) -> list[str]:
    """Map baseline expected_keywords → honeypot fact ids (prefer is_latest=1)."""
    kws = q["expected_keywords"]
    rows = conn.execute(
        "SELECT id, content, is_latest FROM honeypot WHERE content IS NOT NULL"
    ).fetchall()
    scored: list[tuple[int, int, str]] = []
    for hid, content, is_latest in rows:
        cl = (content or "").lower()
        hits = sum(1 for kw in kws if kw.lower() in cl)
        if hits <= 0:
            continue
        # Prefer more keyword hits, then latest
        scored.append((hits, 1 if is_latest else 0, hid))
    scored.sort(key=lambda t: (-t[0], -t[1], t[2]))
    # Keep top matches with at least max(1, ceil(n_kw/3)) hits to avoid noise
    min_hits = max(1, math.ceil(len(kws) / 3))
    ids = [hid for hits, _, hid in scored if hits >= min_hits][:5]
    if not ids and scored:
        ids = [scored[0][2]]
    return ids


def hit_at_k(retrieved: list[str], gt: list[str], k: int = TOP_FINAL) -> bool:
    if not gt:
        return False
    top = set(retrieved[:k])
    return any(g in top for g in gt)


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    s = sorted(values)
    # nearest-rank
    idx = min(len(s) - 1, max(0, int(math.ceil(p / 100.0 * len(s)) - 1)))
    return s[idx]


def arm_current(conn: sqlite3.Connection, question: str, vector: list[float]) -> tuple[list[str], float]:
    t0 = time.perf_counter()
    vec_ids = qdrant_search(vector, TOP_ARM)
    fts_ids = sqlite_fts_search(conn, question, TOP_ARM)
    fused = rrf_fuse([vec_ids, fts_ids])[:TOP_FINAL]
    ms = (time.perf_counter() - t0) * 1000.0
    return fused, ms


def arm_pgvector(
    pg: PgSession, question: str, vector: list[float]
) -> tuple[list[str], float, float]:
    t0 = time.perf_counter()
    vec_ids, ts_ids, server_ms = pg_hybrid_search(pg, question, vector, TOP_ARM)
    fused = rrf_fuse([vec_ids, ts_ids])[:TOP_FINAL]
    wall_ms = (time.perf_counter() - t0) * 1000.0
    return fused, wall_ms, server_ms


def load_import_counts() -> dict:
    path = os.path.join(SPIKE_DIR, "import_counts.json")
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    return {}


def main() -> None:
    if not os.path.exists(SPIKE_VAULT):
        print(f"FATAL: missing {SPIKE_VAULT} — run import_vault.py first", file=sys.stderr)
        sys.exit(1)

    # Smoke checks
    try:
        v = embed_query("ping")
        print(f"embed ok dim={len(v)}")
    except Exception as e:
        print(f"FATAL: embed router unreachable: {e}", file=sys.stderr)
        sys.exit(1)
    try:
        req = urllib.request.Request(f"{QDRANT_URL}/collections/{QDRANT_COLLECTION}")
        with urllib.request.urlopen(req, timeout=5) as resp:
            info = json.loads(resp.read())["result"]
        print(f"qdrant ok points={info.get('points_count')}")
    except Exception as e:
        print(f"FATAL: qdrant unreachable: {e}", file=sys.stderr)
        sys.exit(1)

    conn = sqlite3.connect(f"file:{SPIKE_VAULT}?mode=ro", uri=True)
    pg = PgSession()

    per_q = []
    lat_cur: list[float] = []
    lat_pg_wall: list[float] = []
    lat_pg_server: list[float] = []
    hits_cur = 0
    hits_pg = 0

    try:
        for q in QUESTIONS:
            qid = q["id"]
            print(f"\n=== {qid} ===")
            gt = resolve_ground_truth(conn, q)
            print(f"  gt_ids ({len(gt)}): {gt[:3]}{'...' if len(gt) > 3 else ''}")

            vector = embed_query(q["question"])

            cur_ids, cur_ms = arm_current(conn, q["question"], vector)
            pg_ids, pg_wall, pg_server = arm_pgvector(pg, q["question"], vector)

            cur_hit = hit_at_k(cur_ids, gt)
            pg_hit = hit_at_k(pg_ids, gt)
            hits_cur += int(cur_hit)
            hits_pg += int(pg_hit)
            lat_cur.append(cur_ms)
            lat_pg_wall.append(pg_wall)
            lat_pg_server.append(pg_server)

            print(
                f"  current hit={cur_hit} {cur_ms:.1f}ms top3={cur_ids[:3]} | "
                f"pgvector hit={pg_hit} wall={pg_wall:.1f}ms server={pg_server:.2f}ms "
                f"top3={pg_ids[:3]}"
            )
            per_q.append(
                {
                    "id": qid,
                    "question": q["question"],
                    "category": q["category"],
                    "expected_keywords": q["expected_keywords"],
                    "ground_truth_ids": gt,
                    "current": {
                        "top10_ids": cur_ids,
                        "hit_at_10": cur_hit,
                        "wall_ms": round(cur_ms, 3),
                    },
                    "pgvector": {
                        "top10_ids": pg_ids,
                        "hit_at_10": pg_hit,
                        "wall_ms": round(pg_wall, 3),
                        "server_ms": round(pg_server, 3),
                    },
                }
            )
    finally:
        pg.close()
        conn.close()
    n = len(QUESTIONS)
    # G2 primary metric: in-SQL server_ms (engine). Client wall is reported but
    # inflated by docker-exec transport (host TCP to published 5432 times out).
    results = {
        "date": "2026-08-22",
        "n_questions": n,
        "rrf_k": RRF_K,
        "top_arm": TOP_ARM,
        "top_final": TOP_FINAL,
        "embed": {"url": EMBED_URL, "model": EMBED_MODEL, "dim": 1024},
        "per_question": per_q,
        "recall_at_10": {
            "current": hits_cur / n,
            "pgvector": hits_pg / n,
            "current_hits": hits_cur,
            "pgvector_hits": hits_pg,
        },
        "p50_ms": {
            "current": round(percentile(lat_cur, 50), 3),
            "pgvector": round(percentile(lat_pg_server, 50), 3),
            "pgvector_client_wall": round(percentile(lat_pg_wall, 50), 3),
        },
        "p95_ms": {
            "current": round(percentile(lat_cur, 95), 3),
            "pgvector": round(percentile(lat_pg_server, 95), 3),
            "pgvector_client_wall": round(percentile(lat_pg_wall, 95), 3),
        },
        "latency_note": (
            "p50_ms.pgvector / p95_ms.pgvector are in-SQL clock_timestamp() ms. "
            "Client wall (docker exec -i psql) is higher because host TCP to "
            "127.0.0.1:5432 times out on this CT101 docker-proxy setup; no pip/psycopg."
        ),
        "import_counts": load_import_counts(),
        "note": "478 latest vectors — no HNSW scale benefit expected; measures atomicity + parity.",
    }

    out = os.path.join(SPIKE_DIR, "results.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print("\n=== SUMMARY ===")
    print(json.dumps({k: results[k] for k in ("recall_at_10", "p50_ms", "p95_ms")}, indent=2))
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
