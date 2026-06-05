//! `gzmo health` — probe LLM, embeddings, Neo4j, MCP memory, optional Sovereign.

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::health::{
    format_report, probe_embeddings, probe_librarian, probe_llm_models, probe_mcp_memory,
    probe_neo4j_bolt, probe_qdrant, probe_rerank, probe_sovereign, ProbeResult,
};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::embeddings;
use gzmo_core::tools::ToolRegistry;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::shell::ShellExecTool;

use crate::cli_mcp::McpSession;

pub async fn run(config: &GzmoConfig, _identity: IdentityEngine) -> Result<()> {
    embeddings::assert_vault_backend(&config.memory.vault_backend)?;

    let mut results: Vec<ProbeResult> = Vec::new();

    let prime = config.engine.active_engine_for_mode(gzmo_core::config::EngineMode::Local);
    results.push(probe_llm_models(&prime).await);
    results.push(probe_embeddings(&config.embeddings).await);
    results.push(probe_qdrant(&config.qdrant).await);
    results.push(probe_rerank(&config.rerank).await);
    results.push(probe_librarian(&config.librarian).await);

    if let Some(srv) = config.active_mcp_servers().find(|s| s.name == "memory") {
        if let Some(url) = srv.env.get("NEO4J_URL") {
            results.push(probe_neo4j_bolt(url));
        }

        let mut tools = ToolRegistry::new();
        tools.register(Box::new(FileReadTool));
        tools.register(Box::new(FileWriteTool));
        tools.register(Box::new(DirListTool));
        tools.register(Box::new(FileSearchTool));
        tools.register(Box::new(ShellExecTool::default()));

        let session = McpSession::connect(config, &mut tools).await?;
        results.push(probe_mcp_memory(&tools).await);
        session.close().await;
    }

    if let Some(ref sovereign) = config.engine.sovereign {
        let mut r = probe_sovereign(sovereign).await;
        r.name = "sovereign";
        results.push(r);
    }

    // CLI health is one-shot — no synapse bus needed
    let report = format_report(&results);
    print!("{report}");
    if results.iter().any(|r| !r.ok && r.name != "sovereign") {
        anyhow::bail!("one or more required probes failed");
    }
    Ok(())
}
