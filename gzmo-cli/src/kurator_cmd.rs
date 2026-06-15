//! Kurator CLI — status + manual approve (autospawn runs in daemon).

use std::sync::Arc;

use anyhow::{bail, Result};
use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::kurator_monitor::{self, list_pending_recommendations, take_pending_recommendation};
use gzmo_core::kurator_spawn::{self, build_subagent_runner, spawn_recommendation};
use gzmo_core::memory::scratch::ScratchService;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::synapse::{set_event_source, EventSource, SynapseBus};
use gzmo_core::text_util::truncate_chars;

async fn runner_for_config(config: &GzmoConfig) -> Result<Arc<gzmo_core::subagent::SubagentRunner>> {
    let router = GatewayRouter::new(config);
    let gateway = Arc::clone(router.gateway(TaskKind::Chat));

    let vault = if config.memory.vault_db.exists() {
        Some(Arc::new(SqliteVault::open(&config.memory.vault_db)?))
    } else {
        None
    };

    let scratch = Arc::new(
        ScratchService::from_config(&config.redis, &config.context_memory).await,
    );

    Ok(build_subagent_runner(config, scratch, vault, gateway))
}

pub async fn run(args: &[String], config: &GzmoConfig) -> Result<()> {
    set_event_source(EventSource::GzmoCli);
    let root = std::env::current_dir()?;
    let state_path = kurator_monitor::default_state_path(&root);

    if args.is_empty() || args[0] == "status" {
        let state = kurator_monitor::load_state(&state_path);
        println!("Kurator monitor (phase 3 — autospawn + manual approve)");
        println!("  enabled: {}", config.kurator.enabled);
        println!("  approve_spawns_subagent: {}", config.kurator.approve_spawns_subagent);
        println!("  auto_spawn_on_recommend: {}", config.kurator.auto_spawn_on_recommend);
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
        let runner = runner_for_config(config).await?;
        let bus = SynapseBus::with_path(kurator_spawn::synapse_bus_path(config));

        let result = spawn_recommendation(
            &runner,
            &bus,
            &state_path,
            rec,
            &config.kurator,
            "gzmo kurator approve",
        )
        .await?;

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
