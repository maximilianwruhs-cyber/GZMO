//! # Chaos Triggers — Autonomous threshold-based event firing
//!
//! Monitors ChaosSnapshot state and fires actions when configurable
//! thresholds are crossed. This is the mechanism by which GZMO becomes
//! truly autonomous — the chaos engine doesn't just *inform* the agent,
//! it *compels* it to act.
//!
//! ## Trigger Types
//! - **Threshold**: Fires when a metric crosses a boundary (tension, energy)
//! - **Phase**: Fires on phase transitions (Idle→Build, Build→Drop, etc.)
//! - **Crystallization**: Fires when a thought crystallizes
//! - **Death**: Fires when the engine dies and is reborn
//! - **Periodic**: Fires every N ticks
//!
//! ## Cooldowns
//! Each trigger has a cooldown period to prevent spam-firing.
//! The cooldown is tracked per-trigger via last-fired tick.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::chaos::Phase;
use crate::pulse::ChaosSnapshot;

/// A trigger condition that evaluates against a ChaosSnapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Fire when a metric exceeds a threshold
    Above {
        metric: ChaosMetric,
        threshold: f64,
    },
    /// Fire when a metric drops below a threshold
    Below {
        metric: ChaosMetric,
        threshold: f64,
    },
    /// Fire on phase transition
    PhaseEnter {
        phase: Phase,
    },
    /// Fire when a thought crystallizes
    Crystallization,
    /// Fire on engine death/rebirth
    Death,
    /// Fire every N ticks
    Periodic {
        interval_ticks: u64,
    },
}

/// Metrics that can be monitored by triggers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChaosMetric {
    Tension,
    Energy,
    Valence,
    Temperature,
    LorenzX,
    LorenzY,
    LorenzZ,
    ChaosVal,
}

impl ChaosMetric {
    /// Extract the metric value from a snapshot.
    pub fn extract(&self, snap: &ChaosSnapshot) -> f64 {
        match self {
            Self::Tension => snap.tension,
            Self::Energy => snap.energy,
            Self::Valence => snap.llm_valence as f64,
            Self::Temperature => snap.llm_temperature as f64,
            Self::LorenzX => snap.x,
            Self::LorenzY => snap.y,
            Self::LorenzZ => snap.z,
            Self::ChaosVal => snap.chaos_val,
        }
    }
}

/// What to do when a trigger fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerAction {
    /// Execute a Rust skill by name (e.g., "sound", "dice")
    RunSkill {
        skill_name: String,
        args: String,
    },
    /// Inject a message into the REPL output (displayed to user)
    Notify {
        message: String,
        /// Severity level affects display formatting
        level: NotifyLevel,
    },
    /// Inject an autonomous prompt into the agent loop
    /// This makes the agent "think" something without user input
    InjectPrompt {
        prompt: String,
    },
    /// Emit a custom ChaosEvent back into the engine (meta-feedback)
    EmitEvent {
        tension_delta: f64,
        energy_delta: f64,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotifyLevel {
    Whisper,  // Dim, subtle
    Normal,   // Standard
    Urgent,   // Bold, colored
    Critical, // RED, impossible to miss
}

/// A complete trigger definition.
#[derive(Debug, Clone)]
pub struct ChaosTrigger {
    pub name: String,
    pub condition: TriggerCondition,
    pub action: TriggerAction,
    /// Minimum ticks between firings (cooldown)
    pub cooldown_ticks: u64,
    /// Whether this trigger is currently armed
    pub enabled: bool,
    last_fired: u64,
}

impl ChaosTrigger {
    pub fn new(name: impl Into<String>, condition: TriggerCondition, action: TriggerAction, cooldown_ticks: u64) -> Self {
        Self {
            name: name.into(),
            condition,
            action,
            cooldown_ticks,
            enabled: true,
            last_fired: 0,
        }
    }

    /// Check if this trigger should fire given the current and previous snapshots.
    fn should_fire(&self, snap: &ChaosSnapshot, prev: &ChaosSnapshot) -> bool {
        if !self.enabled { return false; }
        if snap.tick.saturating_sub(self.last_fired) < self.cooldown_ticks { return false; }

        match &self.condition {
            TriggerCondition::Above { metric, threshold } => {
                let val = metric.extract(snap);
                let prev_val = metric.extract(prev);
                // Edge-triggered: fire on crossing, not while above
                val > *threshold && prev_val <= *threshold
            }
            TriggerCondition::Below { metric, threshold } => {
                let val = metric.extract(snap);
                let prev_val = metric.extract(prev);
                val < *threshold && prev_val >= *threshold
            }
            TriggerCondition::PhaseEnter { phase } => {
                snap.phase == *phase && prev.phase != *phase
            }
            TriggerCondition::Crystallization => {
                snap.last_crystallization.is_some()
            }
            TriggerCondition::Death => {
                snap.deaths > prev.deaths
            }
            TriggerCondition::Periodic { interval_ticks } => {
                snap.tick % interval_ticks == 0
            }
        }
    }
}

/// The trigger engine — evaluates all registered triggers against each snapshot.
pub struct TriggerEngine {
    triggers: Vec<ChaosTrigger>,
    prev_snapshot: ChaosSnapshot,
}

/// Result of evaluating triggers for a single tick.
#[derive(Debug)]
pub struct TriggerFired {
    pub trigger_name: String,
    pub action: TriggerAction,
}

impl TriggerEngine {
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
            prev_snapshot: ChaosSnapshot::default(),
        }
    }

    /// Create with default GZMO triggers pre-loaded.
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();

        // ─── Critical Tension Alerts ────────────────────────────
        engine.add(ChaosTrigger::new(
            "tension_critical",
            TriggerCondition::Above {
                metric: ChaosMetric::Tension,
                threshold: 85.0,
            },
            TriggerAction::RunSkill {
                skill_name: "sound".to_string(),
                args: String::new(),
            },
            90, // ~30s cooldown at 174 BPM
        ));

        engine.add(ChaosTrigger::new(
            "tension_calm",
            TriggerCondition::Below {
                metric: ChaosMetric::Tension,
                threshold: 15.0,
            },
            TriggerAction::Notify {
                message: "⚡ Tension critically low — the engine grows dormant…".to_string(),
                level: NotifyLevel::Whisper,
            },
            180, // ~1 min cooldown
        ));

        // ─── Energy Warnings ────────────────────────────────────
        engine.add(ChaosTrigger::new(
            "energy_critical",
            TriggerCondition::Below {
                metric: ChaosMetric::Energy,
                threshold: 10.0,
            },
            TriggerAction::Notify {
                message: "🔋 Energy critical — approaching death threshold!".to_string(),
                level: NotifyLevel::Critical,
            },
            90, // ~30s cooldown
        ));

        // ─── Phase Transitions ──────────────────────────────────
        engine.add(ChaosTrigger::new(
            "phase_drop",
            TriggerCondition::PhaseEnter { phase: Phase::Drop },
            TriggerAction::Notify {
                message: "📉 Phase transition: DROP — energy collapsing, brace for impact.".to_string(),
                level: NotifyLevel::Urgent,
            },
            30, // ~10s cooldown
        ));

        // ─── Death & Rebirth ────────────────────────────────────
        engine.add(ChaosTrigger::new(
            "death_event",
            TriggerCondition::Death,
            TriggerAction::RunSkill {
                skill_name: "sound".to_string(),
                args: String::new(),
            },
            1, // Allow immediate re-fire (deaths are rare)
        ));

        // ─── Crystallization Events ─────────────────────────────
        engine.add(ChaosTrigger::new(
            "crystallization",
            TriggerCondition::Crystallization,
            TriggerAction::Notify {
                message: "🔮 A thought has crystallized — permanent mutation applied.".to_string(),
                level: NotifyLevel::Normal,
            },
            1, // Crystallizations are rare, always notify
        ));

        // ─── Periodic Autonomous Heartbeat ──────────────────────
        engine.add(ChaosTrigger::new(
            "autonomous_pulse",
            TriggerCondition::Periodic { interval_ticks: 520 }, // ~3 minutes
            TriggerAction::InjectPrompt {
                prompt: "[AUTONOMOUS] Your chaos engine has been running for 3 minutes. \
                         Reflect briefly on your current internal state. \
                         If tension is high, consider what's causing it. \
                         If energy is low, conserve effort. \
                         This is an internal monologue — respond in 1-2 sentences.".to_string(),
            },
            520,
        ));

        engine
    }

    /// Register a new trigger.
    pub fn add(&mut self, trigger: ChaosTrigger) {
        info!(name = %trigger.name, "Trigger registered");
        self.triggers.push(trigger);
    }

    /// Enable or disable a trigger by name.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        for t in &mut self.triggers {
            if t.name == name {
                t.enabled = enabled;
                info!(name, enabled, "Trigger toggled");
            }
        }
    }

    /// Evaluate all triggers against the current snapshot.
    /// Returns a vec of fired actions. Call this once per tick.
    pub fn evaluate(&mut self, snap: &ChaosSnapshot) -> Vec<TriggerFired> {
        let mut fired = Vec::new();

        for trigger in &mut self.triggers {
            if trigger.should_fire(snap, &self.prev_snapshot) {
                debug!(name = %trigger.name, tick = snap.tick, "Trigger fired");
                fired.push(TriggerFired {
                    trigger_name: trigger.name.clone(),
                    action: trigger.action.clone(),
                });
                trigger.last_fired = snap.tick;
            }
        }

        self.prev_snapshot = snap.clone();
        fired
    }

    /// Get a diagnostic summary of all triggers.
    pub fn status_summary(&self, current_tick: u64) -> Vec<TriggerStatus> {
        self.triggers.iter().map(|t| {
            let ticks_since = current_tick.saturating_sub(t.last_fired);
            let cooldown_remaining = if ticks_since >= t.cooldown_ticks { 0 } else { t.cooldown_ticks - ticks_since };
            TriggerStatus {
                name: t.name.clone(),
                enabled: t.enabled,
                last_fired: t.last_fired,
                cooldown_remaining,
                condition_summary: format!("{:?}", t.condition),
            }
        }).collect()
    }
}

/// Diagnostic info for a single trigger.
#[derive(Debug, Clone)]
pub struct TriggerStatus {
    pub name: String,
    pub enabled: bool,
    pub last_fired: u64,
    pub cooldown_remaining: u64,
    pub condition_summary: String,
}

impl Default for TriggerEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn snap_with_tension(tick: u64, tension: f64) -> ChaosSnapshot {
        ChaosSnapshot { tick, tension, ..Default::default() }
    }

    fn snap_with_phase(tick: u64, phase: Phase, prev_phase: Phase) -> (ChaosSnapshot, ChaosSnapshot) {
        (
            ChaosSnapshot { tick, phase, ..Default::default() },
            ChaosSnapshot { tick: tick.saturating_sub(1), phase: prev_phase, ..Default::default() },
        )
    }

    #[test]
    fn test_threshold_edge_trigger() {
        let mut engine = TriggerEngine::new();
        engine.add(ChaosTrigger::new(
            "high_tension",
            TriggerCondition::Above { metric: ChaosMetric::Tension, threshold: 80.0 },
            TriggerAction::Notify { message: "Alert!".into(), level: NotifyLevel::Urgent },
            1,
        ));

        // Below threshold — no fire
        let fired = engine.evaluate(&snap_with_tension(1, 50.0));
        assert!(fired.is_empty());

        // Cross threshold — fire
        let fired = engine.evaluate(&snap_with_tension(2, 85.0));
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].trigger_name, "high_tension");

        // Stay above — no re-fire (edge-triggered)
        let fired = engine.evaluate(&snap_with_tension(3, 90.0));
        assert!(fired.is_empty());

        // Drop below and cross again — fire
        let fired = engine.evaluate(&snap_with_tension(4, 70.0));
        assert!(fired.is_empty());
        let fired = engine.evaluate(&snap_with_tension(5, 85.0));
        assert_eq!(fired.len(), 1);
    }

    #[test]
    fn test_cooldown() {
        let mut engine = TriggerEngine::new();
        engine.add(ChaosTrigger::new(
            "periodic",
            TriggerCondition::Periodic { interval_ticks: 10 },
            TriggerAction::Notify { message: "tick".into(), level: NotifyLevel::Normal },
            10, // cooldown = 10 ticks
        ));

        // Tick 10 — fire
        let fired = engine.evaluate(&ChaosSnapshot { tick: 10, ..Default::default() });
        assert_eq!(fired.len(), 1);

        // Tick 20 — fire again (cooldown expired)
        engine.prev_snapshot.tick = 19;
        let fired = engine.evaluate(&ChaosSnapshot { tick: 20, ..Default::default() });
        assert_eq!(fired.len(), 1);
    }

    #[test]
    fn test_phase_transition() {
        let mut engine = TriggerEngine::new();
        engine.add(ChaosTrigger::new(
            "drop_alert",
            TriggerCondition::PhaseEnter { phase: Phase::Drop },
            TriggerAction::Notify { message: "DROP!".into(), level: NotifyLevel::Urgent },
            1,
        ));

        // Start in Build
        engine.prev_snapshot = ChaosSnapshot { tick: 0, phase: Phase::Build, ..Default::default() };

        // Transition to Drop — fire
        let fired = engine.evaluate(&ChaosSnapshot { tick: 1, phase: Phase::Drop, ..Default::default() });
        assert_eq!(fired.len(), 1);

        // Stay in Drop — no re-fire
        let fired = engine.evaluate(&ChaosSnapshot { tick: 2, phase: Phase::Drop, ..Default::default() });
        assert!(fired.is_empty());
    }

    #[test]
    fn test_default_triggers_loaded() {
        let engine = TriggerEngine::with_defaults();
        assert!(engine.triggers.len() >= 7, "Should have at least 7 default triggers");
    }

    #[test]
    fn test_disabled_trigger() {
        let mut engine = TriggerEngine::new();
        engine.add(ChaosTrigger::new(
            "test",
            TriggerCondition::Periodic { interval_ticks: 1 },
            TriggerAction::Notify { message: "x".into(), level: NotifyLevel::Normal },
            0,
        ));
        engine.set_enabled("test", false);

        let fired = engine.evaluate(&ChaosSnapshot { tick: 1, ..Default::default() });
        assert!(fired.is_empty());
    }
}
