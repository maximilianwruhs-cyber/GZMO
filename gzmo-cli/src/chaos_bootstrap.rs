use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, RwLock};
use tokio::task::JoinHandle;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::LlmGateway;
use gzmo_chaos::pulse::{PulseLoop, PulseHandle, ChaosConfig, ChaosSnapshot};
use gzmo_chaos::triggers::{TriggerEngine, TriggerAction, NotifyLevel};
use gzmo_chaos::feedback::ChaosEvent;

/// Handle representing the running chaos runtime.
pub struct ChaosRuntime {
    pub handle: PulseHandle,
    pub feedback_tx: mpsc::Sender<ChaosEvent>,
}

/// Start the PulseLoop chaos engine with the config's [chaos] parameters.
pub fn start_chaos_runtime(config: &GzmoConfig) -> ChaosRuntime {
    let chaos_config: ChaosConfig = config.chaos
        .as_ref()
        .and_then(|v| v.clone().try_into().ok())
        .unwrap_or_default();
    let handle = PulseLoop::start(chaos_config);
    let feedback_tx = handle.feedback_tx.clone();
    ChaosRuntime {
        handle,
        feedback_tx,
    }
}

/// Spawn a background task that bridges chaos snapshots to the LLM gateway parameters,
/// periodically writes snapshot and heartbeat files, and evaluates autonomous triggers.
pub fn spawn_snapshot_bridge(
    snapshot_rx: watch::Receiver<ChaosSnapshot>,
    gateway: Arc<RwLock<Arc<dyn LlmGateway>>>,
    feedback_tx: mpsc::Sender<ChaosEvent>,
    state_dir: PathBuf,
    trigger_notify: Option<mpsc::Sender<String>>,
) -> JoinHandle<()> {
    let mut snapshot_rx = snapshot_rx;
    tokio::spawn(async move {
        let mut triggers = TriggerEngine::with_defaults();
        loop {
            if snapshot_rx.changed().await.is_err() {
                break; // PulseLoop dropped
            }
            let snap = snapshot_rx.borrow_and_update().clone();
            
            // Update gateway LLM parameters from Lorenz coordinates
            {
                let gw = gateway.read().await;
                gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
            }
            
            // Write snapshot files for shell skill backward compat
            if snap.tick % 15 == 0 {
                let json = serde_json::to_string_pretty(&snap).unwrap_or_default();
                // Atomic write: write to .tmp then rename (POSIX rename is atomic)
                let tmp_path = state_dir.join("CHAOS_STATE.json.tmp");
                let target_path = state_dir.join("CHAOS_STATE.json");
                if tokio::fs::write(&tmp_path, json.as_bytes()).await.is_ok() {
                    let _ = tokio::fs::rename(&tmp_path, &target_path).await;
                }
                
                // HEARTBEAT.md — human-readable status (containing the Workstream C rows)
                let heartbeat = format!(
                    "# GZMO Heartbeat\n\n\
                    | Field | Value |\n|---|---|\n\
                    | Status | {} |\n\
                    | Tick | {} |\n\
                    | Energy | {:.1}% |\n\
                    | Phase | {} |\n\
                    | Deaths | {} |\n\
                    | Tension | {:.1}% |\n\
                    | Chaos Val | {:.4} |\n\n\
                    ## Lorenz Attractor\n\n\
                    x={:.3}, y={:.3}, z={:.3}\n\n\
                    ## Thought Cabinet\n\n\
                    | Metric | Value |\n|---|---|\n\
                    | Incubating | {} |\n\
                    | Crystallized | {} |\n\
                    | Gravity mod | {:+.3} |\n\
                    | Friction mod | {:+.3} |\n\
                    | Lorenz ρ mod | {:+.3} |\n\
                    | ρ effective | {:.2} |\n\
                    | ρ mod delta | {:+.3} |\n\
                    | ρ forcing | {:+} |\n\
                    | ρ breath (EMA) | {:+} |\n\
                    | Tension bias | {:+.3} |\n\n\
                    ## LLM Parameters\n\n\
                    Temperature: {:.3}, Max tokens: {}, Valence: {:+.3}\n\n\
                    *Updated: {}*\n",
                    if snap.alive { "ALIVE" } else { "DEAD" },
                    snap.tick, snap.energy, snap.phase, snap.deaths, snap.tension,
                    snap.chaos_val, snap.x, snap.y, snap.z,
                    snap.thoughts_incubating, snap.thoughts_crystallized,
                    snap.mutations.gravity_mod, snap.mutations.friction_mod,
                    snap.mutations.lorenz_rho_mod,
                    snap.rho_effective, snap.rho_mod_delta, snap.rho_forcing_sign,
                    snap.rho_breath_phase,
                    snap.mutations.tension_bias,
                    snap.llm_temperature, snap.llm_max_tokens, snap.llm_valence,
                    snap.timestamp,
                );
                let hb_tmp = state_dir.join("HEARTBEAT.md.tmp");
                let hb_target = state_dir.join("HEARTBEAT.md");
                if tokio::fs::write(&hb_tmp, heartbeat.as_bytes()).await.is_ok() {
                    let _ = tokio::fs::rename(&hb_tmp, &hb_target).await;
                }
            }
            
            // Evaluate autonomous triggers
            let fired = triggers.evaluate(&snap);
            for f in fired {
                match &f.action {
                    TriggerAction::Notify { message, level } => {
                        if let Some(ref notify_tx) = trigger_notify {
                            let prefix = match level {
                                NotifyLevel::Whisper  => format!("\x1b[2m  {message}\x1b[0m"),
                                NotifyLevel::Normal   => format!("  \x1b[36m{message}\x1b[0m"),
                                NotifyLevel::Urgent   => format!("  \x1b[1m\x1b[33m{message}\x1b[0m"),
                                NotifyLevel::Critical => format!("  \x1b[1m\x1b[31m⚠ {message}\x1b[0m"),
                            };
                            let _ = notify_tx.send(prefix).await;
                        }
                    }
                    TriggerAction::EmitEvent { tension_delta, energy_delta } => {
                        let _ = feedback_tx.send(
                            ChaosEvent::Custom {
                                tension_delta: *tension_delta,
                                energy_delta: *energy_delta,
                                thought_seed: None,
                            }
                        ).await;
                    }
                    TriggerAction::RunSkill { skill_name, args } => {
                        if let Some(ref notify_tx) = trigger_notify {
                            let _ = notify_tx.send(
                                format!("__TRIGGER_SKILL__:/{skill_name} {args}")
                            ).await;
                        }
                    }
                    TriggerAction::InjectPrompt { prompt } => {
                        if let Some(ref notify_tx) = trigger_notify {
                            let _ = notify_tx.send(
                                format!("__TRIGGER_INJECT__:{prompt}")
                            ).await;
                        }
                    }
                }
            }
        }
    })
}
