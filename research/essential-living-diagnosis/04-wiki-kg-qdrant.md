# Wiki, KG, and Qdrant seam contracts

## Scope

Side-store contracts on the full living stack: Qdrant honeypot mirror (nightly + incremental + prune + drift), embeddings/rerank degrade paths (VM200), Neo4j KG extract/promote/reconcile living vs lab posture, WikiEngine emit/search vs keep-quality, and `VaultBackend` / `QdrantVault` skeleton status. Horizon-only note on ADR-0010 multi-store weight. Diagnosis only — no storage redesign.

## Contract inventory

### Authoritative stores

| Store | Role | Contract locus |
|-------|------|----------------|
| SQLite `vault.db` (`semantic_vault`, `honeypot`, `evidence`, FTS) | **Source of truth** for facts, supersede (`is_latest`), evidence | `gzmo-core/src/memory/vault.rs`; production backend always `SqliteVault` |
| Neo4j (via MCP `mcp__memory__*`) | Graph tier: entities, relations, observations with provenance notes | `kg_extract.rs` `KgPromoter::promote_to_kg`; dream/ingest/session_distill/spark callers |
| Episodic / Synapse / Obolus | Non-vector side channels (out of this ticket’s deep dive) | Daemon cron consumers |

### Mirrors / acceleration layers

| Surface | Role | Contract locus |
|---------|------|----------------|
| Qdrant collection `honeypot` | **Vector mirror** of honeypot rows that already have 1024-dim embeddings and `is_latest=1` | `scripts/sync-vault-to-qdrant.py`; daemon via `qdrant_sync::sync_vault_to_qdrant` |
| Local SQLite vectors on honeypot/evidence | Same-process vector stream when embeddings present | `vault::recall_rrf` + `search_with_decay` |
| Redis embed cache | Optional cache in front of embed HTTP | `embeddings.rs` (GET/SET fail → drop cache handle, continue) |

### Optional publish / browse surfaces

| Surface | Role | Contract locus |
|---------|------|----------------|
| `wiki/` markdown (`WikiEngine`) | Emit-only Knowledge Gardener: derived pages, never re-ingested | `wiki.rs`, `WIKI.md`, ingest hook + daemon sync/lint |
| OKForge / concept-review gates | Lab/next push path for wiki concepts | `config/gzmo-next.toml` `[wiki] backend = "okforge"`; `scripts/concept-review-gate.sh` |
| Pi `knowledge` / `knowledge_core` Qdrant collections | Separate read-mostly indexes (not living honeypot SoT) | Docs + `PlatformSearchConfig` |

### Qdrant nightly sync and prune

**Observed**

- Daemon loop (`gzmo-cli/src/daemon_cmd.rs` ~794–844): every 60s, if `[qdrant].sync_enabled`, fires once per UTC day at `sync_cron_hour`/`sync_cron_minute` (defaults **1 / 45** in `config.rs`). Calls `sync_vault_to_qdrant` → spawns `python3 scripts/sync-vault-to-qdrant.py`. Failure logs + Synapse `qdrant_sync` fail event; loop continues (stale vectors remain).
- Living appliance sketch enables this: `config/living-appliance.gzmo.toml.example` `[qdrant] enabled=true`, `sync_enabled=true`, cron 1:45, collection `honeypot`, URL localhost:6333.
- Sync preflight requires script + vault file; collection name `"honeypot"` forces SQL source `honeypot`, else `vault` (`qdrant_sync.rs` 69–73).
- Python load for honeypot: `WHERE embedding IS NOT NULL AND length(embedding) >= 4 AND is_latest = 1` (`sync-vault-to-qdrant.py` 72–88). Only 1024-dim vectors upserted. Payload stamps `is_latest: true` on new points (search must **not** filter on that stamp yet — older living points lack the field: `qdrant_recall.rs` 100–102; script comment 132–134).
- After full honeypot upsert, default path runs `prune_honeypot_orphans`: scroll all Qdrant points, delete ids not in `SELECT id FROM honeypot WHERE is_latest = 1` (`sync-vault-to-qdrant.py` 161–202, 267–268). Standalone `scripts/ct101-qdrant-prune-orphans.py` is the same keep-set delete (operator dry-run first).
- Full sync (no `--since`/`--ids`) then runs `scripts/qdrant-post-sync-verify.sh` if present: sample N random embedded latest honeypot ids must exist in Qdrant. Verify failure is **warn-only** (`qdrant_sync.rs` 115–139) — does not fail the overall sync Result once Python exited 0.

### Incremental sync after promote

**Observed**

- API: `sync_vault_to_qdrant_filtered(..., since, ids)` passes `--since` / `--ids` to the Python script; incremental skips post-sync sample verify (`qdrant_sync.rs` 47–141).
- Post-promote hook: `SqliteVault::maybe_incremental_qdrant_sync` after successful `promote_truths_with_origin` (`vault.rs` 2144–2248).
  - Requires vault has Qdrant recall attached (`self.qdrant.is_some()`).
  - **Hard gate:** `GZMO_INSTANCE == "next"` only — living CT101 instance name does **not** take this path.
  - Filters truths with `qualifies_for_honeypot` and not unverified-derived; upserts those ids + `--since` promote start time.
  - Failure is warn-only (“nightly sync remains”).
- Lab same-sitting catch-up (not daemon second writer): `scripts/qdrant-catchup-lab.sh` wraps full `sync-vault-to-qdrant.sh` → `data-next/qdrant-catchup/latest.json` (`docs/LIVING_APPLIANCE.md` 101–102).
- CLI ingest / ingest-dir also attempt full `sync_vault_to_qdrant` after successful runs (non-fatal warn on fail) — operator one-shot, not overnight dual-writer.

**[INFERENCE]** Living overnight metabolism still depends primarily on 01:45 full sync + prune for mirror freshness; same-night distill/spark/ingest after 01:45 remain invisible to Qdrant until next night or manual/lab catch-up (documented F15).

### Drift detection (`is_latest` vs point count)

**Observed**

- `probe_honeypot_qdrant_drift` in `health.rs` 177–274 (also in `run_all` health suite):
  - SQLite: `COUNT(*) FROM honeypot WHERE is_latest = 1` (all latest rows — **not** restricted to rows with embeddings).
  - Qdrant: `GET /collections/{collection}` → `points_count`.
  - `ratio = qdrant_pts / honeypot`; CRITICAL fail if ratio &lt; **0.55**; WARN but still `ok` if &lt; **0.70**; else pass with detail string.
- Doc-dated measured live drift (ADR-0009 / 2026-08-22): honeypot latest **478** vs Qdrant **433** (45-point delta) while facts were small; older CT101 reports show large absolute gaps when embeddings lag (e.g. ~37k latest vs ~24k points in system docs).
- CORE_INSIGHT / CT101 ops: drift WARN often means missing honeypot embeddings, not dead Qdrant — embed backfill then sync (`scripts/ct101-embed-backfill-loop.sh`).
- Backlog items still name “alarm when ratio drops” / sample verify — sample verify exists; ratio probe exists; neither is a continuous paging contract beyond health/ops MCP.

**Gap:** Drift probe does not distinguish (a) missing embeddings, (b) same-night lag, (c) prune failure, (d) orphan excess points (ratio can look healthy if orphans inflate Qdrant count). Prune is inside full honeypot sync unless `--no-prune`.

### Embeddings and rerank failure / degrade

**Observed**

- Defaults: `[embeddings]` disabled; default URL `http://localhost:8002/v1`, model `Qwen3-Embedding-0.6B`. Living docs pin **VM200** embed path (`:8081` in OVERNIGHT_METABOLISM / SOUL-next; config may override). Rerank default URL `http://192.168.31.110:8082/v1`, model `bge-reranker-v2-m3-q8_0.gguf`, **disabled** by default (`config.rs`).
- Boot attach (`open_vault_with_embeddings`):
  - Embeddings **enabled** → construct Embedder, probe `embed("gzmo vault probe")`; empty/unreachable → **hard error** (vault open fails).
  - Rerank enabled → `attach_reranker`: probe fail → **warn, continue without reranker** (`rerank.rs` 100–126).
  - Qdrant enabled → `QdrantRecall::from_config` fail → **warn, continue without Qdrant stream** (`embeddings.rs` 264–277).
- Query-time `recall_rrf` (`vault.rs` 1022–1055, 1096–1711):
  - No embedder / embed fail / empty vector → **FTS-only** (plus keyword/graph/evidence FTS streams); warn logged.
  - Qdrant search `Err` → empty qdrant_ids (silent via `if let Ok`); local vector stream still used if embed succeeded.
  - Rerank fail at apply time → warn, keep RRF/decay order.
- Health: `probe_rerank` fail-closed when enabled; `probe_embeddings` / EmbedHealthPing used in daemon strict startup paths. Redis embed cache miss/error degrades to live HTTP only.

**[INFERENCE]** Living recall is designed to degrade to SQLite FTS/hybrid without vectors rather than hard-fail chat/search when Qdrant/rerank are down; cold boot with embeddings required is stricter than per-query degrade.

### Neo4j / KG extract → promote → reconcile

**Observed**

- **Extract/verify:** `KgPromoter` shared by Dream, Ingest, SessionDistill (`kg_extract.rs`). Default `KgGateConfig`: verify on, min_confidence 0.85, require_evidence, **strict_kg true**.
- **Promote to graph:** `promote_to_kg` batches `mcp__memory__create_entities` / `create_relations` (size 20 via `kg_promotion::KG_BATCH_SIZE`), appends provenance observations. `strict_kg`: any batch failure or incomplete write counts → hard `bail!`; non-strict: warn and continue (`kg_extract.rs` 883–1009).
- Call sites: `dreams.rs`, `ingest.rs`, `session_distill.rs` after pipeline verify; Spark writes hypothesized relations via `mcp__memory__create_relations` with `HYPOTHESIZED_LINK` token (`kg_promotion.rs`, `spark.rs`).
- **Reconcile:** `run_kg_reconcile` needs `mcp__memory__read_graph`; dry-run default **true** (`KgReconcileConfig`). Scans entities for type canonicalization notes; relations for predicate canonicalize → delete old + create new when not dry-run. Daemon loop only if `[kg_reconcile].enabled` (`daemon_cmd.rs` 1081–1135); schedule docs: **04:00** UTC (`INFRASTRUCTURE_MAP.md`). Lab `config/gzmo-next.toml` enables reconcile (hour 4:30) with local Neo4j strict_kg.
- Living appliance example enables Neo4j MCP sidecar bolt but does **not** set `[kg_reconcile]` in the fragment — reconcile remains config-gated; extract/promote still run when engines have MCP tools + strict_kg from dream/ingest sections.
- Known split-brain scar (docs): Neo4j observation quote without matching SQLite evidence row (`INFRASTRUCTURE_MAP` F-row; kg_extract vs vault promote paths diverge by design on write surfaces).

**Living vs lab**

| Aspect | Living (CT101 / appliance) | Lab / GZMO-next |
|--------|----------------------------|-----------------|
| Neo4j sidecar | Production stack; MCP memory | Docker sidecar / shared-mcp-memory |
| KG extract+promote | On dream/ingest/distill when MCP up; strict_kg typically true | Same code; `gzmo-next.toml` `strict_kg = true` |
| KG reconcile writes | Default dry_run; must enable + dry_run=false | Enabled in next toml; still often dry-run unless applied |
| Graph empty risk | MCP down / strict failure aborts promote path for that engine batch | Workstation Neo4j documented as throwaway vs CT101 |

### WikiEngine emit / search vs keep-quality

**Observed**

- Emit: `WikiEngine::emit_source_page` after ingest promotion when `[wiki].enabled && emit_on_ingest` (default emit_on_ingest **true** in code; non-fatal on error) — sources + entities + index + log (`wiki.rs`, `ingest.rs` 321+). Deterministic; no new LLM extraction.
- Search: filesystem grep via `wiki_md::search` / MCP `gzmo_wiki_search` — **emit-only**, never honeypot/Qdrant (`WIKI.md`, `wiki.rs` header). Synthetic/wiki paths blocked from re-ingest (`has_synthetic_frontmatter` / wiki source guards).
- Daemon: daily `WikiEngine::sync` on `[wiki].sync_cron_*` (default **05:30**), weekly lint; optional okforge push backend for next.
- Positioning: cascade’s **optional publish toehold** (`MACHINE.md`, `CASCADING_COMPILER.md`) — not a second RAG store.
- **keep-quality** (`docs/KEEP_QUALITY.md`): pillars are living readiness, felt recall/use, spark, immune, ripen, lymph, attach, airgap honesty — **no wiki page-count or wiki-search row**. Wiki is not a GREEN/RED dependency of `keep-quality-gate.sh`. Concept-review / wiki-push gates are separate lab publish controls.

**[INFERENCE]** Wiki quality is “cascade worked when pages cite real sources” (operator feel), not a hard USP gate; living can be keep-quality GREEN with wiki empty or disabled if honeypot metabolism holds.

### VaultBackend / QdrantVault skeleton

**Observed**

- `vault_backend.rs`: draft trait mirroring SqliteVault API; `QdrantVault::connect` and all trait methods `bail!` / `not_implemented` — intentionally unimplemented.
- `assert_vault_backend`: only `"sqlite"` / empty allowed; `"qdrant"` fails fast at boot (`embeddings.rs` 285–293).
- Callers still use `SqliteVault` directly; Qdrant is recall/sync mirror only, not vault SoT.
- System doc backlog: implement or delete scaffold (`docs/ct101-systems/50-memory-data-plane/vault.md`).

### ADR-0010 horizon (one paragraph only)

**Doc-dated (2026-08-22, Proposed):** At the measured ~1.9k-fact / ~478-active-vector scale, three state stores (SQLite + Qdrant + Neo4j) plus sidecars and a sync cron buy little retrieval power—flat vector scan is trivial—while imposing operational weight and the measured mirror drift failure mode; SOTA agent-memory systems collapse the same job into one store. ADR-0010 therefore frames a **gated clean-sheet one-box prototype** (Postgres+pgvector sole store, no Neo4j, no Qdrant mirror) isolated from CT101, not a living cutover. This diagnosis treats that ADR as horizon context only: the living stack still runs multi-store seams above; no prototype design is in scope here.

## Gaps and drift

| Gap / drift | Nature | Evidence status |
|-------------|--------|-----------------|
| Same-night Qdrant lag (F15) | Post-01:45 vault/honeypot writes invisible to vector stream until next sync or manual/lab catch-up | Observed code schedule + docs |
| Incremental promote sync lab-only | `GZMO_INSTANCE=next` gate; living relies on nightly + CLI ingest full sync | Observed `vault.rs` |
| Drift ratio vs embeddings | Health compares all `is_latest=1` to points; missing embeddings look like “drift” | Observed probe SQL + ops docs |
| Supersede without atomic Qdrant delete | Mitigated by prune on full honeypot sync; orphans linger if sync fails or `--no-prune` | Observed script + INFRASTRUCTURE_MAP |
| Qdrant payload `is_latest` filter unsafe | Stamp on upsert only; recall must SQLite-filter | Observed recall comments |
| Neo4j vs SQLite evidence split-brain | Graph provenance can exist without evidence row | Doc-dated architecture scar |
| KG reconcile default dry-run | Ontology fix-up does not write unless enabled + dry_run false | Observed config default |
| Wiki optional vs keep-quality | Publish toehold; not keep-quality pillar | Observed KEEP_QUALITY.md vs MACHINE.md |
| `QdrantVault` dead config path | Mis-set backend aborts boot; bodies todo | Observed source |
| Sync/verify soft failures | Python fail aborts that tick; post-verify warn-only; incremental fail warn-only | Observed qdrant_sync |
| Live CT101 current counts | Point/row census for “today” | **Unreachable** from this session (no live CT101 probe) |

## Evidence status

| Claim area | Label |
|------------|-------|
| Nightly daemon Qdrant sync/prune/verify | Observed (Rust + Python + daemon_cmd) |
| Incremental after promote | Observed (vault gate `GZMO_INSTANCE=next`) |
| Drift probe thresholds 0.55 / 0.70 | Observed (`health.rs`) |
| Historical drift magnitudes | Doc-dated (ADR-0009 2026-08-22; CT101 system docs) |
| Embed/rerank/Qdrant degrade | Observed (embeddings, rerank, vault recall) |
| KG extract/promote/reconcile | Observed (kg_extract, kg_reconcile, daemon) |
| Wiki emit-only + keep-quality independence | Observed (wiki.rs, WIKI.md, KEEP_QUALITY.md) |
| QdrantVault skeleton | Observed (vault_backend.rs) |
| ADR-0010 multi-store weight | Doc-dated proposed ADR |
| Live sidecar health / current point counts | Unreachable |

## Sources

- `gzmo-core/src/memory/qdrant_sync.rs`
- `gzmo-core/src/memory/qdrant_recall.rs`
- `gzmo-core/src/memory/vault.rs` (`recall_rrf`, `maybe_incremental_qdrant_sync`, `apply_rerank`)
- `gzmo-core/src/memory/vault_backend.rs`
- `gzmo-core/src/memory/embeddings.rs`
- `gzmo-core/src/memory/rerank.rs`
- `gzmo-core/src/memory/kg_extract.rs`, `kg_promotion.rs`
- `gzmo-core/src/kg_reconcile.rs`
- `gzmo-core/src/wiki.rs`, `wiki_md.rs`
- `gzmo-core/src/health.rs` (`probe_honeypot_qdrant_drift`)
- `gzmo-core/src/config.rs` (Qdrant/Wiki/KgReconcile/Embeddings/Rerank defaults)
- `gzmo-core/src/ingest.rs` (wiki emit hook)
- `gzmo-cli/src/daemon_cmd.rs` (qdrant / kg_reconcile / wiki loops)
- `scripts/sync-vault-to-qdrant.py`, `ct101-qdrant-prune-orphans.py`, `qdrant-post-sync-verify.sh`, `qdrant-catchup-lab.sh`
- `config/living-appliance.gzmo.toml.example`, `config/gzmo-next.toml`
- `WIKI.md`, `MACHINE.md`, `docs/KEEP_QUALITY.md`, `docs/CASCADING_COMPILER.md`
- `docs/INFRASTRUCTURE_MAP.md`, `docs/OVERNIGHT_METABOLISM.md`, `docs/LIVING_APPLIANCE.md`
- `docs/ct101-systems/50-memory-data-plane/{qdrant-sync-recall,vault}.md`
- `docs/ADR-0009-pgvector-vault.md`, `docs/ADR-0010-clean-sheet-onebox.md`
