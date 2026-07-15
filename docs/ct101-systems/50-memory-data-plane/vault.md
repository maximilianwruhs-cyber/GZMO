# Vault — SQLite Semantic Store

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Sources:** `gzmo-core/src/memory/vault.rs`, `gzmo-core/src/memory/vault_backend.rs`

---

## Capability

The vault is CT101's **source of truth** for semantic facts: ~**60k** rows in `semantic_vault`, quarantine for low-confidence ingest, WAL-mode SQLite with r2d2 pooling, schema migrations through `PRAGMA user_version`, and the full promote/recall pipeline that feeds honeypot, evidence, and RRF search.

`VaultBackend` is a scaffold seam for a future Qdrant-backed vault; production always uses `SqliteVault`.

---

## How it works

### Open, schema, and migrations

`SqliteVault::open` creates core tables, enables WAL, and applies versioned migrations (confidence column, `content_norm`, honeypot/evidence tables, FTS trigger cleanup):

```58:169:gzmo-core/src/memory/vault.rs
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
        let init_conn = Connection::open(db_path.as_ref())
            .with_context(|| "Failed to open semantic vault database")?;
        // ...
        init_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
        // user_version migrations v1–v5: confidence, content_norm, honeypot, evidence
```

### Unified honeypot recall (RRF)

When honeypot has rows, `recall_rrf` fuses FTS, evidence FTS, graph/keyword, local vector, and Qdrant streams, then optional VM200 rerank:

```543:650:gzmo-core/src/memory/vault.rs
    pub async fn recall_rrf(
        &self,
        query: &str,
        limit: usize,
        container_tag: &str,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        // ...
        let fts_ids = self.honeypot_fts_stream(q, container_tag, PREFETCH_K)?;
        // ... qdrant + local vector via merge_interleaved_rank
        let mut scores = rrf_fuse(&rank_lists);
        // diversify → rerank → top-N
```

### Backend abstraction (draft)

`VaultBackend` trait mirrors `SqliteVault` for future swap; `QdrantVault` bodies are `todo!()`:

```26:67:gzmo-core/src/memory/vault_backend.rs
#[async_trait]
pub trait VaultBackend: Send + Sync {
    async fn store(&self, fact: &SemanticFact) -> Result<()>;
    async fn search_with_decay(&self, q_emb: &[f32], q_text: &str, limit: usize) -> Result<Vec<ScoredFact>>;
    async fn promote_truths(&self, truths: &[ExtractedTruth]) -> Result<()>;
    async fn stale_candidates(&self, limit: usize) -> Result<Vec<SemanticFact>>;
}
```

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config | `[memory] vault_db = "data/vault.db"`, `vault_backend = "sqlite"` |
| Path (CT101) | `/opt/gzmo/data/vault.db` (~664 MiB live) |
| Tables | `semantic_vault`, `quarantine_vault`, `honeypot`, `evidence`, `memory_index`, `knowledge_core` |
| Boot | `embeddings::open_vault_with_embeddings()` attaches embedder, reranker, Qdrant |
| Consumers | ingest, dream, spark, session_distill, `PlatformMemory`, MCP serve |
| CLI | `gzmo memory`, `gzmo health` (counts) |

---

## THINKING nodes

> **THINKING — vault.rs:open/migrations**
> - *Reviewed:* Schema bootstrap through user_version 5; honeypot + evidence FTS created in-process.
> - *Insight:* Migrations are non-destructive and idempotent — safe for CT101 frozen ops.
> - *Risk / limitation:* Large vault (60k facts) makes full-table FTS resync on boot costly if triggers drift.
> - *Enhancement:* Expose migration version in `gzmo health` [CT101-safe].

> **THINKING — vault.rs:recall_rrf**
> - *Reviewed:* Multi-stream RRF with stream-top rescue boost and per-file diversification before rerank.
> - *Insight:* Honeypot is the default cognition substrate; legacy vault-wide search only when honeypot empty.
> - *Risk / limitation:* Qdrant stream failure is silent (empty vec); recall degrades to SQLite-only vectors.
> - *Enhancement:* Log per-stream hit counts in recall debug mode [CT101-safe].

> **THINKING — vault_backend.rs:QdrantVault**
> - *Reviewed:* Skeleton trait + `SqliteVault` adapter; Qdrant impl bails at runtime.
> - *Insight:* Seam exists for GZMO-next without forcing CT101 to implement remote vault.
> - *Risk / limitation:* Mis-set `vault_backend = "qdrant"` fails at boot via `assert_vault_backend`.
> - *Enhancement:* Remove or implement in lab only [GZMO-next].

---

## Advancement

- **CT101:** Keep SQLite authoritative; tune RRF/rerank prefetch and honeypot qualification only.
- **GZMO-next:** Either implement `QdrantVault` with decay payload fields or delete the scaffold and stay SQLite-first per beat-gates.
- **Lab parity:** Vault recall quality gates in `scripts/ingest-quality/` should target honeypot RRF, not raw vault mirror.

---

## Enhancement backlog

1. **[CT101-safe]** Honeypot-vs-vault count drift metric in health probe (60k vault / 37k honeypot baseline).
2. **[CT101-safe]** Batch embed backfill cron for rows missing `embedding` BLOB.
3. **[CT101-safe]** Index maintenance job for `honeypot_fts` / `evidence_fts` after bulk ingest.
4. **[GZMO-next]** Full `QdrantVault` with quarantine collection and decay rescore.
5. **[GZMO-next]** Swap callers to `Arc<dyn VaultBackend>` once Qdrant bodies exist.
