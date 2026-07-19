//! Phase-driven forge lens — Concept / Archetype / Balance design pipeline (CCL-4+).

use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::select_chaos_mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeMode {
    /// Idle-leaning — name, creature identity, Ken Sugimori-era flavor.
    Concept,
    /// Build-leaning — type synergy, deck role, set cohesion.
    Archetype,
    /// Drop-leaning — HP, damage, retreat tuning, and readable attack text.
    Balance,
}

impl ForgeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Concept => "Concept Design",
            Self::Archetype => "Archetype Design",
            Self::Balance => "Balance Design",
        }
    }

    pub fn directive(self) -> &'static str {
        match self {
            Self::Concept => {
                "Prioritize an unforgettable Pokemon name, creature identity, and Ken Sugimori-era flavor. Mechanics serve the concept."
            }
            Self::Archetype => {
                "Design for type synergy, deck role, and set cohesion. Focus on how this card interacts with other cards of its element."
            }
            Self::Balance => {
                "Tune HP, damage, retreat cost, and weakness values. Ensure strict, readable rules/attack text. Every stat counts."
            }
        }
    }
}

pub fn derive_forge_mode(snap: &ChaosSnapshot) -> ForgeMode {
    const MODES: [ForgeMode; 3] = [ForgeMode::Concept, ForgeMode::Archetype, ForgeMode::Balance];
    select_chaos_mode(snap, &MODES)
}
