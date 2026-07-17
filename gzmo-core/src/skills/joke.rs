//! # Joke Skill — `/joke`
//!
//! Structurally engineered joke via LLM (Benign Violation Theory).

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;

use super::llm::{
    accept_creative_output, chaos_index, frame_box, llm_chat, quality_gate_joke, SkillRuntime,
    WHITE, YELLOW,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const SYSTEM_PROMPT: &str = "You are a comedy engine grounded in the Benign Violation Theory (BVT).
A joke triggers laughter if and only if three conditions occur simultaneously:
1. A VIOLATION — something threatens how the world ought to be (social norms, physical laws, logic).
2. A BENIGN CONTEXT — the threat is completely harmless or reframed safely.
3. SIMULTANEOUS PROCESSING — both must be processed at the same neurological millisecond.

Structure your joke using:
- SETUP: Establish a false, highly logical reality. Must be entirely devoid of comedy.
- MISDIRECTION: The invisible cognitive pivot point.
- PUNCHLINE: Violently subverts the expectation while technically complying with the setup's logic.

CRITICAL CONSTRAINTS:
- STRICTLY FORBIDDEN clichés: programming bugs, coffee, bad weather, typical artificial intelligence jokes, or simple 'dad jokes' (Flachwitze).
- Focus on clever, situational irony or absurdist framing.
- Max 280 characters total. Output ONLY the joke text. No labels, no explanation.";

const USER_PROMPT: &str = "Tell me one original, clever joke. Make me laugh.";

const FALLBACKS: [&str; 3] = [
    "Der Optiker fragte, ob ich die Brille zum Lesen oder zum Sehen brauche. Ich sagte: zum Überleben des Kleingedruckten.",
    "Mein Nachbar betet jeden Abend. Nicht aus Glauben — er will, dass der Kühlschrank endlich aufhört zu summen.",
    "Sie nannten mich optimistisch, weil ich bei jeder Absage nach dem Parkplatz gefragt habe.",
];

pub struct JokeSkill {
    pub rt: Arc<SkillRuntime>,
}

#[async_trait]
impl Skill for JokeSkill {
    fn name(&self) -> &str {
        "joke"
    }
    fn description(&self) -> &str {
        "Structurally engineered joke via LLM (BVT)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let mut joke = String::new();
        for _ in 0..3 {
            if let Ok(raw) = llm_chat(&self.rt, SYSTEM_PROMPT, USER_PROMPT, 0.9, 4096, false).await
            {
                if accept_creative_output(&raw, 280, quality_gate_joke) {
                    joke = raw;
                    break;
                }
            }
        }

        if joke.is_empty() {
            let idx = chaos_index(ctx.chaos, FALLBACKS.len());
            joke = FALLBACKS[idx].to_string();
        }

        if joke.is_empty() {
            return Ok(SkillOutput {
                display: format!(
                    "  {RED}✗ LLM offline and no fallback jokes available.{RESET}",
                    RED = super::llm::RED,
                    RESET = super::llm::RESET
                ),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        let body = format!(
            "  {WHITE}{joke}{RESET}",
            WHITE = WHITE,
            RESET = super::llm::RESET
        );
        let display = frame_box("JOKE", &body, "😂", YELLOW);

        let feedback_event = ChaosEvent::JokeGenerated { text: joke.clone() };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
        })
    }
}
