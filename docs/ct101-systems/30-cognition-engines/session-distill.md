# Subsystem — Session Distill

**Source:** `gzmo-core/src/session_distill.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Distills `data/sessions/*.json` chat transcripts into durable `SessionDistill` vault facts, rich episodic entries, and optional KG relations. Runs on nightly cron, archive worker (Redis distill queue), and CLI `gzmo distill`.

---

## 2. How it works

### Engine

```53:86:gzmo-core/src/session_distill.rs
pub struct SessionDistillEngine {
    promoter: KgPromoter,
    summary_gateway: Option<Arc<dyn LlmGateway>>,
    vault: SqliteVault,
    episodic: FileEpisodicStore,
    sessions: SessionManager,
    config: SessionDistillConfig,
    synapse: Option<Arc<SynapseBus>>,
}

pub fn new(
    extract_gateway: Arc<dyn LlmGateway>,
    verify_gateway: Arc<dyn LlmGateway>,
    summary_gateway: Option<Arc<dyn LlmGateway>>,
    // librarian summary when enabled
) -> Self { ... }
```

### Honeypot-safe source path

```34:43:gzmo-core/src/session_distill.rs
pub fn session_distill_source(session_id: &str) -> String {
    let safe: String = session_id.chars().map(/* alnum */).collect();
    format!("sessions/{safe}.md")
}
```

### Transcript dedup

```45:51:gzmo-core/src/session_distill.rs
pub fn distill_transcript_dedup_key(session_id: &str, transcript: &str) -> String {
    let mut h = DefaultHasher::new();
    session_id.hash(&mut h);
    transcript.trim().hash(&mut h);
    format!("{:016x}", h.finish())
}
```

### Archive worker + nightly cron

Daemon spawns `run_distill_worker` at startup (Redis/file queue from agent loop archives):

```184:187:gzmo-cli/src/daemon_cmd.rs
    let distill_worker_handle = tokio::spawn(run_distill_worker(
        Arc::clone(&scratch),
        Arc::clone(&distill_engine),
    ));
```

Nightly cron spawns `gzmo distill` subprocess or lab `synapse-distill-handoff.sh`:

```494:563:gzmo-cli/src/daemon_cmd.rs
    let distill_handle = tokio::spawn(async move {
        if !sd_config.enabled || !sd_config.daemon_scheduled { continue; }
        // cron_due_today → Command::new(&bin).arg("distill")
    });
```

---

## 3. Interfaces

| Interface | Config / path |
|-----------|---------------|
| Enable | `[session_distill] enabled` |
| Cron | `daemon_scheduled`, `cron_hour`, `cron_minute` (default 02:15) |
| Sessions dir | `[session_distill] sessions_dir` |
| Gateways | `DistillExtract`, `DistillVerify`, optional `DistillSummary` |
| Librarian | `[librarian] enabled` + summary gateway |
| Queue | Redis distill jobs or `data/distill-queue/` fallback |
| CLI | `gzmo distill` |

---

## 4. THINKING nodes

> **THINKING — session_distill.rs:dedup key**
> - *Reviewed:* Hash of session_id + transcript prevents double distill.
> - *Insight:* Archive worker and nightly cron won't duplicate same transcript.
> - *Risk / limitation:* Transcript mutation (trim) changes hash → re-distill.
> - *Enhancement:* Dedup on session_id + message count hash. [GZMO-next]

> **THINKING — session_distill.rs:source path**
> - *Reviewed:* `sessions/{id}.md` avoids chat_history exclusion patterns.
> - *Insight:* Spark can anchor on SessionDistill facts in honeypot recall.
> - *Risk / limitation:* Naming collision if manual files use same path.
> - *Enhancement:* Prefix `distilled/` namespace. [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| Synapse pull | Pi events → episodic → feeds dream |
| Lab handoff | `synapse-distill-handoff.sh` when assembly.distill = lab |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Distill backlog depth metric | [CT101-safe] |
| 2 | Per-session distill status in vault | [CT101-safe] |
| 3 | Streaming distill for long sessions | [GZMO-next] |
| 4 | Unified dedup with ingest hash registry | [GZMO-next] |
