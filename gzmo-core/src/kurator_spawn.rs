//! Kurator phase 3 — governed sub-agent spawn (manual approve + daemon autospawn).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Result};

use crate::config::{GzmoConfig, KuratorConfig, RedisConfig};
use crate::context_compress::CcrStore;
use crate::gateway::LlmGateway;
use crate::kurator_monitor::{
    load_state, mark_recommendation_spawned, restore_pending_recommendation,
    take_pending_recommendation, PendingRecommendation,
};
use crate::memory::scratch::ScratchService;
use crate::memory::vault::SqliteVault;
use crate::spawn_gate::{
    self, bypass_gate_for_approved_via, emit_spawn_denied, emit_spawn_executed, evaluate_autospawn,
    record_denial, record_execution,
};
use crate::spawn_prime_budget::{acquire_prime_slot, release_prime_slot};
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

fn project_root_from_kurator_state(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
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
    if crate::discovery_fixer::is_discovery_fix_recommendation(rec) {
        let report_path = crate::discovery_fixer::resolve_discovery_report_path(rec, None);
        let analysis = crate::discovery_fixer::analyze_discovery_report(&report_path)
            .unwrap_or_default();
        let brief = if analysis.has_actionable() {
            crate::discovery_fixer::build_fixer_brief(
                &report_path,
                &rec.session_id,
                &analysis,
                config.spawn_brief_max_chars,
            )
        } else {
            truncate_chars(
                &format!(
                    "Discovery fixer for `{}`.\nTrigger: {}\nRead report at {} and attempt remediation.",
                    rec.session_id,
                    rec.reason,
                    report_path.display(),
                ),
                config.spawn_brief_max_chars,
            )
        };
        return SubagentSpec {
            role: rec.suggested_agent_profile.clone(),
            brief,
            max_iterations: 12,
            depth: 1,
            parent_session: rec.session_id.clone(),
        };
    }

    let brief = truncate_chars(
        &format!(
            "Kurator intervention for Pi session `{session_id}`.\n\
            Trigger: {reason}\n\n\
            Task:\n\
            1. Read session metrics in `data/kurator-monitor.state.json` for this session_id.\n\
            2. Optionally inspect recent Synapse events in `data/Synapse/events.jsonl` (correlation_id = session_id).\n\
            3. Summarize whether the session needs operator attention and what action to take.\n\
            4. Return a concise operator brief.\n\n\
            Do NOT run broad recursive greps for the UUID across /data, /home, or /var.",
            session_id = rec.session_id,
            reason = rec.reason,
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
    redis_cfg: &RedisConfig,
    approved_via: &str,
) -> Result<crate::subagent::SubagentResult> {
    let project_root = project_root_from_kurator_state(state_path);
    let gate_path = spawn_gate::default_state_path(&project_root);

    if !bypass_gate_for_approved_via(approved_via) {
        let kurator_state = load_state(state_path);
        let gate_state = spawn_gate::load_state(&gate_path);
        let decision = evaluate_autospawn(&rec, &config.spawn_gate, &gate_state, &kurator_state);
        if !decision.allowed {
            emit_spawn_denied(bus, &rec, &decision);
            record_denial(&gate_path, &rec, &decision)?;
            bail!("spawn gate denied: {} — {}", decision.code, decision.message);
        }

        let prime = acquire_prime_slot(redis_cfg, &config.spawn_gate).await;
        if let Some(decision) = prime.decision_if_denied() {
            emit_spawn_denied(bus, &rec, decision);
            record_denial(&gate_path, &rec, decision)?;
            bail!("spawn gate denied: {} — {}", decision.code, decision.message);
        }
        if let crate::spawn_prime_budget::PrimeBudgetOutcome::AllowedFailOpen { reason } = &prime {
            tracing::warn!(reason = %reason, "Prime budget fail-open — spawn proceeding");
        }

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

        let spawn_result = runner.spawn(spec).await;
        if spawn_result.is_err() {
            release_prime_slot(redis_cfg, &config.spawn_gate).await;
        }
        let result = spawn_result?;

        emit_agent_spawned(
            bus,
            &thread,
            &agent_profile,
            serde_json::json!({
                "recommendation_id": event_id,
                "approved_via": approved_via,
                "task_id": result.task_id,
                "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
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
                "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
            }),
        );
        emit_spawn_executed(bus, &rec, &result.task_id, approved_via);
        record_execution(&gate_path, &rec, &result.task_id, approved_via)?;
        mark_recommendation_spawned(state_path, &event_id, &result.task_id, rec)?;

        return Ok(result);
    }

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
            "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
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
            "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
        }),
    );
    emit_spawn_executed(bus, &rec, &result.task_id, approved_via);
    record_execution(&gate_path, &rec, &result.task_id, approved_via)?;
    mark_recommendation_spawned(state_path, &event_id, &result.task_id, rec)?;

    Ok(result)
}

/// Fire-and-forget autospawn for freshly emitted `spawn.recommended` events.
pub fn autospawn_new_recommendations(
    runner: Arc<SubagentRunner>,
    bus: Arc<SynapseBus>,
    state_path: PathBuf,
    config: KuratorConfig,
    redis_cfg: RedisConfig,
    subagent_enabled: bool,
    new_recs: Vec<PendingRecommendation>,
) {
    if new_recs.is_empty() {
        return;
    }
    if !config.enabled || !config.approve_spawns_subagent || !subagent_enabled {
        return;
    }

    let project_root = project_root_from_kurator_state(&state_path);
    let gate_path = spawn_gate::default_state_path(&project_root);
    let kurator_state = load_state(&state_path);
    let gate_state = spawn_gate::load_state(&gate_path);

    for rec in new_recs {
        if !spawn_gate::autospawn_enabled_for(
            &rec,
            config.auto_spawn_on_recommend,
            config.spawn_gate.auto_spawn_discovery_fix,
        ) {
            tracing::debug!(
                event_id = %rec.event_id,
                kind = %spawn_gate::spawn_kind(&rec).as_str(),
                "Kurator autospawn skipped (disabled for this kind)"
            );
            continue;
        }

        let decision = evaluate_autospawn(&rec, &config.spawn_gate, &gate_state, &kurator_state);
        if !decision.allowed {
            emit_spawn_denied(&bus, &rec, &decision);
            let _ = record_denial(&gate_path, &rec, &decision);
            tracing::info!(
                event_id = %rec.event_id,
                code = %decision.code,
                "Kurator autospawn denied by spawn gate"
            );
            continue;
        }

        let event_id = rec.event_id.clone();
        let runner = Arc::clone(&runner);
        let bus = Arc::clone(&bus);
        let state_path = state_path.clone();
        let config = config.clone();
        let redis_cfg = redis_cfg.clone();
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
                rec.clone(),
                &config,
                &redis_cfg,
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
                Err(e) => {
                    if let Err(restore_err) =
                        restore_pending_recommendation(&state_path, rec)
                    {
                        tracing::warn!(
                            error = %restore_err,
                            event_id = %event_id,
                            "Kurator autospawn: failed to restore pending recommendation"
                        );
                    }
                    tracing::error!(
                        error = %e,
                        event_id = %event_id,
                        "Kurator autospawn failed"
                    );
                }
            }
        });
    }
}
