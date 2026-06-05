//! delegate_task — spawn a governed sub-agent via SubagentRunner Lite.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;

use crate::subagent::{SubagentRunner, SubagentSpec};
use super::{ToolDef, ToolHandler};

pub struct DelegateTaskTool {
    pub runner: Arc<SubagentRunner>,
    pub session_id: String,
    pub depth: u8,
}

#[async_trait]
impl ToolHandler for DelegateTaskTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "delegate_task".to_string(),
            description: "Delegate a focused sub-task to an isolated sub-agent. \
                Returns a short summary only (not full tool logs). \
                Use for parallel review, research, or scoped analysis."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "role": {
                        "type": "string",
                        "description": "Sub-agent role: reviewer, architect, developer, librarian"
                    },
                    "brief": {
                        "type": "string",
                        "description": "Concise task description and constraints"
                    },
                    "max_iterations": {
                        "type": "integer",
                        "description": "Max tool rounds for sub-agent (default 5, max 15)"
                    }
                },
                "required": ["role", "brief"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let role = args["role"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'role'"))?;
        let brief = args["brief"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'brief'"))?;
        let max_iterations = args["max_iterations"]
            .as_u64()
            .unwrap_or(5)
            .min(15) as usize;

        let spec = SubagentSpec {
            role: role.to_string(),
            brief: brief.to_string(),
            max_iterations,
            depth: self.depth.saturating_add(1),
            parent_session: self.session_id.clone(),
        };

        match self.runner.spawn(spec).await {
            Ok(result) => Ok(serde_json::to_string_pretty(&result)?),
            Err(e) => Ok(format!("Subagent delegation failed: {e}")),
        }
    }
}
