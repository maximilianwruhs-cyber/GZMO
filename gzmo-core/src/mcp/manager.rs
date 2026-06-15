//! # MCP Connection Manager
//!
//! Manages the lifecycle of MCP server connections.

use std::sync::Arc;

use anyhow::Result;
use tracing::{info, warn};

use rmcp::service::Peer;
use rmcp::transport::TokioChildProcess;
use rmcp::RoleClient;
use rmcp::ServiceExt;
use tokio::process::Command;

use crate::tools::ToolRegistry;

use crate::mcp::bridge::{McpClient, McpServerConfig, McpToolBridge};

/// A connected MCP server with its discovered tools.
struct ConnectedServer {
    config: McpServerConfig,
    client: McpClient,
    peer: Arc<Peer<RoleClient>>,
    bridges: Vec<McpToolBridge>,
}

/// Manages all MCP server connections.
pub struct McpManager {
    servers: Vec<ConnectedServer>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    /// Connect to an MCP server, discover its tools.
    /// Returns the number of tools discovered.
    pub async fn connect(&mut self, config: McpServerConfig) -> Result<usize> {
        info!(
            server = %config.name,
            cmd = %config.command,
            args = ?config.args,
            "Connecting to MCP server"
        );

        // Build the Command (owned, not borrowed)
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);
        for (k, v) in &config.env {
            cmd.env(k, v);
        }

        // Create the TokioChildProcess transport — takes owned Command
        let transport = TokioChildProcess::new(cmd)
            .map_err(|e| anyhow::anyhow!("Failed to spawn MCP server '{}': {}", config.name, e))?;

        // Perform MCP handshake
        let client: McpClient = ()
            .serve(transport)
            .await
            .map_err(|e| anyhow::anyhow!("MCP handshake failed for '{}': {}", config.name, e))?;

        // Log server info
        if let Some(peer_info) = client.peer_info() {
            info!(
                server = %config.name,
                peer_name = %peer_info.server_info.name,
                peer_version = %peer_info.server_info.version,
                "MCP handshake complete"
            );
        }

        // Get the Peer handle for calling methods
        let peer: &Peer<RoleClient> = client.peer();

        // Discover tools (handles pagination)
        let tools = peer
            .list_all_tools()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list tools from '{}': {}", config.name, e))?;

        info!(
            server = %config.name,
            tool_count = tools.len(),
            "Discovered MCP tools"
        );

        // We need an Arc<Peer> for the bridges, but Peer is inside RunningService.
        // Clone the Peer into an Arc. Peer implements Clone via its inner Arc fields.
        let peer_arc = Arc::new(peer.clone());

        // Create bridges
        let bridges: Vec<McpToolBridge> = tools
            .into_iter()
            .map(|tool| {
                let prefixed = format!(
                    "mcp__{}__{}",
                    config.name,
                    tool.name.replace(['-', '.'], "_")
                );
                let description: String = tool
                    .description
                    .as_ref()
                    .map(|d: &std::borrow::Cow<'static, str>| d.to_string())
                    .unwrap_or_else(|| format!("MCP tool: {}", tool.name));

                let input_schema = serde_json::Value::Object(tool.input_schema.as_ref().clone());

                info!(tool = %prefixed, mcp_name = %tool.name, "  → discovered");

                McpToolBridge {
                    prefixed_name: prefixed,
                    mcp_tool_name: tool.name.to_string(),
                    description,
                    input_schema,
                    peer: peer_arc.clone(),
                }
            })
            .collect();

        let tool_count = bridges.len();

        self.servers.push(ConnectedServer {
            config,
            client,
            peer: peer_arc,
            bridges,
        });

        Ok(tool_count)
    }

    /// Register all discovered MCP tools into the agent's ToolRegistry.
    pub fn register_all_tools(&mut self, registry: &mut ToolRegistry) {
        for server in &mut self.servers {
            let tool_count = server.bridges.len();
            for bridge in server.bridges.drain(..) {
                info!(tool = %bridge.prefixed_name, server = %server.config.name, "Registering MCP tool");
                registry.register(Box::new(bridge));
            }
            info!(server = %server.config.name, tools = tool_count, "Registered all tools");
        }
    }

    /// Get the total number of connected servers.
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    /// Gracefully shutdown all MCP connections.
    pub async fn shutdown(&mut self) -> Result<()> {
        info!(
            servers = self.servers.len(),
            "Shutting down MCP connections"
        );
        for server in self.servers.drain(..) {
            info!(server = %server.config.name, "Shutting down");
            // Drop the peer Arc first (bridges should already be drained)
            drop(server.peer);
            // Cancel the running service
            if let Err(e) = server.client.cancel().await {
                warn!(server = %server.config.name, error = %e, "Error during shutdown");
            }
        }
        Ok(())
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}
