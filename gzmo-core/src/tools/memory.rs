use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use super::{ToolDef, ToolHandler};
use crate::control_plane::ControlPlaneClient;
use crate::memory::scratch::{ScratchScope, ScratchService};
use crate::memory::vault::SqliteVault;

/// Fail closed: blank query or no vault → error, never invented facts.
fn require_searchable(query: &str, vault_present: bool) -> Result<&str> {
    let q = query.trim();
    if q.is_empty() {
        return Err(anyhow!("empty query"));
    }
    if !vault_present {
        return Err(anyhow!("missing vault"));
    }
    Ok(q)
}

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

        let category = args["category"].as_str().unwrap_or("Episodic");

        let confidence = args["confidence"].as_f64().unwrap_or(1.0);

        self.vault.store_text(fact, category, confidence)?;

        if confidence < 0.85 {
            Ok(format!(
                "Fact quarantined due to low confidence ({}). Awaiting human review.",
                confidence
            ))
        } else {
            Ok(format!(
                "Successfully stored fact in Native Memory (Category: {}): {}",
                category, fact
            ))
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
}

impl MemorySearchTool {
    pub fn new(vault: Arc<SqliteVault>) -> Self {
        Self {
            vault,
            scratch: None,
            scope: None,
            scope_cell: None,
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
        }
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
        let query = require_searchable(query, true)?;

        let limit = args["limit"].as_u64().unwrap_or(5).min(20) as usize;

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

        let (text, results) =
            crate::platform_memory::memory_search_core(&self.vault, query, limit).await?;
        crate::memory::felt_use::touch_hits(
            &self.vault,
            results.iter().map(|(f, _)| Some(&f.id)),
            crate::memory::felt_use::FeltUseKind::Glance,
        );
        Ok(text)
    }
}

/// `memory_search` via the owner socket — chat/REPL must not open the living vault.
pub struct OwnerMemorySearchTool {
    pub client: ControlPlaneClient,
}

#[async_trait]
impl ToolHandler for OwnerMemorySearchTool {
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
        let query = require_searchable(query, true)?;
        let limit = args["limit"].as_u64().unwrap_or(5).min(20) as usize;
        let res = self.client.search(query, limit, true).await?;
        Ok(res.text)
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
        tmp_dir.push(format!(
            "test_vault_{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros()
        ));

        let vault = Arc::new(SqliteVault::open(&tmp_dir).unwrap());

        let record_tool = MemoryRecordTool {
            vault: Arc::clone(&vault),
        };
        let search_tool = MemorySearchTool::new(Arc::clone(&vault));

        let exec_res = record_tool
            .execute(json!({
                "fact": "The user appreciates blunt feedback.",
                "category": "Core"
            }))
            .await
            .unwrap();

        assert!(exec_res.contains("Category: Core"));
        assert!(exec_res.contains("The user appreciates blunt feedback."));

        let search_res = search_tool
            .execute(json!({
                "query": "blunt",
                "limit": 5
            }))
            .await
            .unwrap();

        assert!(search_res.contains("The user appreciates blunt feedback."));

        let empty = search_tool
            .execute(json!({ "query": "" }))
            .await
            .expect_err("empty query must fail closed");
        assert!(
            !empty
                .to_string()
                .contains("The user appreciates blunt feedback."),
            "empty query must not invent recall: {empty}"
        );

        let ws = search_tool
            .execute(json!({ "query": "   \n\t  " }))
            .await
            .expect_err("whitespace query must fail closed");
        assert!(!ws
            .to_string()
            .contains("The user appreciates blunt feedback."));

        let missing_q = search_tool
            .execute(json!({}))
            .await
            .expect_err("missing query must fail closed");
        assert!(!missing_q
            .to_string()
            .contains("The user appreciates blunt feedback."));

        let _ = std::fs::remove_file(tmp_dir);
    }

    #[test]
    fn empty_query_and_missing_vault_fail_closed() {
        assert!(
            require_searchable("", true).is_err(),
            "empty query must fail closed — do not invent recall"
        );
        assert!(require_searchable("   \n", true).is_err());
        assert!(
            require_searchable("blunt", false).is_err(),
            "missing vault must fail closed — no fake facts"
        );
        assert_eq!(require_searchable("blunt", true).unwrap(), "blunt");
    }

    #[tokio::test]
    async fn owner_empty_query_fails_closed_before_socket() {
        let tool = OwnerMemorySearchTool {
            client: ControlPlaneClient::new(std::path::PathBuf::from("/no/such/gzmo.sock"), None),
        };
        let err = tool
            .execute(json!({ "query": "" }))
            .await
            .expect_err("empty query must not hit a missing socket as live recall");
        assert!(
            err.to_string().contains("empty query"),
            "must fail on query, not socket: {err}"
        );
    }
}
