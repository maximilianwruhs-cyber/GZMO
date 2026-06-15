//! Feedback Channel — Bidirectional link between skills and the chaos engine.
//!
//! Skills emit `ChaosEvent`s after execution. The PulseLoop processes them
//! between ticks, modifying tension, energy, and injecting thought seeds.
//! This creates the autopoietic loop: outputs modify the system that generates further outputs.

use serde::Serialize;

/// Sound category for typed feedback
#[derive(Debug, Clone, Serialize)]
pub enum SoundCategory {
    Explosion,
    Thunder,
    Alarm,
    Roar,
    Bell,
    Guitar,
    Drum,
    Wave,
    Chime,
    Piano,
    Wind,
    Hum,
}

/// Events emitted by skills that feed back into the chaos engine.
#[derive(Debug, Clone)]
pub enum ChaosEvent {
    /// Dice rolled — modify tension proportional to outcome distance from mean
    DiceRoll {
        value: u8,
        max: u8,
    },

    /// Sound fired — small energy pulse based on category intensity
    SoundFired {
        category: SoundCategory,
    },

    /// Card forged — inject thought seed from card identity
    CardForged {
        name: String,
        card_type: String,
    },

    /// Pokemon card forged — inject thought seed
    PkmForged {
        name: String,
        element: String,
    },

    /// Poem generated — inject as thought for potential crystallization
    PoemGenerated {
        text: String,
    },

    /// Story generated — inject as thought
    StoryGenerated {
        text: String,
    },

    /// Joke generated — inject as thought
    JokeGenerated {
        text: String,
    },

    /// Transform activated — shift persona, heavy thought injection
    PersonaShift {
        persona: String,
    },

    /// Transform cleared — lighter inverse shift
    PersonaCleared,

    /// Word/definition generated — inject as fact-type thought
    WordGenerated {
        word: String,
        definition: String,
    },

    /// Quote surfaced from lore — inject as quote thought
    QuoteSurfaced {
        text: String,
    },

    /// Stabilize — manual control of attractor parameter
    Stabilize {
        delta_rho: f64,
    },

    /// Socratic mentor turn — shifts chaos so successive teach calls diverge
    MentorTeach {
        topic_preview: String,
        response_preview: String,
        turn_count: u32,
    },

    /// Custom arbitrary event for extensibility
    Custom {
        tension_delta: f64,
        energy_delta: f64,
        thought_seed: Option<ThoughtSeed>,
    },
}

/// A thought seed destined for the Thought Cabinet
#[derive(Debug, Clone)]
pub struct ThoughtSeed {
    pub category: String,
    pub text: String,
}

impl ChaosEvent {
    /// Compute the tension delta for this event.
    /// Positive = increase tension, negative = decrease.
    pub fn tension_delta(&self) -> f64 {
        match self {
            ChaosEvent::DiceRoll { value, max } => {
                if *max == 0 {
                    return 0.0;
                }
                // Normalized distance from midpoint: crit fail → +10, crit success → -10
                let midpoint = *max as f64 / 2.0;
                let value = (*value).clamp(1, *max) as f64;
                let distance = midpoint - value;
                (distance / midpoint) * 10.0
            }
            ChaosEvent::SoundFired { category } => {
                // Aggressive sounds increase tension, calm ones decrease
                match category {
                    SoundCategory::Explosion | SoundCategory::Alarm => 3.0,
                    SoundCategory::Thunder | SoundCategory::Roar => 2.0,
                    SoundCategory::Drum | SoundCategory::Guitar => 0.0,
                    SoundCategory::Bell | SoundCategory::Wave => -1.0,
                    SoundCategory::Chime | SoundCategory::Piano => -2.0,
                    SoundCategory::Wind | SoundCategory::Hum => -3.0,
                }
            }
            ChaosEvent::PersonaShift { .. } => 5.0,  // Identity shifts are stressful
            ChaosEvent::PersonaCleared => -3.0,       // Returning to self is calming
            ChaosEvent::MentorTeach { turn_count, .. } => {
                1.5 + (*turn_count as f64 * 0.3).min(3.0)
            }
            ChaosEvent::Custom { tension_delta, .. } => *tension_delta,
            _ => 0.0,
        }
    }

    /// Compute the energy delta for this event.
    pub fn energy_delta(&self) -> f64 {
        match self {
            ChaosEvent::DiceRoll { value, max } => {
                if *max == 0 {
                    return 0.0;
                }
                // High rolls energize, low rolls drain
                if *value == 1 { -5.0 }
                else if *value == *max { 5.0 }
                else { 0.0 }
            }
            ChaosEvent::SoundFired { .. } => -1.0, // All sounds cost a bit of energy
            ChaosEvent::CardForged { .. } => -2.0,  // Forging is taxing
            ChaosEvent::PkmForged { .. } => -2.0,   // Forging is taxing
            ChaosEvent::PersonaShift { .. } => -3.0, // Identity shifts are expensive
            ChaosEvent::MentorTeach { .. } => -0.5,
            ChaosEvent::Custom { energy_delta, .. } => *energy_delta,
            _ => 0.0,
        }
    }

    /// Extract a thought seed for the Thought Cabinet, if this event produces one.
    pub fn thought_seed(&self) -> Option<ThoughtSeed> {
        match self {
            ChaosEvent::PoemGenerated { text } => Some(ThoughtSeed {
                category: "poem".to_string(),
                text: text.clone(),
            }),
            ChaosEvent::StoryGenerated { text } => Some(ThoughtSeed {
                category: "story".to_string(),
                text: text.clone(),
            }),
            ChaosEvent::JokeGenerated { text } => Some(ThoughtSeed {
                category: "joke".to_string(),
                text: text.clone(),
            }),
            ChaosEvent::CardForged { name, card_type } => Some(ThoughtSeed {
                category: "card".to_string(),
                text: format!("{} ({})", name, card_type),
            }),
            ChaosEvent::PkmForged { name, element } => Some(ThoughtSeed {
                category: "pkm".to_string(),
                text: format!("{} ({})", name, element),
            }),
            ChaosEvent::PersonaShift { persona } => Some(ThoughtSeed {
                category: "persona".to_string(),
                text: format!("Became {}", persona),
            }),
            ChaosEvent::WordGenerated { word, definition } => Some(ThoughtSeed {
                category: "fact".to_string(),
                text: format!("{}: {}", word, definition),
            }),
            ChaosEvent::QuoteSurfaced { text } => Some(ThoughtSeed {
                category: "quote".to_string(),
                text: text.clone(),
            }),
            ChaosEvent::MentorTeach {
                topic_preview,
                response_preview,
                ..
            } => Some(ThoughtSeed {
                category: "mentor".to_string(),
                text: format!("Q: {topic_preview} → {response_preview}"),
            }),
            ChaosEvent::DiceRoll { value, max } => {
                if *max == 0 {
                    return None;
                }
                if *value == 1 {
                    Some(ThoughtSeed {
                        category: "dice_crit_fail".to_string(),
                        text: format!("Rolled {} on D{}", value, max),
                    })
                } else if *value == *max {
                    Some(ThoughtSeed {
                        category: "dice_crit_success".to_string(),
                        text: format!("Rolled {} on D{}", value, max),
                    })
                } else {
                    None
                }
            }
            ChaosEvent::Custom { thought_seed, .. } => thought_seed.clone(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_zero_sided_die_is_inert() {
        let event = ChaosEvent::DiceRoll { value: 0, max: 0 };

        assert_eq!(event.tension_delta(), 0.0);
        assert_eq!(event.energy_delta(), 0.0);
        assert!(event.thought_seed().is_none());
    }

    #[test]
    fn dice_roll_tension_clamps_value_to_die_range() {
        let event = ChaosEvent::DiceRoll { value: 200, max: 20 };

        assert_eq!(event.tension_delta(), -10.0);
    }
}
