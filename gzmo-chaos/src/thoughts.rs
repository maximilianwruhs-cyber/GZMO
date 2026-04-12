/// Thought Cabinet — Disco Elysium-inspired internalization mechanic
///
/// Lore items and skill outputs emitted by the chaos engine have a chance to be
/// "absorbed" into the cabinet as unprocessed thoughts. They incubate for N ticks,
/// imposing cognitive debuffs (increased drain). When incubation completes, they
/// "crystallize" — permanently mutating the engine's physical constants.
///
/// The engine learns from its own output. It becomes autopoietic.

use serde::Serialize;

const MAX_SLOTS: usize = 5; // Expanded from 3 — more capacity for skill output thoughts
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
        "dice_crit" => 10,  // ~3 seconds — critical moments are absorbed instantly
        "sound" => 8,       // ~3 seconds — sensory input is fast
        "persona" => 60,    // ~20 seconds — identity shifts take time
        _ => 30,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IncubatingThought {
    pub category: String,
    pub text_preview: String, // First 80 chars
    pub tick_absorbed: u64,
    pub ticks_remaining: u64,
    pub total_ticks: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrystallizationEvent {
    pub category: String,
    pub text_preview: String,
    pub mutation: MutationEffect,
    pub tick_crystallized: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MutationEffect {
    pub target: String,     // "gravity", "friction", "lorenz_rho", or "tension_bias"
    pub delta: f64,
    pub description: String,
}

/// Accumulated permanent mutations from all crystallized thoughts
#[derive(Debug, Clone, Default, Serialize)]
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

        // Find a free slot
        let free_slot = self.slots.iter().position(|s| s.is_none());
        let Some(idx) = free_slot else {
            return false; // Cabinet full — thought rejected
        };

        let total = incubation_period(category);
        let preview: String = text.chars().take(80).collect();

        self.slots[idx] = Some(IncubatingThought {
            category: category.to_string(),
            text_preview: preview,
            tick_absorbed: current_tick,
            ticks_remaining: total,
            total_ticks: total,
        });

        true
    }

    /// Advance all incubating thoughts by one tick.
    /// Returns any newly crystallized thoughts.
    pub fn tick(&mut self) -> Vec<CrystallizationEvent> {
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

    /// Calculate the mutation and apply it permanently
    fn crystallize(&mut self, thought: &IncubatingThought) -> CrystallizationEvent {
        let mutation = match thought.category.as_str() {
            "joke" => {
                // Humor lightens gravity
                self.mutations.gravity_mod -= 0.1;
                MutationEffect {
                    target: "gravity".to_string(),
                    delta: -0.1,
                    description: "Humor lightens the engine's gravitational pull".to_string(),
                }
            }
            "quote" => {
                // Wisdom perturbs the Lorenz attractor's shape
                self.mutations.lorenz_rho_mod += 0.3;
                MutationEffect {
                    target: "lorenz_rho".to_string(),
                    delta: 0.3,
                    description: "Wisdom reshapes the attractor's orbital topology".to_string(),
                }
            }
            "fact" => {
                // Truth reduces resistance
                self.mutations.friction_mod -= 0.02;
                MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.02,
                    description: "Truth reduces systemic resistance".to_string(),
                }
            }
            "poem" => {
                // Poetry softens gravity and nudges rho
                self.mutations.gravity_mod -= 0.05;
                self.mutations.lorenz_rho_mod += 0.1;
                MutationEffect {
                    target: "gravity+rho".to_string(),
                    delta: -0.05,
                    description: "Verse loosens the engine's grip on determinism".to_string(),
                }
            }
            "story" => {
                // Narrative shifts the attractor significantly
                self.mutations.lorenz_rho_mod += 0.5;
                MutationEffect {
                    target: "lorenz_rho".to_string(),
                    delta: 0.5,
                    description: "Narrative restructures phase space geometry".to_string(),
                }
            }
            "card" => {
                // Forged cards reduce friction (the engine gets "slicker")
                self.mutations.friction_mod -= 0.03;
                MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.03,
                    description: "A forged card greases the gears of chaos".to_string(),
                }
            }
            "dice_crit" => {
                // Critical dice rolls create tension bias (the system "remembers" luck)
                self.mutations.tension_bias -= 2.0; // Good crits calm the system
                MutationEffect {
                    target: "tension_bias".to_string(),
                    delta: -2.0,
                    description: "Fortune's memory lowers the system's baseline anxiety".to_string(),
                }
            }
            "sound" => {
                // Sound experiences leave a subtle friction mark
                self.mutations.friction_mod -= 0.01;
                MutationEffect {
                    target: "friction".to_string(),
                    delta: -0.01,
                    description: "Auditory resonance smooths turbulent transitions".to_string(),
                }
            }
            "persona" => {
                // Persona shifts warp gravity significantly — identity is heavy
                self.mutations.gravity_mod += 0.2;
                self.mutations.lorenz_rho_mod += 0.8;
                MutationEffect {
                    target: "gravity+rho".to_string(),
                    delta: 0.2,
                    description: "Identity crystallization adds existential weight and reshapes the attractor".to_string(),
                }
            }
            _ => MutationEffect {
                target: "none".to_string(),
                delta: 0.0,
                description: "Unknown category — no mutation".to_string(),
            },
        };

        self.mutations.total_crystallized += 1;

        // Clamp all mutation accumulators to prevent unbounded drift
        self.mutations.gravity_mod = self.mutations.gravity_mod.clamp(-5.0, 5.0);
        self.mutations.friction_mod = self.mutations.friction_mod.clamp(-0.5, 0.5);
        self.mutations.lorenz_rho_mod = self.mutations.lorenz_rho_mod.clamp(-10.0, 10.0);
        self.mutations.tension_bias = self.mutations.tension_bias.clamp(-30.0, 30.0);

        CrystallizationEvent {
            category: thought.category.clone(),
            text_preview: thought.text_preview.clone(),
            mutation,
            tick_crystallized: 0, // Caller sets this
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
        active_count as f64 * 0.5 // Each thought adds noise to sigma
    }

    /// Get a snapshot of currently incubating thoughts
    pub fn incubating_snapshot(&self) -> Vec<&IncubatingThought> {
        self.slots.iter().filter_map(|s| s.as_ref()).collect()
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
