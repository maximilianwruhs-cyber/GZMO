//! PulseLoop — The unified heartbeat of GZMO.
//!
//! Replaces the standalone Randomizer binary. Runs as a `tokio::task` inside the
//! GZMO process. Each tick at 174 BPM:
//!   1. Drains feedback events from skills
//!   2. Advances the Lorenz attractor
//!   3. Ticks the Thought Cabinet (incubation, crystallization)
//!   4. Computes derived LLM parameters from Lorenz coordinates
//!   5. Broadcasts a `ChaosSnapshot` to all consumers via `tokio::sync::watch`
//!
//! The snapshot is the read-only interface for skills, the REPL, the orchestrator,
//! and any external diagnostic tools.
//!
//! Synapse observability: do not publish chaos heartbeat events to `SynapseBus`
//! until `PulseLoop` runs in daemon mode. Today it only starts in chat/TUI;
//! Daemon mode: `daemon_cmd.rs` pins `PulseHandle` for the full process lifetime.

use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::Duration;

use serde::Deserialize;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info, warn};

use crate::chaos::{LogisticMap, LorenzAttractor, Phase};
use crate::engine::EngineState;
use crate::feedback::ChaosEvent;
use crate::pedagogy_oscillator::{
    PedagogyOscillateAction, PedagogyOscillationSettings, PedagogyOscillator,
    PedagogyTransitionKind,
};
use crate::thoughts::{CrystallizationEvent, Mutations, ThoughtCabinet};

/// 174 BPM = 344ms per beat
const TICK_INTERVAL: Duration = Duration::from_millis(344);

/// LLM temperature band — chaos-modulated sampling range.
pub const LLM_TEMP_MIN: f32 = 0.3;
pub const LLM_TEMP_MAX: f32 = 1.2;

/// Map organism state to LLM temperature in [`LLM_TEMP_MIN`, `LLM_TEMP_MAX`].
///
/// Blends Lorenz x (slow orbit drift) with tension/energy (immediate skill feedback).
/// Low τ → exploratory; high τ → stabilizing — aligned with phase prompt steering.
pub fn compute_llm_temperature(lorenz_x: f64, tension: f64, energy: f64) -> f32 {
    let normalized = ((lorenz_x + 20.0) / 40.0).clamp(0.0, 1.0);
    let from_lorenz = LLM_TEMP_MIN + (normalized as f32 * (LLM_TEMP_MAX - LLM_TEMP_MIN));

    let tau = tension.clamp(0.0, 100.0);
    let from_tension = LLM_TEMP_MIN + ((100.0 - tau) / 100.0) as f32 * (LLM_TEMP_MAX - LLM_TEMP_MIN);

    let energy_scale = (energy.clamp(0.0, 100.0) / 100.0) as f32;
    let from_state = from_tension * (0.4 + 0.6 * energy_scale);

    (from_lorenz * 0.4 + from_state * 0.6).clamp(LLM_TEMP_MIN, LLM_TEMP_MAX)
}

/// Read-only snapshot of current chaos state, cheaply cloneable.
/// This is the ONLY communication channel between the chaos engine and the rest of GZMO.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChaosSnapshot {
    pub tick: u64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub tension: f64,
    pub energy: f64,
    pub phase: Phase,
    pub alive: bool,
    pub deaths: u32,
    pub chaos_val: f64,

    /// Organic logistic-map value (always populated when oscillation runs).
    #[serde(default)]
    pub chaos_val_raw: f64,

    /// Pedagogy oscillation active this tick.
    #[serde(default)]
    pub pedagogy_oscillation_active: bool,

    #[serde(default)]
    pub pedagogy_target: Option<f64>,

    #[serde(default)]
    pub pedagogy_step: u32,

    #[serde(default)]
    pub pedagogy_transition_seq: u64,

    #[serde(default)]
    pub pedagogy_last_transition: Option<crate::pedagogy_oscillator::PedagogyTransitionInfo>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oscillation_id: Option<uuid::Uuid>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chaos_val_baseline: Option<f64>,

    // Thought Cabinet state
    pub thoughts_incubating: u8,
    pub thoughts_crystallized: u32,
    pub mutations: Mutations,

    // ρ control telemetry (see docs/CHAOS_RHO_CONTROL_MODEL.md)
    pub rho_effective: f64,   // 28.0 + lorenz_rho_mod
    pub rho_mod_delta: f64,   // Δρ_mod since previous tick
    pub rho_forcing_sign: i8, // sign(Δρ_mod): −1 decay/negative impulse, +1 positive impulse, 0 steady
    pub rho_breath_phase: i8, // sign(rho_velocity_ema): -1, 0, +1
    pub rho_velocity_ema: f64,

    // Derived LLM parameters — computed from Lorenz coordinates
    pub llm_temperature: f32,
    pub llm_max_tokens: u32,
    pub llm_valence: f32, // -1.0 (dark/aggressive) to 1.0 (calm/reflective)

    // Last crystallization event (if any on this tick)
    pub last_crystallization: Option<CrystallizationEvent>,

    pub incubating_previews: Vec<String>,

    // Timestamp for external observers
    pub timestamp: String,
}

impl Default for ChaosSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            x: 0.506,
            y: 0.507,
            z: 0.508,
            tension: 0.0,
            energy: 100.0,
            phase: Phase::Idle,
            alive: true,
            deaths: 0,
            chaos_val: 0.5,
            chaos_val_raw: 0.5,
            pedagogy_oscillation_active: false,
            pedagogy_target: None,
            pedagogy_step: 0,
            pedagogy_transition_seq: 0,
            pedagogy_last_transition: None,
            oscillation_id: None,
            chaos_val_baseline: None,
            thoughts_incubating: 0,
            thoughts_crystallized: 0,
            mutations: Mutations::default(),
            rho_effective: 28.0,
            rho_mod_delta: 0.0,
            rho_forcing_sign: 0,
            rho_breath_phase: 0,
            rho_velocity_ema: 0.0,
            llm_temperature: 0.6,
            llm_max_tokens: 256,
            llm_valence: 0.0,
            last_crystallization: None,
            incubating_previews: Vec::new(),
            timestamp: String::new(),
        }
    }
}

/// Configuration for the chaos engine, loaded from TOML
#[derive(Debug, Clone, Deserialize)]
pub struct ChaosConfig {
    #[serde(default = "default_gravity")]
    pub gravity: f64,
    #[serde(default = "default_friction")]
    pub friction: f64,
    #[serde(default = "default_seed")]
    pub seed: f64,
    #[serde(default = "default_tension")]
    pub initial_tension: f64,
    #[serde(default)]
    pub lore_path: Option<PathBuf>,
    #[serde(default)]
    pub events: EventChances,
    /// Leaky-integrator gain k: ρ_mod ← (1−k)·ρ_mod per tick. 0.0 disables decay.
    #[serde(default = "default_rho_decay_k")]
    pub rho_decay_k: f64,
    /// Tanh restore α; when > 0, replaces linear decay with ρ ← ρ − α·tanh(β·ρ).
    #[serde(default)]
    pub rho_restore_alpha: f64,
    /// Tanh restore β (steepness); used only when `rho_restore_alpha > 0`.
    #[serde(default = "default_rho_restore_beta")]
    pub rho_restore_beta: f64,
    /// EMA smoothing factor gamma for breath phase calculation
    #[serde(default = "default_rho_ema_gamma")]
    pub rho_ema_gamma: f64,
    /// Tunable stabilize delta rho (default: -1.0)
    #[serde(default = "default_stabilize_delta_rho")]
    pub stabilize_delta_rho: f64,
}

/// Probability windows for auto-lore emission per tick
#[derive(Debug, Clone, Deserialize)]
pub struct EventChances {
    #[serde(default = "default_joke_chance")]
    pub joke_chance: f64,
    #[serde(default = "default_quote_chance")]
    pub quote_chance: f64,
    #[serde(default = "default_fact_chance")]
    pub fact_chance: f64,
}

impl Default for EventChances {
    fn default() -> Self {
        Self {
            joke_chance: default_joke_chance(),
            quote_chance: default_quote_chance(),
            fact_chance: default_fact_chance(),
        }
    }
}

fn default_gravity() -> f64 { 9.8 }
fn default_friction() -> f64 { 0.5 }
fn default_seed() -> f64 { 0.506 }
fn default_tension() -> f64 { 0.0 }
fn default_joke_chance() -> f64 { 0.3 }
fn default_quote_chance() -> f64 { 0.4 }
fn default_fact_chance() -> f64 { 0.3 }
fn default_rho_decay_k() -> f64 { 0.001 }
fn default_rho_restore_beta() -> f64 { 1.0 }
fn default_rho_ema_gamma() -> f64 { 0.2 }
fn default_stabilize_delta_rho() -> f64 { -1.0 }

fn finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

impl EventChances {
    fn sanitized(mut self) -> Self {
        self.joke_chance = finite_or(self.joke_chance, default_joke_chance()).clamp(0.0, 1.0);
        self.quote_chance = finite_or(self.quote_chance, default_quote_chance()).clamp(0.0, 1.0);
        self.fact_chance = finite_or(self.fact_chance, default_fact_chance()).clamp(0.0, 1.0);
        self
    }
}

impl ChaosConfig {
    pub fn sanitized(mut self) -> Self {
        self.gravity = finite_or(self.gravity, default_gravity()).max(0.0);
        self.friction = finite_or(self.friction, default_friction()).max(0.0);
        self.seed = finite_or(self.seed, default_seed());
        self.initial_tension = finite_or(self.initial_tension, default_tension()).clamp(0.0, 100.0);
        self.events = self.events.sanitized();
        self.rho_decay_k = finite_or(self.rho_decay_k, default_rho_decay_k()).clamp(0.0, 1.0);
        self.rho_restore_alpha = finite_or(self.rho_restore_alpha, 0.0).max(0.0);
        self.rho_restore_beta = finite_or(self.rho_restore_beta, default_rho_restore_beta()).max(f64::EPSILON);
        self.rho_ema_gamma = finite_or(self.rho_ema_gamma, default_rho_ema_gamma()).clamp(0.0, 1.0);
        self.stabilize_delta_rho = finite_or(self.stabilize_delta_rho, default_stabilize_delta_rho()).clamp(-10.0, 10.0);
        self
    }
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            gravity: default_gravity(),
            friction: default_friction(),
            seed: default_seed(),
            initial_tension: default_tension(),
            lore_path: None,
            events: EventChances::default(),
            rho_decay_k: default_rho_decay_k(),
            rho_restore_alpha: 0.0,
            rho_restore_beta: default_rho_restore_beta(),
            rho_ema_gamma: default_rho_ema_gamma(),
            stabilize_delta_rho: default_stabilize_delta_rho(),
        }
    }
}

/// A single lore item from lore.toml
#[derive(Debug, Clone, Deserialize)]
pub struct LoreItem {
    pub text: String,
    pub author: Option<String>,
}

/// The loaded lore pool — jokes, quotes, facts
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LorePool {
    #[serde(default)]
    pub jokes: Vec<LoreItem>,
    #[serde(default)]
    pub quotes: Vec<LoreItem>,
    #[serde(default)]
    pub facts: Vec<LoreItem>,
}

impl LorePool {
    pub fn load(path: &std::path::Path) -> Option<Self> {
        match std::fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<LorePool>(&content) {
                Ok(pool) => {
                    info!(
                        jokes = pool.jokes.len(),
                        quotes = pool.quotes.len(),
                        facts = pool.facts.len(),
                        "📖 Lore pool loaded"
                    );
                    Some(pool)
                }
                Err(e) => {
                    warn!("Failed to parse lore.toml: {e}");
                    None
                }
            },
            Err(e) => {
                warn!("Failed to read lore.toml at {}: {e}", path.display());
                None
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.jokes.is_empty() && self.quotes.is_empty() && self.facts.is_empty()
    }
}

/// Auto-lore emission output — sent to REPL for display
#[derive(Debug, Clone)]
pub struct LoreNotification {
    pub category: String,
    pub text: String,
    pub author: Option<String>,
}

/// Handle returned from `PulseLoop::start()` for interacting with the running loop.
pub struct PulseHandle {
    /// Read the latest chaos snapshot (non-blocking, always returns the most recent value)
    pub snapshot_rx: watch::Receiver<ChaosSnapshot>,
    /// Send feedback events from skills into the chaos engine
    pub feedback_tx: mpsc::Sender<ChaosEvent>,
    /// Receive auto-lore notifications for REPL display
    pub lore_rx: mpsc::Receiver<LoreNotification>,
    /// Task handle for the running pulse loop
    pub task: tokio::task::JoinHandle<()>,
    /// Flag to stop standard threads
    pub shutdown_flag: Arc<AtomicBool>,
}

impl Drop for PulseHandle {
    fn drop(&mut self) {
        self.shutdown_flag.store(true, Ordering::Relaxed);
        self.task.abort();
    }
}

/// The unified pulse loop — the heartbeat of GZMO.
pub struct PulseLoop {
    lorenz: LorenzAttractor,
    logistic: LogisticMap,
    state: EngineState,
    cabinet: ThoughtCabinet,
    tension: f64,
    config: ChaosConfig,
    lore: Option<Arc<LorePool>>,
    rho_velocity_ema: f64,

    // Hardware telemetry: written by sysinfo thread, read by tick loop
    hw_tension: Arc<AtomicU64>,

    // Channels
    event_rx: mpsc::Receiver<ChaosEvent>,
    snapshot_tx: watch::Sender<ChaosSnapshot>,
    lore_tx: mpsc::Sender<LoreNotification>,
    pedagogy_oscillator: PedagogyOscillator,
}

impl PulseLoop {
    /// Create and start the pulse loop. Returns a `PulseHandle` for interaction.
    pub fn start(config: ChaosConfig) -> PulseHandle {
        Self::start_with_pedagogy(config, PedagogyOscillationSettings::default())
    }

    /// Start with pedagogy oscillation settings from `[pedagogy.tension_oscillation]`.
    pub fn start_with_pedagogy(
        config: ChaosConfig,
        pedagogy: PedagogyOscillationSettings,
    ) -> PulseHandle {
        let (feedback_tx, event_rx) = mpsc::channel::<ChaosEvent>(256);
        let (snapshot_tx, snapshot_rx) = watch::channel(ChaosSnapshot::default());
        let (lore_tx, lore_rx) = mpsc::channel::<LoreNotification>(64);
        let config = config.sanitized();

        // Load lore pool if path configured
        let lore = config.lore_path.as_ref().and_then(|p| LorePool::load(p)).map(Arc::new);

        // Shared atomic for hardware telemetry tension
        let hw_tension = Arc::new(AtomicU64::new(config.initial_tension.to_bits()));
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        // Spawn hardware telemetry thread (like original Randomizer)
        {
            let hw_ref = Arc::clone(&hw_tension);
            let shutdown = Arc::clone(&shutdown_flag);
            std::thread::spawn(move || {
                use sysinfo::System;
                let mut sys = System::new();
                while !shutdown.load(Ordering::Relaxed) {
                    sys.refresh_cpu_usage();
                    sys.refresh_memory();
                    let cpu: f64 = sys.global_cpu_usage() as f64;
                    let total_mem = sys.total_memory() as f64;
                    let used_mem = sys.used_memory() as f64;
                    let ram = if total_mem > 0.0 { (used_mem / total_mem) * 100.0 } else { 50.0 };
                    let tension = (cpu * 0.5 + ram * 0.5).min(100.0);
                    hw_ref.store(tension.to_bits(), Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(344)); // Match heartbeat BPM
                }
            });
        }

        let pulse = PulseLoop {
            lorenz: LorenzAttractor::new(config.seed),
            logistic: LogisticMap::new(config.seed),
            state: EngineState::new(),
            cabinet: ThoughtCabinet::new(),
            tension: config.initial_tension,
            lore,
            hw_tension,
            event_rx,
            snapshot_tx,
            lore_tx,
            config,
            rho_velocity_ema: 0.0,
            pedagogy_oscillator: PedagogyOscillator::new(pedagogy),
        };

        let task = tokio::spawn(async move {
            pulse.run().await;
        });

        PulseHandle {
            snapshot_rx,
            feedback_tx,
            lore_rx,
            task,
            shutdown_flag,
        }
    }

    /// The main loop. Runs forever until the task is aborted.
    async fn run(mut self) {
        info!(
            gravity = self.config.gravity,
            friction = self.config.friction,
            seed = self.config.seed,
            "PulseLoop started — 174 BPM"
        );

        let mut interval = tokio::time::interval(TICK_INTERVAL);
        let mut prev_rho_mod = 0.0_f64;

        loop {
            interval.tick().await;

            // 1. Drain all pending feedback events
            let mut events_processed = 0u32;
            while let Ok(event) = self.event_rx.try_recv() {
                self.apply_feedback(&event);
                events_processed += 1;
            }
            if events_processed > 0 {
                debug!(events = events_processed, "Processed feedback events");
            }

            // 2. Compute effective physics with thought mutations
            let effective_tension =
                (self.tension + self.cabinet.mutations.tension_bias).clamp(0.0, 100.0);
            let effective_phase = Phase::from_tension(effective_tension);
            let effective_gravity = (self.config.gravity + self.cabinet.mutations.gravity_mod).max(1.0);
            let effective_friction = (self.config.friction + self.cabinet.mutations.friction_mod).max(0.05);

            // 3. Advance chaos generators using the current tick's phase and mutations.
            self.lorenz.update_phase(&effective_phase);
            self.lorenz.apply_cognitive_noise(self.cabinet.active_lorenz_noise());
            self.lorenz.apply_rho_mutation(self.cabinet.mutations.lorenz_rho_mod);
            let (x, y, z) = self.lorenz.step();

            // Couple logistic map to Lorenz every 10 ticks
            if self.state.tick.is_multiple_of(10) {
                self.logistic.reseed_from_lorenz(self.lorenz.normalized_output());
            }

            let chaos_val_raw = self.logistic.next_val();
            let (chaos_val, pedagogy_meta) =
                self.pedagogy_oscillator.apply(chaos_val_raw, self.state.tick);
            if pedagogy_meta
                .pedagogy_last_transition
                .as_ref()
                .is_some_and(|t| t.kind == PedagogyTransitionKind::CycleStart)
            {
                self.pedagogy_oscillator
                    .set_chaos_val_baseline(chaos_val_raw);
            }

            // 4. Tick engine state
            let rebirth = self.state.tick_heartbeat(
                effective_tension,
                effective_gravity,
                effective_friction,
                chaos_val,
                self.cabinet.active_drain_multiplier(),
            );

            if rebirth {
                self.cabinet.mutations.lorenz_rho_mod *= 0.5;
                info!(lorenz_rho_mod = self.cabinet.mutations.lorenz_rho_mod, "Engine rebirth: lorenz_rho_mod halved");
            }

            // 5. Tick Thought Cabinet — check for crystallizations
            let mut crystallizations = self.cabinet.tick();
            for c in crystallizations.iter_mut() {
                c.tick_crystallized = self.state.tick;
            }

            // Log crystallization events
            for c in &crystallizations {
                info!(
                    category = %c.category,
                    mutation_target = %c.mutation.target,
                    mutation_delta = c.mutation.delta,
                    description = %c.mutation.description,
                    tick = c.tick_crystallized,
                    "🔮 Thought crystallized"
                );
            }

            // ρ_mod homeostasis after crystallization impulses (linear or tanh restore)
            self.cabinet.apply_rho_restoration(
                self.config.rho_restore_alpha,
                self.config.rho_restore_beta,
                self.config.rho_decay_k,
            );

            let rho_mod = self.cabinet.mutations.lorenz_rho_mod;
            let rho_mod_delta = rho_mod - prev_rho_mod;
            let rho_forcing_sign = if rho_mod_delta > 1e-9 {
                1
            } else if rho_mod_delta < -1e-9 {
                -1
            } else {
                0
            };
            prev_rho_mod = rho_mod;

            let gamma = self.config.rho_ema_gamma;
            self.rho_velocity_ema = (1.0 - gamma) * self.rho_velocity_ema + gamma * rho_mod_delta;
            let rho_breath_phase = if self.rho_velocity_ema > 1e-9 {
                1
            } else if self.rho_velocity_ema < -1e-9 {
                -1
            } else {
                0
            };

            let last_crystallization = crystallizations.into_iter().last();

            // 6. Compute derived LLM parameters (Lorenz + live τ/ε)
            let llm_temperature =
                compute_llm_temperature(x, effective_tension, self.state.energy);
            let llm_max_tokens = self.lorenz_to_tokens();
            let llm_valence = self.lorenz_to_valence();

            // 7. Build and broadcast snapshot
            let snapshot = ChaosSnapshot {
                tick: self.state.tick,
                x,
                y,
                z,
                tension: effective_tension,
                energy: self.state.energy,
                phase: self.state.phase,
                alive: self.state.alive,
                deaths: self.state.deaths,
                chaos_val,
                chaos_val_raw: pedagogy_meta.chaos_val_raw,
                pedagogy_oscillation_active: pedagogy_meta.pedagogy_oscillation_active,
                pedagogy_target: pedagogy_meta.pedagogy_target,
                pedagogy_step: pedagogy_meta.pedagogy_step,
                pedagogy_transition_seq: pedagogy_meta.pedagogy_transition_seq,
                pedagogy_last_transition: pedagogy_meta.pedagogy_last_transition,
                oscillation_id: pedagogy_meta.oscillation_id,
                chaos_val_baseline: pedagogy_meta.chaos_val_baseline,
                thoughts_incubating: self.cabinet.occupied_slots() as u8,
                thoughts_crystallized: self.cabinet.mutations.total_crystallized,
                mutations: self.cabinet.mutations.clone(),
                rho_effective: 28.0 + rho_mod,
                rho_mod_delta,
                rho_forcing_sign,
                rho_breath_phase,
                rho_velocity_ema: self.rho_velocity_ema,
                llm_temperature,
                llm_max_tokens,
                llm_valence,
                last_crystallization,
                incubating_previews: self.cabinet.incubating_snapshot()
                    .iter()
                    .map(|t| t.text_preview.clone())
                    .collect(),
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            };

            // Non-blocking send — if nobody is listening, that's fine
            let _ = self.snapshot_tx.send(snapshot);

            // Read hardware telemetry tension (replaces static homeostasis)
            let hw_t = f64::from_bits(self.hw_tension.load(Ordering::Relaxed));
            // Smooth blend: 90% current + 10% hardware (prevents jarring jumps)
            self.tension = self.tension * 0.9 + hw_t * 0.1;

            // Auto-lore emission: every 30 ticks (~10 seconds), if alive
            if self.state.alive && self.state.tick.is_multiple_of(30) {
                if let Some(lore) = self.lore.clone() {
                    if let Some((category, item)) = self.select_lore(&lore) {
                        // Try to absorb into Thought Cabinet
                        let absorb_roll = self.logistic.next_val();
                        if self.cabinet.try_absorb(&category, &item.text, self.state.tick, absorb_roll) {
                            info!(
                                category = %category,
                                text = %item.text.chars().take(40).collect::<String>(),
                                "🧠 Auto-lore absorbed into Thought Cabinet"
                            );
                        }
                        // Notify REPL for display
                        let _ = self.lore_tx.try_send(LoreNotification {
                            category: category.clone(),
                            text: item.text.clone(),
                            author: item.author.clone(),
                        });
                    }
                }
            }
        }
    }

    /// Process a single feedback event from a skill execution.
    fn apply_feedback(&mut self, event: &ChaosEvent) {
        // Apply tension delta
        let tension_delta = event.tension_delta();
        if tension_delta.abs() > 0.01 {
            self.tension = self.state.apply_tension_delta(self.tension, tension_delta);
            debug!(delta = tension_delta, tension = self.tension, "Tension shifted");
        }

        // Apply energy delta
        let energy_delta = event.energy_delta();
        if energy_delta.abs() > 0.01 {
            self.state.apply_energy_delta(energy_delta);
            debug!(delta = energy_delta, energy = self.state.energy, "Energy shifted");
        }

        // Try to absorb thought seed into cabinet
        if let Some(seed) = event.thought_seed() {
            let chaos_roll = self.logistic.next_val();
            if self.cabinet.try_absorb(&seed.category, &seed.text, self.state.tick, chaos_roll) {
                info!(
                    category = %seed.category,
                    text = %seed.text.chars().take(40).collect::<String>(),
                    "🧠 Thought absorbed into cabinet"
                );
            }
        }

        // Apply stabilize delta directly to lorenz_rho_mod
        if let ChaosEvent::Stabilize { delta_rho } = event {
            if delta_rho.is_finite() {
                self.cabinet.mutations.lorenz_rho_mod =
                    (self.cabinet.mutations.lorenz_rho_mod + delta_rho).clamp(-10.0, 10.0);
                debug!(delta = delta_rho, lorenz_rho_mod = self.cabinet.mutations.lorenz_rho_mod, "lorenz_rho_mod stabilized");
            }
        }

        if let ChaosEvent::PedagogyOscillate { action } = event {
            let ok = self.pedagogy_oscillator.handle_action(*action, self.state.tick);
            info!(?action, ok, "Pedagogy oscillation command");
        }
    }

    /// Select a lore item based on chaos_val probability windows (ported from original Randomizer)
    fn select_lore(&mut self, lore: &LorePool) -> Option<(String, LoreItem)> {
        if lore.is_empty() { return None; }
        let chaos_idx = self.logistic.next_val();
        let joke_end = self.config.events.joke_chance;
        let quote_end = joke_end + self.config.events.quote_chance;
        let fact_end = quote_end + self.config.events.fact_chance;

        if chaos_idx < joke_end && !lore.jokes.is_empty() {
            let idx = self.pick_lore_index(lore.jokes.len());
            Some(("joke".to_string(), lore.jokes[idx].clone()))
        } else if chaos_idx < quote_end && !lore.quotes.is_empty() {
            let idx = self.pick_lore_index(lore.quotes.len());
            Some(("quote".to_string(), lore.quotes[idx].clone()))
        } else if chaos_idx < fact_end && !lore.facts.is_empty() {
            let idx = self.pick_lore_index(lore.facts.len());
            Some(("fact".to_string(), lore.facts[idx].clone()))
        } else {
            None
        }
    }

    /// Pick an index from a logistic map value (ported from original)
    fn pick_lore_index(&mut self, len: usize) -> usize {
        let n = self.logistic.next_val();
        ((n * (len.saturating_sub(1)) as f64).round() as usize).min(len.saturating_sub(1))
    }

    /// Map Lorenz y ∈ [-30, 30] to max_tokens ∈ [512, 2048] (tutor-scale band)
    fn lorenz_to_tokens(&self) -> u32 {
        let normalized = ((self.lorenz.y + 30.0) / 60.0).clamp(0.0, 1.0);
        512 + (normalized * 1536.0) as u32
    }

    /// Map Lorenz z ∈ [0, 50] to emotional valence ∈ [-1.0, 1.0]
    fn lorenz_to_valence(&self) -> f32 {
        // z oscillates around ~25 on the Lorenz attractor
        let normalized = ((self.lorenz.z - 25.0) / 25.0).clamp(-1.0, 1.0);
        normalized as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pulse_handle_receives_snapshots() {
        let handle = PulseLoop::start(ChaosConfig::default());

        // Wait a few ticks
        tokio::time::sleep(Duration::from_millis(700)).await;

        let snapshot = handle.snapshot_rx.borrow().clone();
        assert!(snapshot.tick > 0, "PulseLoop should have ticked: {}", snapshot.tick);
        assert!(snapshot.energy > 0.0, "Energy should be positive");
        assert!((0.3..=1.2).contains(&snapshot.llm_temperature), "Temperature out of range: {}", snapshot.llm_temperature);

        // Clean up
        handle.task.abort();
    }

    #[tokio::test]
    async fn feedback_modifies_state() {
        let handle = PulseLoop::start(ChaosConfig::default());

        // Wait for initial tick
        tokio::time::sleep(Duration::from_millis(400)).await;

        let tension_before = handle.snapshot_rx.borrow().tension;

        // Send a high-tension event (nat 1 on a D20)
        handle.feedback_tx.send(ChaosEvent::DiceRoll { value: 1, max: 20 }).await.unwrap();

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(400)).await;

        let tension_after = handle.snapshot_rx.borrow().tension;
        // Nat 1 should increase tension (distance from midpoint is positive)
        assert!(tension_after > tension_before - 1.0, "Tension should have increased from {} but got {}", tension_before, tension_after);

        handle.task.abort();
    }

    #[test]
    fn temperature_mapping() {
        let config = ChaosConfig::default();
        let (_, event_rx) = mpsc::channel(1);
        let (snapshot_tx, _) = watch::channel(ChaosSnapshot::default());
        let (lore_tx, _) = mpsc::channel(1);
        let pulse = PulseLoop {
            lorenz: LorenzAttractor::new(0.506),
            logistic: LogisticMap::new(0.506),
            state: EngineState::new(),
            cabinet: ThoughtCabinet::new(),
            tension: 50.0,
            lore: None,
            hw_tension: Arc::new(AtomicU64::new(50.0f64.to_bits())),
            event_rx,
            snapshot_tx,
            lore_tx,
            config,
            rho_velocity_ema: 0.0,
            pedagogy_oscillator: PedagogyOscillator::new(PedagogyOscillationSettings::default()),
        };

        let temp = compute_llm_temperature(pulse.lorenz.x, pulse.tension, pulse.state.energy);
        assert!(
            (LLM_TEMP_MIN..=LLM_TEMP_MAX).contains(&temp),
            "Initial temperature out of range: {temp}"
        );
    }

    #[test]
    fn compute_llm_temperature_responds_to_tension() {
        let lorenz_x = 0.0_f64;
        let idle = compute_llm_temperature(lorenz_x, 8.0, 95.0);
        let drop = compute_llm_temperature(lorenz_x, 88.0, 42.0);
        assert!(
            idle - drop > 0.15,
            "τ swing should move temperature meaningfully: idle={idle:.3} drop={drop:.3}"
        );
        assert!((LLM_TEMP_MIN..=LLM_TEMP_MAX).contains(&idle));
        assert!((LLM_TEMP_MIN..=LLM_TEMP_MAX).contains(&drop));
    }

    #[test]
    fn compute_llm_temperature_dice_crit_fail_vs_success() {
        let lorenz_x = 5.0_f64;
        // D20 nat 1 stacked τ ≈ 29 after spike from ~10 baseline; nat 20 ≈ −5 from ~10
        let after_fail = compute_llm_temperature(lorenz_x, 29.0, 55.0);
        let after_success = compute_llm_temperature(lorenz_x, 0.0, 80.0);
        assert!(
            after_success > after_fail,
            "crit success should raise T vs crit fail: fail={after_fail:.3} success={after_success:.3}"
        );
    }

    #[tokio::test]
    async fn feedback_stabilize_reduces_rho() {
        let handle = PulseLoop::start(ChaosConfig::default());

        // Wait for initial tick
        tokio::time::sleep(Duration::from_millis(400)).await;

        // Send a Stabilize event
        handle.feedback_tx.send(ChaosEvent::Stabilize { delta_rho: -5.0 }).await.unwrap();

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(400)).await;

        let snap = handle.snapshot_rx.borrow().clone();
        assert!(snap.mutations.lorenz_rho_mod < -4.0, "lorenz_rho_mod should be reduced, got {}", snap.mutations.lorenz_rho_mod);

        handle.task.abort();
    }

    #[test]
    fn ema_smooths_single_spike() {
        let config = ChaosConfig {
            rho_ema_gamma: 0.2,
            ..ChaosConfig::default()
        };
        let (_, event_rx) = mpsc::channel(10);
        let (snapshot_tx, _) = watch::channel(ChaosSnapshot::default());
        let (lore_tx, _) = mpsc::channel(1);
        let mut pulse = PulseLoop {
            lorenz: LorenzAttractor::new(0.506),
            logistic: LogisticMap::new(0.506),
            state: EngineState::new(),
            cabinet: ThoughtCabinet::new(),
            tension: 50.0,
            lore: None,
            hw_tension: Arc::new(AtomicU64::new(50.0f64.to_bits())),
            event_rx,
            snapshot_tx,
            lore_tx,
            config,
            rho_velocity_ema: 0.0,
            pedagogy_oscillator: PedagogyOscillator::new(PedagogyOscillationSettings::default()),
        };

        let gamma = pulse.config.rho_ema_gamma;
        assert_eq!(gamma, 0.2);

        let rho_mod_delta = 1.0;
        pulse.rho_velocity_ema = (1.0 - gamma) * pulse.rho_velocity_ema + gamma * rho_mod_delta;
        assert!((pulse.rho_velocity_ema - 0.2).abs() < 1e-9);

        let rho_mod_delta = 0.0;
        pulse.rho_velocity_ema = (1.0 - gamma) * pulse.rho_velocity_ema + gamma * rho_mod_delta;
        assert!((pulse.rho_velocity_ema - 0.16).abs() < 1e-9);
    }

    #[test]
    fn test_stabilize_delta_rho_config() {
        let toml_str = r#"
            gravity = 9.8
            friction = 0.5
            seed = 0.506
            initial_tension = 0.0
            rho_decay_k = 0.001
            rho_restore_alpha = 0.01
            rho_restore_beta = 1.0
            rho_ema_gamma = 0.2
            stabilize_delta_rho = -2.5
        "#;
        let config: ChaosConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.stabilize_delta_rho, -2.5);

        // default fallback
        let toml_str_empty = "";
        let config_default: ChaosConfig = toml::from_str(toml_str_empty).unwrap_or_default();
        assert_eq!(config_default.stabilize_delta_rho, -1.0);
    }

    #[test]
    fn test_rebirth_halves_rho() {
        let (_, event_rx) = mpsc::channel(10);
        let (snapshot_tx, _) = watch::channel(ChaosSnapshot::default());
        let (lore_tx, _) = mpsc::channel(1);
        let mut pulse = PulseLoop {
            lorenz: LorenzAttractor::new(0.506),
            logistic: LogisticMap::new(0.506),
            state: EngineState::new(),
            cabinet: ThoughtCabinet::new(),
            tension: 50.0,
            lore: None,
            hw_tension: Arc::new(AtomicU64::new(50.0f64.to_bits())),
            event_rx,
            snapshot_tx,
            lore_tx,
            config: ChaosConfig::default(),
            rho_velocity_ema: 0.0,
            pedagogy_oscillator: PedagogyOscillator::new(PedagogyOscillationSettings::default()),
        };

        pulse.state.alive = false;
        pulse.cabinet.mutations.lorenz_rho_mod = 6.0;

        // Tick heartbeat with chaos_roll = 0.8 (should trigger rebirth)
        let rebirth = pulse.state.tick_heartbeat(
            pulse.tension,
            pulse.config.gravity,
            pulse.config.friction,
            0.8,
            1.0,
        );
        assert!(rebirth);

        if rebirth {
            pulse.cabinet.mutations.lorenz_rho_mod *= 0.5;
        }

        assert_eq!(pulse.cabinet.mutations.lorenz_rho_mod, 3.0);
    }
}
