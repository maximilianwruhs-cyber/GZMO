//! Bulk gated ingest: one process, one MCP connection, sequential files.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::info;

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::ingest::IngestEngine;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::qdrant_sync::{self, sync_vault_to_qdrant};
use gzmo_core::synapse::{set_event_source, EventSource, SynapseBus};
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

pub async fn run(config: &GzmoConfig, _identity: IdentityEngine, dir: PathBuf) -> Result<()> {
    info!("╔══════════════════════════════════════════════╗");
    info!("║       GZMO — Gated Directory Ingest          ║");
    info!("╚══════════════════════════════════════════════╝");

    set_event_source(EventSource::GzmoCli);

    let dir = dir.canonicalize().context("ingest-dir path")?;
    if !dir.is_dir() {
        anyhow::bail!("Not a directory: {}", dir.display());
    }

    let mut files: Vec<PathBuf> = Vec::new();
    collect_md(&dir, &mut files)?;
    files.sort();
    if files.is_empty() {
        anyhow::bail!("No .md files under {}", dir.display());
    }

    info!(dir = %dir.display(), count = files.len(), "Ingest-dir queue");

    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let router = GatewayRouter::new(config);
    let gateway = Arc::clone(router.gateway(TaskKind::IngestExtract));
    let verify_gateway = Arc::clone(router.gateway(TaskKind::IngestVerify));

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
    tools.register(Box::new(FileWriteTool));
    tools.register(Box::new(DirListTool));
    tools.register(Box::new(FileSearchTool::default()));
    tools.register(Box::new(ShellExecTool::default()));
    tools.register(Box::new(WebSearchTool::default()));
    tools.register(Box::new(SysMetricsTool));
    tools.register(Box::new(SysKillTool));
    tools.register(Box::new(MemoryRecordTool {
        vault: Arc::clone(&vault),
    }));
    tools.register(Box::new(MemorySearchTool::new(Arc::clone(&vault))));

    let session = McpSession::connect(config, &mut tools).await?;
    let tools = Arc::new(tools);
    let synapse = Arc::new(SynapseBus::new());

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

    let mut ok = 0usize;
    let mut fail = 0usize;
    for (i, path) in files.iter().enumerate() {
        info!(n = i + 1, total = files.len(), file = %path.display(), "Ingest-dir file");
        match engine.ingest_file(path).await {
            Ok(report) => {
                ok += 1;
                println!("[{}/{}] {}", i + 1, files.len(), report.summary);
            }
            Err(e) => {
                fail += 1;
                eprintln!("[{}/{}] FAIL {}: {e}", i + 1, files.len(), path.display());
            }
        }
    }

    session.close().await;

    let project_root = qdrant_sync::discover_project_root();
    if let Err(e) = sync_vault_to_qdrant(&project_root, &config.qdrant, &config.memory.vault_db).await {
        tracing::warn!(error = %e, "Post-ingest-dir Qdrant sync failed (non-fatal)");
    } else {
        info!("Post-ingest-dir Qdrant sync complete");
    }

    println!("Ingest-dir done: ok={ok} fail={fail} (consider recycle-after-wave.sh if RSS high)");
    if fail > 0 {
        anyhow::bail!("{fail} file(s) failed");
    }
    Ok(())
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}
