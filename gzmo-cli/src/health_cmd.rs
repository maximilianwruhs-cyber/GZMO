//! `gzmo health` — probe LLM, embeddings, Neo4j, MCP memory, optional Sovereign.

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::health::{collect_health_probes, format_report};
use gzmo_core::memory::embeddings;
use gzmo_core::tools::ToolRegistry;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::shell::ShellExecTool;

use crate::cli_mcp::McpSession;

pub async fn run(config: &GzmoConfig) -> Result<()> {
    embeddings::assert_vault_backend(&config.memory.vault_backend)?;

    let mut tools = ToolRegistry::new();
    tools.register(Box::new(FileReadTool::default()));
    tools.register(Box::new(FileWriteTool::default()));
    tools.register(Box::new(DirListTool::default()));
    tools.register(Box::new(FileSearchTool::default()));
    tools.register(Box::new(ShellExecTool::default()));

    let mcp_connected = if config.active_mcp_servers().any(|s| s.name == "memory") {
        let session = McpSession::connect(config, &mut tools).await?;
        let results = collect_health_probes(config, Some(&tools)).await;
        session.close().await;
        results
    } else {
        collect_health_probes(config, None).await
    };

    let report = format_report(&mcp_connected);
    print!("{report}");
    if mcp_connected
        .iter()
        .any(|r| !r.ok && r.name != "sovereign")
    {
        anyhow::bail!("one or more required probes failed");
    }
    Ok(())
}
