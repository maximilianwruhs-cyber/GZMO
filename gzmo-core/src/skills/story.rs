//! # Story Skill — `/story [keyword]`
//!
//! Short story generation from a keyword seed.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;

use super::llm::{
    accept_creative_output, fold_lines, frame_box, llm_chat, quality_gate_story, SkillRuntime, BLUE,
    RED, RESET, WHITE,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const SYSTEM_PROMPT: &str = "You are a master of the modern short story, writing in the sparse, tense style of Ernest Hemingway or the surreal, absurd style of Franz Kafka.
Write a very short story based on the keyword provided.

RULES:
- Maximum 500 characters total.
- The story must be complete (beginning, middle, end) but have strong subtext or an unresolved, surprising ending.
- Focus on concrete sensory details, physical objects, and specific textures.
- STRICTLY FORBIDDEN: Fairy tales, happily ever after, 'once upon a time', obvious moral lessons, or cheesy clichés.
- Output ONLY the story text. No titles, no labels, no introduction, no markdown blockquotes.";

pub struct StorySkill {
    pub rt: Arc<SkillRuntime>,
}

#[async_trait]
impl Skill for StorySkill {
    fn name(&self) -> &str {
        "story"
    }
    fn description(&self) -> &str {
        "Short story from a keyword seed (max 500 chars)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let keyword = ctx.args.trim();
        let keyword = if keyword.is_empty() { "chaos" } else { keyword };
        let user_prompt = format!("Write a short story based on the keyword: {keyword}");

        let mut story = String::new();
        for _ in 0..3 {
            if let Ok(raw) = llm_chat(&self.rt, SYSTEM_PROMPT, &user_prompt, 0.85, 4096, false).await {
                if accept_creative_output(&raw, 500, quality_gate_story) {
                    story = raw;
                    break;
                }
            }
        }

        if story.is_empty() {
            return Ok(SkillOutput {
                display: format!(
                    "  {RED}✗ LLM offline or story exceeded limits. The story remains untold.{RESET}"
                ),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        let folded = fold_lines(&story, 50);
        let mut body = String::new();
        for line in folded.lines() {
            body.push_str(&format!("  {WHITE}{line}{RESET}\n", WHITE = WHITE, RESET = RESET));
        }

        let title = format!("STORY — seed: \"{keyword}\"");
        let display = frame_box(&title, body.trim_end(), "📖", BLUE);

        let feedback_event = ChaosEvent::StoryGenerated { text: story.clone() };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
        })
    }
}
