//! Gated Neo4j ontology reconciliation via shared MCP memory tools.

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{info, warn};

use crate::config::KgReconcileConfig;
use crate::gateway::ToolCall;
use crate::memory::kg_promotion::canonicalize_relation_type;
use crate::tools::{ToolRegistry, ToolResult};

const CANONICAL_ENTITY_TYPES: &[&str] = &[
    "PEOPLE", "SYSTEMS", "PROJECTS", "TOOLS", "DECISIONS",
];

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
        _ if !upper.is_empty() => upper,
        _ => "SYSTEMS".to_string(),
    }
}

/// Format MCP `read_graph` JSON (summary or full) for chat/TUI boot context injection.
pub fn format_knowledge_graph_boot_context(output: &str) -> Option<String> {
    let graph: serde_json::Value = serde_json::from_str(output).ok()?;
    let mut block = String::from("\n\n## Persistent Memory (Knowledge Graph)\n\n");
    let mut has_content = false;

    if graph.get("summary").and_then(|s| s.as_bool()) == Some(true) {
        let entities = graph.get("entity_count").and_then(|n| n.as_u64()).unwrap_or(0);
        let relations = graph.get("relation_count").and_then(|n| n.as_u64()).unwrap_or(0);
        let observations = graph
            .get("observation_count")
            .and_then(|n| n.as_u64())
            .unwrap_or(0);
        block.push_str(&format!(
            "- **Overview:** {entities} entities, {relations} relations, {observations} observations\n"
        ));
        has_content = true;

        if let Some(types) = graph.get("entity_types").and_then(|t| t.as_object()) {
            let mut pairs: Vec<(String, u64)> = types
                .iter()
                .filter_map(|(k, v)| v.as_u64().map(|n| (k.clone(), n)))
                .collect();
            pairs.sort_by(|a, b| b.1.cmp(&a.1));
            if !pairs.is_empty() {
                block.push_str("- **Top types:** ");
                let top: Vec<String> = pairs
                    .iter()
                    .take(8)
                    .map(|(t, n)| format!("{t} ({n})"))
                    .collect();
                block.push_str(&top.join(", "));
                block.push('\n');
            }
        }

        if let Some(samples) = graph.get("sample_entities").and_then(|s| s.as_array()) {
            if !samples.is_empty() {
                block.push_str("- **Sample entities:** ");
                let names: Vec<String> = samples
                    .iter()
                    .take(12)
                    .filter_map(|e| {
                        let name = e.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                        let etype = e.get("type").and_then(|t| t.as_str()).unwrap_or("?");
                        Some(format!("{name} ({etype})"))
                    })
                    .collect();
                block.push_str(&names.join(", "));
                block.push('\n');
            }
        }

        if let Some(hint) = graph.get("hint").and_then(|h| h.as_str()) {
            block.push_str(&format!("- **Hint:** {hint}\n"));
        }
    } else if let Some(entities) = graph.get("entities").and_then(|e| e.as_array()) {
        for entity in entities {
            let name = entity.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let etype = entity
                .get("type")
                .or_else(|| entity.get("entityType"))
                .and_then(|t| t.as_str())
                .unwrap_or("?");
            block.push_str(&format!("- **{}** ({})", name, etype));
            if let Some(obs) = entity.get("observations").and_then(|o| o.as_array()) {
                let obs_strs: Vec<&str> = obs.iter().filter_map(|o| o.as_str()).collect();
                if !obs_strs.is_empty() {
                    block.push_str(&format!(": {}", obs_strs.join("; ")));
                }
            }
            block.push('\n');
            has_content = true;
        }

        if let Some(relations) = graph.get("relations").and_then(|r| r.as_array()) {
            if !relations.is_empty() {
                block.push_str("\nRelationships:\n");
                for rel in relations {
                    let from = rel
                        .get("source")
                        .or_else(|| rel.get("from"))
                        .and_then(|f| f.as_str())
                        .unwrap_or("?");
                    let to = rel
                        .get("target")
                        .or_else(|| rel.get("to"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("?");
                    let rtype = rel
                        .get("type")
                        .or_else(|| rel.get("relationType"))
                        .and_then(|r| r.as_str())
                        .unwrap_or("?");
                    block.push_str(&format!("- {} -> ({}) -> {}\n", from, rtype, to));
                    has_content = true;
                }
            }
        }
    }

    if has_content {
        Some(block)
    } else {
        None
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
        arguments: serde_json::json!({ "full": true }),
    };
    let result = tools.dispatch(&call).await;
    if !result.success {
        anyhow::bail!("read_graph failed: {}", result.output);
    }

    let graph: KnowledgeGraph =
        serde_json::from_str(&result.output).context("parse read_graph JSON")?;

    let mut report = ReconcileReport {
        entities_scanned: graph.entities.len(),
        relations_scanned: graph.relations.len(),
        dry_run: cfg.dry_run,
        ..Default::default()
    };

    for entity in &graph.entities {
        let canonical = canonicalize_entity_type(&entity.entity_type);
        if canonical == entity.entity_type.to_uppercase() {
            continue;
        }
        let note = format!(
            "[ontology] canonical type {} (was {})",
            canonical,
            entity.entity_type
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
                        "observations": [note]
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

    #[test]
    fn canonicalize_entity_types() {
        assert_eq!(canonicalize_entity_type("person"), "PEOPLE");
        assert_eq!(canonicalize_entity_type("company"), "SYSTEMS");
        assert_eq!(canonicalize_entity_type("PROJECTS"), "PROJECTS");
    }

    #[test]
    fn relation_canonicalization_matches_kg_promotion() {
        assert_eq!(canonicalize_relation_type("WROTE"), "AUTHORED_BY");
    }

    #[test]
    fn boot_context_formats_summary_payload() {
        let summary = r#"{"summary":true,"entity_count":10,"relation_count":5,"observation_count":20,"entity_types":{"SYSTEM":3},"sample_entities":[{"name":"Neo4j","type":"SYSTEM","observation_count":2}],"hint":"use search"}"#;
        let block = format_knowledge_graph_boot_context(summary).expect("summary block");
        assert!(block.contains("10 entities"));
        assert!(block.contains("Neo4j"));
        assert!(block.contains("use search"));
    }
}
