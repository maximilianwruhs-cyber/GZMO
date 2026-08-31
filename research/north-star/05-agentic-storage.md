# 05 — Agentic durable-state architecture

**Research date:** 2026-08-31  
**Scope:** Compare multi-store status quo vs single-store candidates for one-writer, air-gapped, bi-temporal, evidence-grounded hybrid memory. No migration; final store choice deferred to ticket 10.

## Executive finding

At measured living scale (~0.5k–3k active honeypot rows, embeddings already in SQLite BLOBs), **Qdrant and Neo4j are not earning their operational weight as durable planes**. They are *derived* surfaces that introduce documented split-brain and same-night lag, while local flat vector scan + FTS5 already implement hybrid recall inside the SQLite SoT.

**Pareto frontier for GZMO’s one-node ladder (not a final pick):**

| Profile | Durable SoT | Derived indexes | Ephemeral | Why on the frontier |
|---------|-------------|-----------------|-----------|---------------------|
| **Minimum / low-end** | Embedded SQLite (WAL) holding facts, evidence, bi-temporal supersession, FTS, BLOB vectors, durable queues, audit/candidate ledgers | Optional none; graph = SQL tables/CTEs | Process memory or optional Redis-class cache | Lowest RAM/ops; already production SoT; crash recovery and backup are file-local |
| **Release reference** | Same SQLite single-file SoT *or* local PostgreSQL 16 + pgvector as sole process-owned store | FTS + vectors co-located; graph as SQL projections | Optional local cache only | Single ACID writer; no mirror drift; proven hybrid path |
| **Forge / large corpus** | PostgreSQL 16 + pgvector (HNSW when N justifies) | Same; optional rebuildable ANN only if measured | Cache only | Scale-up, PITR, concurrent readers; still one physical node |

**Reject as durable architecture (keep only if an explicit capability profile opts into derived acceleration after SoT is correct):** status-quo SQLite + Qdrant mirror + Neo4j KG + Redis-as-queue. Redis in the living appliance is configured with RDB save disabled and is already treated as degrade-to-memory/file — it must not own durable claims.

**Depth / deletion test:** `VaultBackend` + `QdrantVault` fail the deletion test today (one real adapter; skeleton methods bail). Do **not** invent a generic repository seam until two *required* ladder adapters exist. Prefer one deep module (`promote` / `recall` / `claim` / `audit_append`) with a single concrete store per installation.

ADR-0009/0010 remain **evidence**, not binding product law: spike showed pgvector can match 3-way RRF quality at small N; they do **not** prove Postgres is required on minimum hardware.

## Decision-relevant facts

### 1. Correct the stale schema and count baseline

| Claim source | Stale claim | Corrected (code / later docs) |
|--------------|-------------|-------------------------------|
| `docs/INFRASTRUCTURE_MAP.md` §3 | `vault.db` schema **v7** | Code migrations through **`PRAGMA user_version = 10`**: utility_score (v8/v9 repair), bi-temporal `valid_from`/`valid_to`, `gate_event`, `failure_cases` (`gzmo-core/src/memory/vault.rs`) |
| ADR-0009 (2026-08-22) | schema v7; 1870 vault / 1774 honeypot / **478** latest / 1747 evidence | Same-day frozen backup (ADR-0010): **1881** / **1785** honeypot / **487** latest / **1758** evidence; 37 MB; integrity_check ok |
| Older CT101 system docs | ~60k vault / ~37k latest / ~24k Qdrant | **Doc-dated snapshots** from other epochs; not the 2026-08-22 living small-corpus freeze |
| `docs/ct101-systems/50-memory-data-plane/vault.md` | migrations “through v5”; `todo!()` on QdrantVault | Code: v10; `QdrantVault` uses `bail!` / `not_implemented` (essential-living-diagnosis/01) |
| Live felt-use depth (2026-08-30) | — | Another census: latest **3005**, utility_positive **78** (`research/felt-use-shipped-vs-opportunity.md`) — scale still “flat-scan trivial” |

**Authoritative living contracts (observed, 2026-08 diagnosis):**  
`research/essential-living-diagnosis/01-memory-plane.md`, `04-wiki-kg-qdrant.md`.

### 2. Layer separation (required vocabulary)

| Layer | Owns | Must survive crash | Examples in GZMO today |
|-------|------|--------------------|------------------------|
| **Durable SoT** | Facts, quarantine, evidence quotes, supersession flags, gate events, failure cases, dedup keys | Yes | `semantic_vault`, `quarantine_vault`, `honeypot`, `evidence`, `failure_cases`, `distill_dedup`, `ingest_dedup` in `vault.db` |
| **Derived indexes** | Rebuildable search/graph acceleration | Prefer yes, but always rebuildable from SoT | `honeypot_fts` / `evidence_fts`; Qdrant `honeypot` collection; Neo4j entities/relations; in-vault `knowledge_core` export table |
| **Ephemeral cache** | Session scratch, embed HTTP cache | No | Redis scratch / in-memory fallback (`scratch.rs`); Redis embed cache |
| **Immutable audit / ledgers** | Append-only evaluation, candidates, promotions, energy, operator-signed events | Yes (append-only semantics) | Partial: `failure_cases`, episodic `memory/*.md`, Synapse events; **not** a unified candidate/eval ledger yet (north-star gap) |

Wiki markdown and sibling `knowledge_core.db` (profile schema) are **emit/derived**, not SoT (`01-memory-plane` dual-schema gap).

### 3. Option A — Status quo: SQLite + Qdrant + Neo4j + Redis

**Topology (docs + compose):** workstation/daemon owns `vault.db`; LXC/docker sidecars: Qdrant `:6333`, Neo4j `:7687`, Redis `:6379` (`docs/INFRASTRUCTURE_MAP.md`; `deploy/living-appliance/docker-compose.yml`).

**One-writer transactions (SoT only):**  
`promote_truths_with_origin` → `BEGIN IMMEDIATE` + per-truth `SAVEPOINT promote_one` + lifecycle supersede (`vault.rs`, `lifecycle.rs`). SQLite WAL: single writer, concurrent readers ([SQLite WAL](https://www.sqlite.org/wal.html)).

**Bi-temporal supersession:**  
`is_latest`, `valid_from`/`valid_to`, `gate_event='supersede'`, `supersedes_id`, `honeypot_as_of` / chain APIs (v10 + lifecycle). **Atomic only inside SQLite.** Qdrant upsert is *after* commit and often nightly; supersede does not atomically delete vector points (`INFRASTRUCTURE_MAP` L308; `04-wiki-kg-qdrant`).

**Evidence / provenance:**  
`evidence` + FTS + embeddings in SoT. Neo4j observations can exist **without** matching evidence rows (documented split-brain).

**Vector + FTS fusion:**  
`recall_rrf`: honeypot FTS, evidence FTS, graph-or-keyword, **interleaved Qdrant + local `search_with_decay`**, evidence vectors → RRF (`k=60`) → diversify → optional rerank → utility select. Local path already flat-scans BLOB embeddings with cosine + keyword blend (`search_with_decay`).

**Graph projections:**  
Neo4j via MCP (`kg_extract` / `promote_to_kg`); not transactional with vault. Appliance defaults **~1.5 GiB heap + 1.5 GiB pagecache** for Neo4j alone (`docker-compose.yml`) — heavy for minimum ladder.

**Queue claims:**  
Redis list LPUSH/BRPOP for distill, with **file fallback** directory; scratch degrades to memory (`scratch.rs`). Appliance Redis: `redis-server --maxmemory 1gb --maxmemory-policy allkeys-lru --save ""` — **explicitly non-durable RDB**.

**Backups / crash recovery:**  
SQLite: WAL + online backup API / `VACUUM INTO` / `.backup` ([backup API](https://www.sqlite.org/backup.html)); ADR-0010 freeze used WAL-safe `.backup` + `PRAGMA integrity_check`. Multi-store restore requires coordinating vault + Qdrant + Neo4j + Redis state — high operator burden; nuclear purge scripts encode that coupling.

**ADR-0009 measured drift (2026-08-22 18:25):** latest honeypot **478** vs Qdrant **433** (**45-point** divergence). Health probe ratios 0.55 critical / 0.70 warn (`health.rs`) conflate missing embeddings with true mirror lag.

**Spike evidence (ADR-0010):** pgvector one-box spike **GO** — `memoryarena-12q` recall@10 **8/12** parity with 3-way RRF; in-SQL p50 **5.2 ms**; lossless import; clean teardown. At this N, HNSW is not the justification — atomicity and fewer moving parts are.

**Rust drivers today:** `rusqlite` 0.32 bundled + `r2d2`/`r2d2_sqlite`; `redis` 0.27 async; Qdrant via HTTP (`reqwest`); Neo4j via external MCP process — not in-process graph driver.

**Operational weight:** 3 stateful sidecars + sync cron + drift probes + Python sync script + dual instance gates (`GZMO_INSTANCE=next` incremental Qdrant only). Fails air-gap **minimum** simplicity and one-node resource floors.

### 4. Option B — PostgreSQL 16 + pgvector single-store

**Primary sources:** [pgvector](https://github.com/pgvector/pgvector) v0.8.x (exact NN by default; HNSW/IVFFlat optional; cosine `<=>`; ACID + JOINs + PITR claim on project README); [PostgreSQL 16 continuous archiving / PITR](https://www.postgresql.org/docs/16/continuous-archiving.html); ADR-0009/0010 schema sketch.

**Fits:**

- One-writer promote + supersede + embedding + FTS (`tsvector`/GIN) + evidence FK in **one transaction** → drift eliminated by construction.
- Graph as `entities`/`relations` SQL projections + CTEs (ADR-0010) — deletes Neo4j requirement for agent-memory scale.
- Durable `ingest_queue` with `pending|claimed|done|quarantined` + `SKIP LOCKED` / advisory locks (prototype intent) — replaces Redis durable-queue temptation.
- Candidate / evaluation / audit / energy ledgers as ordinary tables (append-heavy).
- Backup: `pg_dump` logical and/or base backup + WAL archive for true PITR (ops complexity higher than SQLite file copy).
- Scale-up: HNSW when vector count leaves “flat trivial”; concurrent readers without SQLite write hiccups.
- Hybrid retrieval can collapse multi-roundtrip RRF into SQL (ADR-0009 motivation); spike p50 5.2 ms in-SQL.

**Costs:**

- Always-on postmaster, shared_buffers, extension install, air-gapped image staging (`pgvector/pgvector:pg16`).
- Rust cutover surface: living code is synchronous `rusqlite` throughout `vault.rs` / `honeypot.rs`; would need `sqlx`/`tokio-postgres`/etc. — large migration if in-place (ADR-0009 risk). Clean-sheet prototype path (ADR-0010) isolates that risk.
- Minimum hardware: higher RAM floor than embedded SQLite; may be wrong for “tiny” capability profile.
- Blast radius: one store holds all durable state (mitigate with backups + integrity checks) — acceptable under one-node doctrine if backup/rollback floors hold.

**NeuronDB / exotic PG ML extensions:** ADR-0009 **rejected** (air-gap packaging, maintenance). Still rejected here unless new primary evidence appears.

### 5. Option C — Embedded SQLite + credible vectors (BLOB flat scan and/or sqlite-vec)

**Already true in tree:**

- Embeddings stored as BLOBs on honeypot/evidence/vault.
- `search_with_decay` full-table cosine + keyword; used in production RRF even when Qdrant attached.
- FTS5 Porter virtual tables with explicit row sync (post-v4 no broken triggers).
- WAL, `busy_timeout=5000`, pool max 5, `BEGIN IMMEDIATE` promote.
- Bi-temporal + failure_cases + utility already landed (v8–v10).

**sqlite-vec ([asg017/sqlite-vec](https://github.com/asg017/sqlite-vec), docs [alexgarcia.xyz/sqlite-vec](https://alexgarcia.xyz/sqlite-vec/)):**

- Pure C extension, `vec0` virtual tables, KNN via `MATCH`, metadata columns, Rust crate exists.
- **Pre-v1:** upstream marks breaking changes expected — treat as *optional accelerator*, not constitutional dependency.
- At hundreds–low thousands of 1024-d vectors, flat scan remains the honest baseline; extension is optimization, not architecture.

**Queues / ledgers without Redis:** SQLite tables + single-writer claim (`UPDATE … WHERE status='pending' LIMIT 1` under IMMEDIATE) or filesystem queue (already distill fallback). Durable claims belong in SoT, not Redis.

**Graph:** entity/relation tables + recursive CTEs; hop depth for agent memory is small. Neo4j GDS/APOC unrestricted procedures are out of scope for sealed air-gap minimum.

**Backup / recovery:** online backup API, `VACUUM INTO`, integrity_check, single-file ship on portable media — excellent for boot-appliance and air-gap. Continuous replication (e.g. third-party Litestream-class tools) is optional forge tooling, not required for correctness.

**Rust:** keep `rusqlite` + bundled SQLite; optional loadable extension for sqlite-vec. No multi-service client surface.

**Limits:** single writer (matches doctrine); very large WAL transactions discouraged historically (mitigated in modern SQLite but keep promote batches bounded); no first-class PITR comparable to Postgres continuous archiving without add-on tooling; ANN quality/scale ceiling below dedicated engines / pgvector HNSW for multi-million vectors (not current GZMO problem).

### 6. Option D — Simpler credible alternatives (screened)

| Alternative | Verdict for durable SoT |
|-------------|-------------------------|
| Qdrant-as-SoT (`QdrantVault`) | **Reject.** Scaffold only; payload-centric store is weak for bi-temporal supersession, evidence FK, FTS, audit SQL, queue claims. Config path already fail-fasts non-sqlite backends. |
| Neo4j-as-SoT | **Reject.** Graph-first wrong fit for quotable evidence + FTS + ledgers; heavy heap; MCP indirection. |
| Redis-as-SoT | **Reject.** Appliance disables persistence; wrong durability class. |
| DuckDB / OLAP embeds | Interesting analytics side-car; not primary OLTP writer with mature multi-reader app patterns for this daemon — **out of Pareto for SoT** without separate spike. |
| Multiple SQLite files as “distributed” SoT | **Avoid** for cross-table atomicity (SQLite multi-DB attach transactions are not atomic as a set per WAL docs). Prefer one durable file/cluster. |

### 7. Capability-ladder mapping (research only)

| Concern | Min | Reference | Forge |
|---------|-----|-----------|-------|
| Durable SoT | SQLite single file | SQLite **or** local Postgres+pgvector | Postgres+pgvector |
| Vectors | BLOB flat scan | Flat or sqlite-vec / pgvector exact | pgvector HNSW when N warrants |
| FTS | FTS5 | FTS5 or `tsvector` | same |
| Graph | SQL projections | SQL projections | SQL (+ optional derived graph only if profile declares) |
| Queue | SQLite table or files | same / `SKIP LOCKED` | same |
| Audit/candidates | Append tables in SoT | same | same + PITR |
| Cache | memory | optional Redis/local | optional |
| Sidecars required | **0** | 0–1 (postgres only if chosen) | 1 (postgres) |

Declaring “Neo4j required” or “Qdrant required” on minimum profile would violate resource/sovereignty floors.

### 8. Codebase-design (depth / deletion)

- **Deep module to preserve/extend:** one writer API that hides schema, lifecycle classification, evidence attach, FTS maintenance, vector write, queue claim, and audit append. Callers should not know Qdrant/Neo4j/Redis.
- **Shallow / delete-candidates:** `VaultBackend` trait and `QdrantVault` (complexity vanishes on delete; callers already use `SqliteVault`). Qdrant sync Python + drift ratio as *product* dependencies.
- **Real second adapter rule:** only introduce a store seam if the hardware ladder **requires** two concurrent production adapters (e.g. SQLite min + Postgres forge) *and* both ship. Until ticket 10 decides ladder binding, keep research options — do not pre-build repository abstraction soup.
- **Derived rebuild jobs** (FTS rebuild, vector reindex, graph projection refresh) are internal implementation, not a second SoT.

### 9. Migration implications (evidence for ticket 10 / 13 — no migration now)

1. **Facts are freezable:** ADR-0010 already treated 1881 facts as analyzable archive, not cutover constraint.
2. **In-place rusqlite→sqlx** is the expensive path; clean-sheet one-box then cutover ADR is the lower-risk Postgres path if chosen.
3. **Simplifying to SQLite-only** is mostly *deletion and queue/ledger schema completion*, not a store migration: stop requiring sidecars; keep promote/recall; move durable queue into SoT; project graph in SQL; keep Qdrant/Neo4j off the constitutional path.
4. **Hybrid path:** SQLite SoT forever + optional Postgres replica is a *second writer/sync* hazard — avoid unless explicitly designed as derived read model with rebuild, not dual SoT.
5. **knowledge_core dual schema** must be healed in any future design (ripen table vs profile sibling DB) — independent of engine choice.

## Options and trade-offs

| Criterion | A Multi-store | B PG+pgvector | C SQLite(+vec) |
|-----------|---------------|---------------|----------------|
| One-writer ACID across fact+vector+graph | Partial (SQLite only) | Full | Full (SQL graph) |
| Bi-temporal supersession | Yes in SQLite; mirrors lag | Yes atomic | Yes atomic |
| Evidence faithfulness | SoT strong; graph weak | Strong if single schema | Strong |
| FTS+vector fusion | App RRF multi-hop | SQL or app | App RRF (or SQL+vec0) |
| Queue durability | Redis weak; file OK | First-class | First-class |
| Audit/candidate ledgers | Incomplete; multi-store awkward | Natural | Natural |
| Backup / PITR | Multi-artifact hard | Strong PITR | Strong file snapshot; weaker PITR |
| Crash recovery | SQLite good; sidecars independent failure | Single recovery story | Single file |
| Low-end fit | Poor (Neo4j RAM) | Medium | Best |
| Scale-up | Qdrant helps vectors only | Best balanced | Adequate to mid |
| Rust fit | Mixed clients | New driver stack | Current stack |
| Ops weight | Highest | Medium | Lowest |
| Air-gap packaging | 3 images + JVM graph | 1 image + ext | Binary + db file |
| Matches spike evidence | Baseline | Parity GO | Already half of baseline |

**Pareto survivors:** **C** (min→reference) and **B** (reference→forge). **A** is dominated on ops, drift, and low-end RAM once SQL graph + local vectors are accepted.

## Constraints for GZMO

1. **One physical node; runtime air-gap; local containers allowed** — sidecars must be optional capability declarations, not silent dependencies.
2. **One writer** for durable memory (ADR-0003 doctrine). Enqueue ≠ promote. No dual overnight writers; no raw Qdrant/Neo4j nutrient path (OpenClaw contracts).
3. **Non-compensable floors:** faithfulness (evidence-linked asserts), sovereignty (no cloud store), reliability/rollback (integrity_check / PITR / signed backups), resource floors (declare Neo4j-class RAM as forge-only), auditability (append ledgers).
4. **Memory evolution autonomous; schema/code/model/security expansion operator-signed** — store schema migrations are signed capability changes, not silent daemon self-edits.
5. **Adaptive ladder:** minimum must boot and improve memory without Qdrant/Neo4j/Redis. Reference may choose SQLite or Postgres. Forge may require Postgres+pgvector.
6. **Do not treat wiki, Qdrant, Neo4j, Redis, or `knowledge_core.db` as SoT.**
7. **No generic multi-adapter repository** until two real adapters are ladder-required.
8. **Final architecture selection is ticket 10**; this brief supplies evidence only.

## Unknowns

- Live CT101 row/point census **today** (2026-08-31) — unreachable in this session; use dated snapshots with labels.
- Whether release-reference hardware RAM budget comfortably hosts Postgres 16 + embed + local LLM simultaneously (depends on tickets 01/03).
- sqlite-vec production maturity on bundled `rusqlite` + Windows/Linux appliance images at v1 stability — pre-v1 risk.
- Exact multi-hop graph query set that would *fail* on SQL projections (no measured Neo4j-only recall gain in ADR spikes).
- Preferred durable queue claim shape under SQLite vs `SKIP LOCKED` (design detail for ticket 10).
- Unified candidate/evaluation/audit ledger schema (north-star evolution tickets) — engine-agnostic but must live in SoT.
- Air-gapped Postgres extension upgrade story across years (operator-signed supply chain).

## Primary sources

### Local (repository)

- `gzmo-core/src/memory/vault.rs` — WAL open, migrations v1–v10, `BEGIN IMMEDIATE` promote, `search_with_decay`, `recall_rrf`, failure_cases
- `gzmo-core/src/memory/lifecycle.rs` — supersede / bi-temporal fields
- `gzmo-core/src/memory/honeypot.rs` — qualify, FTS row sync, evidence
- `gzmo-core/src/memory/recall_rrf.rs` — RRF fusion constants
- `gzmo-core/src/memory/qdrant_sync.rs`, `qdrant_recall.rs` — mirror + assertable filter
- `gzmo-core/src/memory/vault_backend.rs` — scaffold trait / `QdrantVault`
- `gzmo-core/src/memory/scratch.rs` — Redis ephemeral + file distill fallback
- `gzmo-core/src/health.rs` — honeypot/Qdrant drift probe thresholds
- `Cargo.toml` — `rusqlite`, `r2d2_sqlite`, `redis`
- `deploy/living-appliance/docker-compose.yml` — Redis `--save ""`; Neo4j heap/pagecache; Qdrant image pin
- `docs/ADR-0009-pgvector-vault.md`, `docs/ADR-0010-clean-sheet-onebox.md`
- `docs/INFRASTRUCTURE_MAP.md` — multi-store topology and failure rows (schema v7 claim **stale**)
- `research/essential-living-diagnosis/01-memory-plane.md`, `04-wiki-kg-qdrant.md`
- `research/felt-use-shipped-vs-opportunity.md` — 2026-08-30 depth census
- `scripts/sync-vault-to-qdrant.py` — mirror/prune semantics

### External

- SQLite WAL: https://www.sqlite.org/wal.html  
- SQLite Online Backup API: https://www.sqlite.org/backup.html  
- pgvector README (v0.8.6 lineage): https://github.com/pgvector/pgvector  
- PostgreSQL 16 PITR: https://www.postgresql.org/docs/16/continuous-archiving.html  
- sqlite-vec: https://github.com/asg017/sqlite-vec · https://alexgarcia.xyz/sqlite-vec/  
- Qdrant overview: https://qdrant.tech/documentation/overview/  

### Evidence labels

| Area | Label |
|------|-------|
| Schema v10, promote IMMEDIATE, local vector scan, RRF | Observed source |
| Drift 45 pts; spike 8/12 @ 5.2 ms; freeze counts | Doc-dated 2026-08-22 ADRs |
| Felt-use latest 3005 | Doc-dated 2026-08-30 research |
| sqlite-vec pre-v1; pgvector HNSW dims | Vendor docs as of research date |
| Live CT101 “today” counts | Unreachable |
