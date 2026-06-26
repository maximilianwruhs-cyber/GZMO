use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use crate::memory::scratch::{ScratchScope, ScratchService};
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
    pub scratch: Option<Arc<ScratchService>>,
    pub scope: Option<ScratchScope>,
    /// Orchestrator/daemon: scope updated per job step via shared cell.
    pub scope_cell: Option<Arc<std::sync::Mutex<ScratchScope>>>,
    /// Optional wiki search layer — wiki hits are appended to recall results
    /// so the LLM can access curated synthesis pages alongside vault facts.
    pub wiki: Option<crate::wiki::WikiEngine>,
}

impl MemorySearchTool {
    pub fn new(vault: Arc<SqliteVault>) -> Self {
        Self {
            vault,
            scratch: None,
            scope: None,
            scope_cell: None,
            wiki: None,
        }
    }

    pub fn with_scratch(
        vault: Arc<SqliteVault>,
        scratch: Arc<ScratchService>,
        scope: ScratchScope,
    ) -> Self {
        Self {
            vault,
            scratch: Some(scratch),
            scope: Some(scope),
            scope_cell: None,
            wiki: None,
        }
    }

    pub fn with_orchestrator_scratch(
        vault: Arc<SqliteVault>,
        scratch: Arc<ScratchService>,
        scope_cell: Arc<std::sync::Mutex<ScratchScope>>,
    ) -> Self {
        Self {
            vault,
            scratch: Some(scratch),
            scope: None,
            scope_cell: Some(scope_cell),
            wiki: None,
        }
    }

    /// Attach a wiki engine for fused vault+wiki search.
    pub fn with_wiki(mut self, wiki: crate::wiki::WikiEngine) -> Self {
        self.wiki = Some(wiki);
        self
    }

    fn effective_scope(&self) -> Option<ScratchScope> {
        if let Some(cell) = &self.scope_cell {
            return cell.lock().ok().map(|g| g.clone());
        }
        self.scope.clone()
    }
}

#[async_trait]
impl ToolHandler for MemorySearchTool {
    fn definition(&self) -> ToolDef {
        ToolDef {
            name: "memory_search".to_string(),
            description: "Recall relevant context from the curated honeypot memory layer (high-confidence facts) using keywords.".to_string(),
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

        if let (Some(scratch), Some(scope)) = (&self.scratch, self.effective_scope()) {
            return crate::platform_memory::memory_search_into_scratch(
                &self.vault,
                Arc::clone(scratch),
                &scope,
                query,
                limit,
            )
            .await;
        }

        let results = self.vault.search_recall(query, limit).await?;
        if results.is_empty() && self.wiki.is_none() {
            return Ok(format!("No relevant memories found for query: '{}'", query));
        }
        let mut out = String::new();
        out.push_str(&format!("Honeypot recall for '{}':\n\n", query));
        for (fact, score) in &results {
            let dt = fact.created_at.format("%Y-%m-%d").to_string();
            out.push_str(&format!("- [{}] (Score: {:.2}) {}\n", dt, score, fact.content));
        }

        // Append wiki search results as a bonus stream
        if let Some(wiki) = &self.wiki {
            let wiki_hits = wiki.search(query, 3);
            if !wiki_hits.is_empty() {
                if results.is_empty() {
                    out.push_str(&format!("Honeypot recall for '{}':\n\n", query));
                }
                out.push_str("\n--- Wiki Knowledge Base ---\n");
                for hit in wiki_hits {
                    out.push_str(&format!(
                        "- **{}** (wiki/{}) — {}\n",
                        hit.title, hit.path, hit.snippet.chars().take(120).collect::<String>()
                    ));
                }
            }
        }

        if results.is_empty() && self.wiki.is_some() {
            // Only wiki results found — note that vault was empty
            out = format!("Honeypot recall for '{}':\n\n(no vault hits — wiki only)\n{}", query, out);
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
        let search_tool = MemorySearchTool::new(Arc::clone(&vault));
        
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
