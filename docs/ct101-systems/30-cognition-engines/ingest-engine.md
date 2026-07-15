# Subsystem — Ingest Engine

**Sources:** `gzmo-core/src/ingest.rs`, `gzmo-core/src/ingest_prep.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Gated document ingest: classify and prepare files, extract entities/relations via LLM, verify, dedupe by content hash, promote to vault + Neo4j, optionally emit wiki source pages. Replaces ungated headless watcher prompts.

---

## 2. How it works

### Engine

```28:89:gzmo-core/src/ingest.rs
pub struct IngestEngine {
    promoter: KgPromoter,
    vault: SqliteVault,
    episodic: FileEpisodicStore,
    config: IngestConfig,
    synapse: Option<Arc<SynapseBus>>,
    wiki: Option<WikiConfig>,
}

pub fn new_with_verify(
    extract_gateway: Arc<dyn LlmGateway>,
    verify_gateway: Arc<dyn LlmGateway>,
    // ...
) -> Self {
    Self {
        promoter: KgPromoter::new(extract_gateway, tools, config.kg_gate())
            .with_verify_gateway(verify_gateway),
        wiki: None,
    }
}

pub fn with_wiki(mut self, wiki: WikiConfig) -> Self {
    self.wiki = Some(wiki);
    self
}
```

### Wiki circular guard

```91:114:gzmo-core/src/ingest.rs
    fn is_wiki_source(&self, path: &Path) -> bool {
        path.components()
            .any(|c| c.as_os_str() == std::ffi::OsStr::new(wiki_dir))
    }

    pub async fn ingest_file(&self, path: &Path) -> Result<IngestReport> {
        if self.is_wiki_source(path) {
            return Ok(IngestReport::skipped_wiki(path));
        }
        let content_hash = ingest_content_hash(&prepared.body);
        if self.vault.ingest_dedup_seen(&content_hash)? {
            // skip duplicate
        }
```

### Document prep (`ingest_prep.rs`)

```14:53:gzmo-core/src/ingest_prep.rs
pub enum DocClass {
    AgentSpec,
    Reference,
    ChatExport,
    Narrative,
}

pub fn split_frontmatter(raw: &str) -> (Frontmatter, String) {
    // YAML --- block parsing
}

pub fn classify_document(file_name: &str, frontmatter: &Frontmatter, body: &str) -> DocClass {
    // chat_history + USER:/MODEL: heuristics
}
```

### Watcher integration

Triggered from `watcher.rs` → `engine.ingest_file()` when `[ingest] enabled`.

---

## 3. Interfaces

| Interface | Config |
|-----------|--------|
| Enable | `[ingest] enabled` |
| Gateways | `TaskKind::IngestExtract`, `TaskKind::IngestVerify` |
| Dedup | SHA256 content hash in vault |
| Wiki emit | `[wiki] emit_on_ingest` + `with_wiki()` |
| KG gate | `[ingest] kg_gate` thresholds |
| Watchers | `[orchestration.watchers.*]` directories |

---

## 4. THINKING nodes

> **THINKING — ingest.rs:dedup hash**
> - *Reviewed:* Same content re-ingest skipped at vault layer.
> - *Insight:* Watcher fingerprint (path+mtime) + content hash = two-layer dedup.
> - *Risk / limitation:* Minor whitespace change bypasses content hash.
> - *Enhancement:* Normalize whitespace before hash. [CT101-safe]

> **THINKING — ingest_prep.rs:DocClass**
> - *Reviewed:* Heuristic classification drives extract prompts.
> - *Insight:* Chat exports get different treatment than agent specs.
> - *Risk / limitation:* Misclassification on edge-case filenames.
> - *Enhancement:* Explicit frontmatter `doc_class` override. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| markitdown | Watcher converts PDF/DOCX before ingest |
| Research corpus | Bulk ingest via inbox watchers on CT101 |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Configurable markitdown path (via watcher) | [CT101-safe] |
| 2 | Whitespace-normalized content hash | [CT101-safe] |
| 3 | Batch ingest CLI `gzmo ingest --dir` | [GZMO-next] |
| 4 | Ingest queue with retry for LLM failures | [GZMO-next] |
