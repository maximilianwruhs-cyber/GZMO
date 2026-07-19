//! Phase-driven forge lens — Vision / Set / Play design pipeline (CCL-4+).

use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::select_chaos_mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeMode {
    /// Idle-leaning — flavor, name, worldbuilding first.
    Vision,
    /// Build-leaning — draft archetype, synergy, signpost.
    Set,
    /// Drop-leaning — mana cost, power level, templating discipline.
    Play,
}

impl ForgeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vision => "Vision Design",
            Self::Set => "Set Design",
            Self::Play => "Play Design",
        }
    }

    pub fn directive(self) -> &'static str {
        match self {
            Self::Vision => {
                "Prioritize an unforgettable name and flavor that embodies the color's philosophy. Mechanics serve the story."
            }
            Self::Set => {
                "Design as a Limited draft signpost — clear synergy hook, archetype identity, two abilities that teach the deck."
            }
            Self::Play => {
                "Tune mana cost and power level for tabletop balance. Strict templating; every word earns its place."
            }
        }
    }
}

pub fn derive_forge_mode(snap: &ChaosSnapshot) -> ForgeMode {
    const MODES: [ForgeMode; 3] = [ForgeMode::Vision, ForgeMode::Set, ForgeMode::Play];
    select_chaos_mode(snap, &MODES)
}
