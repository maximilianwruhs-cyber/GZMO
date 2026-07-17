//! # Tool Dispatch System
//!
//! Pluggable tool registry. Each tool implements `ToolHandler`.

pub mod fs;
pub mod jail;
pub mod profile;
pub mod shell;
pub mod sysadmin;
pub mod web;
pub mod web_browse;
pub mod memory;
pub mod delegate;

pub use delegate::DelegateTaskTool;
pub use jail::PathJail;
pub use profile::{register_for_profile, CapabilityProfile, ToolRegisterOpts};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::gateway::ToolCall;
use crate::text_util::truncate_chars;

/// A registered tool that the agent can invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// The result of executing a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub success: bool,
    pub output: String,
}

/// Implement this for each concrete tool.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn definition(&self) -> ToolDef;
    async fn execute(&self, args: serde_json::Value) -> Result<String>;
}

/// Central registry of all available tools.
pub struct ToolRegistry {
    handlers: HashMap<String, Box<dyn ToolHandler>>,
    audit: bool,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            audit: false,
        }
    }

    pub fn set_audit(&mut self, audit: bool) {
        self.audit = audit;
    }

    pub fn register(&mut self, handler: Box<dyn ToolHandler>) {
        let def = handler.definition();
        tracing::info!(tool = %def.name, "Registered tool");
        self.handlers.insert(def.name.clone(), handler);
    }

    pub fn definitions(&self) -> Vec<ToolDef> {
        self.handlers.values().map(|h| h.definition()).collect()
    }

    pub async fn dispatch(&self, call: &ToolCall) -> ToolResult {
        let result = match self.handlers.get(&call.function_name) {
            Some(handler) => match handler.execute(call.arguments.clone()).await {
                Ok(output) => ToolResult {
                    call_id: call.id.clone(),
                    success: true,
                    output,
                },
                Err(e) => ToolResult {
                    call_id: call.id.clone(),
                    success: false,
                    output: format!("Tool error: {e}"),
                },
            },
            None => ToolResult {
                call_id: call.id.clone(),
                success: false,
                output: format!("Unknown tool: {}", call.function_name),
            },
        };

        if self.audit {
            let args_preview = truncate_chars(&call.arguments.to_string(), 160);
            let out_preview = truncate_chars(&result.output, 120);
            tracing::info!(
                target: "gzmo_tool_audit",
                tool = %call.function_name,
                call_id = %call.id,
                success = result.success,
                args = %args_preview,
                output = %out_preview,
                "tool_audit"
            );
        }

        result
    }

    pub fn len(&self) -> usize {
        self.handlers.len()
    }
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
