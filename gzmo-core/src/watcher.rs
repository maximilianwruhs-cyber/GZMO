use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::{Mutex, Semaphore};
use tracing::{error, info, warn};

use crate::config::WatcherConfig;
use crate::orchestrator::{execute_headless, OrchestratorContext};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;

/// Launch configured directory watchers using the `notify` crate.
///
/// Concurrency is gated by a `Semaphore` (one ingest at a time). Each path is
/// debounced and deduplicated by size+mtime fingerprint before ingest runs.
pub async fn start_watchers(
    watcher_configs: &HashMap<String, WatcherConfig>,
    ctx: Arc<OrchestratorContext>,
) -> anyhow::Result<()> {
    if watcher_configs.is_empty() {
        return Ok(());
    }

    let active_watchers: Vec<(String, WatcherConfig)> = watcher_configs
        .iter()
        .filter(|(_, w)| !w.disabled)
        .map(|(k, w)| (k.clone(), w.clone()))
        .collect();

    if active_watchers.is_empty() {
        return Ok(());
    }

    let concurrency_gate = Arc::new(Semaphore::new(1));
    let ingest_fingerprints = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    for (name, config) in active_watchers {
        let ctx = Arc::clone(&ctx);
        let gate = Arc::clone(&concurrency_gate);
        let fingerprints = Arc::clone(&ingest_fingerprints);
        let name_clone = name.clone();

        info!(
            watcher = %name,
            dir = %config.directory,
            debounce_secs = config.debounce_secs,
            "Starting reactive file watcher"
        );

        tokio::spawn(async move {
            if let Err(e) = run_watcher(name_clone.clone(), config, gate, fingerprints, ctx).await {
                error!(watcher = %name_clone, "Watcher crashed: {e}");
            }
        });
    }

    Ok(())
}

async fn run_watcher(
    name: String,
    config: WatcherConfig,
    gate: Arc<Semaphore>,
    ingest_fingerprints: Arc<Mutex<HashMap<String, String>>>,
    ctx: Arc<OrchestratorContext>,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<PathBuf>(100);
    let debounce = Duration::from_secs(config.debounce_secs.max(1));
    let mut debounce_tasks: HashMap<PathBuf, tokio::task::JoinHandle<()>> = HashMap::new();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            if matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(notify::event::ModifyKind::Data(_))
            ) {
                for path in event.paths {
                    let _ = tx.blocking_send(path);
                }
            }
        }
    })?;

    let target_dir = Path::new(&config.directory);
    if !target_dir.exists() {
        warn!(watcher = %name, dir = %target_dir.display(), "Watcher directory does not exist, creating it now");
        tokio::fs::create_dir_all(target_dir).await?;
    }

    watcher.watch(target_dir, RecursiveMode::Recursive)?;
    info!(watcher = %name, "Watcher natively engaged");

    while let Some(path) = rx.recv().await {
        if !path_matches_watcher(&path, &config) {
            continue;
        }

        if let Some(handle) = debounce_tasks.remove(&path) {
            handle.abort();
        }

        let path_for_task = path.clone();
        let name_c = name.clone();
        let config_c = config.clone();
        let gate_c = Arc::clone(&gate);
        let fp_c = Arc::clone(&ingest_fingerprints);
        let ctx_c = Arc::clone(&ctx);

        let handle = tokio::spawn(async move {
            tokio::time::sleep(debounce).await;
            if let Err(e) =
                process_watched_file(&name_c, &config_c, &path_for_task, gate_c, fp_c, ctx_c).await
            {
                error!(watcher = %name_c, file = %path_for_task.display(), "Watcher ingest failed: {e}");
            }
        });
        debounce_tasks.insert(path, handle);
    }

    Ok(())
}

fn path_matches_watcher(path: &Path, config: &WatcherConfig) -> bool {
    if !path.is_file() {
        return false;
    }
    if path
        .components()
        .any(|c| c.as_os_str() == ".gzmo_converted")
    {
        return false;
    }
    // Never watch the agent-owned wiki/ layer as a raw ingest source — emitted
    // pages are derived from vault facts and re-ingesting them is circular.
    if path.components().any(|c| c.as_os_str() == "wiki") {
        return false;
    }
    if let Some(ref pattern) = config.pattern {
        if let Some(ext) = pattern.strip_prefix("*.") {
            match path.extension().and_then(|s: &std::ffi::OsStr| s.to_str()) {
                Some(file_ext) if file_ext == ext => {}
                _ => return false,
            }
        } else {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if !file_name.contains(pattern) {
                return false;
            }
        }
    }
    true
}

async fn file_fingerprint(path: &Path) -> anyhow::Result<String> {
    let meta = tokio::fs::metadata(path).await?;
    let modified = meta
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(format!("{}:{}:{}", path.display(), meta.len(), modified))
}

async fn process_watched_file(
    name: &str,
    config: &WatcherConfig,
    path: &Path,
    gate: Arc<Semaphore>,
    ingest_fingerprints: Arc<Mutex<HashMap<String, String>>>,
    ctx: Arc<OrchestratorContext>,
) -> anyhow::Result<()> {
    info!(watcher = %name, file = %path.display(), "Reactive event triggered (debounced)");

    let fingerprint = file_fingerprint(path).await?;
    {
        let map = ingest_fingerprints.lock().await;
        if map.get(path.to_string_lossy().as_ref()) == Some(&fingerprint) {
            info!(
                watcher = %name,
                file = %path.display(),
                "Skipping ingest — file unchanged since last successful ingest"
            );
            return Ok(());
        }
    }

    let _permit = match gate.acquire().await {
        Ok(p) => p,
        Err(_) => return Ok(()),
    };

    let ingest_path = match maybe_convert_to_markdown(path).await {
        Ok(p) => p,
        Err(e) => {
            warn!(watcher = %name, file = %path.display(), "markitdown conversion failed, using raw file: {e}");
            path.to_path_buf()
        }
    };

    info!(watcher = %name, file = %path.display(), "Processing file ingest");

    if let Some(ref chaos_tx) = ctx.chaos_feedback_tx {
        let _ = chaos_tx
            .send(gzmo_chaos::feedback::ChaosEvent::Custom {
                tension_delta: -5.0,
                energy_delta: 20.0,
                thought_seed: Some(gzmo_chaos::feedback::ThoughtSeed {
                    category: "inbox".to_string(),
                    text: format!(
                        "File ingested: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                }),
            })
            .await;
        info!(watcher = %name, "Chaos engine: +20 energy injected from inbox");
    }

    if let Some(ref engine) = ctx.ingest_engine {
        match engine.ingest_file(&ingest_path).await {
            Ok(report) => {
                ingest_fingerprints
                    .lock()
                    .await
                    .insert(path.to_string_lossy().to_string(), fingerprint);
                info!(
                    watcher = %name,
                    file = %path.display(),
                    kg_entities = report.kg_entities_written,
                    kg_relations = report.kg_relations_written,
                    promoted_entities = report.entities_promoted,
                    summary = %report.summary,
                    "Gated ingest complete"
                );
            }
            Err(e) => error!(watcher = %name, "Gated ingest failed: {e}"),
        }
    } else {
        let active_prompt = config
            .prompt
            .replace("{file_path}", &ingest_path.display().to_string());
        warn!(
            watcher = %name,
            "IngestEngine unavailable — falling back to headless prompt (ungated)"
        );
        if let Err(e) = execute_headless(&ctx, name, &active_prompt).await {
            error!(watcher = %name, "Watcher headless cycle failed: {e}");
        } else {
            ingest_fingerprints
                .lock()
                .await
                .insert(path.to_string_lossy().to_string(), fingerprint);
        }
    }

    Ok(())
}

/// Absolute path to the markitdown CLI (installed via `uv tool install`).
const MARKITDOWN_BIN: &str = "/home/maximilian-wruhs/.local/bin/markitdown";

async fn maybe_convert_to_markdown(path: &Path) -> anyhow::Result<PathBuf> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    const TEXT_EXTS: &[&str] = &[
        "md", "markdown", "txt", "text", "csv", "tsv", "json", "xml", "log", "yaml", "yml",
    ];
    if ext.is_empty() || TEXT_EXTS.contains(&ext.as_str()) {
        return Ok(path.to_path_buf());
    }

    let convert_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".gzmo_converted");
    tokio::fs::create_dir_all(&convert_dir).await?;

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".to_string());
    let out = convert_dir.join(format!("{file_name}.md"));

    let src = path.to_path_buf();
    let out_arg = out.clone();
    let result = tokio::task::spawn_blocking(move || {
        std::process::Command::new(MARKITDOWN_BIN)
            .arg(&src)
            .arg("-o")
            .arg(&out_arg)
            .output()
    })
    .await??;

    if !result.status.success() {
        anyhow::bail!(
            "markitdown exited {}: {}",
            result.status,
            String::from_utf8_lossy(&result.stderr).trim()
        );
    }

    info!(src = %path.display(), dst = %out.display(), "Converted document to Markdown (markitdown)");
    Ok(out)
}
