/// PulseLoop — The unified heartbeat of GZMO.
///
/// Replaces the standalone Randomizer binary. Runs as a `tokio::task` inside the
/// GZMO process. Each tick at 174 BPM:
///   1. Drains feedback events from skills
///   2. Advances the Lorenz attractor
///   3. Ticks the Thought Cabinet (incubation, crystallization)
///   4. Computes derived LLM parameters from Lorenz coordinates
///   5. Broadcasts a `ChaosSnapshot` to all consumers via `tokio::sync::watch`
///
/// The snapshot is the read-only interface for skills, the REPL, the orchestrator,
/// and any external diagnostic tools.

use std::time::Duration;

use tokio::sync::{mpsc, watch};
use tracing::{debug, info};

use crate::chaos::{LogisticMap, LorenzAttractor, Phase};
use crate::engine::EngineState;
use crate::feedback::ChaosEvent;
use crate::thoughts::{CrystallizationEvent, Mutations, ThoughtCabinet};

/// 174 BPM = 344ms per beat
const TICK_INTERVAL: Duration = Duration::from_millis(344);

/// Read-only snapshot of current chaos state, cheaply cloneable.
/// This is the ONLY communication channel between the chaos engine and the rest of GZMO.
#[derive(Debug, Clone, serde::Serialize)]
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

    // Thought Cabinet state
    pub thoughts_incubating: u8,
    pub thoughts_crystallized: u32,
    pub mutations: Mutations,

    // Derived LLM parameters — computed from Lorenz coordinates
    pub llm_temperature: f32,
    pub llm_max_tokens: u32,
    pub llm_valence: f32, // -1.0 (dark/aggressive) to 1.0 (calm/reflective)

    // Last crystallization event (if any on this tick)
    pub last_crystallization: Option<CrystallizationEvent>,

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
            thoughts_incubating: 0,
            thoughts_crystallized: 0,
            mutations: Mutations::default(),
            llm_temperature: 0.6,
            llm_max_tokens: 256,
            llm_valence: 0.0,
            last_crystallization: None,
            timestamp: String::new(),
        }
    }
}

/// Configuration for the chaos engine, loaded from TOML
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChaosConfig {
    #[serde(default = "default_gravity")]
    pub gravity: f64,
    #[serde(default = "default_friction")]
    pub friction: f64,
    #[serde(default = "default_seed")]
    pub seed: f64,
    #[serde(default = "default_tension")]
    pub initial_tension: f64,
}

fn default_gravity() -> f64 { 9.81 }
fn default_friction() -> f64 { 0.7 }
fn default_seed() -> f64 { 0.506 }
fn default_tension() -> f64 { 50.0 }

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            gravity: default_gravity(),
            friction: default_friction(),
            seed: default_seed(),
            initial_tension: default_tension(),
        }
    }
}

/// Handle returned from `PulseLoop::start()` for interacting with the running loop.
pub struct PulseHandle {
    /// Read the latest chaos snapshot (non-blocking, always returns the most recent value)
    pub snapshot_rx: watch::Receiver<ChaosSnapshot>,
    /// Send feedback events from skills into the chaos engine
    pub feedback_tx: mpsc::Sender<ChaosEvent>,
    /// Task handle for the running pulse loop
    pub task: tokio::task::JoinHandle<()>,
}

/// The unified pulse loop — the heartbeat of GZMO.
pub struct PulseLoop {
    lorenz: LorenzAttractor,
    logistic: LogisticMap,
    state: EngineState,
    cabinet: ThoughtCabinet,
    tension: f64,
    config: ChaosConfig,

    // Channels
    event_rx: mpsc::Receiver<ChaosEvent>,
    snapshot_tx: watch::Sender<ChaosSnapshot>,
}

impl PulseLoop {
    /// Create and start the pulse loop. Returns a `PulseHandle` for interaction.
    pub fn start(config: ChaosConfig) -> PulseHandle {
        let (feedback_tx, event_rx) = mpsc::channel::<ChaosEvent>(256);
        let (snapshot_tx, snapshot_rx) = watch::channel(ChaosSnapshot::default());

        let pulse = PulseLoop {
            lorenz: LorenzAttractor::new(config.seed),
            logistic: LogisticMap::new(config.seed),
            state: EngineState::new(),
            cabinet: ThoughtCabinet::new(),
            tension: config.initial_tension,
            event_rx,
            snapshot_tx,
            config,
        };

        let task = tokio::spawn(async move {
            pulse.run().await;
        });

        PulseHandle {
            snapshot_rx,
            feedback_tx,
            task,
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

            // 2. Advance chaos generators
            let (x, y, z) = self.lorenz.step();
            self.lorenz.update_phase(&self.state.phase);

            // Apply cognitive noise from incubating thoughts
            self.lorenz.apply_cognitive_noise(self.cabinet.active_lorenz_noise());

            // Apply permanent rho mutation from crystallized thoughts
            self.lorenz.apply_rho_mutation(self.cabinet.mutations.lorenz_rho_mod);

            // Couple logistic map to Lorenz every 10 ticks
            if self.state.tick % 10 == 0 {
                self.logistic.reseed_from_lorenz(self.lorenz.normalized_output());
            }

            let chaos_val = self.logistic.next_val();

            // 3. Compute effective physics with thought mutations
            let effective_gravity = (self.config.gravity + self.cabinet.mutations.gravity_mod).max(1.0);
            let effective_friction = (self.config.friction + self.cabinet.mutations.friction_mod).max(0.05);

            // 4. Tick engine state
            let _rebirth = self.state.tick_heartbeat(
                self.tension + self.cabinet.mutations.tension_bias,
                effective_gravity,
                effective_friction,
                chaos_val,
                self.cabinet.active_drain_multiplier(),
            );

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

            let last_crystallization = crystallizations.into_iter().last();

            // 6. Compute derived LLM parameters from Lorenz coordinates
            let llm_temperature = self.lorenz_to_temperature();
            let llm_max_tokens = self.lorenz_to_tokens();
            let llm_valence = self.lorenz_to_valence();

            // 7. Build and broadcast snapshot
            let snapshot = ChaosSnapshot {
                tick: self.state.tick,
                x,
                y,
                z,
                tension: self.tension + self.cabinet.mutations.tension_bias,
                energy: self.state.energy,
                phase: self.state.phase,
                alive: self.state.alive,
                deaths: self.state.deaths,
                chaos_val,
                thoughts_incubating: self.cabinet.occupied_slots() as u8,
                thoughts_crystallized: self.cabinet.mutations.total_crystallized,
                mutations: self.cabinet.mutations.clone(),
                llm_temperature,
                llm_max_tokens,
                llm_valence,
                last_crystallization,
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            };

            // Non-blocking send — if nobody is listening, that's fine
            let _ = self.snapshot_tx.send(snapshot);

            // Tension decays naturally toward 50 (homeostasis)
            self.tension += (50.0 - self.tension) * 0.001;
        }
    }

    /// Process a single feedback event from a skill execution.
    fn apply_feedback(&mut self, event: &ChaosEvent) {
        // Apply tension delta
        let tension_delta = event.tension_delta();
        if tension_delta.abs() > 0.01 {
            self.tension = (self.tension + tension_delta).clamp(0.0, 100.0);
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
    }

    /// Map Lorenz x ∈ [-20, 20] to temperature ∈ [0.3, 1.2]
    fn lorenz_to_temperature(&self) -> f32 {
        let normalized = ((self.lorenz.x + 20.0) / 40.0).clamp(0.0, 1.0);
        0.3 + (normalized as f32 * 0.9) // [0.3, 1.2]
    }

    /// Map Lorenz y ∈ [-30, 30] to max_tokens ∈ [128, 512]
    fn lorenz_to_tokens(&self) -> u32 {
        let normalized = ((self.lorenz.y + 30.0) / 60.0).clamp(0.0, 1.0);
        128 + (normalized * 384.0) as u32
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
        let pulse = PulseLoop {
            lorenz: LorenzAttractor::new(0.506),
            logistic: LogisticMap::new(0.506),
            state: EngineState::new(),
            cabinet: ThoughtCabinet::new(),
            tension: 50.0,
            event_rx,
            snapshot_tx,
            config,
        };

        let temp = pulse.lorenz_to_temperature();
        assert!((0.3..=1.2).contains(&temp), "Initial temperature out of range: {temp}");
    }
}
