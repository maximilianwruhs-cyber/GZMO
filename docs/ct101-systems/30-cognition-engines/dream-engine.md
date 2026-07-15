# Subsystem — Dream Engine

**Sources:** `gzmo-core/src/dreams.rs`, `gzmo-core/src/dreams_md.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Nightly consolidation of episodic daily logs into structured vault facts and Neo4j relations. Pipeline: **Light** (compress) → **REM** (LLM extract) → **Verify** → **Deep** (KG + honeypot promotion). Writes narrative to `DREAMS.md` while preserving spark sections.

---

## 2. How it works

### Engine structure

```44:92:gzmo-core/src/dreams.rs
pub struct DreamEngine {
    episodic: FileEpisodicStore,
    vault: SqliteVault,
    promoter: KgPromoter,
    dreams: DreamsConfig,
    synapse: Option<Arc<SynapseBus>>,
}

pub fn new_with_verify(
    extract_gateway: Arc<dyn LlmGateway>,
    verify_gateway: Arc<dyn LlmGateway>,
    // ...
) -> Self {
    Self {
        promoter: KgPromoter::new(extract_gateway, tools, dreams.kg_gate())
            .with_verify_gateway(verify_gateway),
        ...
    }
}
```

### Consolidate entry

```94:120:gzmo-core/src/dreams.rs
pub async fn consolidate(&self, date: NaiveDate) -> Result<DreamReport> {
    let raw = self.episodic.read_day(date).await?;
    if raw.trim().is_empty() { /* skip */ }
    let filtered = filter_episodic_for_consolidation(&raw, &self.dreams.exclude_episodic_substrings);
    if filtered.trim().len() < self.dreams.min_consolidation_chars { /* skip ops noise */ }
```

### Honeypot REM substrate (M3)

```142:167:gzmo-core/src/dreams.rs
        if self.dreams.honeypot_rem_enabled && self.vault.cognition_uses_honeypot() {
            match self.vault.build_honeypot_rem_context(...).await {
                Ok(hp) if !hp.trim().is_empty() => {
                    rem_input = format!("{}\n\n### HONEYPOT ASSOCIATIONS (M3)\n{}\n", ...);
                }
            }
        }
        let compressed = self.light_phase(&rem_input);
        let chunks = chunk_text_for_llm(&compressed, self.dreams.chunk_chars);
        // promoter.run_pipeline per chunk
```

### DREAMS.md merge (spark preservation)

```46:56:gzmo-core/src/dreams_md.rs
pub async fn write_dream_narrative(path: &Path, narrative: &str) -> Result<()> {
    let existing = if path.exists() {
        tokio::fs::read_to_string(path).await.unwrap_or_default()
    } else { String::new() };
    let merged = merge_dream_narrative(&existing, narrative);
    tokio::fs::write(path, merged).await?;
}
```

```11:20:gzmo-core/src/dreams_md.rs
pub fn split_dream_and_spark(content: &str) -> (String, String) {
    if let Some(pos) = content.find("\n## Spark") {
        let dream = content[..pos].trim_end().to_string();
        let spark = content[pos + 1..].to_string();
        return (dream, spark);
    }
```

### Daemon cron loop

```319:368:gzmo-cli/src/daemon_cmd.rs
    let dream_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_consolidated: Option<NaiveDate> = None;
        loop {
            // cron_hour/minute check for yesterday's date
            match dream_engine_clone.consolidate(yesterday).await {
                Ok(report) => {
                    write_dream_narrative(&dreams_path, &report.narrative).await?;
                }
            }
        }
    });
```

---

## 3. Interfaces

| Interface | Config / path |
|-----------|---------------|
| Enable | `[dreams] enabled` |
| Cron | `cron_hour`, `cron_minute` (default 01:00 UTC) |
| Output | `[skills] dreams_path` → `/opt/gzmo/DREAMS.md` |
| Gateways | `TaskKind::DreamExtract`, `TaskKind::DreamVerify` |
| Episodic source | `memory/{date}.md` |
| Honeypot REM | `honeypot_rem_enabled`, `honeypot_rem_anchor_limit` |

---

## 4. THINKING nodes

> **THINKING — dreams.rs:ops noise filter**
> - *Reviewed:* `exclude_episodic_substrings` + `min_consolidation_chars` skip janitor meta.
> - *Insight:* Prevents dream from re-extracting daemon heartbeat/spark logs as entities.
> - *Risk / limitation:* Legitimate short human notes may be skipped.
> - *Enhancement:* Source-tag episodic entries (human vs internal) instead of substring filter. [GZMO-next]

> **THINKING — dreams_md.rs:spark split**
> - *Reviewed:* Dream rewrite preserves everything from first `\n## Spark` onward.
> - *Insight:* Spark narrative accumulates in DREAMS.md without nightly overwrite.
> - *Risk / limitation:* Malformed spark headings could split incorrectly.
> - *Enhancement:* YAML frontmatter block between dream and spark sections. [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| `session-to-dream.sh` | Lab backend when `assembly.dream = lab` |
| CT101 live | Inline Rust DreamEngine + cloud GLM extract/verify |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Tune honeypot REM limits for 60k vault | [CT101-safe] |
| 2 | Episodic source tagging (human/internal) | [GZMO-next] |
| 3 | Dream report metrics to Synapse | [CT101-safe] |
| 4 | Multi-day catch-up after long outage | [CT101-safe] |
