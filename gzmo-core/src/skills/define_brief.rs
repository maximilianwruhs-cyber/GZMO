use gzmo_chaos::chaos::Phase;
use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::{AttractorMeta, AttractorPromptInput, select_chaos_mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefineMode {
    PoeticEtymology,
    ScientificEtymology,
    MechanicalEtymology,
    SurrealEtymology,
}

pub struct DefineBrief {
    pub meta: AttractorMeta,
    pub mode: DefineMode,
}

pub struct DefineBriefInput<'a> {
    pub term: &'a str,
    pub snap: &'a ChaosSnapshot,
    pub recent_themes: &'a [String],
    pub call_serial: u64,
    pub attempt: u32,
    pub instant_nanos: u64,
}

impl DefineBrief {
    pub fn new(input: DefineBriefInput<'_>) -> Self {
        let mode = Self::derive_mode(input.snap);
        let meta = AttractorMeta::from_input(AttractorPromptInput {
            seed_label: "term",
            seed: input.term,
            snap: input.snap,
            recent_themes: input.recent_themes,
            call_serial: input.call_serial,
            attempt: input.attempt,
            instant_nanos: input.instant_nanos,
            max_chars: 800,
            extra_rules: &[],
        });
        Self { meta, mode }
    }

    fn derive_mode(snap: &ChaosSnapshot) -> DefineMode {
        const MODES: [DefineMode; 4] = [
            DefineMode::PoeticEtymology,
            DefineMode::ScientificEtymology,
            DefineMode::MechanicalEtymology,
            DefineMode::SurrealEtymology,
        ];
        select_chaos_mode(snap, &MODES)
    }

    pub fn system_prompt(&self) -> &'static str {
        match self.mode {
            DefineMode::PoeticEtymology => {
                "You are a poetic lexicographer. For the given term, provide standard dictionary info \
                 but focus the etymology and usage on the poetic beauty, philosophical weight, and ancient origins of the word. \
                 Output EXACTLY in this format:\n\
                 WORD: [word]\n\
                 PRONUNCIATION: [IPA pronunciation]\n\
                 PART OF SPEECH: [part of speech]\n\
                 DEFINITION: [definition]\n\
                 ETYMOLOGY: [poetic etymology]\n\
                 USAGE: [literary example sentence]\n\
                 No other text."
            }
            DefineMode::ScientificEtymology => {
                "You are a scientific, precise lexicographer. For the given term, provide standard dictionary info \
                 focusing on dry, factual linguistic evolution and historical origins. \
                 Output EXACTLY in this format:\n\
                 WORD: [word]\n\
                 PRONUNCIATION: [IPA pronunciation]\n\
                 PART OF SPEECH: [part of speech]\n\
                 DEFINITION: [definition]\n\
                 ETYMOLOGY: [scientific etymology]\n\
                 USAGE: [precise, factual example sentence]\n\
                 No other text."
            }
            DefineMode::MechanicalEtymology => {
                "You are a structural lexicographer. For the given term, provide standard dictionary info \
                 focusing on the word's functional components, causal linkages, and active usage. \
                 Output EXACTLY in this format:\n\
                 WORD: [word]\n\
                 PRONUNCIATION: [IPA pronunciation]\n\
                 PART OF SPEECH: [part of speech]\n\
                 DEFINITION: [definition]\n\
                 ETYMOLOGY: [structural etymology]\n\
                 USAGE: [functional example sentence]\n\
                 No other text."
            }
            DefineMode::SurrealEtymology => {
                "You are a surrealist lexicographer. For the given term, provide standard dictionary info \
                 focusing on absurd connections, forgotten dream-like origins, and strange usage. \
                 Output EXACTLY in this format:\n\
                 WORD: [word]\n\
                 PRONUNCIATION: [IPA pronunciation]\n\
                 PART OF SPEECH: [part of speech]\n\
                 DEFINITION: [definition]\n\
                 ETYMOLOGY: [surreal etymology]\n\
                 USAGE: [dream-like, surreal example sentence]\n\
                 No other text."
            }
        }
    }

    pub fn user_prompt(&self) -> String {
        self.meta.user_prompt(
            "Term",
            800,
            &[
                "Provide WORD, PRONUNCIATION, PART OF SPEECH, DEFINITION, ETYMOLOGY, and USAGE.",
                "Ensure every field has its corresponding label prefix.",
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;

    #[test]
    fn define_mode_varies_by_phase() {
        let mut idle = ChaosSnapshot::default();
        idle.phase = Phase::Idle;
        idle.chaos_val = 0.05;

        let mut drop = idle.clone();
        drop.phase = Phase::Drop;

        let idle_mode = DefineBrief::new(DefineBriefInput {
            term: "lexicon",
            snap: &idle,
            recent_themes: &[],
            call_serial: 1,
            attempt: 1,
            instant_nanos: 0,
        })
        .mode;

        let drop_mode = DefineBrief::new(DefineBriefInput {
            term: "lexicon",
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
