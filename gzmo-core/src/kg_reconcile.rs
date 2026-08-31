//! Gated Neo4j ontology reconciliation via shared MCP memory tools.

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{info, warn};

use crate::config::KgReconcileConfig;
use crate::gateway::ToolCall;
use crate::memory::kg_promotion::canonicalize_relation_type;
use crate::tools::{ToolRegistry, ToolResult};

const CANONICAL_ENTITY_TYPES: &[&str] = &["PEOPLE", "SYSTEMS", "PROJECTS", "TOOLS", "DECISIONS"];

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphEntity {
    name: String,
    #[serde(rename = "type")]
    entity_type: String,
    #[serde(default)]
    observations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GraphRelation {
    source: String,
    target: String,
    #[serde(rename = "relationType")]
    relation_type: String,
}

#[derive(Debug, Deserialize)]
struct KnowledgeGraph {
    #[serde(default)]
    entities: Vec<GraphEntity>,
    #[serde(default)]
    relations: Vec<GraphRelation>,
}

#[derive(Debug, Clone, Default)]
pub struct ReconcileReport {
    pub entities_scanned: usize,
    pub relations_scanned: usize,
    pub entity_notes_added: usize,
    pub relations_recanonicalized: usize,
    pub relations_deleted: usize,
    pub dry_run: bool,
}

/// Canonicalize entity type labels toward the shared ontology.
pub fn canonicalize_entity_type(raw: &str) -> String {
    let upper = raw.trim().to_uppercase();
    if upper.is_empty() {
        return String::new();
    }
    if CANONICAL_ENTITY_TYPES.contains(&upper.as_str()) {
        return upper;
    }
    match upper.as_str() {
        "PERSON" | "PEOPLE" | "HUMAN" | "USER" => "PEOPLE".to_string(),
        "SYSTEM" | "SERVICE" | "INFRA" | "INFRASTRUCTURE" => "SYSTEMS".to_string(),
        "PROJECT" | "REPO" | "CODEBASE" => "PROJECTS".to_string(),
        "TOOL" | "CLI" | "APP" | "APPLICATION" => "TOOLS".to_string(),
        "DECISION" | "POLICY" | "CHOICE" => "DECISIONS".to_string(),
        "COMPANY" | "ORG" | "ORGANIZATION" | "LOCATION" | "CONCEPT" | "EVENT" => {
            "SYSTEMS".to_string()
        }
        _ => upper,
    }
}

pub async fn run_kg_reconcile(
    tools: &ToolRegistry,
    cfg: &KgReconcileConfig,
) -> Result<ReconcileReport> {
    if !tools.has_tool("mcp__memory__read_graph") {
        anyhow::bail!("mcp__memory__read_graph not available — connect Neo4j MCP first");
    }

    let call = ToolCall {
        id: "kg_reconcile_read".into(),
        function_name: "mcp__memory__read_graph".to_string(),
        arguments: serde_json::json!({}),
    };
    let result = tools.dispatch(&call).await;
    if !result.success {
        anyhow::bail!("read_graph failed: {}", result.output);
    }

    let output = result.output.trim();
    if output.is_empty() {
        return Ok(ReconcileReport {
            dry_run: cfg.dry_run,
            ..Default::default()
        });
    }

    let graph: KnowledgeGraph = serde_json::from_str(output).context("parse read_graph JSON")?;

    let mut report = ReconcileReport {
        entities_scanned: graph.entities.len(),
        relations_scanned: graph.relations.len(),
        dry_run: cfg.dry_run,
        ..Default::default()
    };

    for entity in &graph.entities {
        let canonical = canonicalize_entity_type(&entity.entity_type);
        if canonical.is_empty() {
            continue; // Fail closed, no-op for empty entity types
        }
        if canonical == entity.entity_type.to_uppercase() {
            continue;
        }
        let note = format!(
            "[ontology] canonical type {} (was {})",
            canonical, entity.entity_type
        );
        if cfg.dry_run {
            info!(entity = %entity.name, from = %entity.entity_type, to = %canonical, "dry-run entity note");
            report.entity_notes_added += 1;
            continue;
        }
        if tools.has_tool("mcp__memory__add_observations") {
            let call = ToolCall {
                id: format!("kg_note_{}", entity.name),
                function_name: "mcp__memory__add_observations".to_string(),
                arguments: serde_json::json!({
                    "observations": [{
                        "entityName": entity.name,
                        "contents": [note]
                    }]
                }),
            };
            if let ToolResult { success: true, .. } = tools.dispatch(&call).await {
                report.entity_notes_added += 1;
            }
        }
    }

    let mut to_delete: Vec<GraphRelation> = Vec::new();
    let mut to_create: Vec<serde_json::Value> = Vec::new();

    for rel in &graph.relations {
        // Find out if the relation type sanitizes to empty BEFORE it falls back to RELATED_TO.
        let is_empty_sanitized = {
            let mut out = String::with_capacity(rel.relation_type.len());
            let mut prev_underscore = false;
            for ch in rel.relation_type.chars() {
                if ch.is_ascii_alphanumeric() {
                    out.push(ch.to_ascii_uppercase());
                    prev_underscore = false;
                } else if !prev_underscore {
                    out.push('_');
                    prev_underscore = true;
                }
            }
            out.trim_matches('_').is_empty()
        };

        if is_empty_sanitized {
            continue;
        }

        let canon = canonicalize_relation_type(&rel.relation_type);
        if canon.is_empty() {
            continue;
        }

        if canon == rel.relation_type {
            continue;
        }
        to_delete.push(rel.clone());
        to_create.push(serde_json::json!({
            "source": rel.source,
            "target": rel.target,
            "relationType": canon,
        }));
    }

    if cfg.dry_run {
        report.relations_recanonicalized = to_create.len();
        info!(
            entities = report.entities_scanned,
            relations = report.relations_scanned,
            would_fix = to_create.len(),
            "KG reconcile dry-run complete"
        );
        return Ok(report);
    }

    if !to_delete.is_empty() && tools.has_tool("mcp__memory__delete_relations") {
        let payload: Vec<serde_json::Value> = to_delete
            .iter()
            .map(|r| {
                serde_json::json!({
                    "source": r.source,
                    "target": r.target,
                    "relationType": r.relation_type,
                })
            })
            .collect();
        let call = ToolCall {
            id: "kg_reconcile_del".into(),
            function_name: "mcp__memory__delete_relations".to_string(),
            arguments: serde_json::json!({ "relations": payload }),
        };
        if let ToolResult { success: true, .. } = tools.dispatch(&call).await {
            report.relations_deleted = to_delete.len();
        } else {
            warn!("delete_relations batch failed");
        }
    }

    if !to_create.is_empty() && tools.has_tool("mcp__memory__create_relations") {
        for chunk in to_create.chunks(20) {
            let call = ToolCall {
                id: format!("kg_reconcile_create_{}", chunk.len()),
                function_name: "mcp__memory__create_relations".to_string(),
                arguments: serde_json::json!({ "relations": chunk }),
            };
            if let ToolResult { success: true, .. } = tools.dispatch(&call).await {
                report.relations_recanonicalized += chunk.len();
            }
        }
    }

    info!(
        entities = report.entities_scanned,
        relations_fixed = report.relations_recanonicalized,
        "KG reconcile complete"
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolDef;
    use crate::tools::ToolHandler;
    use async_trait::async_trait;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn canonicalize_entity_types() {
        assert_eq!(canonicalize_entity_type("person"), "PEOPLE");
        assert_eq!(canonicalize_entity_type("company"), "SYSTEMS");
        assert_eq!(canonicalize_entity_type("PROJECTS"), "PROJECTS");
        assert_eq!(canonicalize_entity_type(""), "");
        assert_eq!(canonicalize_entity_type("   "), "");
    }

    #[test]
    fn relation_canonicalization_matches_kg_promotion() {
        assert_eq!(canonicalize_relation_type("WROTE"), "AUTHORED_BY");
    }

    struct MockReadGraphTool {
        output: String,
    }

    #[async_trait]
    impl ToolHandler for MockReadGraphTool {
        fn definition(&self) -> ToolDef {
            ToolDef {
                name: "mcp__memory__read_graph".to_string(),
                description: "".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<String> {
            Ok(self.output.clone())
        }
    }

    #[tokio::test]
    async fn test_empty_graph_is_noop() {
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(MockReadGraphTool {
            output: "   ".to_string(),
        }));

        let cfg = KgReconcileConfig {
            dry_run: false,
            ..Default::default()
        };
        let report = run_kg_reconcile(&tools, &cfg).await.unwrap();

        assert_eq!(report.entities_scanned, 0);
        assert_eq!(report.relations_scanned, 0);
    }

    struct MockAddObsTool {
        called: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl ToolHandler for MockAddObsTool {
        fn definition(&self) -> ToolDef {
            ToolDef {
                name: "mcp__memory__add_observations".to_string(),
                description: "".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<String> {
            *self.called.lock().await = true;
            Ok("ok".to_string())
        }
    }

    struct MockCreateRelTool {
        called: Arc<Mutex<bool>>,
    }

    #[async_trait]
    impl ToolHandler for MockCreateRelTool {
        fn definition(&self) -> ToolDef {
            ToolDef {
                name: "mcp__memory__create_relations".to_string(),
                description: "".to_string(),
                parameters: serde_json::json!({}),
            }
        }
        async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<String> {
            *self.called.lock().await = true;
            Ok("ok".to_string())
        }
    }

    #[tokio::test]
    async fn test_empty_relation_and_entity_types_are_noops() {
        let mut tools = ToolRegistry::new();
        let graph_json = serde_json::json!({
            "entities": [
                {
                    "name": "E1",
                    "type": "   ",
                    "observations": []
                }
            ],
            "relations": [
                {
                    "source": "E1",
                    "target": "E2",
                    "relationType": "!@#"
                }
            ]
        });

        tools.register(Box::new(MockReadGraphTool {
            output: graph_json.to_string(),
        }));

        let obs_called = Arc::new(Mutex::new(false));
        tools.register(Box::new(MockAddObsTool {
            called: obs_called.clone(),
        }));

        let create_rel_called = Arc::new(Mutex::new(false));
        tools.register(Box::new(MockCreateRelTool {
            called: create_rel_called.clone(),
        }));

        let cfg = KgReconcileConfig {
            dry_run: false,
            ..Default::default()
        };
        let report = run_kg_reconcile(&tools, &cfg).await.unwrap();

        assert_eq!(report.entities_scanned, 1);
        assert_eq!(report.relations_scanned, 1);
        assert_eq!(report.entity_notes_added, 0);
        assert_eq!(report.relations_recanonicalized, 0);

        assert!(!*obs_called.lock().await);
        assert!(!*create_rel_called.lock().await);
    }
}
