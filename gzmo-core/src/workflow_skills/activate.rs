//! Tool: `activate_workflow_skill` — model-invoked workflow activation.

use std::sync::Arc;

use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::json;

use crate::tools::{ToolDef, ToolHandler};

use super::{SharedWorkflowSession, WorkflowSkillIndex};

pub struct ActivateWorkflowSkillTool {
    pub index: Arc<WorkflowSkillIndex>,
    pub session: SharedWorkflowSession,
}

#[async_trait]
impl ToolHandler for ActivateWorkflowSkillTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "activate_workflow_skill".to_string(),
            description: "Activate a GZMO workflow skill (grill, tdd, diagnose, review, handoff) \
                by name. Returns the full skill contract — follow it immediately. \
                Use when the task matches a listed workflow skill."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Workflow skill name (e.g. grill, tdd, diagnose, review, handoff)"
                    },
                    "args": {
                        "type": "string",
                        "description": "Optional topic or focus for the workflow"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        if !self.index.model_can_activate {
            bail!("Model activation of workflow skills is disabled in config");
        }
        let name = args["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'name'"))?
            .trim();
        let topic = args["args"].as_str().unwrap_or("").trim();
        if name.is_empty() {
            bail!("Workflow skill name is empty");
        }
        if !self.index.has(name) {
            bail!(
                "Unknown workflow skill '{}'. Available: {}",
                name,
                self.index.names().join(", ")
            );
        }

        let inject = self.index.activate(&self.session, name, topic)?;
        Ok(inject)
    }
}
