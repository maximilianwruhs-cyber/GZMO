//! Manual spark trigger — one serendipitous recall cycle on demand.

use std::sync::Arc;

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::config::TaskKind;
use gzmo_core::synapse::set_event_source;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::embeddings;

use gzmo_core::spark::{append_spark_to_dreams, SparkEngine};
use gzmo_core::synapse::SynapseBus;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

pub async fn run(config: &GzmoConfig, _identity: &IdentityEngine, date: Option<NaiveDate>) -> Result<()> {
    let date = date.unwrap_or_else(|| Utc::now().date_naive());

    info!("╔══════════════════════════════════════════════╗");
    info!("║          GZMO — Manual Spark Cycle           ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(
        enabled = config.spark.enabled,
        verify = config.spark.verify,
        min_confidence = config.spark.min_confidence,
        "Spark settings"
    );

    // Set event source for this thread (CLI)
    set_event_source(gzmo_core::synapse::EventSource::GzmoCli);

    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let router = GatewayRouter::new(config);
    let gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::SparkHypothesis));
    let verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::SparkVerify));

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
    info!(path = %synapse.path.display(), "SynapseBus initialized (CLI spark)");

    let engine = SparkEngine::new_with_verify(
        (*vault).clone(),
        FileEpisodicStore::new(&config.memory.directory),
        gateway,
        verify_gateway,
        tools,
        config.spark.clone(),
        Some(Arc::clone(&synapse)),
    );

    let report = engine.run(date).await?;
    append_spark_to_dreams(&config.skills.dreams_path, &report.section).await?;

    info!(
        promoted = report.promoted,
        kg_relations = report.kg_relations_written,
        "Spark complete"
    );

    println!("{}", report.section);
    println!("Appended to {}", config.skills.dreams_path.display());

    session.close().await;
    Ok(())
}
