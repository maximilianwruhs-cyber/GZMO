//! # Quote Skill — `/quote`
//!
//! Serves a verified historical quote from lore.toml,
//! selected by chaos engine state.

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::ChaosEvent;

use super::{Skill, SkillContext, SkillOutput, SkillType};

const WHITE: &str = "\x1b[97m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";

pub struct QuoteSkill;

#[derive(serde::Deserialize)]
struct LoreFile {
    #[serde(default)]
    quotes: Vec<QuoteEntry>,
}

#[derive(serde::Deserialize)]
struct QuoteEntry {
    text: String,
    #[serde(default = "default_author")]
    author: String,
}

fn default_author() -> String {
    "Unknown".to_string()
}

#[async_trait]
impl Skill for QuoteSkill {
    fn name(&self) -> &str {
        "quote"
    }
    fn description(&self) -> &str {
        "Serve a verified historical quote from the Lore Pool"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mechanical
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        // Try multiple lore.toml locations
        let lore_paths = [
            std::path::PathBuf::from("../Randomizer/lore.toml"),
            std::path::PathBuf::from("lore.toml"),
            std::path::PathBuf::from("data/lore.toml"),
        ];

        let lore_content = lore_paths
            .iter()
            .find_map(|p| std::fs::read_to_string(p).ok());

        let content = match lore_content {
            Some(c) => c,
            None => {
                return Ok(SkillOutput {
                    display: format!("  {RED}✗ lore.toml not found{RESET}"),
                    feedback: vec![],
                    inject_to_conversation: false,
                })
            }
        };

        let lore: LoreFile = match toml::from_str(&content) {
            Ok(l) => l,
            Err(e) => {
                return Ok(SkillOutput {
                    display: format!("  {RED}✗ Failed to parse lore.toml: {e}{RESET}"),
                    feedback: vec![],
                    inject_to_conversation: false,
                })
            }
        };

        if lore.quotes.is_empty() {
            return Ok(SkillOutput {
                display: format!("  {RED}✗ No quotes found in lore.toml{RESET}"),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        let idx = ((ctx.chaos.chaos_val * 10000.0 + ctx.chaos.x.abs() * 100.0) as usize)
            % lore.quotes.len();
        let quote = &lore.quotes[idx];

        let display = format!(
            "\n{DIM}┌─────────────────────────────────────────────────┐{RESET}\n\
             {BOLD}{CYAN}  📜 QUOTE{RESET}\n\
             {DIM}├─────────────────────────────────────────────────┤{RESET}\n\n\
               {WHITE}\"{}\"{RESET}\n\n\
               {DIM}— {}{RESET}\n\n\
             {DIM}└─────────────────────────────────────────────────┘{RESET}\n",
            quote.text, quote.author,
        );

        // Send feedback to chaos engine — quotes affect the autopoietic loop
        let feedback_event = ChaosEvent::QuoteSurfaced {
            text: quote.text.clone(),
        };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
        })
    }
}
