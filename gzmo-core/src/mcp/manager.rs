//! # MCP Connection Manager
//!
//! Manages the lifecycle of MCP server connections.

use std::process::Stdio;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use tracing::{debug, info, warn};

use rmcp::ServiceExt;
use rmcp::service::Peer;
use rmcp::RoleClient;
use rmcp::transport::TokioChildProcess;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;

use crate::tools::ToolRegistry;

use crate::mcp::bridge::{McpClient, McpServerConfig, McpToolBridge};

pub type SharedMcpPeer = Arc<RwLock<Arc<Peer<RoleClient>>>>;

/// A connected MCP server with its discovered tools.
struct ConnectedServer {
    config: McpServerConfig,
    client: McpClient,
    peer_slot: SharedMcpPeer,
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
        let (client, peer_slot, bridges) = Self::spawn_server(&config).await?;
        let tool_count = bridges.len();
        self.servers.push(ConnectedServer {
            config,
            client,
            peer_slot,
            bridges,
        });
        Ok(tool_count)
    }

    async fn spawn_server(
        config: &McpServerConfig,
    ) -> Result<(McpClient, SharedMcpPeer, Vec<McpToolBridge>)> {
        info!(
            server = %config.name,
            cmd = %config.command,
            args = ?config.args,
            "Connecting to MCP server"
        );

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);
        for (k, v) in &config.env {
            cmd.env(k, v);
        }

        // Default rmcp stderr is Inherit — FastMCP banners / INFO logs smash the TUI
        // alternate screen. Pipe and drain into tracing instead.
        let (transport, stderr_opt) = TokioChildProcess::builder(cmd)
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn MCP server '{}': {}", config.name, e))?;

        if let Some(stderr) = stderr_opt {
            let server = config.name.clone();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    debug!(server = %server, %line, "mcp stderr");
                }
            });
        }

        let client: McpClient = ().serve(transport).await.map_err(|e| {
            anyhow!("MCP handshake failed for '{}': {}", config.name, e)
        })?;

        if let Some(peer_info) = client.peer_info() {
            info!(
                server = %config.name,
                peer_name = %peer_info.server_info.name,
                peer_version = %peer_info.server_info.version,
                "MCP handshake complete"
            );
        }

        let peer: &Peer<RoleClient> = client.peer();
        let tools = peer.list_all_tools().await.map_err(|e| {
            anyhow!("Failed to list tools from '{}': {}", config.name, e)
        })?;

        info!(
            server = %config.name,
            tool_count = tools.len(),
            "Discovered MCP tools"
        );

        let peer_arc = Arc::new(peer.clone());
        let peer_slot = Arc::new(RwLock::new(peer_arc));

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
                    peer: peer_slot.clone(),
                }
            })
            .collect();

        Ok((client, peer_slot, bridges))
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

    /// Probe MCP servers; reconnect with exponential backoff on failure.
    pub async fn ensure_healthy(&mut self) -> Result<()> {
        const MAX_ATTEMPTS: u32 = 3;
        const BACKOFF_MS: u64 = 1000;

        for server in &mut self.servers {
            let healthy = {
                let peer = server.peer_slot.read().await;
                peer.list_all_tools().await.is_ok()
            };
            if healthy {
                continue;
            }

            warn!(server = %server.config.name, "MCP server unhealthy — reconnecting");
            let config = server.config.clone();
            for attempt in 1..=MAX_ATTEMPTS {
                match Self::spawn_server(&config).await {
                    Ok((new_client, new_slot, _bridges)) => {
                        *server.peer_slot.write().await = new_slot.read().await.clone();
                        let old_client = std::mem::replace(&mut server.client, new_client);
                        if let Err(e) = old_client.cancel().await {
                            warn!(server = %config.name, error = %e, "Error shutting down stale MCP client");
                        }
                        info!(server = %config.name, attempt, "MCP server reconnected");
                        break;
                    }
                    Err(e) => {
                        warn!(
                            server = %config.name,
                            attempt,
                            error = %e,
                            "MCP reconnect attempt failed"
                        );
                        if attempt == MAX_ATTEMPTS {
                            return Err(e);
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(
                            BACKOFF_MS * attempt as u64,
                        ))
                        .await;
                    }
                }
            }
        }
        Ok(())
    }

    /// Gracefully shutdown all MCP connections.
    pub async fn shutdown(&mut self) -> Result<()> {
        info!(servers = self.servers.len(), "Shutting down MCP connections");
        for server in self.servers.drain(..) {
            info!(server = %server.config.name, "Shutting down");
            drop(server.peer_slot);
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
