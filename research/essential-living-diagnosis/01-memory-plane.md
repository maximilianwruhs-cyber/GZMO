# Memory plane contracts inventory

## Scope

Authoritative living-memory data-plane contracts as implemented in `gzmo-core/src/memory/` and stated in `docs/ct101-systems/50-memory-data-plane/`. Non-goals: metabolism scheduling clocks, MCP transport, live CT101 probing. Active-bet boundary: diagnose only; `felt-use-mass-growth` is not replaced.

## Contract inventory

### 1. SQLite vault SoT APIs and migrations

**Observed** — Production SoT is `SqliteVault` (`gzmo-core/src/memory/vault.rs`). Open path: `SqliteVault::open` creates base tables, enables WAL + `busy_timeout=5000`, builds r2d2 pool (`max_size=5`), then applies `PRAGMA user_version` migrations through **v10**.

| Version | Contract |
|---------|----------|
| bootstrap | `semantic_vault`, `quarantine_vault`, `memory_index`, `idx_vault_decay` |
| v1 | `semantic_vault.confidence` |
| v2 | `source_file`, `content_norm` + index |
| v3 | `honeypot` + `honeypot_fts` (Porter FTS5) |
| v4 | Drop broken FTS triggers `trg_honeypot_{ai,ad,au}` |
| v5 | `evidence` + `evidence_fts` |
| v6 | `distill_dedup` |
| v7 | `ingest_dedup` |
| v8 | `honeypot.utility_score REAL NOT NULL DEFAULT 0.0`; seed `utility_score = CAST(recall_count AS REAL)` where zero and `recall_count > 0`; index `idx_honeypot_utility (is_latest, utility_score DESC)` |
| v9 | **Repair**: if `user_version` advanced without column, re-`ALTER` + re-seed + re-index `utility_score` via `PRAGMA table_info(honeypot)` |
| v10 | Bi-temporal `valid_from` / `valid_to`, `gate_event` default `'promote'`, `failure_cases` table, `idx_honeypot_valid` |

Primary write API:

- `SqliteVault::promote_truths` → `promote_truths_with_origin(..., "ingest")`
- `SqliteVault::promote_truths_with_origin(truths, origin)` — embeds, `BEGIN IMMEDIATE`, per-truth `SAVEPOINT promote_one`
  - `confidence < 0.85` → `quarantine_vault` + `failure_cases` kind `verify_fail`
  - vault duplicate by `content_norm` → `promote_corroborate_vault`
  - else → `promote_new_vault_truth` (lifecycle against latest honeypot entity)
  - commit → optional `seed_core_pin_bonded`, `reinforce_outcome_from_new_truths` when `origin == "session_distill"`, incremental Qdrant only if `GZMO_INSTANCE=next`

Hybrid search / recall entrypoints (inherent `SqliteVault`, not only trait):

- `recall_rrf(query, limit, container_tag)` — primary cognition path when honeypot populated
- `search_with_decay` / `search_with_decay_reranked`, `keyword_search`
- `reinforce` / `reinforce_by` / `reinforce_felt` / `reinforce_outcome_from_new_truths`
- `ripen_gate_census`, `knowledge_core_row_count`, `honeypot_as_of`, `get_memory_chain`, `filter_assertable_honeypot_ids`, `take_assertable_prefetch`

Boot attach: `embeddings::open_vault_with_embeddings` probes embedder, attaches reranker, optional `QdrantRecall`.

`VaultBackend` trait (`vault_backend.rs`) mirrors a subset of inherent APIs (`store`, `store_text`, `search_with_decay`, `keyword_search`, `promote_truths`, `reinforce`, `recall_failures`, `recent`, `count`, `stale_candidates`). Callers still use `SqliteVault` directly; trait adapter delegates.

Config SoT path: `[memory] vault_db`, `vault_backend` default `"sqlite"` (`config.rs` `default_vault_backend`; `gzmo.toml.example`).

### 2. Honeypot qualification and FTS

**Observed** — `honeypot::HONEYPOT_MIN_CONFIDENCE = 0.85`.

`qualifies_for_honeypot(truth)` requires:

1. `confidence >= 0.85`
2. non-empty `source_file`
3. `source_file` path must **not** contain: `chat_history`, `chat_session`, `quelltext`, `sources`, `notebooklm`, `drive_clean`, `/takeout/`, `takeout_curated`
4. content must not start with `[relation:`
5. not boilerplate (`sources do not contain`, `migration_id`, `takeout drive`, `curated research corpus consolidated`)

Insert paths:

- `insert_honeypot_lifecycle` — new row, `is_latest=1`, `container_tag='obolus'`, optional `graph_rel` / `supersedes_id`, then `sync_honeypot_fts_row`, `valid_from` coalesce, `profile::invalidate_profile_cache(Some("obolus"))`
- `upsert_honeypot_row` — `ON CONFLICT(id) DO UPDATE` with **utility lock**: when `COALESCE(utility_score,0) > 1.0`, content/embedding/origin/decay/source/promoted_at/`is_latest` are **not** overwritten; confidence still `MAX`; FTS resynced from **saved** row content

FTS:

- Per-row: `sync_honeypot_fts_row` DELETE+INSERT by honeypot `rowid` (post-v4, no triggers)
- On recall: `ensure_honeypot_fts_synced` rebuilds full index if latest count mismatch or stale `is_latest=0` rows linger in FTS
- Evidence twin: `upsert_evidence_row` / `sync_evidence_fts_row` / `ensure_evidence_fts_synced`

Honeypot gate on promote: qualify **and** `!is_unverified_derived(truth, origin)`.

### 3. Lifecycle / supersession / `is_latest`

**Observed** — `lifecycle.rs`:

- `LifecycleKind`: `Duplicate | Extends | Contradicts | Unrelated | Derives`
- `graph_rel`: Extends→`"extends"`, Contradicts→`"update"`, Derives→`"derives"`
- `classify_truth_pair`: normalize equality → Duplicate; `contradicts_heuristic` (entity tag + strong/soft negation / predicate tail) → Contradicts; `is_extension` → Extends; else Unrelated
- `is_unverified_derived`: `[derives:` prefix always; origins `dream|verified_dream|spark|verified_spark|session_distill` need evidence text or `confidence >= 0.92`
- `find_latest_honeypot_by_entity`: `is_latest=1 AND container_tag=?` LIKE entity
- `supersede_honeypot`: `is_latest=0`, `valid_to = COALESCE(valid_to, now)`, `gate_event='supersede'` where `id` or `vault_id` match and still latest

Promote wiring (`promote_new_vault_truth`):

| Kind | Vault | Honeypot |
|------|-------|----------|
| Duplicate (entity match) | corroborate existing vault_id | upsert |
| Contradicts | insert new vault row | supersede old; insert lifecycle with `supersedes_id` |
| Extends | insert new vault row | insert lifecycle; **old stays `is_latest=1`** |
| Unrelated/Derives or no entity | insert new vault row | insert if qualifies |

`maybe_region_rewrite` only for `origin == "verified_dream"` (entity cluster supersession).

Default recall filters `is_latest = 1`. Bi-temporal read: `honeypot_as_of`. Chain: `get_memory_chain`. Qdrant hits re-gated by `filter_assertable_honeypot_ids` / `take_assertable_prefetch` (no Qdrant payload `is_latest` filter — living points predate stamp).

### 4. Recall path: RRF, rerank, utility, reinforce

**Observed** — `recall_rrf` (`vault.rs` + `recall_rrf.rs`):

Constants: `RRF_K=60`, `PREFETCH_K=50`, `QDRANT_PREFETCH_K=100`, `RERANK_PREFETCH=40`, `MAX_PER_SOURCE_FILE=5`, `UTILITY_POOL_LAMBDA=0.05`, stream-top rescue `0.025` on top-5 per list, rerank-stage per-file cap `12`.

Streams fused via `rrf_fuse`:

1. honeypot FTS (narrow then broad match)
2. evidence FTS (if non-empty)
3. graph entity stream else keyword stream
4. interleaved Qdrant + local `search_with_decay` vector ids (`merge_interleaved_rank`)
5. evidence vector stream

Pipeline order: fuse → stream-top rescue → sort → diversify → `apply_rerank` → **`apply_utility_select`** (`honeypot_utility_scores` + `apply_utility_boost`) → truncate `limit`. Empty honeypot → `search_recall_legacy`.

`apply_utility_boost`: pool-relative add `λ * (u - u_min) / span`; does not invent hits; equal utility preserves relevance order.

Felt use (`felt_use.rs` → `reinforce_felt`):

| Kind | recall_weight | utility_weight |
|------|---------------|----------------|
| Glance | 1 | 0 |
| Cited | 3 | 3 |
| Bonded | 5 | 5 |
| Outcome | 3 | 8 |

`reinforce_felt` updates `semantic_vault.confirmation_count`/`last_accessed_at` when recall_delta>0; honeypot `recall_count`, `last_recalled_at`, `utility_score += utility_delta`. `reinforce_by` sets both deltas equal. Glance cannot mint Q.

### 5. Scratch + distill queue

**Observed** — `scratch.rs`:

- Scopes: `Main { session_id }`, `Sub { session_id, task_id }`, `Orch { job, step }` → keys `gzmo:scratch:...`
- `ScratchService::from_config`: Redis when enabled (3s connect, 15s reconnect backoff, process-local fallback map); else memory backend
- APIs: `write` / `read` / `clear` / `format_for_inject` (`[RECALL]` block via `format_recall_block` + token budget)
- Distill: `DistillJob { session_id, transcript, source: MainArchive | SubArchive }`; `enqueue_distill` LPUSH Redis list `distill_queue`, else write `{uuid}.json` under `distill_fallback_dir`; `pop_distill_job` BRPOP or oldest fallback file
- Config: `[redis] enabled, url, distill_queue, distill_fallback_dir`; `[context_memory] scratch_max_tokens`

### 6. Ripen / knowledge_core honesty

**Observed** — `ripen.rs` `ripen_honeypot`: group_by_entity → resolve_contradictions → optional `export_cards` into **vault connection** table `knowledge_core` (`id, label, entity_type, summary, confidence, contradiction_resolved, supporting_facts, created_at, exported_at`). Defaults: `dedup_threshold=0.95`, `min_entries_for_card=5`, `min_confidence=0.85`, `max_cards=50`, `export=true`.

Honesty / gate census (`SqliteVault::ripen_gate_census(min_confidence, min_recall)` → `RipenGateCensus`):

- `latest`: `is_latest=1` count
- `nonzero_recall`: latest with `recall_count > 0`
- `dual`: latest with conf≥min and recall≥min
- `dual_origin`: dual restricted to origins `ingest|verified_dream|session_distill`

`knowledge_core_row_count(path)` opens a **sibling file** and `COUNT(*)` — separate from in-vault ripen export.

Profile static load (`profile.rs` `load_static_from_core`) opens sibling `knowledge_core.db` and queries `SELECT entity_tag, summary_md ... ORDER BY version` — **different schema** than ripen `export_cards`.

### 7. Dead paths: `QdrantVault` / fail-fast config

**Observed**:

- `QdrantVault::connect` bails: not implemented; use `vault_backend = "sqlite"`
- All `VaultBackend` methods on `QdrantVault` → `not_implemented`
- `embeddings::assert_vault_backend`: only `"sqlite"` or empty OK; `"qdrant"` and unknown → hard error
- Call sites: `daemon_cmd.rs`, `health_cmd.rs` (and peers) call assert before open
- **Alive** Qdrant path is mirror/recall only: `qdrant_sync::sync_vault_to_qdrant*` shells `scripts/sync-vault-to-qdrant.py` (`--source honeypot` when collection name is `honeypot`); `QdrantRecall::search_ids` HTTP search for RRF stream B′

## Gaps and drift

Facts only — named missing/broken contracts vs docs or cross-module seams.

1. **Migration ceiling drift (docs vs code)** — `docs/.../vault.md` and THINKING nodes still describe schema bootstrap “through user_version 5”. **Code** migrations run through **v10** (`utility_score` v8/v9 repair, bi-temporal + `failure_cases` v10). Observed in `vault.rs:133-351` vs `vault.md` lines citing v1–v5 only.

2. **`QdrantVault` body wording** — `vault.md` / module comment say bodies are `todo!()`. **Code** uses `anyhow::bail!` / `not_implemented` (connect fails immediately). Fail-fast config path is accurate; `todo!()` wording is stale.

3. **knowledge_core dual store / dual schema** — SYSTEM diagram and lifecycle-ripen doc treat `knowledge_core` as a vault.db table. `ripen::export_cards` writes that in-vault table. `profile::load_static_from_core` and `knowledge_core_row_count` expect sibling **`knowledge_core.db`** with columns `entity_tag`, `summary_md`, `version`. **Contract gap**: ripen export schema ≠ profile/export-script consumer schema; honesty census does not prove profile can read ripened cards.

4. **Ripen cadence wording** — `ripen.rs` module doc says “hourly background job”; `lifecycle-ripen.md` / SYSTEM say orchestrator `honeypot_ripen` **midnight UTC**. Code API is clock-agnostic (`ripen_honeypot`); schedule is outside this plane.

5. **Upsert utility lock undocumented in 50-plane honeypot.md** — `upsert_honeypot_row` freezes content fields when `utility_score > 1.0`. Honeypot subsystem report covers qualify + FTS insert but not this lock.

6. **Doc live counts are dated snapshots** — SYSTEM.md table “Live count (2026-07-14)” 60,031 / 37,807 / 24,322 is **Doc-dated**, not re-verified here. Other docs (ADR-0009, CORE audits) cite different smaller counts from other dates — expected snapshot drift, not a code contract change.

7. **vault.md tables list** includes `knowledge_core` beside vault tables without distinguishing in-DB ripen export vs sibling file used by profile/honesty count.

8. **Missing contract (named only)** — No automatic FTS purge of superseded rows except full rebuild on recall when stale detected; supersede does not call `sync_honeypot_fts_row` to delete old FTS row immediately (heal is lazy via `ensure_honeypot_fts_synced`).

## Evidence status

| Item | Label |
|------|-------|
| Migrations v1–v10, utility_score repair | Observed (`vault.rs` open) |
| promote_truths / lifecycle / qualify / FTS | Observed |
| recall_rrf streams, utility boost, felt_use weights | Observed |
| scratch + distill queue | Observed |
| ripen export + gate census | Observed |
| QdrantVault + assert_vault_backend fail-fast | Observed |
| Qdrant mirror/recall (not vault backend) | Observed |
| knowledge_core schema split (ripen vs profile) | Observed |
| Live CT101 row/point counts | Unreachable (out of ticket scope; doc counts Doc-dated) |
| Whether living CT101 `knowledge_core.db` matches profile SQL | Unreachable |
| Felt-use mass growth operational health | Out of scope (active bet; not diagnosed as replace/kill) |

## Sources

- `gzmo-core/src/memory/mod.rs`
- `gzmo-core/src/memory/vault.rs` (`open`, migrations, `reinforce_felt`, `recall_rrf`, `apply_utility_select`, `promote_truths_with_origin`, `promote_new_vault_truth`, `ripen_gate_census`, FTS ensure, assertable filter)
- `gzmo-core/src/memory/honeypot.rs` (`HONEYPOT_MIN_CONFIDENCE`, `qualifies_for_honeypot`, insert/upsert/FTS/evidence)
- `gzmo-core/src/memory/lifecycle.rs`
- `gzmo-core/src/memory/felt_use.rs`
- `gzmo-core/src/memory/recall_rrf.rs`
- `gzmo-core/src/memory/scratch.rs`
- `gzmo-core/src/memory/ripen.rs`
- `gzmo-core/src/memory/profile.rs` (`knowledge_core_path`, `load_static_from_core`)
- `gzmo-core/src/memory/vault_backend.rs`
- `gzmo-core/src/memory/embeddings.rs` (`assert_vault_backend`, `open_vault_with_embeddings`)
- `gzmo-core/src/memory/qdrant_sync.rs`, `qdrant_recall.rs`
- `gzmo-cli/src/daemon_cmd.rs`, `health_cmd.rs` (assert call sites)
- `gzmo-core/src/config.rs` (`vault_backend` default)
- `docs/ct101-systems/50-memory-data-plane/SYSTEM.md`, `vault.md`, `honeypot.md`, `lifecycle-ripen.md`, `qdrant-sync-recall.md`, `scratch-redis.md`, `embeddings-rerank.md`, `evidence.md`
