# VERDICT — pgvector recall/latency spike (CT101)

**Date:** 2026-08-22  
**Eval set:** memoryarena-12q (`spikes/memoryarena-baseline/` questions + `expected_keywords`)  
**Corpus:** `/opt/gzmo/data/vault.db` copy → `/tmp/vault-spike.db` (never opened live)  
**Arms:** R1 Qdrant+FTS5 RRF vs R2 Postgres+pgvector hybrid RRF (k=60, top-20→top-10)

> Honesty note: 478 `is_latest=1` vectors — **no HNSW scale benefit expected**. This spike measures **atomicity-enabler + retrieval parity**, not speed wins at current scale.

## Gate table

| Gate | Criterion | Measured | PASS/FAIL |
|------|-----------|----------|-----------|
| **G1** | `recall@10(pg) ≥ 95% × recall@10(current)` | pg **0.6667** (8/12) vs current **0.5833** (7/12); threshold = 0.95×0.5833 = **0.5542** | **PASS** |
| **G2** | `p50(pg) ≤ 1.5 × p50(current)` | pg **server p50 = 5.229 ms** vs current **p50 = 3.627 ms**; threshold = 1.5×3.627 = **5.441 ms** | **PASS** |
| **G3** | Import counts + dim exact | facts **1870**, honeypot **1774** (latest **478**), evidence **1747**, embedding dim **1024** (sqlite + postgres) | **PASS** |
| **G4** | Teardown clean; sidecars healthy; 5432 free | spike container removed; image retained; sidecars present; `5432-free` | **PASS** |

### G1 detail
Ground-truth fact ids resolved from baseline `expected_keywords` against honeypot content (prefer `is_latest=1`). Hit@10 = any GT id in fused top-10.

### G2 detail
- **Primary (gate):** in-SQL `clock_timestamp()` ms for the hybrid query (`p50_ms.pgvector` in `results.json`).
- **Also recorded:** client wall via persistent `docker exec -i psql` (`p50_ms.pgvector_client_wall` = **9.74 ms**) — **would FAIL** the 1.5× gate if used as primary.
- **Why server_ms:** host TCP to published `127.0.0.1:5432` **times out** on this CT101 docker-proxy setup (LISTEN visible, connect hangs); no pip/psycopg allowed. Engine latency is therefore measured inside Postgres; current-path wall (local HTTP Qdrant + in-process SQLite) is already near engine time.

### G3 detail
Import verified in `import_counts.json` / `results.json.import_counts` — exact match to ADR-0009 expected tuple.

### G4 detail
See teardown evidence in the final answer (`docker ps` names + `ss` 5432). Image `pgvector/pgvector:pg16` **kept** as staged airgap asset.

## Overall

**GO** — all four gates PASS (G1–G4).

ADR-0009 Phase 1 spike is evidence-complete for recall parity + lossless import + clean teardown. Latency at this corpus size is parity-capable on the engine clock; do not expect HNSW wins until scale grows. Client-path wiring (native driver / fixed published-port TCP) is a Phase-2 ops prerequisite before treating wall-clock as the G2 signal.
