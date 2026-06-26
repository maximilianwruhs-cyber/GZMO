//! Distill persisted chat sessions into SessionDistill vault + rich episodic.

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::config::TaskKind;
use gzmo_core::synapse::{set_event_source, SynapseBus};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::embeddings;
use gzmo_core::session_distill::SessionDistillEngine;
use gzmo_core::memory::qdrant_sync;
use gzmo_core::synapse_reader;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

async fn build_distill_engine(
    config: &GzmoConfig,
) -> Result<(SessionDistillEngine, McpSession)> {
    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let router = GatewayRouter::new(config);
    let verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DistillVerify));
    let extract_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DistillExtract));
    let summary_gateway: Option<Arc<dyn LlmGateway>> = config
        .session_distill
        .librarian_summary
        .then(|| Arc::clone(router.gateway(TaskKind::DistillSummary)));

    let vault = Arc::new(
        embeddings::open_vault_with_embeddings(
            &config.memory.vault_db,
            &config.embeddings,
            &config.redis,
            &config.rerank,
            &config.qdrant,
            &config.recall,
        )
        .await?,
    );

    let mut tools = ToolRegistry::new();
    tools.register(Box::new(FileReadTool::default()));
    tools.register(Box::new(FileWriteTool));
    tools.register(Box::new(DirListTool));
    tools.register(Box::new(FileSearchTool::default()));
    tools.register(Box::new(ShellExecTool::default()));
    tools.register(Box::new(WebSearchTool::default()));
    tools.register(Box::new(SysMetricsTool));
    tools.register(Box::new(SysKillTool));
    tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(&vault) }));
    tools.register(Box::new(MemorySearchTool::new(Arc::clone(&vault))));

    let session = McpSession::connect(config, &mut tools).await?;
    let tools = Arc::new(tools);

    let synapse = Arc::new(SynapseBus::new());
    info!(path = %synapse.path.display(), "SynapseBus initialized (CLI distill)");

    let engine = SessionDistillEngine::new(
        (*vault).clone(),
        FileEpisodicStore::new(&config.memory.directory),
        &config.session_distill.sessions_dir,
        extract_gateway,
        verify_gateway,
        summary_gateway,
        tools,
        config.session_distill.clone(),
        Some(Arc::clone(&synapse)),
    );

    Ok((engine, session))
}

pub async fn run(config: &GzmoConfig, _identity: &IdentityEngine, session_id: Option<String>) -> Result<()> {
    info!("╔══════════════════════════════════════════════╗");
    info!("║       GZMO — Session Distill (→ dream)       ║");
    info!("╚══════════════════════════════════════════════╝");

    set_event_source(gzmo_core::synapse::EventSource::GzmoCli);

    let (engine, session) = build_distill_engine(config).await?;

    if let Some(id) = session_id {
        let report = engine.distill_one(&id).await?;
        println!("{}", report.summary);
    } else {
        let reports = engine.distill_all().await?;
        for r in &reports {
            println!("{}", r.summary);
        }
        let promoted: usize = reports.iter().map(|r| r.vault_truths).sum();
        println!("Total vault truths from sessions: {promoted}");
    }

    session.close().await;
    Ok(())
}

/// Distill a Pi agent session JSONL (`gzmo distill pi <path>`).
pub async fn run_pi(
    config: &GzmoConfig,
    _identity: &IdentityEngine,
    pi_session_path: &Path,
    start_turn: usize,
    max_turns: Option<usize>,
) -> Result<()> {
    info!(
        path = %pi_session_path.display(),
        start_turn,
        ?max_turns,
        "GZMO — Pi session distill (session_end → vault)"
    );

    set_event_source(gzmo_core::synapse::EventSource::GzmoCli);

    let (engine, session) = build_distill_engine(config).await?;
    let report = if start_turn > 0 || max_turns.is_some() {
        engine.distill_pi_jsonl_range(pi_session_path, start_turn, max_turns).await?
    } else {
        engine.distill_pi_jsonl(pi_session_path).await?
    };
    if !report.skipped {
        let project_root = qdrant_sync::discover_project_root();
        let distill_state = synapse_reader::default_distill_state_path(&project_root);
        let path_key = pi_session_path.to_string_lossy().to_string();
        if let Err(e) = synapse_reader::mark_pi_session_distilled(&path_key, &distill_state) {
            tracing::warn!(error = %e, "Failed to mark Pi session distilled (dedup state)");
        }
    }
    println!("{}", report.summary);
    session.close().await;
    Ok(())
}
