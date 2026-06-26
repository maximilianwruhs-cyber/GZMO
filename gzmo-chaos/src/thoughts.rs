//! Thought Cabinet — Disco Elysium-inspired internalization mechanic
//!
//! Lore items and skill outputs emitted by the chaos engine have a chance to be
//! "absorbed" into the cabinet as unprocessed thoughts. They incubate for N ticks,
//! imposing cognitive debuffs (increased drain). When incubation completes, they
//! "crystallize" — permanently mutating the engine's physical constants.
//!
//! The engine learns from its own output. It becomes autopoietic.

use serde::{Deserialize, Serialize};

const MAX_SLOTS: usize = 7; // Expanded from 3 — more capacity for skill output thoughts
const ABSORB_THRESHOLD: f64 = 0.82; // ~18% chance per emission

/// Incubation periods by category (in ticks at 174 BPM)
fn incubation_period(category: &str) -> u64 {
    match category {
        "joke" => 15,       // ~5 seconds — humor is absorbed quickly
        "quote" => 30,      // ~10 seconds — wisdom takes time to process
        "fact" => 45,       // ~15 seconds — truth requires deep contemplation
        "poem" => 25,       // ~8 seconds — verse resonates moderately
        "story" => 40,      // ~14 seconds — narrative needs digestion
        "card" => 35,       // ~12 seconds — a forged card leaves an imprint
        "card_mythic" => 45, // mythic resonance lingers longer
        "pkm" => 35,
        "pkm_ex" => 45,
        "dice_crit" | "dice_crit_fail" | "dice_crit_success" => 10,
        "dice_catastrophe" => 12,
        "dice_resonance" => 20,
        "dice_oracle" => 30,
        "dice_spark" => 18,
        "dice_crystallize" | "dice_bifurcation" => 25,
        "dice_legendary" => 15,
        "dice_cascade" => 22,
        "sound" => 8,       // ~3 seconds — sensory input is fast
        "persona" => 60,    // ~20 seconds — identity shifts take time
        _ => 30,
    }
}

/// Apply ±20% jitter to base incubation using a chaos roll (0.0–1.0).
/// Minimum 3 ticks to prevent instant crystallization.
fn incubation_with_jitter(base: u64, chaos_roll: f64) -> u64 {
    let jitter = (chaos_roll - 0.5) * 0.4;
    let adjusted = base as f64 * (1.0 + jitter);
    adjusted.round().max(3.0) as u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncubatingThought {
    pub category: String,
    pub text_preview: String, // First 80 chars
    pub tick_absorbed: u64,
    pub ticks_remaining: u64,
    pub total_ticks: u64,
    /// Number of same-category thoughts fused into this one while incubating.
    /// Fused thoughts produce amplified mutations on crystallize (1.0 + 0.5 × count).
    #[serde(default)]
    pub fusion_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystallizationEvent {
    pub category: String,
    pub text_preview: String,
    pub mutation: MutationEffect,
    pub tick_crystallized: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationEffect {
    pub target: String,     // "gravity", "friction", "lorenz_rho", or "tension_bias"
    pub delta: f64,
    pub description: String,
}

/// Accumulated permanent mutations from all crystallized thoughts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mutations {
    pub gravity_mod: f64,      // Additive modifier (negative = lighter)
    pub friction_mod: f64,     // Additive modifier (negative = smoother)
    pub lorenz_rho_mod: f64,   // Additive modifier to Lorenz rho parameter
    pub tension_bias: f64,     // Permanent tension offset
    pub total_crystallized: u32,
}

pub struct ThoughtCabinet {
    slots: Vec<Option<IncubatingThought>>,
    pub mutations: Mutations,
}

impl ThoughtCabinet {
    pub fn new() -> Self {
        Self {
            slots: vec![None; MAX_SLOTS],
            mutations: Mutations::default(),
        }
    }

    /// Attempt to absorb a lore/skill item as an unprocessed thought.
    /// Returns true if absorbed (slot available and chaos roll passes threshold).
    pub fn try_absorb(&mut self, category: &str, text: &str, current_tick: u64, chaos_roll: f64) -> bool {
        if chaos_roll < ABSORB_THRESHOLD {
            return false;
        }

        // Fusion: if a same-category thought is already incubating, boost it
        // instead of consuming a new slot. Fused thoughts crystalize with
        // amplified mutations and a longer incubation period.
        if let Some(existing) = self.slots.iter_mut()
            .filter_map(|s| s.as_mut())
            .find(|t| t.category == category)
        {
            existing.fusion_count = existing.fusion_count.saturating_add(1);
            let extension = incubation_period(category) / 3;
            existing.ticks_remaining = existing.ticks_remaining.saturating_add(extension);
            existing.total_ticks = existing.total_ticks.saturating_add(extension);
            return true;
        }

        // Find a free slot
        let free_slot = self.slots.iter().position(|s| s.is_none());
        let Some(idx) = free_slot else {
            return false; // Cabinet full — thought rejected
        };

        let base = incubation_period(category);
        let total = incubation_with_jitter(base, chaos_roll);
        let preview: String = text.chars().take(80).collect();

        self.slots[idx] = Some(IncubatingThought {
            category: category.to_string(),
            text_preview: preview,
            tick_absorbed: current_tick,
            ticks_remaining: total,
            total_ticks: total,
            fusion_count: 0,
        });

        true
    }

    /// Advance all incubating thoughts by one tick.
    /// Returns any newly crystallized thoughts.
    pub fn tick(&mut self) -> Vec<CrystallizationEvent> {
        self.decay_mutations();

        // First pass: decrement and collect indices of completed thoughts
        let mut completed: Vec<(usize, IncubatingThought)> = Vec::new();

        for (i, slot) in self.slots.iter_mut().enumerate() {
            if let Some(thought) = slot {
                if thought.ticks_remaining > 0 {
                    thought.ticks_remaining -= 1;
                }

                if thought.ticks_remaining == 0 {
                    completed.push((i, thought.clone()));
                }
            }
        }

        // Second pass: crystallize completed thoughts and free slots
        let mut crystallized = Vec::new();
        for (i, thought) in completed {
            let event = self.crystallize(&thought);
            crystallized.push(event);
            self.slots[i] = None;
        }

        crystallized
    }

    /// Calculate the mutation and apply it permanently.
    ///
    /// Crystallization impulses on `lorenz_rho_mod` (Δρ): joke −0.2, quote +0.3,
    /// poem +0.1, story +0.5, persona +0.8. Per-tick decay (1−k)ρ_mod runs in
    /// `pulse.rs`. See `docs/CHAOS_RHO_CONTROL_MODEL.md`.
    fn crystallize(&mut self, thought: &IncubatingThought) -> CrystallizationEvent {
        let fusion_mult = 1.0 + 0.5 * thought.fusion_count as f64;
        let category_label = if thought.fusion_count > 0 {
            format!("{}[fused x{}]", thought.category, thought.fusion_count + 1)
        } else {
            thought.category.clone()
        };

        // Compute base deltas and mutation effect per category.
        // Deltas are applied below, amplified by fusion_mult for fused thoughts.
        let (dg, df, dr, dt, mutation) = match thought.category.as_str() {
            "joke" => {
                ( -0.1, 0.0, -0.2, 0.0, MutationEffect {
                    target: "gravity+rho".to_string(),
                    delta: -0.1,
                    description: "Humor lightens gravity and cools attractor intensity".to_string(),
                })
            }
            "quote" => {
                ( 0.0, 0.0, 0.3, 0.0, MutationEffect {
                    target: "lorenz_rho".to_string(),
                    delta: 0.3,
                    description: "Wisdom reshapes the attractor's orbital topology".to_string(),
                })
            }
            "fact" => {
                ( 0.0, -0.02, 0.0, 0.0, MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.02,
                    description: "Truth reduces systemic resistance".to_string(),
                })
            }
            "poem" => {
                ( -0.05, 0.0, 0.1, 0.0, MutationEffect {
                    target: "gravity+rho".to_string(),
                    delta: -0.05,
                    description: "Verse loosens the engine's grip on determinism".to_string(),
                })
            }
            "story" => {
                ( 0.0, 0.0, 0.5, 0.0, MutationEffect {
                    target: "lorenz_rho".to_string(),
                    delta: 0.5,
                    description: "Narrative restructures phase space geometry".to_string(),
                })
            }
            "card" => {
                ( 0.0, -0.03, 0.0, 0.0, MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.03,
                    description: "A forged card greases the gears of chaos".to_string(),
                })
            }
            "pkm" => {
                ( 0.0, -0.03, 0.0, 0.0, MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.03,
                    description: "A forged pokemon card greases the gears of chaos".to_string(),
                })
            }
            "pkm_ex" => {
                ( 0.0, -0.03, 0.0, 0.0, MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.03,
                    description: "An ex pokemon card greases the gears of chaos".to_string(),
                })
            }
            "dice_crit" | "dice_crit_success" => {
                ( 0.0, 0.0, 0.0, -2.0, MutationEffect {
                    target: "tension_bias".to_string(),
                    delta: -2.0,
                    description: "Fortune's memory lowers the system's baseline anxiety".to_string(),
                })
            }
            "dice_crit_fail" => {
                ( 0.0, 0.0, 0.0, 2.0, MutationEffect {
                    target: "tension_bias".to_string(),
                    delta: 2.0,
                    description: "Misfortune's memory raises the system's baseline anxiety".to_string(),
                })
            }
            "dice_catastrophe" => {
                ( 0.0, 0.0, 0.5, 3.0, MutationEffect {
                    target: "rho+tension".to_string(),
                    delta: 0.5,
                    description: "Phase collapse scars the attractor and elevates baseline dread".to_string(),
                })
            }
            "dice_resonance" => {
                ( 0.0, -0.02, 0.4, 0.0, MutationEffect {
                    target: "rho+friction".to_string(),
                    delta: 0.4,
                    description: "Lorenz–Logistic coupling reshapes rho and greases the field".to_string(),
                })
            }
            "dice_oracle" => {
                ( 0.0, -0.01, 0.15, 0.0, MutationEffect {
                    target: "friction+rho".to_string(),
                    delta: 0.15,
                    description: "Oracle insight smooths friction and nudges the attractor".to_string(),
                })
            }
            "dice_spark" => {
                ( 0.0, 0.0, 0.2, 0.0, MutationEffect {
                    target: "lorenz_rho".to_string(),
                    delta: 0.2,
                    description: "Creative spark raises attractor intensity".to_string(),
                })
            }
            "dice_crystallize" => {
                ( -0.1, 0.0, 0.0, 0.0, MutationEffect {
                    target: "gravity".to_string(),
                    delta: -0.1,
                    description: "Spontaneous nucleation lightens the engine's gravity well".to_string(),
                })
            }
            "dice_bifurcation" => {
                ( 0.0, -0.02, 0.0, 0.0, MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.02,
                    description: "Period-3 window discovery smooths turbulent transitions".to_string(),
                })
            }
            "dice_legendary" => {
                ( -0.05, 0.0, 1.0, 0.0, MutationEffect {
                    target: "rho+gravity".to_string(),
                    delta: 1.0,
                    description: "Legendary crystallization expands ρ and transcends parameter space".to_string(),
                })
            }
            "dice_cascade" => {
                ( 0.0, -0.01, 0.25, 0.0, MutationEffect {
                    target: "rho+friction".to_string(),
                    delta: 0.25,
                    description: "Wild magic imprints the pantheon echo on the attractor".to_string(),
                })
            }
            "sound" => {
                ( 0.0, -0.01, 0.0, 0.0, MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.01,
                    description: "Auditory resonance smooths turbulent transitions".to_string(),
                })
            }
            "persona" => {
                ( 0.2, 0.0, 0.8, 0.0, MutationEffect {
                    target: "gravity+rho".to_string(),
                    delta: 0.2,
                    description: "Identity crystallization adds existential weight and reshapes the attractor".to_string(),
                })
            }
            _ => {
                ( 0.0, 0.0, 0.0, 0.0, MutationEffect {
                    target: "none".to_string(),
                    delta: 0.0,
                    description: "Unknown category — no mutation".to_string(),
                })
            },
        };

        // Apply deltas amplified by fusion multiplier
        self.mutations.gravity_mod += dg * fusion_mult;
        self.mutations.friction_mod += df * fusion_mult;
        self.mutations.lorenz_rho_mod += dr * fusion_mult;
        self.mutations.tension_bias += dt * fusion_mult;

        self.mutations.total_crystallized += 1;

        // Clamp all mutation accumulators to prevent unbounded drift
        self.mutations.gravity_mod = self.mutations.gravity_mod.clamp(-5.0, 5.0);
        self.mutations.friction_mod = self.mutations.friction_mod.clamp(-0.5, 0.5);
        self.mutations.lorenz_rho_mod = self.mutations.lorenz_rho_mod.clamp(-10.0, 10.0);
        self.mutations.tension_bias = self.mutations.tension_bias.clamp(-30.0, 30.0);

        // Amplify the MutationEffect description for fused thoughts
        let mutation = if thought.fusion_count > 0 {
            MutationEffect {
                target: mutation.target,
                delta: mutation.delta * fusion_mult,
                description: format!("{} [fusion x{}]", mutation.description, thought.fusion_count + 1),
            }
        } else {
            mutation
        };

        CrystallizationEvent {
            category: category_label,
            text_preview: thought.text_preview.clone(),
            mutation,
            tick_crystallized: 0,
        }
    }

    /// Total drain multiplier from all actively incubating thoughts.
    /// Each incubating thought adds 15% drain overhead (cognitive load).
    pub fn active_drain_multiplier(&self) -> f64 {
        let active_count = self.slots.iter().filter(|s| s.is_some()).count();
        1.0 + (active_count as f64 * 0.15)
    }

    /// Active Lorenz perturbation from incubating thoughts (cognitive noise)
    pub fn active_lorenz_noise(&self) -> f64 {
        let active_count = self.slots.iter().filter(|s| s.is_some()).count();
        active_count as f64 * 0.5
    }

    /// Forgetting curve: pull gravity_mod, friction_mod, tension_bias toward zero.
    /// 0.0002 per tick ≈ 30% decay over ~10 min at 174 BPM.
    fn decay_mutations(&mut self) {
        const RATE: f64 = 0.0002;
        self.mutations.gravity_mod *= 1.0 - RATE;
        self.mutations.friction_mod *= 1.0 - RATE;
        self.mutations.tension_bias *= 1.0 - RATE;
        if self.mutations.gravity_mod.abs() < 0.001 { self.mutations.gravity_mod = 0.0; }
        if self.mutations.friction_mod.abs() < 0.001 { self.mutations.friction_mod = 0.0; }
        if self.mutations.tension_bias.abs() < 0.001 { self.mutations.tension_bias = 0.0; }
    }

    /// Get a snapshot of currently incubating thoughts
    pub fn incubating_snapshot(&self) -> Vec<&IncubatingThought> {
        self.slots.iter().filter_map(|s| s.as_ref()).collect()
    }

    /// Leaky integrator dissipation: ρ_mod ← (1−k)·ρ_mod each tick (k from `ChaosConfig.rho_decay_k`).
    pub fn apply_rho_decay(&mut self, k: f64) {
        if !k.is_finite() || k <= 0.0 {
            return;
        }
        let k = k.clamp(0.0, 1.0);
        self.mutations.lorenz_rho_mod *= 1.0 - k;
        self.mutations.lorenz_rho_mod = self.mutations.lorenz_rho_mod.clamp(-10.0, 10.0);
    }

    /// Bounded homeostatic restore (MASTER Phase I): ρ ← ρ − α·tanh(β·ρ).
    /// Falls back to linear `apply_rho_decay(k)` when `alpha <= 0`.
    pub fn apply_rho_restoration(&mut self, alpha: f64, beta: f64, k: f64) {
        if alpha.is_finite() && alpha > 0.0 {
            let beta = if beta > 0.0 { beta } else { 1.0 };
            let rho = self.mutations.lorenz_rho_mod;
            let restore = (alpha * (beta * rho).tanh()).clamp(-rho.abs(), rho.abs());
            self.mutations.lorenz_rho_mod -= restore;
        } else {
            self.apply_rho_decay(k);
        }
        self.mutations.lorenz_rho_mod = self.mutations.lorenz_rho_mod.clamp(-10.0, 10.0);
    }

    /// Number of occupied slots
    pub fn occupied_slots(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Total available slots
    pub fn max_slots(&self) -> usize {
        MAX_SLOTS
    }
}

impl Default for ThoughtCabinet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joke_cools_rho() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.lorenz_rho_mod = 3.0;
        assert!(cabinet.try_absorb("joke", "test", 0, 0.9));
        while cabinet.occupied_slots() > 0 {
            cabinet.tick();
        }
        assert!((cabinet.mutations.lorenz_rho_mod - 2.8).abs() < f64::EPSILON);
    }

    #[test]
    fn tanh_restoration_pulls_toward_zero() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.lorenz_rho_mod = 5.0;
        cabinet.apply_rho_restoration(0.01, 1.0, 0.001);
        assert!(cabinet.mutations.lorenz_rho_mod < 5.0);
        assert!(cabinet.mutations.lorenz_rho_mod > 4.98);
    }

    #[test]
    fn tanh_restoration_falls_back_to_linear_when_alpha_zero() {
        let mut tanh_cab = ThoughtCabinet::new();
        let mut linear_cab = ThoughtCabinet::new();
        tanh_cab.mutations.lorenz_rho_mod = 4.0;
        linear_cab.mutations.lorenz_rho_mod = 4.0;
        tanh_cab.apply_rho_restoration(0.0, 1.0, 0.001);
        linear_cab.apply_rho_decay(0.001);
        assert!((tanh_cab.mutations.lorenz_rho_mod - linear_cab.mutations.lorenz_rho_mod).abs() < 1e-12);
    }

    #[test]
    fn rho_decay_halves_over_half_life() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.lorenz_rho_mod = 8.0;
        let k = 0.001_f64;
        let half_life = (0.693_f64 / k).round() as u64;
        for _ in 0..half_life {
            cabinet.apply_rho_decay(k);
        }
        assert!(cabinet.mutations.lorenz_rho_mod < 4.5);
        assert!(cabinet.mutations.lorenz_rho_mod > 3.5);
    }

    #[test]
    fn rho_decay_does_not_invert_with_large_gain() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.lorenz_rho_mod = 4.0;
        cabinet.apply_rho_decay(2.0);

        assert_eq!(cabinet.mutations.lorenz_rho_mod, 0.0);
    }

    #[test]
    fn tanh_restoration_does_not_overshoot_zero() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.lorenz_rho_mod = 0.25;
        cabinet.apply_rho_restoration(10.0, 10.0, 0.001);

        assert_eq!(cabinet.mutations.lorenz_rho_mod, 0.0);
    }

    fn crystallize_category(category: &str) -> Mutations {
        let mut cabinet = ThoughtCabinet::new();
        assert!(cabinet.try_absorb(category, "test seed", 0, 0.9));
        while cabinet.occupied_slots() > 0 {
            cabinet.tick();
        }
        cabinet.mutations
    }

    #[test]
    fn dice_cascade_crystallizes_rho() {
        let m = crystallize_category("dice_cascade");
        assert!((m.lorenz_rho_mod - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn dice_legendary_crystallizes_rho_plus_one() {
        let m = crystallize_category("dice_legendary");
        assert!((m.lorenz_rho_mod - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dice_crystallize_lightens_gravity() {
        let m = crystallize_category("dice_crystallize");
        assert!((m.gravity_mod - (-0.1)).abs() < f64::EPSILON);
    }

    #[test]
    fn dice_catastrophe_scars_rho_and_tension() {
        let m = crystallize_category("dice_catastrophe");
        assert!((m.lorenz_rho_mod - 0.5).abs() < f64::EPSILON);
        assert!((m.tension_bias - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dice_crit_fail_raises_tension_bias() {
        let m = crystallize_category("dice_crit_fail");
        assert!((m.tension_bias - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn dice_spark_raises_rho() {
        let m = crystallize_category("dice_spark");
        assert!((m.lorenz_rho_mod - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn dice_oracle_nudges_rho_and_friction() {
        let m = crystallize_category("dice_oracle");
        assert!((m.lorenz_rho_mod - 0.15).abs() < f64::EPSILON);
        assert!((m.friction_mod - (-0.01)).abs() < f64::EPSILON);
    }
    #[test]
    fn jitter_at_midpoint_returns_base() {
        assert_eq!(incubation_with_jitter(30, 0.5), 30);
        assert_eq!(incubation_with_jitter(15, 0.5), 15);
        assert_eq!(incubation_with_jitter(60, 0.5), 60);
    }

    #[test]
    fn jitter_below_midpoint_shortens() {
        assert_eq!(incubation_with_jitter(30, 0.0), 24);
    }

    #[test]
    fn jitter_above_midpoint_lengthens() {
        assert_eq!(incubation_with_jitter(30, 1.0), 36);
    }

    #[test]
    fn jitter_clamps_to_minimum_3_ticks() {
        assert_eq!(incubation_with_jitter(8, 0.0), 6);
        assert_eq!(incubation_with_jitter(2, 0.0), 3);
    }

    #[test]
    fn jitter_produces_different_incubation_times_for_same_category() {
        let short = incubation_with_jitter(incubation_period("joke"), 0.1);
        let base_t = incubation_with_jitter(incubation_period("joke"), 0.5);
        let long = incubation_with_jitter(incubation_period("joke"), 0.9);
        assert!(short < base_t);
        assert!(long > base_t);
    }

    #[test]
    fn mutation_decay_pulls_toward_zero() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.gravity_mod = -4.0;
        cabinet.mutations.friction_mod = -0.3;
        cabinet.mutations.tension_bias = 10.0;
        cabinet.tick();
        assert!(cabinet.mutations.gravity_mod.abs() < 4.0);
        assert!(cabinet.mutations.friction_mod.abs() < 0.3);
        assert!(cabinet.mutations.tension_bias < 10.0);
    }

    #[test]
    fn mutation_decay_clips_near_zero() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.gravity_mod = 0.0005;
        cabinet.mutations.friction_mod = -0.0005;
        cabinet.tick();
        assert_eq!(cabinet.mutations.gravity_mod, 0.0);
        assert_eq!(cabinet.mutations.friction_mod, 0.0);
    }

    #[test]
    fn mutation_decay_does_not_affect_rho() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.mutations.lorenz_rho_mod = 3.0;
        cabinet.tick();
        assert_eq!(cabinet.mutations.lorenz_rho_mod, 3.0);
    }

    /// Absorb a category, tick to crystallize, return final Mutations
    fn absorb_and_crystallize(cabinet: &mut ThoughtCabinet, category: &str, chaos_roll: f64) -> Mutations {
        assert!(cabinet.try_absorb(category, "test seed", 0, chaos_roll));
        while cabinet.occupied_slots() > 0 {
            cabinet.tick();
        }
        cabinet.mutations.clone()
    }

    #[test]
    fn fusion_same_category_boosts_instead_of_consuming_slot() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.try_absorb("joke", "first joke", 0, 0.9);
        assert_eq!(cabinet.occupied_slots(), 1);

        // Same category → fuses into existing thought, doesn't use second slot
        cabinet.try_absorb("joke", "second joke", 1, 0.9);
        assert_eq!(cabinet.occupied_slots(), 1);
    }

    #[test]
    fn fusion_amplifies_joke_mutation() {
        let mut cabinet = ThoughtCabinet::new();
        // Fuse 3 jokes into one slot
        cabinet.try_absorb("joke", "j1", 0, 0.9);
        cabinet.try_absorb("joke", "j2", 1, 0.9);
        cabinet.try_absorb("joke", "j3", 2, 0.9);

        while cabinet.occupied_slots() > 0 {
            cabinet.tick();
        }

        // fusion_count=2, fusion_mult = 1.0 + 0.5*2 = 2.0
        // gravity: -0.1 * 2.0 = -0.2, rho: -0.2 * 2.0 = -0.4
        assert!((cabinet.mutations.gravity_mod - (-0.2)).abs() < 0.01,
            "expected gravity ≈ -0.2, got {}", cabinet.mutations.gravity_mod);
        assert!((cabinet.mutations.lorenz_rho_mod - (-0.4)).abs() < 0.01,
            "expected rho ≈ -0.4, got {}", cabinet.mutations.lorenz_rho_mod);
    }

    #[test]
    fn fusion_label_includes_fusion_count() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.try_absorb("story", "epic tale", 0, 0.9);
        cabinet.try_absorb("story", "epic sequel", 1, 0.9);

        let mut events = Vec::new();
        while cabinet.occupied_slots() > 0 {
            let evts = cabinet.tick();
            events.extend(evts);
        }

        // Should be exactly one crystallization event with fused label
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].category, "story[fused x2]");
    }

    #[test]
    fn fusion_extends_incubation_period() {
        let mut cabinet = ThoughtCabinet::new();
        // Roll=0.9 (>0.82 threshold) — jitter adds ~16% to base
        cabinet.try_absorb("joke", "j1", 0, 0.9);

        cabinet.try_absorb("joke", "j2", 1, 0.9);  // fuses, extends by 15/3 = 5 ticks

        let thought = cabinet.slots.iter().find_map(|s| s.as_ref()).unwrap();
        // After jitter: base 15 + ~16% ≈ 17-18, fusion extends by 5 → ~22-23
        assert!(thought.ticks_remaining > 15,
            "fusion should extend incubation beyond base: got {}", thought.ticks_remaining);
        assert_eq!(thought.fusion_count, 1);
    }

    #[test]
    fn different_categories_dont_fuse() {
        let mut cabinet = ThoughtCabinet::new();
        cabinet.try_absorb("joke", "funny", 0, 0.9);
        cabinet.try_absorb("quote", "wise", 1, 0.9);

        // Different categories → two separate slots
        assert_eq!(cabinet.occupied_slots(), 2);
    }
}
