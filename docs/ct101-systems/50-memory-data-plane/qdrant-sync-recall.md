# Qdrant Sync & Recall — Vector Mirror & RRF Stream

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Sources:** `gzmo-core/src/memory/qdrant_sync.rs`, `gzmo-core/src/memory/qdrant_recall.rs`, `gzmo-core/src/memory/recall_rrf.rs`

---

## Capability

Qdrant on CT101 (`192.168.31.202:6333`, collection **`honeypot`**) holds **~24k** vector points — a synced mirror of honeypot rows with embeddings, not the full 60k vault. Nightly cron runs `scripts/sync-vault-to-qdrant.py`. At query time, `QdrantRecall` supplies stream B′ for RRF fusion alongside SQLite FTS/vector streams.

---

## How it works

### Nightly sync

```12:72:gzmo-core/src/memory/qdrant_sync.rs
pub async fn sync_vault_to_qdrant(project_root: &Path, cfg: &QdrantConfig, vault_db: &Path) -> Result<()> {
    if !cfg.enabled || !cfg.sync_enabled { return Ok(()); }
    let sync_source = if cfg.collection == "honeypot" { "honeypot" } else { "vault" };
    tokio::process::Command::new("python3")
        .arg(&script).arg("--db").arg(&vault_db)
        .arg("--url").arg(&cfg.url).arg("--collection").arg(&cfg.collection)
        .arg("--source").arg(sync_source)
```

Orchestrator cron: **01:45 UTC** (`[qdrant]` in gzmo.toml).

### Runtime recall

```58:119:gzmo-core/src/memory/qdrant_recall.rs
    pub async fn search_ids(&self, vector: &[f32], limit: usize) -> Result<Vec<Uuid>> {
        // POST /collections/{collection}/points/search
    }
```

Used inside `vault::recall_rrf` — merged with local SQLite vector hits via `merge_interleaved_rank` to avoid double-counting correlated lists.

### RRF helpers

```49:88:gzmo-core/src/memory/recall_rrf.rs
pub fn rrf_fuse(rank_lists: &[Vec<Uuid>]) -> HashMap<Uuid, f64> {
    // contrib = 1 / (RRF_K + rank), RRF_K = 60
}

pub fn merge_interleaved_rank(a: &[Uuid], b: &[Uuid], cap: usize) -> Vec<Uuid> { /* dedupe interleave */ }

pub fn diversify_by_source_file(/* max 5 per source_file default */) -> Vec<(RecallCandidate, f64)>
```

Constants: `PREFETCH_K=50`, `QDRANT_PREFETCH_K=100` (overfetch before SQLite `is_latest` filter), `RERANK_PREFETCH=40`, `MAX_PER_SOURCE_FILE=5`.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config `[qdrant]` | `enabled`, `url`, `collection = "honeypot"`, `sync_enabled` |
| Sidecar | Docker `sidecar-qdrant` on CT101 ports 6333/6334 |
| Script | `scripts/sync-vault-to-qdrant.py` |
| Health | Point count vs honeypot — expect gap (~37k vs ~24k) when embeddings missing |
| Pi | Optional second collection via `with_collection` for Pi knowledge payloads |

---

## THINKING nodes

> **THINKING — qdrant_sync.rs:sync_source**
> - *Reviewed:* Production uses `honeypot` source when collection name is honeypot.
> - *Insight:* Enforces architecture rule: no `SELECT * FROM semantic_vault` mirror.
> - *Risk / limitation:* Python script failure only logged; daemon continues with stale vectors.
> - *Enhancement:* Synapse `health.fail` event on sync exit non-zero [CT101-safe].

> **THINKING — qdrant_recall.rs:search_hits**
> - *Reviewed:* HTTP JSON API, 30s timeout, empty vector early return. Recall overfetches then SQLite-filters `is_latest`.
> - *Insight:* Sidecar HTTP avoids gRPC dependency in core crate. Payload may stamp `is_latest: true` on new upserts.
> - *Risk / limitation:* Living Qdrant points predate the stamp. A payload filter `is_latest=true` **now** would empty the vector stream.
> - *Enhancement:* Enable Qdrant payload filter only after a full living re-sync that stamps every remaining point [CT101-safe, gated].

> **THINKING — recall_rrf.rs:merge_interleaved_rank**
> - *Reviewed:* Interleaves Qdrant vs local IDs before single RRF list.
> - *Insight:* Prevents duplicate rank contributions from correlated vector stores.
> - *Risk / limitation:* If Qdrant stale, interleave still boosts wrong IDs until next sync.
> - *Enhancement:* Staleness watermark in Qdrant payload + recall penalty [GZMO-next].

---

## Advancement

- **CT101:** Treat SQLite honeypot count as canonical; Qdrant is acceleration layer only.
- **GZMO-next:** Rust-native sync replacing Python script for beat-gate parity.

---

## Enhancement backlog

1. **[CT101-safe]** Post-sync verify: sample 10 IDs exist in both honeypot and Qdrant.
2. **[CT101-safe]** Alert when Qdrant points / honeypot ratio drops below threshold.
3. **[CT101-safe]** Incremental sync (delta since `promoted_at`) vs full rebuild.
4. **[GZMO-next]** In-process Qdrant upsert on honeypot promote (eliminate nightly lag).
5. **[GZMO-next]** Hybrid sparse+dense collection schema per MEMORY_ARCHITECTURE_SPEC.
