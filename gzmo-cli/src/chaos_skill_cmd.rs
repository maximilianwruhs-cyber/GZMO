//! `gzmo chaos skill <command> [args]` — external skill runner with daemon IPC.

use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, Result};
use gzmo_chaos::feedback_ipc;
use gzmo_chaos::pulse::ChaosSnapshot;
use tokio::sync::mpsc;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::{TurboQuantGateway, VllmConfig};
use gzmo_core::skills::dispatch::{self, data_dir, load_live_chaos_snapshot};
use gzmo_core::skills::registry::build_chaos_skill_registry;
use gzmo_core::skills::NestedDispatch;
use gzmo_core::synapse::SynapseBus;
use gzmo_core::synapse_writer::{
    self, claim_skill_invoke, disabled_claim, emit_skill_complete, emit_skill_error,
    gate_bypass_from_env,
};

use crate::chaos_bootstrap;

/// Strip `--json` from skill args (may appear before or after die type).
pub fn strip_json_flag(args: &str) -> (String, bool) {
    let mut json = false;
    let cleaned: Vec<&str> = args
        .split_whitespace()
        .filter(|t| {
            if *t == "--json" {
                json = true;
                false
            } else {
                true
            }
        })
        .collect();
    (cleaned.join(" "), json)
}

fn synapse_bus_path(config: &GzmoConfig) -> std::path::PathBuf {
    std::env::var("GZMO_SYNAPSE_BUS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| data_dir(config).join("Synapse/events.jsonl"))
}

pub async fn run(config: &GzmoConfig, cmd: &str, args: &str, json_flag: bool) -> Result<()> {
    let (args, json_from_args) = strip_json_flag(args);
    let json = json_flag || json_from_args;

    let root = std::env::current_dir()?;
    let bus_path = synapse_bus_path(config);
    let gate_state_path = synapse_writer::default_gate_state_path(&root);
    let tool_call_id = std::env::var("GZMO_SYNAPSE_TOOL_CALL_ID").ok();
    let bypass = gate_bypass_from_env();

    let claim = if bypass {
        disabled_claim(tool_call_id.clone())
    } else {
        match claim_skill_invoke(
            &bus_path,
            &gate_state_path,
            &config.synapse_writer,
            cmd,
            &args,
            tool_call_id.as_deref(),
        ) {
            Ok(c) => c,
            Err(e) => {
                bail!(
                    "synapse writer gate: {e} (set GZMO_SYNAPSE_GATE_BYPASS=1 for headless CLI)"
                );
            }
        }
    };

    let started = Instant::now();
    let bus = SynapseBus::with_path(bus_path);

    let registry = build_chaos_skill_registry(&config.pedagogy);
    let profile = config.engine.active_engine();
    let inbox = feedback_ipc::default_inbox_path(data_dir(config));
    let daemon = dispatch::daemon_running();

    let (fallback_snap, feedback_tx, _runtime_keepalive) = if daemon {
        let (tx, _rx) = mpsc::channel(8);
        (
            load_live_chaos_snapshot(data_dir(config), &ChaosSnapshot::default()),
            tx,
            None,
        )
    } else {
        let runtime = chaos_bootstrap::start_chaos_runtime(config);
        let snap = runtime.handle.snapshot_rx.borrow().clone();
        let tx = runtime.handle.feedback_tx.clone();
        (snap, tx, Some(runtime))
    };

    let snap = load_live_chaos_snapshot(data_dir(config), &fallback_snap);

    let gateway = TurboQuantGateway::new(VllmConfig::from(profile.clone()));
    gateway.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
    let gateway_dyn: Arc<dyn gzmo_core::gateway::LlmGateway> = Arc::new(gateway);

    let nested = gzmo_core::skills::NestedDispatch {
        registry: Some(&registry),
        profile: Some(&profile),
        depth: 0,
    };
    let ctx = dispatch::skill_context(
        &snap,
        &feedback_tx,
        &args,
        Some(gateway_dyn.as_ref()),
        None,
        config,
        nested,
    );

    let dispatch_result =
        match dispatch::dispatch_skill(&registry, cmd, ctx, &profile).await {
            Ok(r) => r,
            Err(e) => {
                emit_skill_error(&bus, &claim, cmd, &e.to_string());
                return Err(e);
            }
        };

    if daemon {
        for event in &dispatch_result.output.feedback {
            feedback_ipc::append_event(&inbox, event)?;
        }
    }

    emit_skill_complete(&bus, &claim, cmd, started.elapsed().as_millis() as u64);

    if json {
        let mut payload = dispatch_result.output.evidence.unwrap_or_else(|| {
            serde_json::json!({
                "skill": cmd,
                "display": dispatch_result.output.display,
                "feedback_count": dispatch_result.output.feedback.len(),
            })
        });
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("daemon_inbox".into(), serde_json::Value::Bool(daemon));
            obj.insert("cmd".into(), serde_json::Value::String(cmd.to_string()));
            if !dispatch_result.output.display.is_empty() {
                obj.insert(
                    "display_plain".into(),
                    serde_json::Value::String(
                        gzmo_core::text_util::pi_skill_display(&dispatch_result.output.display),
                    ),
                );
            }
        }
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if !dispatch_result.output.display.is_empty() {
        print!("{}", dispatch_result.output.display);
    } else if !dispatch_result.output.feedback.is_empty() {
        println!("(skill ok — chaos feedback queued)");
    } else {
        emit_skill_error(&bus, &claim, cmd, "skill produced no output");
        anyhow::bail!("skill produced no output");
    }

    Ok(())
}
