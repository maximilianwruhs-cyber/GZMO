# Subsystem — Wiki Engine

**Sources:** `gzmo-core/src/wiki.rs`, `gzmo-core/src/wiki_md.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Deterministic **Knowledge Gardener** for the git-tracked `wiki/` layer. Emits entity/source pages from verified ingest facts (no new LLM extraction), rebuilds `index.md`, lints structure, and provides grep-class search — emit-only retrieval without Qdrant feedback loops.

---

## 2. How it works

### WikiEngine operations

```1:13:gzmo-core/src/wiki.rs
//! Operations map to the "Knowledge Gardener" duties in `WIKI.md`:
//! - WikiEngine::emit_source_page — called from IngestEngine on promotion.
//! - WikiEngine::sync — rebuild index.md from pages on disk.
//! - WikiEngine::lint — structural health report.
//! - WikiEngine::search / file_back.
```

### Emit from ingest (verified facts only)

```78:86:gzmo-core/src/wiki.rs
    pub async fn emit_source_page(
        &self,
        source_file: &str,
        entities: &[VerifiedEntity],
        relations: &[VerifiedRelation],
        date: NaiveDate,
    ) -> Result<EmitReport> {
        if !self.config.enabled { return Ok(EmitReport::default()); }
```

### Synthetic page guard (`wiki_md.rs`)

```27:30:gzmo-core/src/wiki_md.rs
    /// Marks pages emitted by the engine — the ingest guard refuses to ingest
    /// any file carrying this flag, preventing derived-fact feedback loops.
    pub gzmo_synthetic: bool,
```

```37:48:gzmo-core/src/wiki_md.rs
impl PageFrontmatter {
    pub fn new(page_type: &str, title: &str, date: &str) -> Self {
        Self { gzmo_synthetic: true, ... }
    }
}
```

### Lexical search

```201:204:gzmo-core/src/wiki_md.rs
/// Naive lexical search over wiki/**/*.md. Title/heading matches weigh more.
/// Deliberately simple: avoids honeypot/Qdrant feedback.
pub fn search(dir: &Path, query: &str, limit: usize) -> Vec<SearchHit> {
```

### Daemon cron loops

```648:717:gzmo-cli/src/daemon_cmd.rs
    // Wiki sync — daily (default after Qdrant sync)
    WikiEngine::new(wiki_sync_cfg.clone()).sync().await
    // Wiki lint — weekly (default Sunday 06:00 UTC)
    WikiEngine::new(wiki_lint_cfg.clone()).lint().await
```

---

## 3. Interfaces

| Interface | Config / path |
|-----------|---------------|
| Enable | `[wiki] enabled` |
| Directory | `[wiki] directory` → `/opt/gzmo/wiki/` |
| Index | `[wiki] index_path` → `wiki/index.md` |
| Log | `[wiki] log_path` → `wiki/log.md` |
| Sync cron | `sync_cron_hour`, `sync_cron_minute` |
| Lint cron | `lint_cron_dow`, `lint_cron_hour` |
| Emit | `[wiki] emit_on_ingest` |

---

## 4. THINKING nodes

> **THINKING — wiki.rs:emit-only retrieval**
> - *Reviewed:* search() greps markdown; no vault/Qdrant write on read.
> - *Insight:* Breaks RAG feedback loop — wiki is derived, not ingested.
> - *Risk / limitation:* Lexical search misses semantic paraphrases.
> - *Enhancement:* Optional read-only Qdrant wiki collection (separate from honeypot). [GZMO-next]

> **THINKING — wiki_md.rs:gzmo_synthetic**
> - *Reviewed:* All engine-emitted pages flag synthetic in frontmatter.
> - *Insight:* Double guard with watcher wiki/ path exclusion.
> - *Risk / limitation:* Hand-edited pages without flag could be ingested.
> - *Enhancement:* Lint rule: human pages must have `gzmo_synthetic: false`. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| Obsidian | Frontmatter compatible with Dataview |
| CT101 | `/opt/gzmo/wiki/` self-written OKF layer |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Lint enforces synthetic flag semantics | [CT101-safe] |
| 2 | Auto-stale pages when source fact superseded | [GZMO-next] |
| 3 | Wiki search exposed as MCP tool | [GZMO-next] |
| 4 | Entity merge on duplicate slugs | [CT101-safe] |
