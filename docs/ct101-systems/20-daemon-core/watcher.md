# Subsystem — File Watcher

**Source:** `gzmo-core/src/watcher.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Reactive document ingestion: watches configured directories with `notify`, debounces events, deduplicates by file fingerprint, optionally converts binaries to Markdown via markitdown, and routes to **IngestEngine** (gated) or headless orchestrator prompt (fallback).

---

## 2. How it works

### Startup

```18:59:gzmo-core/src/watcher.rs
pub async fn start_watchers(
    watcher_configs: &HashMap<String, WatcherConfig>,
    ctx: Arc<OrchestratorContext>,
) -> anyhow::Result<()> {
    let concurrency_gate = Arc::new(Semaphore::new(1));
    let ingest_fingerprints = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    for (name, config) in active_watchers {
        tokio::spawn(async move {
            run_watcher(name_clone.clone(), config, gate, fingerprints, ctx).await
        });
    }
}
```

### Event filter + debounce

```73:127:gzmo-core/src/watcher.rs
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(...)) {
            for path in event.paths {
                let _ = tx.blocking_send(path);
            }
        }
    })?;
    // ...
    while let Some(path) = rx.recv().await {
        if !path_matches_watcher(&path, &config) { continue; }
        // abort prior debounce task, spawn new sleep(debounce)
    }
```

### Path guards (anti-circular ingest)

```132:157:gzmo-core/src/watcher.rs
fn path_matches_watcher(path: &Path, config: &WatcherConfig) -> bool {
    if path.components().any(|c| c.as_os_str() == ".gzmo_converted") {
        return false;
    }
    if path.components().any(|c| c.as_os_str() == "wiki") {
        return false;
    }
    // pattern matching for extension
}
```

### Ingest dispatch

```226:260:gzmo-core/src/watcher.rs
    if let Some(ref engine) = ctx.ingest_engine {
        match engine.ingest_file(&ingest_path).await {
            Ok(report) => {
                ingest_fingerprints.lock().await.insert(path_key, fingerprint);
            }
            Err(e) => error!(watcher = %name, "Gated ingest failed: {e}"),
        }
    } else {
        execute_headless(&ctx, name, &active_prompt).await?;
    }
```

Chaos feedback on ingest: `-5 tension`, `+20 energy` with inbox thought seed.

---

## 3. Interfaces

| Interface | Config |
|-----------|--------|
| Watcher definitions | `[orchestration.watchers.<name>]` |
| Directory | `directory = "/path/to/inbox"` |
| Pattern | `pattern = "*.md"` or substring |
| Debounce | `debounce_secs` (min 1) |
| Prompt fallback | `prompt` with `{file_path}` placeholder |
| Markitdown binary | hardcoded path in source (should be config on CT101) |
| Ingest gate | `[ingest] enabled` → `OrchestratorContext.ingest_engine` |

---

## 4. THINKING nodes

> **THINKING — watcher.rs:Semaphore(1)**
> - *Reviewed:* Only one ingest at a time globally across all watchers.
> - *Insight:* Protects vault/Neo4j from concurrent write storms.
> - *Risk / limitation:* Large batch drops can queue behind single slow file.
> - *Enhancement:* Per-watcher semaphores with global cap. [GZMO-next]

> **THINKING — watcher.rs:wiki exclusion**
> - *Reviewed:* Paths under `wiki/` never ingested.
> - *Insight:* Prevents circular facts from agent-emitted wiki pages.
> - *Risk / limitation:* Symlink or path outside `wiki/` component could bypass.
> - *Enhancement:* Canonical path check against `[wiki].directory`. [CT101-safe]

> **THINKING — watcher.rs:MARKITDOWN_BIN**
> - *Reviewed:* Hardcoded `/home/maximilian-wruhs/.local/bin/markitdown`.
> - *Insight:* CT101 user is `maximilian` — path may differ on production.
> - *Risk / limitation:* PDF/DOCX ingest fails silently to raw file on missing binary.
> - *Enhancement:* `[ingest] markitdown_path` in gzmo.toml. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| IngestEngine | Preferred path — extract/verify/promote pipeline |
| Headless fallback | Legacy when `[ingest] enabled = false` |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Configurable markitdown path | [CT101-safe] |
| 2 | Canonical wiki path guard | [CT101-safe] |
| 3 | Watcher metrics (files/hour, skip rate) | [CT101-safe] |
| 4 | Multi-file batch ingest API | [GZMO-next] |
