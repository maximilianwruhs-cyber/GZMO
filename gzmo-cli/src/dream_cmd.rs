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
use gzmo_core::config::TaskKind;
use gzmo_core::dreams::DreamEngine;
use gzmo_core::dreams_md::write_dream_narrative;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::synapse::set_event_source;

use gzmo_core::synapse::SynapseBus;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

/// Compact oversized `DREAMS.md` and optionally archive cold sessions.
pub async fn run_compact(
    config: &GzmoConfig,
    max_chars: Option<usize>,
    archive_sessions_days: Option<i64>,
    dry_run: bool,
) -> Result<()> {
    use gzmo_core::dreams_md::{
        archive_cold_sessions, compact_dreams_md, DEFAULT_DREAMS_COMPACT_MAX_CHARS,
    };

    let max_chars = max_chars.unwrap_or_else(|| {
        std::env::var("GZMO_DREAMS_MAX_CHARS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_DREAMS_COMPACT_MAX_CHARS)
    });
    let mut report = compact_dreams_md(&config.skills.dreams_path, max_chars, dry_run)?;
    if let Some(days) = archive_sessions_days {
        report.sessions_archived =
            archive_cold_sessions(&config.session_distill.sessions_dir, days, dry_run)?;
    }

    if !report.compacted && report.sessions_archived == 0 {
        println!(
            "Dream compact: nothing to do ({} chars ≤ max {})",
            report.before_chars, max_chars
        );
        return Ok(());
    }

    println!(
        "Dream compact{}: DREAMS {} → {} chars{}",
        if dry_run { " (dry-run)" } else { "" },
        report.before_chars,
        report.after_chars,
        if report.compacted {
            ""
        } else {
            " (under budget)"
        }
    );
    if let Some(p) = &report.archived_dreams {
        println!("  archive: {}", p.display());
    }
    if report.sessions_archived > 0 {
        println!(
            "  sessions archived: {} (older than {}d)",
            report.sessions_archived,
            archive_sessions_days.unwrap_or(0)
        );
    }
    Ok(())
}

/// Run a one-shot dream consolidation for `date` (defaults to today).
pub async fn run(
    config: &GzmoConfig,
    _identity: &IdentityEngine,
    date: Option<NaiveDate>,
) -> Result<()> {
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
    let gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamExtract));
    let verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamVerify));

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
    tools.register(Box::new(MemoryRecordTool {
        vault: Arc::clone(&vault),
    }));
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
