use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch, RwLock};
use tokio::task::JoinHandle;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::pedagogy::{
    compute_knowledge_delta, delta_to_json, empty_knowledge_state,
    knowledge_state_for_cycle_start, snapshot_to_json, KnowledgeDelta, KnowledgeStateSnapshot,
};
use gzmo_core::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};
use gzmo_chaos::pulse::{PulseLoop, PulseHandle, ChaosConfig, ChaosSnapshot};
use gzmo_chaos::pedagogy_oscillator::{PedagogyOscillationSettings, PedagogyTransitionKind};
use gzmo_chaos::triggers::{TriggerEngine, TriggerAction, NotifyLevel};
use gzmo_chaos::feedback::ChaosEvent;
use serde_json::json;
use uuid::Uuid;

/// Tracks discovery spawns and knowledge snapshots for the active oscillation cycle.
#[derive(Debug, Default)]
pub struct PedagogyOscillationContext {
    pub oscillation_id: Option<Uuid>,
    pub chaos_val_baseline: Option<f64>,
    pub knowledge_state_before: KnowledgeStateSnapshot,
    pub knowledge_state_after: KnowledgeStateSnapshot,
    pub knowledge_delta: KnowledgeDelta,
    pub spawned_tasks: Vec<serde_json::Value>,
}

/// Daemon-side pedagogy oscillation hooks (discovery spawn on low phase).
#[derive(Clone)]
pub struct PedagogyBridgeConfig {
    pub oscillation: PedagogyOscillationSettings,
    pub scripts_root: String,
    pub gzmo_root: PathBuf,
    pub gzmo_config: Arc<GzmoConfig>,
    pub cycle_ctx: Arc<Mutex<PedagogyOscillationContext>>,
}

fn build_pedagogy_event_data(
    t: &gzmo_chaos::pedagogy_oscillator::PedagogyTransitionInfo,
    snap: &ChaosSnapshot,
    ctx: &PedagogyOscillationContext,
) -> serde_json::Value {
    let osc_id = t.oscillation_id.or(snap.oscillation_id);
    let baseline = ctx.chaos_val_baseline.or(snap.chaos_val_baseline).unwrap_or(snap.chaos_val_raw);
    let mut data = json!({
        "oscillation_id": osc_id.map(|u| u.to_string()),
        "step": t.step,
        "target": t.target,
        "label": t.label,
        "duration_secs": t.duration_secs,
        "is_low_phase": t.is_low_phase,
        "chaos_val": snap.chaos_val,
        "chaos_val_raw": snap.chaos_val_raw,
        "chaos_val_baseline": baseline,
    });
    if t.kind == PedagogyTransitionKind::CycleStart {
        data["knowledge_state_before"] = snapshot_to_json(&ctx.knowledge_state_before);
    }
    if t.kind == PedagogyTransitionKind::CycleComplete {
        data["knowledge_state_after"] = snapshot_to_json(&ctx.knowledge_state_after);
        data["knowledge_delta"] = delta_to_json(&ctx.knowledge_delta);
        data["spawned_tasks"] = json!(ctx.spawned_tasks);
    }
    data
}

/// Handle representing the running chaos runtime.
pub struct ChaosRuntime {
    pub handle: PulseHandle,
    pub feedback_tx: mpsc::Sender<ChaosEvent>,
    pub restore_policy: String,
}

/// Start the PulseLoop chaos engine with the config's [chaos] parameters.
pub fn start_chaos_runtime(config: &GzmoConfig) -> ChaosRuntime {
    let chaos_config: ChaosConfig = config.chaos
        .as_ref()
        .and_then(|v| v.clone().try_into().ok())
        .unwrap_or_default();
    let pedagogy = config.pedagogy.tension_oscillation.to_chaos_settings();
    let restore_policy = if chaos_config.rho_restore_alpha > 0.0 {
        format!("tanh (α={:.2}, β={:.2})", chaos_config.rho_restore_alpha, chaos_config.rho_restore_beta)
    } else {
        format!("linear (k={:.4})", chaos_config.rho_decay_k)
    };
    let handle = PulseLoop::start_with_pedagogy(chaos_config, pedagogy);
    let feedback_tx = handle.feedback_tx.clone();
    ChaosRuntime {
        handle,
        feedback_tx,
        restore_policy,
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
    synapse: Option<Arc<SynapseBus>>,
    synapse_source: EventSource,
    restore_policy: String,
    pedagogy_bridge: Option<PedagogyBridgeConfig>,
) -> JoinHandle<()> {
    let mut snapshot_rx = snapshot_rx;
    tokio::spawn(async move {
        let mut triggers = TriggerEngine::with_defaults();
        let mut last_pedagogy_seq: u64 = 0;
        loop {
            if snapshot_rx.changed().await.is_err() {
                break; // PulseLoop dropped
            }
            let snap = snapshot_rx.borrow_and_update().clone();

            if let Some(ref pb) = pedagogy_bridge {
                if snap.pedagogy_transition_seq > last_pedagogy_seq {
                    if let Some(ref t) = snap.pedagogy_last_transition {
                        if t.kind == PedagogyTransitionKind::CycleStart {
                            if let Ok(mut ctx) = pb.cycle_ctx.lock() {
                                ctx.oscillation_id = t.oscillation_id.or(snap.oscillation_id);
                                ctx.chaos_val_baseline =
                                    snap.chaos_val_baseline.or(Some(snap.chaos_val_raw));
                                ctx.knowledge_state_before = knowledge_state_for_cycle_start(
                                    &pb.gzmo_config.memory.vault_db,
                                );
                                ctx.knowledge_state_after = empty_knowledge_state();
                                ctx.knowledge_delta = KnowledgeDelta::default();
                                ctx.spawned_tasks.clear();
                            }
                        }
                        if t.kind == PedagogyTransitionKind::CycleComplete {
                            if let Ok(mut ctx) = pb.cycle_ctx.lock() {
                                ctx.knowledge_state_after = knowledge_state_for_cycle_start(
                                    &pb.gzmo_config.memory.vault_db,
                                );
                                ctx.knowledge_delta = compute_knowledge_delta(
                                    &ctx.knowledge_state_before,
                                    &ctx.knowledge_state_after,
                                );
                            }
                        }
                        if let Some(ref bus) = synapse {
                            let event_type = match t.kind {
                                PedagogyTransitionKind::CycleStart => {
                                    EventType::PedagogyOscillationStart
                                }
                                PedagogyTransitionKind::StepEnter => {
                                    EventType::PedagogyOscillationStep
                                }
                                PedagogyTransitionKind::CycleComplete => {
                                    EventType::PedagogyOscillationComplete
                                }
                            };
                            let ctx = pb.cycle_ctx.lock().unwrap_or_else(|e| e.into_inner());
                            let data = build_pedagogy_event_data(t, &snap, &ctx);
                            let corr = t.oscillation_id.or(snap.oscillation_id);
                            bus.append(&SynapseEvent::with_envelope(
                                event_type,
                                synapse_source,
                                corr,
                                None,
                                Some(data),
                            ));
                        }
                        if t.is_low_phase
                            && pb.oscillation.spawn_discovery_on_low
                            && t.kind == PedagogyTransitionKind::StepEnter
                        {
                            let scripts = pb.scripts_root.clone();
                            let root = pb.gzmo_root.clone();
                            let snap_clone = snap.clone();
                            let config = pb.gzmo_config.clone();
                            let synapse_bus = synapse.clone();
                            let cycle_ctx = pb.cycle_ctx.clone();
                            let osc_id = t.oscillation_id.or(snap.oscillation_id);
                            tokio::spawn(async move {
                                if let Ok(session) = gzmo_core::pedagogy::PedagogySession::load(&config.pedagogy).await {
                                    if session.ops_mode || !session.auto_triggers_enabled {
                                        tracing::info!("Pedagogy session is in ops_mode or auto_triggers_enabled is false — skipping pedagogy oscillation discovery spawn");
                                        return;
                                    }
                                }
                                let bus = synapse_bus.as_deref();
                                match crate::low_tension_dialogue::spawn_discovery_if_allowed(
                                    &config,
                                    bus,
                                    &scripts,
                                    &snap_clone,
                                    &root,
                                    Some("pedagogy oscillation low phase"),
                                    osc_id,
                                    "pedagogy_oscillation",
                                )
                                .await
                                {
                                    Ok(true) => {
                                        if let Ok(mut ctx) = cycle_ctx.lock() {
                                            ctx.spawned_tasks.push(json!({
                                                "trigger": "pedagogy_oscillation",
                                                "oscillation_id": osc_id.map(|u| u.to_string()),
                                                "spawned_at": chrono::Utc::now().to_rfc3339(),
                                            }));
                                        }
                                    }
                                    Ok(false) => tracing::info!(
                                        "Pedagogy oscillation discovery deferred by ObolusGate"
                                    ),
                                    Err(e) => tracing::warn!(
                                        error = %e,
                                        "Pedagogy oscillation discovery spawn failed"
                                    ),
                                }
                            });
                        }
                        if t.kind == PedagogyTransitionKind::CycleComplete {
                            if let Ok(mut ctx) = pb.cycle_ctx.lock() {
                                ctx.oscillation_id = None;
                                ctx.chaos_val_baseline = None;
                                ctx.spawned_tasks.clear();
                                ctx.knowledge_state_before = empty_knowledge_state();
                                ctx.knowledge_state_after = empty_knowledge_state();
                                ctx.knowledge_delta = KnowledgeDelta::default();
                            }
                        }
                    }
                    last_pedagogy_seq = snap.pedagogy_transition_seq;
                }
            }

            // Drain external skill feedback (Pi / `gzmo chaos skill`)
            let inbox = state_dir.join("chaos_feedback_inbox.jsonl");
            let drained = gzmo_chaos::feedback_ipc::drain_inbox(&inbox);
            if !drained.is_empty() {
                let mut by_type: HashMap<String, usize> = HashMap::new();
                for event in &drained {
                    let label = gzmo_chaos::feedback_ipc::event_type_label(event).to_string();
                    *by_type.entry(label).or_default() += 1;
                    let _ = gzmo_chaos::feedback_ipc::append_audit(&state_dir, event, "drained");
                    let _ = feedback_tx.send(event.clone()).await;
                }
                if let Some(ref bus) = synapse {
                    let count = drained.len();
                    let summary: Vec<String> = by_type
                        .iter()
                        .map(|(k, v)| format!("{k}×{v}"))
                        .collect();
                    let display_plain = format!(
                        "Drained {count} chaos feedback event(s): {}",
                        summary.join(", ")
                    );
                    bus.append(&SynapseEvent::with_data(
                        EventType::ChaosFeedbackDrained,
                        synapse_source,
                        serde_json::json!({
                            "count": count,
                            "by_type": by_type,
                            "display_plain": display_plain,
                        }),
                    ));
                }
            }
            
            // Update gateway LLM parameters from Lorenz coordinates
            {
                let gw = gateway.read().await;
                gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
            }

            // Soul hook: correlate crystallization impulses with ρ telemetry on Synapse
            if let (Some(ref c), Some(ref bus)) = (&snap.last_crystallization, &synapse) {
                bus.append(&SynapseEvent::with_data(
                    EventType::ThoughtCrystallized,
                    synapse_source,
                    serde_json::json!({
                        "category": c.category,
                        "mutation_target": c.mutation.target,
                        "mutation_delta": c.mutation.delta,
                        "description": c.mutation.description,
                        "tick": snap.tick,
                        "rho_effective": snap.rho_effective,
                        "rho_mod_delta": snap.rho_mod_delta,
                        "rho_forcing_sign": snap.rho_forcing_sign,
                        "rho_breath_phase": snap.rho_breath_phase,
                        "thoughts_crystallized": snap.thoughts_crystallized,
                    }),
                ));
            }

            // Write snapshot files for shell skill backward compat (more often during oscillation)
            if snap.tick % 15 == 0 || snap.pedagogy_oscillation_active {
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
                    *Updated: {}*\n",
                    if snap.alive { "ALIVE" } else { "DEAD" },
                    snap.tick, snap.energy, snap.phase, snap.deaths, snap.tension,
                    snap.chaos_val,
                    restore_policy,
                    snap.x, snap.y, snap.z,
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

                if let Some(ref bus) = synapse {
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
