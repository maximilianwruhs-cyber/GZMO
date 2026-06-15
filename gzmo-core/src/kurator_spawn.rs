//! Kurator phase 3 — governed sub-agent spawn (manual approve + daemon autospawn).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;

use crate::config::{GzmoConfig, KuratorConfig};
use crate::context_compress::CcrStore;
use crate::gateway::LlmGateway;
use crate::kurator_monitor::{
    mark_recommendation_spawned, take_pending_recommendation, PendingRecommendation,
};
use crate::memory::scratch::ScratchService;
use crate::memory::vault::SqliteVault;
use crate::subagent::{SubagentRunner, SubagentSpec};
use crate::synapse::SynapseBus;
use crate::synapse_writer::{emit_agent_result, emit_agent_spawned, ForumThread};
use crate::text_util::truncate_chars;

pub fn synapse_bus_path(config: &GzmoConfig) -> PathBuf {
    std::env::var("GZMO_SYNAPSE_BUS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            config
                .memory
                .vault_db
                .parent()
                .unwrap_or_else(|| Path::new("data"))
                .join("Synapse/events.jsonl")
        })
}

pub fn build_subagent_runner(
    config: &GzmoConfig,
    scratch: Arc<ScratchService>,
    vault: Option<Arc<SqliteVault>>,
    gateway: Arc<dyn LlmGateway>,
) -> Arc<SubagentRunner> {
    let ccr = CcrStore::new(&config.redis, &config.context_compress);
    let system_prompt = std::fs::read_to_string(&config.identity.soul_path)
        .unwrap_or_else(|_| "You are a focused GZMO sub-agent.".to_string());
    let serpapi_key = std::env::var("SERPAPI_API_KEY").unwrap_or_default();

    Arc::new(SubagentRunner::new(
        config.subagent.clone(),
        config.context_compress.clone(),
        ccr,
        scratch,
        gateway,
        vault,
        system_prompt,
        serpapi_key,
    ))
}

pub fn spec_from_recommendation(
    rec: &PendingRecommendation,
    config: &KuratorConfig,
) -> SubagentSpec {
    let brief = truncate_chars(
        &format!(
            "Kurator spawn recommendation for session {}.\nReason: {}\nComplete a focused assist task and return a concise summary.",
            rec.session_id, rec.reason
        ),
        config.spawn_brief_max_chars,
    );
    SubagentSpec {
        role: rec.suggested_agent_profile.clone(),
        brief,
        max_iterations: 8,
        depth: 1,
        parent_session: rec.session_id.clone(),
    }
}

/// Spawn a governed sub-agent for a recommendation and write Forum Romanum bus events.
pub async fn spawn_recommendation(
    runner: &SubagentRunner,
    bus: &SynapseBus,
    state_path: &Path,
    rec: PendingRecommendation,
    config: &KuratorConfig,
    approved_via: &str,
) -> Result<crate::subagent::SubagentResult> {
    let event_id = rec.event_id.clone();
    let session_id = rec.session_id.clone();
    let agent_profile = rec.suggested_agent_profile.clone();
    let spec = spec_from_recommendation(&rec, config);

    let reply_to = uuid::Uuid::parse_str(&event_id).ok();
    let thread = ForumThread::from_session(&session_id);
    let thread = if let Some(id) = reply_to {
        thread.with_reply_to(id)
    } else {
        thread
    };

    let result = runner.spawn(spec).await?;

    emit_agent_spawned(
        bus,
        &thread,
        &agent_profile,
        serde_json::json!({
            "recommendation_id": event_id,
            "approved_via": approved_via,
            "task_id": result.task_id,
        }),
    );
    emit_agent_result(
        bus,
        &thread,
        &agent_profile,
        &format!("{:?}", result.status).to_lowercase(),
        serde_json::json!({
            "task_id": result.task_id,
            "summary": result.summary,
            "llm_calls": result.llm_calls,
            "tool_calls": result.tool_calls,
        }),
    );
    mark_recommendation_spawned(state_path, &event_id, &result.task_id, rec)?;

    Ok(result)
}

/// Fire-and-forget autospawn for freshly emitted `spawn.recommended` events.
pub fn autospawn_new_recommendations(
    runner: Arc<SubagentRunner>,
    bus: Arc<SynapseBus>,
    state_path: PathBuf,
    config: KuratorConfig,
    subagent_enabled: bool,
    new_recs: Vec<PendingRecommendation>,
) {
    if new_recs.is_empty() {
        return;
    }
    if !config.enabled
        || !config.auto_spawn_on_recommend
        || !config.approve_spawns_subagent
        || !subagent_enabled
    {
        return;
    }

    for rec in new_recs {
        let event_id = rec.event_id.clone();
        let runner = Arc::clone(&runner);
        let bus = Arc::clone(&bus);
        let state_path = state_path.clone();
        let config = config.clone();
        tokio::spawn(async move {
            let rec = match take_pending_recommendation(&state_path, &event_id) {
                Ok(rec) => rec,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        event_id = %event_id,
                        "Kurator autospawn: recommendation unavailable"
                    );
                    return;
                }
            };
            match spawn_recommendation(
                &runner,
                &bus,
                &state_path,
                rec,
                &config,
                "kurator autospawn",
            )
            .await
            {
                Ok(result) => tracing::info!(
                    event_id = %event_id,
                    task_id = %result.task_id,
                    status = ?result.status,
                    "Kurator autospawn complete"
                ),
                Err(e) => tracing::error!(
                    error = %e,
                    event_id = %event_id,
                    "Kurator autospawn failed"
                ),
            }
        });
    }
}
