//! Manual gated ingest for a single document.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::config::TaskKind;
use gzmo_core::synapse::set_event_source;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::ingest::IngestEngine;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::qdrant_sync::{self, sync_vault_to_qdrant};

use gzmo_core::synapse::SynapseBus;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

pub async fn run(
    config: &GzmoConfig,
    _identity: IdentityEngine,
    path: PathBuf,
    dry_run: bool,
) -> Result<()> {
    info!("╔══════════════════════════════════════════════╗");
    info!("║         GZMO — Gated Document Ingest         ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(
        file = %path.display(),
        verify = config.ingest.verify,
        min_confidence = config.ingest.min_confidence,
        "Ingest settings"
    );

    // Set event source for this thread (CLI)
    set_event_source(gzmo_core::synapse::EventSource::GzmoCli);

    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let router = GatewayRouter::new(config);
    let gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::IngestExtract));
    let verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::IngestVerify));

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
    tools.register(Box::new(FileReadTool::default()));
    tools.register(Box::new(FileWriteTool::default()));
    tools.register(Box::new(DirListTool::default()));
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
    info!(path = %synapse.path.display(), "SynapseBus initialized (CLI ingest)");

    let engine = IngestEngine::new_with_verify(
        (*vault).clone(),
        FileEpisodicStore::new(&config.memory.directory),
        gateway,
        verify_gateway,
        tools,
        config.ingest.clone(),
        Some(Arc::clone(&synapse)),
    )
    .with_wiki(config.wiki.clone());

    let report = if dry_run {
        engine.ingest_file_dry_run(&path).await?
    } else {
        engine.ingest_file(&path).await?
    };

    println!("{}", report.summary);
    println!(
        "KG: {} entities, {} relations | vault truths: {}",
        report.kg_entities_written, report.kg_relations_written, report.vault_truths
    );

    session.close().await;

    if !dry_run {
        let project_root = qdrant_sync::discover_project_root();
        if let Err(e) =
            sync_vault_to_qdrant(&project_root, &config.qdrant, &config.memory.vault_db).await
        {
            tracing::warn!(error = %e, "Post-ingest Qdrant sync failed (non-fatal)");
        }
    }

    Ok(())
}
