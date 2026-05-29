//! Daemon mode — heartbeat + dreams + orchestrator.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{Timelike, Utc};
use tracing::{error, info};

use gzmo_core::config::GzmoConfig;
use gzmo_core::daemon::{FileChangeCheck, HealthPing, HeartbeatEngine};
use gzmo_core::dreams::DreamEngine;
use gzmo_core::gateway::{LlmGateway, TurboQuantGateway, VllmConfig};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::mcp::{bridge::McpServerConfig, manager::McpManager};
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

pub async fn run(config: &GzmoConfig, identity: IdentityEngine) -> Result<()> {
    let soul = identity.snapshot().await;

    info!("╔══════════════════════════════════════════════╗");
    info!("║            GZMO — Daemon Mode                ║");
    info!("║       100% Local · Air-Gapped · Rust         ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(persona = %soul.persona_name, "Identity loaded");

    // Ensure directories
    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Heartbeat
    let heartbeat_interval = Duration::from_secs(config.agent.heartbeat_interval_secs);
    let mut heartbeat = HeartbeatEngine::new(heartbeat_interval);
    heartbeat.add_check(FileChangeCheck {
        watch_dir: config.memory.directory.to_string_lossy().to_string(),
        since: Duration::from_secs(config.agent.heartbeat_interval_secs),
    });
    heartbeat.add_check(HealthPing {
        url: format!("{}/models", config.engine.active_engine().url),
        service_name: "LLM Engine".to_string(),
    });

    // Gateway + Tools for dream cycle
    let active_profile = config.engine.active_engine();
    let dream_gateway: Arc<dyn LlmGateway> =
        Arc::new(TurboQuantGateway::new(VllmConfig::from(active_profile)));

    let dream_vault = SqliteVault::open(&config.memory.vault_db)?;
    let dream_vault = Arc::new(dream_vault);

    let mut dream_tools = ToolRegistry::new();
    dream_tools.register(Box::new(FileReadTool));
    dream_tools.register(Box::new(FileWriteTool));
    dream_tools.register(Box::new(DirListTool));
    dream_tools.register(Box::new(FileSearchTool));
    dream_tools.register(Box::new(ShellExecTool::default()));
    dream_tools.register(Box::new(WebSearchTool::default()));
    dream_tools.register(Box::new(SysMetricsTool));
    dream_tools.register(Box::new(SysKillTool));
    dream_tools.register(Box::new(MemoryRecordTool {
        vault: Arc::clone(&dream_vault),
    }));
    dream_tools.register(Box::new(MemorySearchTool {
        vault: Arc::clone(&dream_vault),
    }));

    // MCP for dreams
    let mut dream_mcp = McpManager::new();
    for server in config.active_mcp_servers() {
        match dream_mcp
            .connect(McpServerConfig {
                name: server.name.clone(),
                command: server.command.clone(),
                args: server.args.clone(),
                env: server.env.clone(),
            })
            .await
        {
            Ok(count) => info!(server = %server.name, tools = count, "Dream MCP connected"),
            Err(e) => error!(server = %server.name, "Dream MCP failed: {}", e),
        }
    }
    dream_mcp.register_all_tools(&mut dream_tools);
    let dream_tools = Arc::new(dream_tools);

    let dream_episodic = Arc::new(FileEpisodicStore::new(&config.memory.directory));
    let orch_gateway = Arc::clone(&dream_gateway);
    let dream_engine = Arc::new(DreamEngine::new(
        FileEpisodicStore::new(&config.memory.directory),
        (*dream_vault).clone(),
        dream_gateway,
        Arc::clone(&dream_tools),
    ));
    let dream_engine_clone = Arc::clone(&dream_engine);
    let dreams_path = config.skills.dreams_path.clone();

    info!("All subsystems online — entering daemon loop");

    // Orchestrator (cron jobs)
    let orch_ctx = Arc::new(gzmo_core::orchestrator::OrchestratorContext {
        gateway: orch_gateway,
        tools: Arc::clone(&dream_tools),
        system_prompt: format!(
            "{}\n\n---\nYou are {} in BACKGROUND MODE.\nToday is {}.\nBe concise and action-oriented.",
            soul.raw_markdown, soul.persona_name, Utc::now().format("%Y-%m-%d %H:%M UTC"),
        ),
        vault: Some(Arc::clone(&dream_vault)),
        episodic: Some(Arc::clone(&dream_episodic)),
        chaos_feedback_tx: None, // Daemon mode doesn't run PulseLoop yet
    });

    let orch_jobs = config.orchestration.jobs.clone();
    let _scheduler =
        match gzmo_core::orchestrator::start_orchestrator(orch_jobs, Arc::clone(&orch_ctx)).await {
            Ok(s) => {
                info!("Orchestrator online");
                Some(s)
            }
            Err(e) => {
                error!("Orchestrator failed: {e}");
                None
            }
        };

    // Watchers
    let orch_watchers = config.orchestration.watchers.clone();
    if let Err(e) = gzmo_core::watcher::start_watchers(&orch_watchers, orch_ctx).await {
        error!("Watchers failed: {e}");
    }

    // Heartbeat task
    let heartbeat_handle = tokio::spawn(async move {
        heartbeat
            .run(|anomalies| async move {
                info!(count = anomalies.len(), "Heartbeat triggered");
                for a in &anomalies {
                    info!(anomaly = %a);
                }
            })
            .await
    });

    // Dream cycle task
    let dream_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let now = Utc::now();
            if now.hour() == 1 {
                let yesterday = now.date_naive() - chrono::Duration::days(1);
                info!(date = %yesterday, "Dream consolidation starting");
                match dream_engine_clone.consolidate(yesterday).await {
                    Ok(report) => {
                        info!(entities = report.entities_extracted, "Dream cycle complete");
                        let _ = tokio::fs::write(&dreams_path, &report.narrative).await;
                    }
                    Err(e) => error!("Dream failed: {e}"),
                }
                tokio::time::sleep(Duration::from_secs(7200)).await;
            }
        }
    });

    let _identity = identity;

    tokio::select! {
        _ = heartbeat_handle => error!("Heartbeat exited"),
        _ = dream_handle => error!("Dream cycle exited"),
    }

    Ok(())
}
