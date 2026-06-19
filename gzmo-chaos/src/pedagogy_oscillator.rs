//! Pedagogy chaos_val setpoint controller for structured discovery oscillation.

use std::time::Instant;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One phase target in an oscillation sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillationStep {
    pub target: f64,
    pub duration_secs: u32,
    #[serde(default)]
    pub label: String,
}

/// Settings loaded from `[pedagogy.tension_oscillation]` (via gzmo-core).
#[derive(Debug, Clone)]
pub struct PedagogyOscillationSettings {
    pub enabled: bool,
    pub spawn_discovery_on_low: bool,
    pub low_phase_threshold: f64,
    pub cooldown_secs: u64,
    pub blend_ticks: u64,
    pub sequence: Vec<OscillationStep>,
}

impl Default for PedagogyOscillationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            spawn_discovery_on_low: true,
            low_phase_threshold: 0.55,
            cooldown_secs: 3600,
            blend_ticks: 8,
            sequence: vec![
                OscillationStep {
                    target: 0.9,
                    duration_secs: 60,
                    label: "High tension — confirmation machine".to_string(),
                },
                OscillationStep {
                    target: 0.5,
                    duration_secs: 60,
                    label: "Low tension — discovery machine".to_string(),
                },
                OscillationStep {
                    target: 0.9,
                    duration_secs: 60,
                    label: "High tension — confirmation machine".to_string(),
                },
            ],
        }
    }
}

/// CLI / inbox trigger for the oscillator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PedagogyOscillateAction {
    Start,
    Stop,
}

/// Metadata exposed on each ChaosSnapshot tick.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PedagogyOscillationMeta {
    pub pedagogy_oscillation_active: bool,
    pub pedagogy_target: Option<f64>,
    pub pedagogy_step: u32,
    pub chaos_val_raw: f64,
    pub pedagogy_transition_seq: u64,
    pub pedagogy_last_transition: Option<PedagogyTransitionInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oscillation_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chaos_val_baseline: Option<f64>,
}

/// Emitted when the state machine advances (for daemon / Synapse).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedagogyTransitionInfo {
    pub kind: PedagogyTransitionKind,
    pub step: u32,
    pub target: f64,
    pub label: String,
    pub duration_secs: u32,
    pub is_low_phase: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oscillation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PedagogyTransitionKind {
    CycleStart,
    StepEnter,
    CycleComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunState {
    Idle,
    Running {
        step_idx: usize,
        step_started_tick: u64,
    },
}

/// State machine controlling effective `chaos_val` during pedagogy cycles.
#[derive(Debug)]
pub struct PedagogyOscillator {
    settings: PedagogyOscillationSettings,
    state: RunState,
    last_completed_at: Option<Instant>,
    transition_seq: u64,
    pending_transition: Option<PedagogyTransitionInfo>,
    current_oscillation_id: Option<Uuid>,
    chaos_val_baseline: Option<f64>,
}

const TICK_INTERVAL_SECS: f64 = 344.0 / 1000.0;

impl PedagogyOscillator {
    pub fn new(settings: PedagogyOscillationSettings) -> Self {
        Self {
            settings,
            state: RunState::Idle,
            last_completed_at: None,
            transition_seq: 0,
            pending_transition: None,
            current_oscillation_id: None,
            chaos_val_baseline: None,
        }
    }

    pub fn settings(&self) -> &PedagogyOscillationSettings {
        &self.settings
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, RunState::Running { .. })
    }

    pub fn current_oscillation_id(&self) -> Option<Uuid> {
        self.current_oscillation_id
    }

    pub fn chaos_val_baseline(&self) -> Option<f64> {
        self.chaos_val_baseline
    }

    pub fn request_start(&mut self, tick: u64) -> bool {
        if !self.settings.enabled || self.settings.sequence.is_empty() {
            return false;
        }
        if let RunState::Running { .. } = self.state {
            return false;
        }
        if let Some(last) = self.last_completed_at {
            if last.elapsed().as_secs() < self.settings.cooldown_secs {
                return false;
            }
        }
        self.begin_step(0, tick);
        true
    }

    pub fn request_stop(&mut self) {
        if matches!(self.state, RunState::Running { .. }) {
            self.complete_cycle();
        }
    }

    pub fn handle_action(&mut self, action: PedagogyOscillateAction, tick: u64) -> bool {
        match action {
            PedagogyOscillateAction::Start => self.request_start(tick),
            PedagogyOscillateAction::Stop => {
                self.request_stop();
                true
            }
        }
    }

    /// Advance timing, apply setpoint blend, return snapshot meta.
    pub fn apply(&mut self, chaos_val_raw: f64, tick: u64) -> (f64, PedagogyOscillationMeta) {
        if let RunState::Running {
            step_idx,
            step_started_tick,
        } = self.state
        {
            let step = &self.settings.sequence[step_idx];
            let duration_ticks =
                ((step.duration_secs as f64) / TICK_INTERVAL_SECS).ceil() as u64;
            let elapsed_ticks = tick.saturating_sub(step_started_tick);

            if elapsed_ticks >= duration_ticks.max(1) {
                let next = step_idx + 1;
                if next >= self.settings.sequence.len() {
                    self.complete_cycle();
                } else {
                    self.begin_step(next, tick);
                }
            }
        }

        let (effective, active, target, step_no) = self.effective_value(chaos_val_raw, tick);
        let transition = self.pending_transition.take();

        let meta = PedagogyOscillationMeta {
            pedagogy_oscillation_active: active,
            pedagogy_target: target,
            pedagogy_step: step_no,
            chaos_val_raw,
            pedagogy_transition_seq: self.transition_seq,
            pedagogy_last_transition: transition,
            oscillation_id: self.current_oscillation_id,
            chaos_val_baseline: self.chaos_val_baseline,
        };

        (effective, meta)
    }

    fn effective_value(&self, raw: f64, tick: u64) -> (f64, bool, Option<f64>, u32) {
        let RunState::Running {
            step_idx,
            step_started_tick,
        } = self.state
        else {
            return (raw, false, None, 0);
        };

        let step = &self.settings.sequence[step_idx];
        let target = step.target.clamp(0.0, 1.0);
        let blend = self.settings.blend_ticks.max(1);
        let elapsed_ticks = tick.saturating_sub(step_started_tick);
        let duration_ticks =
            ((step.duration_secs as f64) / TICK_INTERVAL_SECS).ceil() as u64;

        let effective = if elapsed_ticks < blend {
            let t = (elapsed_ticks as f64 / blend as f64).clamp(0.0, 1.0);
            raw + (target - raw) * t
        } else if elapsed_ticks + blend >= duration_ticks {
            let remaining = duration_ticks.saturating_sub(elapsed_ticks);
            let t = if blend > 0 {
                1.0 - (remaining as f64 / blend as f64).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let next_target = self
                .settings
                .sequence
                .get(step_idx + 1)
                .map(|s| s.target.clamp(0.0, 1.0))
                .unwrap_or(raw);
            target + (next_target - target) * t
        } else {
            target
        };

        (
            effective.clamp(0.0, 1.0),
            true,
            Some(target),
            (step_idx + 1) as u32,
        )
    }

    fn begin_step(&mut self, step_idx: usize, tick: u64) {
        let step = self.settings.sequence[step_idx].clone();
        let is_low = step.target <= self.settings.low_phase_threshold;
        let kind = if step_idx == 0 {
            PedagogyTransitionKind::CycleStart
        } else {
            PedagogyTransitionKind::StepEnter
        };

        if step_idx == 0 {
            self.current_oscillation_id = Some(Uuid::new_v4());
            self.chaos_val_baseline = None;
        }

        self.state = RunState::Running {
            step_idx,
            step_started_tick: tick,
        };
        self.transition_seq += 1;
        self.pending_transition = Some(PedagogyTransitionInfo {
            kind,
            step: (step_idx + 1) as u32,
            target: step.target,
            label: step.label,
            duration_secs: step.duration_secs,
            is_low_phase: is_low,
            oscillation_id: self.current_oscillation_id,
        });
    }

    fn complete_cycle(&mut self) {
        self.state = RunState::Idle;
        self.last_completed_at = Some(Instant::now());
        self.transition_seq += 1;
        self.pending_transition = Some(PedagogyTransitionInfo {
            kind: PedagogyTransitionKind::CycleComplete,
            step: 0,
            target: 0.0,
            label: "cycle complete".to_string(),
            duration_secs: 0,
            is_low_phase: false,
            oscillation_id: self.current_oscillation_id,
        });
        self.current_oscillation_id = None;
        self.chaos_val_baseline = None;
    }

    /// Record chaos baseline at cycle start (called from snapshot bridge on CycleStart).
    pub fn set_chaos_val_baseline(&mut self, raw: f64) {
        if self.chaos_val_baseline.is_none() && self.is_active() {
            self.chaos_val_baseline = Some(raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_settings() -> PedagogyOscillationSettings {
        PedagogyOscillationSettings {
            enabled: true,
            sequence: vec![
                OscillationStep {
                    target: 0.9,
                    duration_secs: 2,
                    label: "high".to_string(),
                },
                OscillationStep {
                    target: 0.5,
                    duration_secs: 2,
                    label: "low".to_string(),
                },
            ],
            blend_ticks: 2,
            cooldown_secs: 0,
            ..Default::default()
        }
    }

    #[test]
    fn start_applies_setpoint_after_blend() {
        let mut osc = PedagogyOscillator::new(test_settings());
        assert!(osc.handle_action(PedagogyOscillateAction::Start, 1));

        let (v0, m0) = osc.apply(0.2, 1);
        assert!(m0.pedagogy_oscillation_active);
        assert_eq!(m0.pedagogy_step, 1);
        assert!(m0.oscillation_id.is_some());

        for tick in 2..=5 {
            let (v, m) = osc.apply(0.2, tick);
            if tick >= 3 {
                assert!(
                    (v - 0.9).abs() < 0.05,
                    "tick {tick} expected ~0.9 got {v}"
                );
                assert!((m.pedagogy_target.unwrap() - 0.9).abs() < f64::EPSILON);
            }
            let _ = v;
        }
    }

    #[test]
    fn new_oscillation_id_each_cycle() {
        let mut osc = PedagogyOscillator::new(test_settings());
        assert!(osc.handle_action(PedagogyOscillateAction::Start, 1));
        let id1 = osc.apply(0.2, 1).1.oscillation_id;
        for tick in 2..=20 {
            osc.apply(0.2, tick);
        }
        assert!(osc.handle_action(PedagogyOscillateAction::Start, 21));
        let id2 = osc.apply(0.2, 21).1.oscillation_id;
        assert_ne!(id1, id2);
    }

    #[test]
    fn cooldown_blocks_immediate_restart() {
        let mut settings = test_settings();
        settings.cooldown_secs = 3600;
        settings.sequence = vec![OscillationStep {
            target: 0.9,
            duration_secs: 1,
            label: "x".to_string(),
        }];
        let mut osc = PedagogyOscillator::new(settings);
        assert!(osc.handle_action(PedagogyOscillateAction::Start, 1));
        osc.apply(0.5, 1);
        osc.apply(0.5, 2);
        assert!(!osc.handle_action(PedagogyOscillateAction::Start, 3));
    }
}
