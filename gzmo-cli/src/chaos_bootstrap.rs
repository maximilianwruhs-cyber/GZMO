use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, RwLock};
use tokio::task::JoinHandle;

use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::pulse::{ChaosConfig, ChaosSnapshot, PulseHandle, PulseLoop};
use gzmo_chaos::triggers::{NotifyLevel, TriggerAction, TriggerEngine};
use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};

use crate::tui::action::Action;

/// Handle representing the running chaos runtime.
pub struct ChaosRuntime {
    pub handle: PulseHandle,
    pub feedback_tx: mpsc::Sender<ChaosEvent>,
    pub restore_policy: String,
}

/// ADR-0003: chaos is opt-in for chat. Default off when key absent.
pub fn enabled_in_chat(config: &GzmoConfig) -> bool {
    config
        .chaos
        .as_ref()
        .and_then(|v| v.get("enabled_in_chat"))
        .and_then(|x| x.as_bool())
        .unwrap_or(false)
}

/// Chat boot: live PulseLoop when enabled, else static idle snapshot (no CPU pulse).
pub struct ChatChaosBoot {
    pub enabled: bool,
    /// Kept alive so a disabled boot's watch sender is not dropped.
    pub _snapshot_tx: Option<watch::Sender<ChaosSnapshot>>,
    pub runtime: Option<ChaosRuntime>,
    pub snapshot_rx: watch::Receiver<ChaosSnapshot>,
    pub feedback_tx: mpsc::Sender<ChaosEvent>,
    pub restore_policy: String,
}

pub fn boot_chat_chaos(config: &GzmoConfig) -> ChatChaosBoot {
    if enabled_in_chat(config) {
        let runtime = start_chaos_runtime(config);
        let snapshot_rx = runtime.handle.snapshot_rx.clone();
        let feedback_tx = runtime.feedback_tx.clone();
        let restore_policy = runtime.restore_policy.clone();
        ChatChaosBoot {
            enabled: true,
            _snapshot_tx: None,
            runtime: Some(runtime),
            snapshot_rx,
            feedback_tx,
            restore_policy,
        }
    } else {
        let (snapshot_tx, snapshot_rx) = watch::channel(ChaosSnapshot::default());
        let (feedback_tx, _feedback_rx) = mpsc::channel(8);
        ChatChaosBoot {
            enabled: false,
            _snapshot_tx: Some(snapshot_tx),
            runtime: None,
            snapshot_rx,
            feedback_tx,
            restore_policy: "quarantined".into(),
        }
    }
}

/// Start the PulseLoop chaos engine with the config's [chaos] parameters.
pub fn start_chaos_runtime(config: &GzmoConfig) -> ChaosRuntime {
    let chaos_config: ChaosConfig = config
        .chaos
        .as_ref()
        .and_then(|v| v.clone().try_into().ok())
        .unwrap_or_default();
    let restore_policy = if chaos_config.rho_restore_alpha > 0.0 {
        format!(
            "tanh (α={:.2}, β={:.2})",
            chaos_config.rho_restore_alpha, chaos_config.rho_restore_beta
        )
    } else {
        format!("linear (k={:.4})", chaos_config.rho_decay_k)
    };
    let handle = PulseLoop::start(chaos_config);
    let feedback_tx = handle.feedback_tx.clone();
    ChaosRuntime {
        handle,
        feedback_tx,
        restore_policy,
    }
}

/// Options for snapshot bridge liveness (TUI uses immediate flush + heartbeat).
#[derive(Debug, Clone, Copy)]
pub struct SnapshotBridgeOpts {
    /// Process the current watch value before awaiting `changed()`.
    pub immediate_flush: bool,
    /// Rebroadcast the latest snapshot on this interval even when watch is quiet.
    pub heartbeat_ms: Option<u64>,
}

impl SnapshotBridgeOpts {
    pub const STDIO: Self = Self {
        immediate_flush: false,
        heartbeat_ms: None,
    };

    pub const TUI: Self = Self {
        immediate_flush: true,
        heartbeat_ms: Some(200),
    };
}

/// Spawn a background task that bridges chaos snapshots to the LLM gateway parameters,
/// periodically writes snapshot and heartbeat files, and evaluates autonomous triggers.
pub fn spawn_snapshot_bridge(
    snapshot_rx: watch::Receiver<ChaosSnapshot>,
    gateway: Arc<RwLock<Arc<dyn LlmGateway>>>,
    feedback_tx: mpsc::Sender<ChaosEvent>,
    state_dir: PathBuf,
    trigger_notify: Option<mpsc::Sender<String>>,
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    synapse: Option<Arc<SynapseBus>>,
    synapse_source: EventSource,
    restore_policy: String,
    interactive: bool,
    opts: SnapshotBridgeOpts,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut snapshot_rx = snapshot_rx;
        let mut triggers = if interactive {
            TriggerEngine::with_defaults_interactive()
        } else {
            TriggerEngine::with_defaults()
        };

        if opts.immediate_flush {
            let snap = snapshot_rx.borrow().clone();
            process_snapshot(
                snap,
                &gateway,
                &state_dir,
                &restore_policy,
                synapse.as_ref(),
                synapse_source,
                &mut triggers,
                &feedback_tx,
                trigger_notify.as_ref(),
                action_tx.as_ref(),
            )
            .await;
        }

        let mut heartbeat = opts
            .heartbeat_ms
            .map(|ms| tokio::time::interval(std::time::Duration::from_millis(ms)));

        loop {
            let got_change = if let Some(ref mut hb) = heartbeat {
                tokio::select! {
                    changed = snapshot_rx.changed() => {
                        if changed.is_err() {
                            break;
                        }
                        true
                    }
                    _ = hb.tick() => false,
                }
            } else if snapshot_rx.changed().await.is_err() {
                break;
            } else {
                true
            };

            let snap = if got_change {
                snapshot_rx.borrow_and_update().clone()
            } else {
                snapshot_rx.borrow().clone()
            };

            process_snapshot(
                snap,
                &gateway,
                &state_dir,
                &restore_policy,
                synapse.as_ref(),
                synapse_source,
                &mut triggers,
                &feedback_tx,
                trigger_notify.as_ref(),
                action_tx.as_ref(),
            )
            .await;
        }
    })
}

async fn process_snapshot(
    snap: ChaosSnapshot,
    gateway: &Arc<RwLock<Arc<dyn LlmGateway>>>,
    state_dir: &PathBuf,
    restore_policy: &str,
    synapse: Option<&Arc<SynapseBus>>,
    synapse_source: EventSource,
    triggers: &mut TriggerEngine,
    feedback_tx: &mpsc::Sender<ChaosEvent>,
    trigger_notify: Option<&mpsc::Sender<String>>,
    action_tx: Option<&mpsc::UnboundedSender<Action>>,
) {
    {
        let gw = gateway.read().await;
        gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
    }

    if let Some(tx) = action_tx {
        let _ = tx.send(Action::ChaosSnapshot(snap.clone()));
    }

    if snap.tick % 15 == 0 {
        write_chaos_artifacts(state_dir, &snap, restore_policy, synapse, synapse_source).await;
    }

    let fired = triggers.evaluate(&snap);
    for f in fired {
        dispatch_trigger(&f.action, feedback_tx, trigger_notify, action_tx).await;
    }
}

async fn write_chaos_artifacts(
    state_dir: &PathBuf,
    snap: &ChaosSnapshot,
    restore_policy: &str,
    synapse: Option<&Arc<SynapseBus>>,
    synapse_source: EventSource,
) {
    let json = serde_json::to_string_pretty(snap).unwrap_or_default();
    let tmp_path = state_dir.join("CHAOS_STATE.json.tmp");
    let target_path = state_dir.join("CHAOS_STATE.json");
    if tokio::fs::write(&tmp_path, json.as_bytes()).await.is_ok() {
        let _ = tokio::fs::rename(&tmp_path, &target_path).await;
    }

    let cheapcheck_block = {
        let existing = tokio::fs::read_to_string(state_dir.join("HEARTBEAT.md"))
            .await
            .unwrap_or_default();
        extract_cheapcheck_block(&existing)
    };
    let heartbeat = format!(
        "# GZMO Heartbeat\n\n\
        | Field | Value |\n|---|---|\n\
        | Status | {} |\n\
        | Tick | {} |\n\
        | Energy | {:.1}% |\n\
        | Phase | {} |\n\
        | Deaths | {} |\n\
        | Tension | {:.1}% |\n\
        | Chaos Val | {:.4} |\n\
        | Restore Policy | {} |\n\n\
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
        *Updated: {}*\n\
        {cheapcheck_block}",
        if snap.alive { "ALIVE" } else { "DEAD" },
        snap.tick,
        snap.energy,
        snap.phase,
        snap.deaths,
        snap.tension,
        snap.chaos_val,
        restore_policy,
        snap.x,
        snap.y,
        snap.z,
        snap.thoughts_incubating,
        snap.thoughts_crystallized,
        snap.mutations.gravity_mod,
        snap.mutations.friction_mod,
        snap.mutations.lorenz_rho_mod,
        snap.rho_effective,
        snap.rho_mod_delta,
        snap.rho_forcing_sign,
        snap.rho_breath_phase,
        snap.mutations.tension_bias,
        snap.llm_temperature,
        snap.llm_max_tokens,
        snap.llm_valence,
        snap.timestamp,
    );
    let hb_tmp = state_dir.join("HEARTBEAT.md.tmp");
    let hb_target = state_dir.join("HEARTBEAT.md");
    if tokio::fs::write(&hb_tmp, heartbeat.as_bytes())
        .await
        .is_ok()
    {
        let _ = tokio::fs::rename(&hb_tmp, &hb_target).await;
    }

    if let Some(bus) = synapse {
        let data = serde_json::json!({
            "tick": snap.tick,
            "rho_mod": snap.mutations.lorenz_rho_mod,
            "rho_effective": snap.rho_effective,
            "rho_mod_delta": snap.rho_mod_delta,
            "rho_forcing_sign": snap.rho_forcing_sign,
            "rho_breath_phase": snap.rho_breath_phase,
            "rho_velocity_ema": snap.rho_velocity_ema,
        });
        bus.append(&SynapseEvent::with_data(
            EventType::SenseChaosRho,
            synapse_source,
            data,
        ));
    }
}

async fn dispatch_trigger(
    action: &TriggerAction,
    feedback_tx: &mpsc::Sender<ChaosEvent>,
    trigger_notify: Option<&mpsc::Sender<String>>,
    action_tx: Option<&mpsc::UnboundedSender<Action>>,
) {
    match action {
        TriggerAction::Notify { message, level } => {
            if let Some(tx) = action_tx {
                let formatted = match level {
                    NotifyLevel::Whisper => format!("[dim] {}", message),
                    NotifyLevel::Normal => message.clone(),
                    NotifyLevel::Urgent => format!("⚠ {}", message),
                    NotifyLevel::Critical => format!("⚠⚠ {}", message),
                };
                let _ = tx.send(Action::TriggerNotification(formatted));
            } else if let Some(notify_tx) = trigger_notify {
                let prefix = match level {
                    NotifyLevel::Whisper => format!("\x1b[2m  {message}\x1b[0m"),
                    NotifyLevel::Normal => format!("  \x1b[36m{message}\x1b[0m"),
                    NotifyLevel::Urgent => format!("  \x1b[1m\x1b[33m{message}\x1b[0m"),
                    NotifyLevel::Critical => format!("  \x1b[1m\x1b[31m⚠ {message}\x1b[0m"),
                };
                let _ = notify_tx.send(prefix).await;
            }
        }
        TriggerAction::EmitEvent {
            tension_delta,
            energy_delta,
        } => {
            let _ = feedback_tx
                .send(ChaosEvent::Custom {
                    tension_delta: *tension_delta,
                    energy_delta: *energy_delta,
                    thought_seed: None,
                })
                .await;
        }
        TriggerAction::RunSkill { skill_name, args } => {
            if let Some(tx) = action_tx {
                let _ = tx.send(Action::TriggerSkill(skill_name.clone(), args.clone()));
            } else if let Some(notify_tx) = trigger_notify {
                let _ = notify_tx
                    .send(format!("__TRIGGER_SKILL__:/{skill_name} {args}"))
                    .await;
            }
        }
        TriggerAction::InjectPrompt { prompt } => {
            if let Some(tx) = action_tx {
                let _ = tx.send(Action::TriggerInject(prompt.clone()));
            } else if let Some(notify_tx) = trigger_notify {
                let _ = notify_tx.send(format!("__TRIGGER_INJECT__:{prompt}")).await;
            }
        }
    }
}

fn extract_cheapcheck_block(body: &str) -> String {
    let start = gzmo_core::daemon::CHEAPCHECK_START;
    let end = gzmo_core::daemon::CHEAPCHECK_END;
    if let (Some(s), Some(e)) = (body.find(start), body.find(end)) {
        if e > s {
            return format!(
                "\n{}\n{}\n{}\n",
                start,
                body[s + start.len()..e].trim(),
                end
            );
        }
    }
    format!(
        "\n{}\n## CheapCheck probes\n\n| Check | Status | Detail |\n|---|---|---|\n| (pending) | — | awaiting first heartbeat tick |\n\n{}\n",
        start, end
    )
}
