# Honeypot — Curated Memory Crystal

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Source:** `gzmo-core/src/memory/honeypot.rs`

---

## Capability

The honeypot layer holds **~37k** curated facts (`is_latest=1`, `container_tag=obolus`) eligible for agent recall, Qdrant sync, and Dream/Spark cognition. It enforces a **0.85 confidence floor**, source-file provenance, anti-boilerplate filters, and manual FTS5 sync (triggers removed in schema v4).

---

## How it works

### Qualification gate

Facts must meet confidence, have a non-empty `source_file`, and pass origin/content heuristics:

```12:35:gzmo-core/src/memory/honeypot.rs
pub const HONEYPOT_MIN_CONFIDENCE: f32 = 0.85;

pub fn qualifies_for_honeypot(truth: &ExtractedTruth) -> bool {
    if truth.confidence < HONEYPOT_MIN_CONFIDENCE {
        return false;
    }
    let Some(sf) = &truth.source_file else { return false; };
    // reject chat_history, quelltext, [relation:], boilerplate
```

### Insert / upsert + FTS sync

Lifecycle inserts preserve supersession metadata; upserts refresh embeddings and invalidate profile cache:

```44:128:gzmo-core/src/memory/honeypot.rs
pub fn insert_honeypot_lifecycle(/* ... */) -> Result<()> {
    conn.execute("INSERT INTO honeypot ( /* ... */ ) VALUES ( /* ... */ )", /* ... */)?;
    sync_honeypot_fts_row(conn, vault_id, &truth.content, content_norm)?;
    crate::memory::profile::invalidate_profile_cache(Some("obolus"));
}

pub fn sync_honeypot_fts_row(conn: &Connection, vault_id: &str, content: &str, content_norm: &str) -> Result<()> {
    // DELETE + INSERT into honeypot_fts by rowid
```

Evidence rows share the same transaction pattern via `upsert_evidence_row`.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Table | `honeypot` — FK to `semantic_vault(id)`; indexes on `is_latest`, `content_norm`, `source_file` |
| FTS | `honeypot_fts` virtual table (Porter tokenizer) |
| Promotion | Called from `vault::promote_truths_with_origin` after lifecycle classification |
| Qdrant | Nightly sync reads honeypot-only (`--source honeypot`) → **24k** points (subset without embeddings / stale) |
| Profile | `profile::invalidate_profile_cache` on every honeypot write |

---

## THINKING nodes

> **THINKING — honeypot.rs:qualifies_for_honeypot**
> - *Reviewed:* Confidence + source_file + blocklist for chat exports and relation rows.
> - *Insight:* Honeypot is intentionally narrower than vault — RAG quality over coverage.
> - *Risk / limitation:* Missing `source_file` drops otherwise good facts (common in legacy rows).
> - *Enhancement:* Backfill `source_file` from ingest manifest [CT101-safe].

> **THINKING — honeypot.rs:sync_honeypot_fts_row**
> - *Reviewed:* Manual FTS sync after every insert/upsert (post-v4 trigger removal).
> - *Insight:* Correctness over automation — broken triggers were worse than explicit sync.
> - *Risk / limitation:* Bulk promote paths must not skip FTS sync or recall goes dark.
> - *Enhancement:* Batch FTS rebuild command `gzmo memory rebuild-fts` [CT101-safe].

> **THINKING — honeypot.rs:evidence coupling**
> - *Reviewed:* `upsert_evidence_row` lives in honeypot module; FK to honeypot id.
> - *Insight:* Evidence tier is honeypot-scoped, not vault-scoped — aligns with M2 architecture.
> - *Risk / limitation:* Orphan evidence if honeypot row superseded without cascade cleanup.
> - *Enhancement:* Supersede hook to mark evidence stale [GZMO-next].

---

## Advancement

- **CT101:** Monitor honeypot/Qdrant ratio (37k vs 24k) — gap is expected for rows lacking embeddings or post-sync drift.
- **GZMO-next:** Explicit `memory_type` and golden-approved gate from MEMORY_ARCHITECTURE_SPEC §2.

---

## Enhancement backlog

1. **[CT101-safe]** Health check: honeypot `is_latest` count vs Qdrant collection points.
2. **[CT101-safe]** Report top `origin` values and rejection reasons from qualify gate.
3. **[CT101-safe]** Periodic `sync_honeypot_fts` full rebuild after large ingest waves.
4. **[GZMO-next]** Golden-approved flag and anti-entity registry in qualify path.
5. **[GZMO-next]** Separate container tags per stack (obolus vs pi-knowledge) with recall filter.
