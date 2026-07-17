//! # MCP Tool Bridge
//!
//! Wraps a single MCP tool as a `ToolHandler` so it can be registered
//! in the agent's `ToolRegistry` and called natively by the agentic loop.

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::RoleClient;

use crate::mcp::manager::SharedMcpPeer;
use crate::tools::{ToolDef, ToolHandler};

/// The live MCP client type.
/// We store a Peer<RoleClient> directly since that's the interface we call
/// methods on (list_tools, call_tool, etc.).
pub type McpClient = RunningService<RoleClient, ()>;

/// Bridges a single MCP tool into the ToolHandler trait.
pub struct McpToolBridge {
    /// The prefixed name: `mcp__{server_name}__{tool_name}`
    pub prefixed_name: String,

    /// The original MCP tool name (sent to the server during call_tool)
    pub mcp_tool_name: String,

    /// Human-readable description from the MCP server
    pub description: String,

    /// JSON Schema for the tool's input parameters
    pub input_schema: serde_json::Value,

    /// The live MCP peer handle (swappable on reconnect via manager watchdog)
    pub peer: SharedMcpPeer,
}

#[async_trait]
impl ToolHandler for McpToolBridge {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: self.prefixed_name.clone(),
            description: self.description.clone(),
            parameters: self.input_schema.clone(),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        tracing::info!(
            tool = %self.prefixed_name,
            mcp_name = %self.mcp_tool_name,
            "Invoking MCP tool"
        );

        // Build the CallToolRequestParams
        let arguments = match args {
            serde_json::Value::Object(map) => Some(map),
            _ => None,
        };

        let params = CallToolRequestParams {
            meta: None,
            name: self.mcp_tool_name.clone().into(),
            arguments,
            task: None,
        };

        // Execute via JSON-RPC over stdio
        let peer = self.peer.read().await;
        let result = peer.call_tool(params).await.map_err(|e| {
            anyhow::anyhow!("MCP call_tool failed for '{}': {}", self.mcp_tool_name, e)
        })?;

        // Extract text content from the result.
        // Content = Annotated<RawContent>, which has an `raw` field.
        let mut output = String::new();
        for content in &result.content {
            // Use serde to extract text — Content is tagged-union with "type" field.
            // RawContent::Text has a RawTextContent with .text field.
            // The Annotated wrapper means content.raw is the RawContent.
            if let Some(text) = content.as_text() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(&text.text);
            }
        }

        if result.is_error.unwrap_or(false) {
            anyhow::bail!(
                "MCP tool '{}' returned error: {}",
                self.mcp_tool_name,
                output
            );
        }

        if output.is_empty() {
            // Try structured_content as fallback
            if let Some(structured) = &result.structured_content {
                Ok(serde_json::to_string_pretty(structured)?)
            } else {
                Ok("(empty response)".to_string())
            }
        } else {
            Ok(output)
        }
    }
}

/// Configuration for a single MCP server connection.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Human-readable name (used as tool name prefix `mcp__{name}__{tool}`).
    pub name: String,
    /// Executable to run (e.g. "npx", "uvx").
    pub command: String,
    /// Arguments to pass to the command.
    pub args: Vec<String>,
    /// Optional environment variables for the child process.
    pub env: HashMap<String, String>,
}
