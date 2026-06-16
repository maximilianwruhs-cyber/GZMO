//! Kurator CLI — status + manual approve + discovery fixer autospawn.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Result};
use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::discovery_fixer::analyze_discovery_report;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::kurator_monitor::{
    self, list_pending_recommendations, process_discovery_report, take_pending_recommendation,
};
use gzmo_core::kurator_spawn::{self, autospawn_new_recommendations, build_subagent_runner, spawn_recommendation};
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
        println!("  auto_spawn_discovery_fix: {}", config.kurator.spawn_gate.auto_spawn_discovery_fix);
        println!("  spawn_gate_enabled: {}", config.kurator.spawn_gate.enabled);
        println!("  prime_budget_enabled: {}", config.kurator.spawn_gate.prime_budget_enabled);
        println!("  prime_spawn_budget_per_hour: {}", config.kurator.spawn_gate.prime_spawn_budget_per_hour);
        println!("  discovery_fixer_enabled: {}", config.kurator.discovery_fixer_enabled);
        println!("  fixer_agent_profile: {}", config.kurator.fixer_agent_profile);
        println!("  sessions tracked: {}", state.sessions.len());
        println!("  recommendations emitted: {}", state.recommendations_emitted);
        println!("  pending approvals: {}", state.pending_recommendations.len());
        println!("  approved spawns: {}", state.spawn_history.len());
        if let Some(at) = state.last_eval_at {
            println!("  last eval: {at}");
        }
        for rec in list_pending_recommendations(&state_path) {
            let kind = rec
                .kind
                .as_deref()
                .or_else(|| {
                    if gzmo_core::discovery_fixer::is_discovery_fix_recommendation(&rec) {
                        Some("discovery_fix")
                    } else {
                        Some("pi_metrics")
                    }
                })
                .unwrap_or("pi_metrics");
            println!(
                "  pending {} session={} profile={} kind={} reason={}",
                rec.event_id,
                rec.session_id,
                rec.suggested_agent_profile,
                kind,
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

    if args[0] == "fix-from-discovery" {
        let mut report_path = None;
        let mut session_id = String::new();
        let mut spawn_now = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--report" => {
                    i += 1;
                    if i >= args.len() {
                        bail!("--report requires a path");
                    }
                    report_path = Some(PathBuf::from(&args[i]));
                }
                "--session-id" => {
                    i += 1;
                    if i >= args.len() {
                        bail!("--session-id requires a value");
                    }
                    session_id = args[i].clone();
                }
                "--spawn" => spawn_now = true,
                other => bail!("unknown arg for fix-from-discovery: {other}"),
            }
            i += 1;
        }

        let report_path = report_path.ok_or_else(|| anyhow::anyhow!("--report is required"))?;
        if session_id.is_empty() {
            session_id = format!("discovery-{}", chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ"));
        }
        if !config.kurator.enabled {
            bail!("kurator disabled in gzmo.toml");
        }
        if !config.kurator.discovery_fixer_enabled {
            bail!("kurator discovery_fixer_enabled is false");
        }

        let analysis = analyze_discovery_report(&report_path)?;
        println!(
            "Discovery report: {} actionable ({} FAIL, {} GAP)",
            analysis.actionable_count(),
            analysis.fail_count,
            analysis.gap_count
        );
        if !analysis.has_actionable() {
            println!("No FAIL/GAP findings — fixer spawn skipped");
            return Ok(());
        }

        let bus = SynapseBus::with_path(kurator_spawn::synapse_bus_path(config));
        let pending = process_discovery_report(
            &bus,
            &state_path,
            &config.kurator,
            &report_path,
            &session_id,
        )?;

        let Some(rec) = pending else {
            println!("Fixer recommendation not emitted (already claimed or below threshold)");
            return Ok(());
        };

        println!("Emitted spawn.recommended");
        println!("  recommendation_id: {}", rec.event_id);
        println!("  session_id: {}", rec.session_id);
        println!("  profile: {}", rec.suggested_agent_profile);
        println!("  reason: {}", rec.reason);

        let should_spawn = spawn_now
            || (config.kurator.spawn_gate.auto_spawn_discovery_fix
                && config.kurator.approve_spawns_subagent
                && config.subagent.enabled);

        if should_spawn {
            let runner = runner_for_config(config).await?;
            if spawn_now {
                let taken = take_pending_recommendation(&state_path, &rec.event_id)?;
                let result = spawn_recommendation(
                    &runner,
                    &bus,
                    &state_path,
                    taken,
                    &config.kurator,
                    &config.redis,
                    "gzmo kurator fix-from-discovery",
                )
                .await?;
                println!("Fixer sub-agent spawned");
                println!("  task_id: {}", result.task_id);
                println!("  status: {:?}", result.status);
                println!("  summary: {}", truncate_chars(&result.summary, 500));
            } else {
                autospawn_new_recommendations(
                    runner,
                    Arc::new(bus),
                    state_path,
                    config.kurator.clone(),
                    config.redis.clone(),
                    config.subagent.enabled,
                    vec![rec],
                );
                println!("Fixer autospawn queued (daemon-style background spawn)");
            }
        } else {
            println!("Autospawn disabled — run `gzmo kurator approve {}` to spawn", rec.event_id);
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
            &config.redis,
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

    eprintln!(
        "Usage: gzmo kurator status | gzmo kurator approve <id> | gzmo kurator fix-from-discovery --report <path> [--session-id <id>] [--spawn]"
    );
    Ok(())
}
