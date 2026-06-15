use gzmo_chaos::chaos::Phase;
use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::{AttractorMeta, AttractorPromptInput, select_chaos_mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordMode {
    CalmOrganic,
    SharpIndustrial,
    TenseMechanical,
    SurrealAbsurd,
}

pub struct WordBrief {
    pub meta: AttractorMeta,
    pub mode: WordMode,
}

pub struct WordBriefInput<'a> {
    pub seed: &'a str,
    pub snap: &'a ChaosSnapshot,
    pub recent_themes: &'a [String],
    pub call_serial: u64,
    pub attempt: u32,
    pub instant_nanos: u64,
}

impl WordBrief {
    pub fn new(input: WordBriefInput<'_>) -> Self {
        let mode = Self::derive_mode(input.snap);
        let meta = AttractorMeta::from_input(AttractorPromptInput {
            seed_label: "theme",
            seed: input.seed,
            snap: input.snap,
            recent_themes: input.recent_themes,
            call_serial: input.call_serial,
            attempt: input.attempt,
            instant_nanos: input.instant_nanos,
            max_chars: 512,
            extra_rules: &[],
        });
        Self { meta, mode }
    }

    fn derive_mode(snap: &ChaosSnapshot) -> WordMode {
        const MODES: [WordMode; 4] = [
            WordMode::CalmOrganic,
            WordMode::SharpIndustrial,
            WordMode::TenseMechanical,
            WordMode::SurrealAbsurd,
        ];
        select_chaos_mode(snap, &MODES)
    }

    pub fn system_prompt(&self) -> &'static str {
        match self.mode {
            WordMode::CalmOrganic => {
                "You are a neologist. Invent a new pronounceable word that sounds natural, soft, and positive. \
                 Focus on organic, flowing sounds. Output EXACTLY in this format:\n\
                 WORD: [word] ([pronunciation])\n\
                 DEFINITION: [definition]\n\
                 EXAMPLE: [sentence]\n\
                 No other text."
            }
            WordMode::SharpIndustrial => {
                "You are a neologist. Invent a new pronounceable word that sounds harsh, metallic, and technical. \
                 Focus on sharp, industrial consonant sounds. Output EXACTLY in this format:\n\
                 WORD: [word] ([pronunciation])\n\
                 DEFINITION: [definition]\n\
                 EXAMPLE: [sentence]\n\
                 No other text."
            }
            WordMode::TenseMechanical => {
                "You are a neologist. Invent a new pronounceable word that sounds precise and rhythmic. \
                 Focus on gear-like, mechanical syllables. Output EXACTLY in this format:\n\
                 WORD: [word] ([pronunciation])\n\
                 DEFINITION: [definition]\n\
                 EXAMPLE: [sentence]\n\
                 No other text."
            }
            WordMode::SurrealAbsurd => {
                "You are a neologist. Invent a new pronounceable word that sounds absurd, strange, and dream-like. \
                 Focus on unexpected, surreal phonemes. Output EXACTLY in this format:\n\
                 WORD: [word] ([pronunciation])\n\
                 DEFINITION: [definition]\n\
                 EXAMPLE: [sentence]\n\
                 No other text."
            }
        }
    }

    pub fn user_prompt(&self) -> String {
        self.meta.user_prompt(
            "Theme",
            512,
            &[
                "Invent one completely new pronounceable word.",
                "Ensure the prefix and syllables are pronounceable.",
                "WORD line must contain the pronunciation in parentheses.",
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;

    #[test]
    fn word_mode_uses_chaos_coordinates() {
        let mut idle = ChaosSnapshot::default();
        idle.phase = Phase::Idle;
        idle.chaos_val = 0.05;
        idle.x = 0.1;
        idle.y = 0.2;

        let mut build = idle.clone();
        build.phase = Phase::Build;

        let idle_mode = WordBrief::new(WordBriefInput {
            seed: "nature",
            snap: &idle,
            recent_themes: &[],
            call_serial: 1,
            attempt: 1,
            instant_nanos: 0,
        })
        .mode;

        let build_mode = WordBrief::new(WordBriefInput {
            seed: "nature",
            snap: &build,
            recent_themes: &[],
            call_serial: 1,
            attempt: 1,
            instant_nanos: 0,
        })
        .mode;

        assert_ne!(idle_mode, build_mode);
    }
}
