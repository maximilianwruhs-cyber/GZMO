//! Learner profile tools for Agentic Teacher memory.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;

use super::{ToolDef, ToolHandler};
use crate::config::PedagogyConfig;
use crate::pedagogy::LearnerStore;

pub struct LearnerRecallTool {
    store: LearnerStore,
}

impl LearnerRecallTool {
    pub fn new(config: &PedagogyConfig) -> Self {
        Self {
            store: LearnerStore::new(config),
        }
    }
}

#[async_trait]
impl ToolHandler for LearnerRecallTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "learner_recall".to_string(),
            description: "Recall the active learner's profile: mastery, misconceptions, and teaching preferences.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String> {
        let profile = self.store.load().await?;
        Ok(profile.prompt_block(4000))
    }
}

pub struct LearnerUpdateTool {
    store: LearnerStore,
}

impl LearnerUpdateTool {
    pub fn new(config: &PedagogyConfig) -> Self {
        Self {
            store: LearnerStore::new(config),
        }
    }
}

#[async_trait]
impl ToolHandler for LearnerUpdateTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "learner_update".to_string(),
            description:
                "Update learner profile: mastery, misconception, interest, or teaching modality."
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "field": {
                        "type": "string",
                        "enum": ["mastery", "misconception", "interest", "modality_effective", "modality_ineffective"],
                        "description": "Which learner profile list to append to"
                    },
                    "value": {
                        "type": "string",
                        "description": "Short phrase to record"
                    }
                },
                "required": ["field", "value"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let field = args["field"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing field"))?;
        let value = args["value"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing value"))?
            .trim()
            .to_string();
        if value.is_empty() {
            return Err(anyhow!("value must not be empty"));
        }

        let mut profile = self.store.load().await?;
        match field {
            "mastery" => profile.semantic.mastery_vectors.push(value),
            "misconception" => profile.semantic.misconceptions.push(value),
            "interest" => profile.semantic.interests.push(value),
            "modality_effective" => profile.procedural.effective_modalities.push(value),
            "modality_ineffective" => profile.procedural.ineffective_modalities.push(value),
            other => return Err(anyhow!("Unknown field: {other}")),
        }
        profile.updated_at = Some(chrono::Utc::now());
        self.store.save(&profile).await?;
        Ok(format!("Learner profile updated: {field}"))
    }
}
