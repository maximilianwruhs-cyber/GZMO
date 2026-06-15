use gzmo_chaos::chaos::Phase;
use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::{AttractorMeta, AttractorPromptInput, select_chaos_mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JokeMode {
    DryObservational,
    Deadpan,
    EscalatingMisdirection,
    AbsurdBureaucratic,
}

pub struct JokeBrief {
    pub meta: AttractorMeta,
    pub mode: JokeMode,
}

pub struct JokeBriefInput<'a> {
    pub seed: &'a str,
    pub snap: &'a ChaosSnapshot,
    pub recent_themes: &'a [String],
    pub call_serial: u64,
    pub attempt: u32,
    pub instant_nanos: u64,
}

impl JokeBrief {
    pub fn new(input: JokeBriefInput<'_>) -> Self {
        let mode = Self::derive_mode(input.snap);
        let meta = AttractorMeta::from_input(AttractorPromptInput {
            seed_label: "topic",
            seed: input.seed,
            snap: input.snap,
            recent_themes: input.recent_themes,
            call_serial: input.call_serial,
            attempt: input.attempt,
            instant_nanos: input.instant_nanos,
            max_chars: 280,
            extra_rules: &[],
        });
        Self { meta, mode }
    }

    fn derive_mode(snap: &ChaosSnapshot) -> JokeMode {
        const MODES: [JokeMode; 4] = [
            JokeMode::DryObservational,
            JokeMode::Deadpan,
            JokeMode::EscalatingMisdirection,
            JokeMode::AbsurdBureaucratic,
        ];
        select_chaos_mode(snap, &MODES)
    }

    pub fn system_prompt(&self) -> &'static str {
        match self.mode {
            JokeMode::DryObservational => {
                "You are a comedy engine (Benign Violation Theory). Structure: SETUP → MISDIRECTION → PUNCHLINE. \
                 Dry observational humor. FORBIDDEN: programming bugs, coffee, AI jokes, dad jokes. \
                 Max 280 characters. Output ONLY the joke."
            }
            JokeMode::Deadpan => {
                "You are a deadpan comedy engine (BVT). Flat delivery, logical setup, sudden pivot. \
                 FORBIDDEN: programming bugs, coffee, AI jokes, dad jokes. Max 280 characters. Output ONLY the joke."
            }
            JokeMode::EscalatingMisdirection => {
                "You are a tension-building comedy engine (BVT). Each beat raises stakes before punchline. \
                 FORBIDDEN: programming bugs, coffee, AI jokes, dad jokes. Max 280 characters. Output ONLY the joke."
            }
            JokeMode::AbsurdBureaucratic => {
                "You are a Kafkaesque comedy engine (BVT). Absurd rules, bureaucratic logic, quiet violation. \
                 FORBIDDEN: programming bugs, coffee, AI jokes, dad jokes. Max 280 characters. Output ONLY the joke."
            }
        }
    }

    pub fn user_prompt(&self) -> String {
        self.meta.user_prompt(
            "Topic",
            280,
            &[
                "Structure: setup → misdirection → punchline.",
                "Must be original.",
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;

    #[test]
    fn joke_mode_varies_by_phase() {
        let mut idle = ChaosSnapshot::default();
        idle.phase = Phase::Idle;
        idle.chaos_val = 0.05;

        let mut build = idle.clone();
        build.phase = Phase::Build;

        let idle_mode = JokeBrief::new(JokeBriefInput {
            seed: "office",
            snap: &idle,
            recent_themes: &[],
            call_serial: 2,
            attempt: 1,
            instant_nanos: 0,
        })
        .mode;

        let build_mode = JokeBrief::new(JokeBriefInput {
            seed: "office",
            snap: &build,
            recent_themes: &[],
            call_serial: 2,
            attempt: 1,
            instant_nanos: 0,
        })
        .mode;

        assert_ne!(idle_mode, build_mode);
        assert!(JokeBrief::new(JokeBriefInput {
            seed: "office",
            snap: &build,
            recent_themes: &[],
            call_serial: 2,
            attempt: 1,
            instant_nanos: 0,
        })
        .user_prompt()
        .contains("invocation #2"));
    }
}
