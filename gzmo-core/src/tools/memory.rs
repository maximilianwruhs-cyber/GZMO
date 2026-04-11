use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use crate::memory::vault::SqliteVault;
use super::{ToolDef, ToolHandler};

/// Tool to record a new fact into the Sovereign Native Memory (SqliteVault).
pub struct MemoryRecordTool {
    pub vault: Arc<SqliteVault>,
}

#[async_trait]
impl ToolHandler for MemoryRecordTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "memory_record".to_string(),
            description: "Store a new factual observation, user preference, or structural memory securely into the permanent local Knowledge Vault.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "fact": {
                        "type": "string",
                        "description": "A concise declarative sentence containing the fact to store. Ex: 'The host system prefers the Nemotron model due to VRAM limits.'"
                    },
                    "category": {
                        "type": "string",
                        "description": "Category: 'Core' (Never decays), 'Procedural' (Long half-life), or 'Episodic' (Standard decay)",
                        "enum": ["Core", "Procedural", "Episodic"]
                    },
                    "confidence": {
                        "type": "number",
                        "description": "Your certainty of this fact (0.0 to 1.0). Facts < 0.85 will be quarantined for human review."
                    }
                },
                "required": ["fact", "category", "confidence"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let fact = args["fact"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'fact' parameter"))?;
            
        let category = args["category"]
            .as_str()
            .unwrap_or("Episodic");

        let confidence = args["confidence"]
            .as_f64()
            .unwrap_or(1.0);

        self.vault.store_text(fact, category, confidence)?;
        
        if confidence < 0.85 {
            Ok(format!("Fact quarantined due to low confidence ({}). Awaiting human review.", confidence))
        } else {
            Ok(format!("Successfully stored fact in Native Memory (Category: {}): {}", category, fact))
        }
    }
}

/// Tool to search the Native Memory (SqliteVault).
pub struct MemorySearchTool {
    pub vault: Arc<SqliteVault>,
}

#[async_trait]
impl ToolHandler for MemorySearchTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "memory_search".to_string(),
            description: "Recall relevant context, past failures, or learned entities from the permanent local Knowledge Vault using keywords.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The keyword string to search for historically."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max number of results to return (default 5, max 20)"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<String> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'query' parameter"))?;
            
        let limit = args["limit"]
            .as_u64()
            .unwrap_or(5)
            .min(20) as usize;

        // Since we killed the embedding node, we rely on Native Rust BM25 text search
        let results = self.vault.keyword_search(query, limit)?;
        
        if results.is_empty() {
            return Ok(format!("No relevant memories found for query: '{}'", query));
        }
        
        let mut out = String::new();
        out.push_str(&format!("Vault Results for '{}':\n\n", query));
        for (fact, score) in results {
            let dt = fact.created_at.format("%Y-%m-%d").to_string();
            out.push_str(&format!("- [{}] (Score: {:.2}) {}\n", dt, score, fact.content));
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio;

    #[tokio::test]
    async fn test_native_memory_tools() {
        let mut tmp_dir = env::temp_dir();
        tmp_dir.push(format!("test_vault_{}.db", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()));
        
        let vault = Arc::new(SqliteVault::open(&tmp_dir).unwrap());
        
        let record_tool = MemoryRecordTool { vault: Arc::clone(&vault) };
        let search_tool = MemorySearchTool { vault: Arc::clone(&vault) };
        
        let exec_res = record_tool.execute(json!({
            "fact": "The user appreciates blunt feedback.",
            "category": "Core"
        })).await.unwrap();
        
        assert!(exec_res.contains("Category: Core"));
        assert!(exec_res.contains("The user appreciates blunt feedback."));
        
        let search_res = search_tool.execute(json!({
            "query": "blunt",
            "limit": 5
        })).await.unwrap();
        
        assert!(search_res.contains("The user appreciates blunt feedback."));
        
        let _ = std::fs::remove_file(tmp_dir);
    }
}
