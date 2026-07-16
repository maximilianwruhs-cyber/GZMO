//! Manual dream trigger — run a single autoDream consolidation on demand.
//!
//! Mirrors the dream-engine wiring the daemon builds for its 01:00 cycle, but
//! runs `consolidate(date)` once and exits. Useful for operating the brain
//! ("dream now") and for exercising the verification firewall without waiting
//! for the nightly window.

use std::sync::Arc;

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::dreams::DreamEngine;
use gzmo_core::dreams_md::write_dream_narrative;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::config::TaskKind;
use gzmo_core::synapse::set_event_source;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::embeddings;

use gzmo_core::synapse::SynapseBus;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

/// Run a one-shot dream consolidation for `date` (defaults to today).
pub async fn run(config: &GzmoConfig, _identity: &IdentityEngine, date: Option<NaiveDate>) -> Result<()> {
    let date = date.unwrap_or_else(|| Utc::now().date_naive());

    info!("╔══════════════════════════════════════════════╗");
    info!("║        GZMO — Manual Dream Consolidation     ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(
        date = %date,
        verify = config.dreams.verify,
        min_confidence = config.dreams.min_confidence,
        verify_temperature = config.dreams.verify_temperature,
        "Dream settings"
    );

    // Set event source for this thread (CLI)
    set_event_source(gzmo_core::synapse::EventSource::GzmoCli);

    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let router = GatewayRouter::new(config);
    let gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::DreamExtract));
    let verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::DreamVerify));

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

    // Connect MCP servers (memory → Neo4j) so verified facts can be written.
    let session = McpSession::connect(config, &mut tools).await?;
    let tools = Arc::new(tools);

    let synapse = Arc::new(SynapseBus::new());
    info!(path = %synapse.path.display(), "SynapseBus initialized (CLI dream)");

    let engine = DreamEngine::new_with_verify(
        FileEpisodicStore::new(&config.memory.directory),
        (*vault).clone(),
        gateway,
        verify_gateway,
        tools,
        config.dreams.clone(),
        Some(Arc::clone(&synapse)),
    );

    info!(date = %date, "Starting consolidation");
    let report = engine.consolidate(date).await?;

    // Persist the narrative to DREAMS.md and echo it for inspection.
    let dreams_path = &config.skills.dreams_path;
    write_dream_narrative(dreams_path, &report.narrative).await?;

    info!(
        entities_extracted = report.entities_extracted,
        relations_extracted = report.relations_extracted,
        kg_entities_written = report.kg_entities_written,
        kg_relations_written = report.kg_relations_written,
        truths_promoted = report.truths_promoted,
        "Consolidation complete"
    );

    println!("\n{}\n", report.narrative);
    println!("DREAMS.md written to {}", dreams_path.display());

    session.close().await;
    Ok(())
}
