//! Shared bootstrap helpers for stdio chat and `--repl` TUI parity.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::tools::ToolRegistry;
use gzmo_core::types::{Message, Role, SoulContext};
use gzmo_core::workflow_skills::{
    load_from_config, SharedWorkflowSession, WorkflowSessionState, WorkflowSkillIndex,
};

/// Boot the Knowledge Graph MCP and return a context block for the system prompt.
pub async fn boot_knowledge_graph(tools: &ToolRegistry) -> Option<String> {
    let call = gzmo_core::gateway::ToolCall {
        id: "boot_kg_read".to_string(),
        function_name: "mcp__memory__read_graph".to_string(),
        arguments: serde_json::json!({}),
    };
    let result = tools.dispatch(&call).await;
    if !result.success || result.output.trim().is_empty() {
        return None;
    }

    let graph: serde_json::Value = serde_json::from_str(&result.output).ok()?;
    let mut block = String::from("\n\n## Persistent Memory (Knowledge Graph)\n\n");
    let mut has_content = false;

    if let Some(entities) = graph.get("entities").and_then(|e| e.as_array()) {
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

    if has_content {
        Some(block)
    } else {
        None
    }
}

/// Build the operator system prompt shared by stdio chat and TUI.
#[allow(dead_code)] // kept for callers that do not pass workflow context
pub fn build_system_prompt(
    soul: &SoulContext,
    memory_context: Option<&str>,
    vault_context: Option<&str>,
    tool_names: &[String],
    today: &str,
) -> String {
    build_system_prompt_with_workflows(
        soul,
        memory_context,
        vault_context,
        tool_names,
        today,
        None,
        None,
    )
}

/// System prompt including workflow skill index and optional last-handoff pointer.
pub fn build_system_prompt_with_workflows(
    soul: &SoulContext,
    memory_context: Option<&str>,
    vault_context: Option<&str>,
    tool_names: &[String],
    today: &str,
    workflows: Option<&WorkflowSkillIndex>,
    last_handoff: Option<&std::path::Path>,
) -> String {
    let wf_block = workflows.map(|w| w.prompt_index_block()).unwrap_or_default();
    let handoff_block = last_handoff
        .map(|p| {
            format!(
                "\n\n## Latest handoff\nPrevious session left a handoff at `{}`. \
                 Read it with file_read if continuing that work.",
                p.display()
            )
        })
        .unwrap_or_default();
    format!(
        "{}{}{}{}{}

---
You are {}. Today is {}.
Available tools: {}.
Use ecosystem_status for a grounded stack/overnight snapshot (slash /status is operator-only).
Use memory_search when you need prior facts (results land in scratch for this turn only).
Use delegate_task for focused sub-work; you receive a short summary, not full subagent logs.
Use activate_workflow_skill (or slash /grill /tdd /diagnose /review /handoff) for engineering discipline.",
        soul.raw_markdown,
        memory_context.unwrap_or(""),
        vault_context.unwrap_or(""),
        wf_block,
        handoff_block,
        soul.persona_name,
        today,
        if tool_names.is_empty() {
            "none".to_string()
        } else {
            tool_names.join(", ")
        }
    )
}

/// Load workflow skill index + empty session state from config.
pub fn boot_workflow_skills(
    config: &GzmoConfig,
) -> Result<(Arc<WorkflowSkillIndex>, SharedWorkflowSession)> {
    let index = Arc::new(load_from_config(&config.workflow_skills)?);
    let session = Arc::new(Mutex::new(WorkflowSessionState::default()));
    Ok((index, session))
}

/// Activate a workflow skill and push inject message into `messages`.
/// Returns Ok(true) if activated.
pub fn activate_workflow_slash(
    index: &WorkflowSkillIndex,
    session: &SharedWorkflowSession,
    name: &str,
    args: &str,
    messages: &mut Vec<Message>,
) -> Result<bool> {
    if !index.has(name) {
        return Ok(false);
    }
    let inject = index.activate(session, name, args)?;
    messages.push(Message {
        role: Role::System,
        content: inject,
        is_meta: true,
        tool_calls: None,
        tool_call_id: None,
    });
    Ok(true)
}

/// Status lines for active workflows / last handoff (for `/status` surfaces).
pub fn workflow_status_lines(
    index: &WorkflowSkillIndex,
    session: &SharedWorkflowSession,
) -> Vec<String> {
    let mut lines = Vec::new();
    if index.is_empty() {
        lines.push("Workflow skills: (none loaded)".into());
        return lines;
    }
    lines.push(format!(
        "Workflow skills: {} ({})",
        index.names().join(", "),
        index.dir().display()
    ));
    if let Ok(state) = session.lock() {
        if state.active_names().is_empty() {
            lines.push("Active workflows: (none)".into());
        } else {
            lines.push(format!(
                "Active workflows: {}",
                state.active_names().join(", ")
            ));
        }
        if let Some(ref p) = state.last_handoff {
            lines.push(format!("Last handoff: {}", p.display()));
        } else if let Some(p) = index.latest_handoff() {
            lines.push(format!("Latest handoff on disk: {}", p.display()));
        }
    }
    lines
}

/// Open the semantic vault (embed + rerank) used by memory_search.
pub async fn open_semantic_vault(config: &GzmoConfig) -> Option<Arc<SqliteVault>> {
    match embeddings::open_vault_with_embeddings(
        &config.memory.vault_db,
        &config.embeddings,
        &config.redis,
        &config.rerank,
        &config.qdrant,
    )
    .await
    {
        Ok(v) => Some(Arc::new(v)),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to open vault — continuing without it");
            None
        }
    }
}

/// Ping the configured LLM engine `/models` endpoint.
pub async fn ping_engine(config: &GzmoConfig) -> (&'static str, String) {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let active = config.engine.active_engine();
    let health_url = format!("{}/models", active.url);
    let start = std::time::Instant::now();

    for _ in 0..15 {
        let req = http.get(&health_url);
        let req = if !active.api_key.is_empty() {
            req.bearer_auth(&active.api_key)
        } else {
            req
        };

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                return ("ONLINE", format!("{}ms", start.elapsed().as_millis()));
            }
            _ => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
        }
    }
    ("OFFLINE", String::new())
}
