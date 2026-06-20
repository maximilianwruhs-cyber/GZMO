//! One-shot CLI MCP lifecycle — connect on start, shutdown on exit (avoids uvx zombies).

use std::sync::Arc;

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::mcp::bridge::McpServerConfig;
use gzmo_core::mcp::manager::McpManager;
use gzmo_core::tools::ToolRegistry;
use tracing::{error, info};

pub struct McpSession {
    mcp: McpManager,
}

impl McpSession {
    pub async fn connect(config: &GzmoConfig, tools: &mut ToolRegistry) -> Result<Self> {
        let mut mcp = McpManager::new().with_obolus_config(Arc::new(config.clone()));
        for server in config.active_mcp_servers() {
            match mcp
                .connect(McpServerConfig {
                    name: server.name.clone(),
                    command: server.command.clone(),
                    args: server.args.clone(),
                    env: server.env.clone(),
                })
                .await
            {
                Ok(count) => info!(server = %server.name, tools = count, "MCP connected"),
                Err(e) => error!(server = %server.name, "MCP failed: {e}"),
            }
        }
        mcp.register_all_tools(tools);
        Ok(Self { mcp })
    }

    pub async fn close(mut self) {
        if let Err(e) = self.mcp.shutdown().await {
            error!("MCP shutdown: {e}");
        }
    }
}

/// Run `f`, then always shut down MCP (even when `f` returns Err).
pub async fn with_mcp<R>(
    config: &GzmoConfig,
    tools: &mut ToolRegistry,
    f: impl std::future::Future<Output = Result<R>>,
) -> Result<R> {
    let session = McpSession::connect(config, tools).await?;
    let result = f.await;
    session.close().await;
    result
}
