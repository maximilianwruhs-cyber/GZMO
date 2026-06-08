//! Distill persisted chat sessions into SessionDistill vault + rich episodic.

use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::config::TaskKind;
use gzmo_core::synapse::set_event_source;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::embeddings;
use gzmo_core::session_distill::SessionDistillEngine;
use gzmo_core::synapse::SynapseBus;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

pub async fn run(config: &GzmoConfig, _identity: &IdentityEngine, session_id: Option<String>) -> Result<()> {
    info!("╔══════════════════════════════════════════════╗");
    info!("║       GZMO — Session Distill (→ dream)       ║");
    info!("╚══════════════════════════════════════════════╝");

    // Set event source for this thread (CLI)
    set_event_source(gzmo_core::synapse::EventSource::GzmoCli);

    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Use Obolus GatewayRouter for distill routing
    let router = GatewayRouter::new(config);
    let verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DistillVerify));
    let extract_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DistillExtract));
    let summary_gateway: Option<Arc<dyn LlmGateway>> = config
        .session_distill
        .librarian_summary
        .then(|| Arc::clone(router.gateway(TaskKind::DistillSummary)))
        .filter(|_| config.librarian.enabled);

    let vault = Arc::new(
        embeddings::open_vault_with_embeddings(
            &config.memory.vault_db,
            &config.embeddings,
            &config.redis,
            &config.rerank,
            &config.qdrant,
        )
        .await?,
    );

    let mut tools = ToolRegistry::new();
    tools.register(Box::new(FileReadTool));
    tools.register(Box::new(FileWriteTool));
    tools.register(Box::new(DirListTool));
    tools.register(Box::new(FileSearchTool));
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
