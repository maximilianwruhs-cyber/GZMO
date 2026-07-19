use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::{select_chaos_mode, AttractorMeta, AttractorPromptInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoemMode {
    QuietLandscape,
    ConcreteIndustrial,
    TenseRhythm,
    FracturedVerse,
}

pub struct PoemBrief {
    pub meta: AttractorMeta,
    pub mode: PoemMode,
}

pub struct PoemBriefInput<'a> {
    pub seed: &'a str,
    pub snap: &'a ChaosSnapshot,
    pub recent_themes: &'a [String],
    pub call_serial: u64,
    pub attempt: u32,
    pub instant_nanos: u64,
}

impl PoemBrief {
    pub fn new(input: PoemBriefInput<'_>) -> Self {
        let mode = Self::derive_mode(input.snap);
        let meta = AttractorMeta::from_input(AttractorPromptInput {
            seed_label: "motif",
            seed: input.seed,
            snap: input.snap,
            recent_themes: input.recent_themes,
            call_serial: input.call_serial,
            attempt: input.attempt,
            instant_nanos: input.instant_nanos,
            max_chars: 180,
            extra_rules: &[],
        });
        Self { meta, mode }
    }

    fn derive_mode(snap: &ChaosSnapshot) -> PoemMode {
        const MODES: [PoemMode; 4] = [
            PoemMode::QuietLandscape,
            PoemMode::ConcreteIndustrial,
            PoemMode::TenseRhythm,
            PoemMode::FracturedVerse,
        ];
        select_chaos_mode(snap, &MODES)
    }

    pub fn system_prompt(&self) -> &'static str {
        match self.mode {
            PoemMode::QuietLandscape => {
                "You are a contemporary poet. Write calm, observational verse about still landscapes \
                 and quiet physical details. Ban predictable end-rhymes and abstract words \
                 (soul, fate, eternity, Seele, Schicksal). Max 180 characters. Output ONLY the poem."
            }
            PoemMode::ConcreteIndustrial => {
                "You are a contemporary German-influenced poet. Focus on concrete industrial textures: \
                 metal, rust, oil, glass, stone. Ban predictable end-rhymes and abstract words. \
                 Max 180 characters. Output ONLY the poem."
            }
            PoemMode::TenseRhythm => {
                "You are a poet of mounting pressure. Use short lines, ticking rhythm, withheld release. \
                 Ban predictable end-rhymes and abstract words. Max 180 characters. Output ONLY the poem."
            }
            PoemMode::FracturedVerse => {
                "You are a surrealist poet. Disjointed images, impossible juxtapositions, quiet dread. \
                 Ban predictable end-rhymes and abstract words. Max 180 characters. Output ONLY the poem."
            }
        }
    }

    pub fn user_prompt(&self) -> String {
        self.meta.user_prompt(
            "Motif",
            180,
            &[
                "Use concrete sensory details only.",
                "No titles or commentary.",
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;

    #[test]
    fn poem_mode_varies_by_phase() {
        let mut idle = ChaosSnapshot::default();
        idle.phase = Phase::Idle;
        idle.chaos_val = 0.05;

        let mut drop = idle.clone();
        drop.phase = Phase::Drop;

        let idle_mode = PoemBrief::new(PoemBriefInput {
            seed: "rust",
            snap: &idle,
            recent_themes: &[],
            call_serial: 1,
            attempt: 1,
            instant_nanos: 0,
        })
        .mode;

        let drop_mode = PoemBrief::new(PoemBriefInput {
            seed: "rust",
            snap: &drop,
            recent_themes: &[],
            call_serial: 1,
            attempt: 1,
            instant_nanos: 0,
        })
        .mode;

        assert_ne!(idle_mode, drop_mode);
    }
}
