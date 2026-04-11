use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn, error};

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use crate::config::WatcherConfig;
use crate::orchestrator::{execute_headless, OrchestratorContext};

/// Launch configured directory watchers using the `notify` crate.
/// 
/// Note: The concurrency is strictly gated by a `tokio::sync::Semaphore` to prevent
/// spawning hundreds of simultaneous headless agent instances and crashing the LLM.
pub async fn start_watchers(
    watcher_configs: &HashMap<String, WatcherConfig>,
    ctx: Arc<OrchestratorContext>,
) -> anyhow::Result<()> {
    if watcher_configs.is_empty() {
        return Ok(());
    }

    // Filter active watchers
    let active_watchers: Vec<(String, WatcherConfig)> = watcher_configs
        .iter()
        .filter(|(_, w)| !w.disabled)
        .map(|(k, w)| (k.clone(), w.clone()))
        .collect();

    if active_watchers.is_empty() {
        return Ok(());
    }

    // We only allow 1 active headless background run at a time to serialize LLM load
    let concurrency_gate = Arc::new(Semaphore::new(1));

    for (name, config) in active_watchers {
        let ctx = Arc::clone(&ctx);
        let gate = Arc::clone(&concurrency_gate);
        let name_clone = name.clone();
        
        info!(
            watcher = %name,
            dir = %config.directory,
            "Starting reactive file watcher"
        );

        // Spawn a background task for each watcher
        tokio::spawn(async move {
            if let Err(e) = run_watcher(name_clone.clone(), config, gate, ctx).await {
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
    ctx: Arc<OrchestratorContext>,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<PathBuf>(100);

    // Setup notify async event handler
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            // We only care about file creations or significant modifications
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

    watcher.watch(target_dir, RecursiveMode::NonRecursive)?;
    info!(watcher = %name, "Watcher natively engaged");

    // Event loop
    while let Some(path) = rx.recv().await {
        // Debounce / Filter
        if !path.is_file() {
            continue;
        }

        // Apply basic pattern matching (e.g., "*.csv", "*.pdf")
        if let Some(ref pattern) = config.pattern {
            if let Some(ext) = pattern.strip_prefix("*.") {
                if let Some(file_ext) = path.extension().and_then(|s: &std::ffi::OsStr| s.to_str()) {
                    if file_ext != ext {
                        continue;
                    }
                } else {
                    continue; // File has no extension
                }
            } else {
                // If the user supplied a raw string without *. like "invoice"
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if !file_name.contains(pattern) {
                    continue;
                }
            }
        }

        info!(watcher = %name, file = %path.display(), "Reactive event triggered");

        // Wait for concurrency slot (prevent LLM meltdown)
        let _permit = match gate.acquire().await {
            Ok(p) => p,
            Err(_) => break, // graceful shutdown if semaphore closes
        };

        // Construct dynamic prompt
        let active_prompt = config.prompt.replace("{file_path}", &path.display().to_string());
        
        info!(watcher = %name, file = %path.display(), "Spawning headless cognitive cycle");

        if let Err(e) = execute_headless(&ctx, &name, &active_prompt).await {
            error!(watcher = %name, "Watcher headless cycle failed: {e}");
        }
    }

    Ok(())
}
