//! # Word Skill — `/word`
//!
//! Invent a brand new word with definition and example.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;

use super::llm::{llm_chat, SkillRuntime, BOLD, CYAN, DIM, GREEN, RED, RESET, WHITE};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const SYSTEM_PROMPT: &str = "You are a neologist — an inventor of words.
Create one completely new word that does not exist in any language.
The word should:
- Sound natural and pronounceable
- Have a specific, useful meaning that fills a gap in language
- Come with a believable etymology

Format your response EXACTLY like this (3 lines only):
WORD: [the new word] ([pronunciation])
DEFINITION: [clear definition]
EXAMPLE: [one example sentence using the word]

No other text. No commentary.";

const USER_PROMPT: &str = "Invent a new word.";

pub struct WordSkill {
    pub rt: Arc<SkillRuntime>,
}

fn format_word_lines(result: &str) -> String {
    let mut out = String::new();
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("WORD:") {
            out.push_str(&format!("  {BOLD}{CYAN}WORD:{rest}{RESET}\n"));
        } else if let Some(rest) = trimmed.strip_prefix("DEFINITION:") {
            out.push_str(&format!("  {WHITE}DEFINITION:{rest}{RESET}\n"));
        } else if let Some(rest) = trimmed.strip_prefix("EXAMPLE:") {
            out.push_str(&format!("  {DIM}EXAMPLE:{rest}{RESET}\n"));
        } else {
            out.push_str(&format!("  {trimmed}\n"));
        }
    }
    out
}

#[async_trait]
impl Skill for WordSkill {
    fn name(&self) -> &str {
        "word"
    }
    fn description(&self) -> &str {
        "Invent a new word with definition and example"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let result = match llm_chat(&self.rt, SYSTEM_PROMPT, USER_PROMPT, 0.95, 256, false).await {
            Ok(r) if !r.is_empty() => r,
            _ => {
                return Ok(SkillOutput {
                    display: format!(
                        "  {RED}✗ LLM offline. Cannot invent words without a brain.{RESET}"
                    ),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
        };

        let body = format_word_lines(&result);
        let display = super::llm::frame_box("NEW WORD", &body, "🔤", GREEN);

        let word = result
            .lines()
            .find_map(|l| l.trim().strip_prefix("WORD:").map(|s| s.trim().to_string()))
            .unwrap_or_else(|| "unknown".to_string());
        let definition = result
            .lines()
            .find_map(|l| {
                l.trim()
                    .strip_prefix("DEFINITION:")
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_default();

        let feedback_event = ChaosEvent::WordGenerated { word, definition };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
        })
    }
}
