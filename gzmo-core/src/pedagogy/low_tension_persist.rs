//! Persist low-tension Socratic dialogues to Neo4j via MCP memory tools.

use anyhow::Result;
use gzmo_chaos::pulse::ChaosSnapshot;
use tracing::warn;
use uuid::Uuid;

use crate::gateway::ToolCall;
use crate::memory::kg_promotion::is_valid_relation_endpoints;
use crate::pedagogy::low_tension_opening::LowTensionOpening;
use crate::tools::{ToolRegistry, ToolResult};

/// Write a SOCRATIC_DIALOGUE entity and link it to concepts + learner.
pub async fn persist_socratic_dialogue(
    tools: &ToolRegistry,
    learner_id: &str,
    opening: &LowTensionOpening,
    response: Option<&str>,
    snap: &ChaosSnapshot,
    trigger: &str,
) -> Result<()> {
    if !tools.has_tool("mcp__memory__create_entities") {
        return Ok(());
    }

    let dialogue_name = format!(
        "SocraticDialogue-{}-{}",
        snap.tick,
        &Uuid::new_v4().to_string()[..8]
    );
    let mut observations = vec![
        format!(
            "[low_tension] trigger={trigger} tension={:.1}% tick={} phase={}",
            snap.tension, snap.tick, snap.phase
        ),
        format!(
            "[opening] {}",
            opening.prompt.chars().take(500).collect::<String>()
        ),
    ];
    if let Some(r) = response {
        observations.push(format!(
            "[response] {}",
            r.chars().take(500).collect::<String>()
        ));
    }
    for (id, title) in opening
        .concept_ids
        .iter()
        .zip(opening.concept_titles.iter())
    {
        observations.push(format!("[concept] {title} ({id})"));
    }

    let entity_call = ToolCall {
        id: format!("ltd_entity_{}", Uuid::new_v4()),
        function_name: "mcp__memory__create_entities".to_string(),
        arguments: serde_json::json!({
            "entities": [{
                "name": dialogue_name,
                "type": "SOCRATIC_DIALOGUE",
                "observations": observations,
            }]
        }),
    };
    match tools.dispatch(&entity_call).await {
        ToolResult { success: true, .. } => {}
        ToolResult { output, .. } => {
            warn!("SOCRATIC_DIALOGUE entity write failed: {output}");
            return Ok(());
        }
    }

    if !tools.has_tool("mcp__memory__create_relations") {
        return Ok(());
    }

    let learner_name = format!("Learner-{learner_id}");
    let _ = tools
        .dispatch(&ToolCall {
            id: format!("ltd_learner_{}", Uuid::new_v4()),
            function_name: "mcp__memory__create_entities".to_string(),
            arguments: serde_json::json!({
                "entities": [{
                    "name": learner_name,
                    "type": "LEARNER",
                    "observations": [format!("[learner_id] {learner_id}")],
                }]
            }),
        })
        .await;

    let mut relations = Vec::new();
    if is_valid_relation_endpoints(&dialogue_name, &learner_name, "DIALOGUE_WITH") {
        relations.push(serde_json::json!({
            "source": dialogue_name,
            "target": learner_name,
            "relationType": "DIALOGUE_WITH",
        }));
    }
    for (id, title) in opening
        .concept_ids
        .iter()
        .zip(opening.concept_titles.iter())
    {
        let concept_name = if title.len() >= 2 {
            title.clone()
        } else {
            id.clone()
        };
        let _ = tools
            .dispatch(&ToolCall {
                id: format!("ltd_concept_{}", Uuid::new_v4()),
                function_name: "mcp__memory__create_entities".to_string(),
                arguments: serde_json::json!({
                    "entities": [{
                        "name": concept_name,
                        "type": "CONCEPT",
                        "observations": [format!("[graph_node] {id}")],
                    }]
                }),
            })
            .await;
        if is_valid_relation_endpoints(&dialogue_name, &concept_name, "DIALOGUE_ABOUT") {
            relations.push(serde_json::json!({
                "source": dialogue_name,
                "target": concept_name,
                "relationType": "DIALOGUE_ABOUT",
            }));
        }
    }

    if relations.is_empty() {
        return Ok(());
    }

    let rel_call = ToolCall {
        id: format!("ltd_rel_{}", Uuid::new_v4()),
        function_name: "mcp__memory__create_relations".to_string(),
        arguments: serde_json::json!({ "relations": relations }),
    };
    if let ToolResult {
        success: false,
        output,
        ..
    } = tools.dispatch(&rel_call).await
    {
        warn!("SOCRATIC_DIALOGUE relations write failed: {output}");
    }
    Ok(())
}

/// Search Neo4j for prior low-tension openings about the same concepts.
pub async fn prior_opening_hints(tools: &ToolRegistry, concept_ids: &[String]) -> Vec<String> {
    if !tools.has_tool("mcp__memory__search_memories") || concept_ids.is_empty() {
        return vec![];
    }
    let query = format!("SOCRATIC_DIALOGUE low_tension {}", concept_ids.join(" "));
    let call = ToolCall {
        id: format!("ltd_search_{}", Uuid::new_v4()),
        function_name: "mcp__memory__search_memories".to_string(),
        arguments: serde_json::json!({ "query": query, "limit": 5 }),
    };
    let ToolResult {
        success: true,
        output,
        ..
    } = tools.dispatch(&call).await
    else {
        return vec![];
    };
    parse_opening_hints(&output)
}

fn parse_opening_hints(search_output: &str) -> Vec<String> {
    let mut hints = Vec::new();
    for line in search_output.lines() {
        if let Some(rest) = line.strip_prefix("[opening]") {
            let stem: String = rest.trim().chars().take(120).collect();
            if !stem.is_empty() {
                hints.push(stem);
            }
        }
    }
    hints.truncate(3);
    hints
}
