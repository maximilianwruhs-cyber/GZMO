//! Kurator CLI — status + phase-2 approve (spawn governed sub-agent).

use std::sync::Arc;

use anyhow::{bail, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::context_compress::CcrStore;
use gzmo_core::gateway::{TurboQuantGateway, VllmConfig};
use gzmo_core::kurator_monitor::{
    self, list_pending_recommendations, mark_recommendation_spawned, take_pending_recommendation,
};
use gzmo_core::memory::scratch::ScratchService;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::subagent::{SubagentRunner, SubagentSpec};
use gzmo_core::synapse::{set_event_source, EventSource, SynapseBus};
use gzmo_core::synapse_writer::{emit_agent_result, emit_agent_spawned, ForumThread};
use gzmo_core::text_util::truncate_chars;

async fn build_subagent_runner(config: &GzmoConfig) -> Result<Arc<SubagentRunner>> {
    let profile = config.engine.active_engine();
    let gateway = TurboQuantGateway::new(VllmConfig::from(profile.clone()));
    let gateway_dyn: Arc<dyn gzmo_core::gateway::LlmGateway> = Arc::new(gateway);

    let vault = if config.memory.vault_db.exists() {
        Some(Arc::new(SqliteVault::open(&config.memory.vault_db)?))
    } else {
        None
    };

    let scratch = Arc::new(
        ScratchService::from_config(&config.redis, &config.context_memory).await,
    );
    let ccr = CcrStore::new(&config.redis, &config.context_compress);

    let system_prompt = std::fs::read_to_string(&config.identity.soul_path)
        .unwrap_or_else(|_| "You are a focused GZMO sub-agent.".to_string());

    let serpapi_key = std::env::var("SERPAPI_API_KEY").unwrap_or_default();

    Ok(Arc::new(SubagentRunner::new(
        config.subagent.clone(),
        config.context_compress.clone(),
        ccr,
        scratch,
        gateway_dyn,
        vault,
        system_prompt,
        serpapi_key,
    )))
}

fn synapse_bus_path(config: &GzmoConfig) -> std::path::PathBuf {
    std::env::var("GZMO_SYNAPSE_BUS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            config
                .memory
                .vault_db
                .parent()
                .unwrap_or(std::path::Path::new("data"))
                .join("Synapse/events.jsonl")
        })
}

pub async fn run(args: &[String], config: &GzmoConfig) -> Result<()> {
    set_event_source(EventSource::GzmoCli);
    let root = std::env::current_dir()?;
    let state_path = kurator_monitor::default_state_path(&root);

    if args.is_empty() || args[0] == "status" {
        let state = kurator_monitor::load_state(&state_path);
        println!("Kurator monitor (phase 2 — approve spawns sub-agent)");
        println!("  enabled: {}", config.kurator.enabled);
        println!("  approve_spawns_subagent: {}", config.kurator.approve_spawns_subagent);
        println!("  sessions tracked: {}", state.sessions.len());
        println!("  recommendations emitted: {}", state.recommendations_emitted);
        println!("  pending approvals: {}", state.pending_recommendations.len());
        println!("  approved spawns: {}", state.spawn_history.len());
        if let Some(at) = state.last_eval_at {
            println!("  last eval: {at}");
        }
        for rec in list_pending_recommendations(&state_path) {
            println!(
                "  pending {} session={} profile={} reason={}",
                rec.event_id,
                rec.session_id,
                rec.suggested_agent_profile,
                truncate_chars(&rec.reason, 120)
            );
        }
        for (sid, m) in &state.sessions {
            println!(
                "  session {sid}: turns={} tokens={} skill_errors={}",
                m.turn_count,
                m.input_tokens + m.output_tokens,
                m.skill_errors
            );
        }
        return Ok(());
    }

    if args[0] == "approve" {
        if args.len() < 2 {
            bail!("usage: gzmo kurator approve <recommendation-id|session-id>");
        }
        if !config.kurator.enabled {
            bail!("kurator disabled in gzmo.toml");
        }
        if !config.kurator.approve_spawns_subagent {
            bail!("kurator approve_spawns_subagent is false");
        }
        if !config.subagent.enabled {
            bail!("subagent disabled — enable [subagent] in gzmo.toml");
        }

        let target = &args[1];
        let rec = take_pending_recommendation(&state_path, target)?;
        let event_id = rec.event_id.clone();
        let session_id = rec.session_id.clone();
        let agent_profile = rec.suggested_agent_profile.clone();
        let runner = build_subagent_runner(config).await?;

        let brief = truncate_chars(
            &format!(
                "Kurator spawn recommendation for session {}.\nReason: {}\nComplete a focused assist task and return a concise summary.",
                session_id, rec.reason
            ),
            config.kurator.spawn_brief_max_chars,
        );

        let spec = SubagentSpec {
            role: agent_profile.clone(),
            brief,
            max_iterations: 8,
            depth: 1,
            parent_session: session_id.clone(),
        };

        let bus = SynapseBus::with_path(synapse_bus_path(config));
        let reply_to = uuid::Uuid::parse_str(&event_id).ok();
        let thread = ForumThread::from_session(&session_id);
        let thread = if let Some(id) = reply_to {
            thread.with_reply_to(id)
        } else {
            thread
        };

        let result = runner.spawn(spec).await?;

        emit_agent_spawned(
            &bus,
            &thread,
            &agent_profile,
            serde_json::json!({
                "recommendation_id": event_id,
                "approved_via": "gzmo kurator approve",
                "task_id": result.task_id,
            }),
        );
        emit_agent_result(
            &bus,
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
        mark_recommendation_spawned(&state_path, &event_id, &result.task_id, rec)?;

        println!("Kurator approve: sub-agent spawned");
        println!("  recommendation_id: {}", event_id);
        println!("  session_id: {}", session_id);
        println!("  task_id: {}", result.task_id);
        println!("  status: {:?}", result.status);
        println!("  summary: {}", truncate_chars(&result.summary, 500));
        return Ok(());
    }

    eprintln!("Usage: gzmo kurator status | gzmo kurator approve <id>");
    Ok(())
}
