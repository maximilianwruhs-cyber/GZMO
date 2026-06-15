use gzmo_chaos::chaos::Phase;
use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::build_nonce;
use super::attractor_common::select_chaos_mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryMode {
    HemingwayCalm,
    HemingwaySparse,
    RisingTension,
    KafkaSurreal,
}

pub struct StoryBrief {
    pub keyword: String,
    pub tick: u64,
    pub phase: Phase,
    pub valence: f32,
    pub temperature: f32,
    pub rho_effective: f64,
    pub call_serial: u64,
    pub nonce: u64,
    pub cabinet_echo: Option<String>,
    pub mode: StoryMode,
    pub anti_repeat_hint: String,
}

pub struct StoryBriefInput<'a> {
    pub keyword: &'a str,
    pub snap: &'a ChaosSnapshot,
    pub recent_themes: &'a [String],
    pub call_serial: u64,
    pub attempt: u32,
    pub instant_nanos: u64,
}

impl StoryBrief {
    pub fn new(input: StoryBriefInput<'_>) -> Self {
        let keyword = input.keyword.trim().to_string();
        let snap = input.snap;
        let mode = Self::derive_mode(snap);
        let nonce = build_nonce(
            &keyword,
            snap.tick,
            input.call_serial,
            input.attempt,
            input.instant_nanos,
        );

        let cabinet_echo = snap.incubating_previews.first().cloned();

        let anti_repeat_hint = if input.recent_themes.is_empty() {
            String::new()
        } else {
            format!(
                "Avoid repeating these prior story themes/images: {}.",
                input.recent_themes.join(", ")
            )
        };

        Self {
            keyword,
            tick: snap.tick,
            phase: snap.phase,
            valence: snap.llm_valence,
            temperature: snap.llm_temperature,
            rho_effective: snap.rho_effective,
            call_serial: input.call_serial,
            nonce,
            cabinet_echo,
            mode,
            anti_repeat_hint,
        }
    }

    fn derive_mode(snap: &ChaosSnapshot) -> StoryMode {
        const MODES: [StoryMode; 4] = [
            StoryMode::HemingwayCalm,
            StoryMode::HemingwaySparse,
            StoryMode::RisingTension,
            StoryMode::KafkaSurreal,
        ];
        select_chaos_mode(snap, &MODES)
    }

    pub fn system_prompt(&self) -> &'static str {
        match self.mode {
            StoryMode::HemingwayCalm => {
                "You write short stories in Ernest Hemingway's calm, modern style. \
                 Focus on quiet observations of nature, still landscapes, and inner peace. \
                 Use short, declarative sentences. Avoid metaphor. Output ONLY story text."
            }
            StoryMode::HemingwaySparse => {
                "You write short stories in Ernest Hemingway's classic sparse, concrete style. \
                 Focus on physical objects, raw action, and subtext (the iceberg theory). \
                 No flowery adjectives or metaphors. Output ONLY story text."
            }
            StoryMode::RisingTension => {
                "You write short stories in a tense, building style. \
                 Focus on rising physical pressure, clockwork precision, ticking mechanisms, \
                 and pending collapse. Withhold the ending. Output ONLY story text."
            }
            StoryMode::KafkaSurreal => {
                "You write short stories in Franz Kafka's surreal, disorienting style. \
                 Focus on absurd bureaucracy, impossible spaces, self-contradicting rules, \
                 and quiet anxiety. Output ONLY story text."
            }
        }
    }

    pub fn user_prompt(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Keyword: {}", self.keyword));
        lines.push(format!(
            "Attractor state: tick {}, phase {}, valence {:.2}, rho {:.2}, invocation #{}",
            self.tick, self.phase, self.valence, self.rho_effective, self.call_serial
        ));
        lines.push(format!("Nonce: {} (unique per invocation)", self.nonce));
        
        if let Some(echo) = &self.cabinet_echo {
            lines.push(format!("Incorporate or contrast this incubating thought: \"{}\"", echo));
        }

        if !self.anti_repeat_hint.is_empty() {
            lines.push(self.anti_repeat_hint.clone());
        }

        lines.push("Rules:".to_string());
        lines.push("- Maximum 500 characters.".to_string());
        lines.push("- Must have a complete but unresolved narrative arc.".to_string());
        lines.push("- Output ONLY the story text, no title, no quotes, no markdown blockquotes.".to_string());

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;
    use gzmo_chaos::pulse::ChaosSnapshot;

    #[test]
    fn test_derive_mode_varies_by_phase() {
        let mut idle = ChaosSnapshot::default();
        idle.phase = Phase::Idle;
        idle.chaos_val = 0.05;
        idle.x = 0.1;
        idle.y = 0.2;

        let mut build = idle.clone();
        build.phase = Phase::Build;

        assert_ne!(
            StoryBrief::derive_mode(&idle),
            StoryBrief::derive_mode(&build)
        );
    }

    #[test]
    fn test_story_prompt_includes_tick_and_nonce() {
        let mut snap = ChaosSnapshot::default();
        snap.tick = 4242;
        snap.phase = Phase::Build;
        snap.llm_valence = 0.5;

        let brief = StoryBrief::new(StoryBriefInput {
            keyword: "lighthouse",
            snap: &snap,
            recent_themes: &[],
            call_serial: 99,
            attempt: 1,
            instant_nanos: 123_456,
        });
        let user_prompt = brief.user_prompt();

        assert!(user_prompt.contains("tick 4242"));
        assert!(user_prompt.contains("phase Build"));
        assert!(user_prompt.contains("invocation #99"));
        assert!(user_prompt.contains(&format!("Nonce: {}", brief.nonce)));
    }

    #[test]
    fn test_nonce_differs_for_same_tick_different_serial() {
        let n1 = build_nonce("chaos", 100, 1, 1, 0);
        let n2 = build_nonce("chaos", 100, 2, 1, 0);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_nonce_differs_for_same_tick_different_attempt() {
        let n1 = build_nonce("chaos", 100, 1, 1, 0);
        let n2 = build_nonce("chaos", 100, 1, 2, 0);
        assert_ne!(n1, n2);
    }
}
