//! # Language Skill — `/language [code]`
//!
//! Switch output language for all generative skills.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use super::llm::{frame_box, get_language, SkillRuntime, BOLD, DIM, GREEN, RED, RESET};
use super::{Skill, SkillContext, SkillOutput, SkillType};

fn lang_name(code: &str) -> &str {
    static NAMES: &[(&str, &str)] = &[
        ("en", "English"),
        ("de", "Deutsch"),
        ("ja", "日本語"),
        ("fr", "Français"),
        ("es", "Español"),
        ("it", "Italiano"),
        ("pt", "Português"),
        ("zh", "中文"),
        ("ko", "한국어"),
        ("ru", "Русский"),
        ("ar", "العربية"),
        ("hi", "हिन्दी"),
        ("nl", "Nederlands"),
        ("pl", "Polski"),
        ("sv", "Svenska"),
        ("tr", "Türkçe"),
    ];
    NAMES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, n)| *n)
        .unwrap_or(code)
}

pub struct LanguageSkill {
    pub rt: Arc<SkillRuntime>,
}

#[async_trait]
impl Skill for LanguageSkill {
    fn name(&self) -> &str {
        "language"
    }
    fn description(&self) -> &str {
        "Switch output language for generative skills"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let code = ctx.args.trim().to_lowercase();

        if code.is_empty() {
            let current = get_language(&self.rt.skills_dir);
            let display = format!(
                "{DIM}Current language: {BOLD}{current}{RESET}\n\
                 {DIM}Usage: /language <code>  (en, de, ja, fr, es, it, pt, zh, ko, ru, ar, hi){RESET}\n\
                 {DIM}Reset: /language en{RESET}",
                DIM = DIM,
                BOLD = BOLD,
                RESET = RESET
            );
            return Ok(SkillOutput {
                display,
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        if code.len() < 2 || code.len() > 3 || !code.chars().all(|c| c.is_ascii_lowercase()) {
            return Ok(SkillOutput {
                display: format!(
                    "  {RED}✗ Invalid language code: {code}. Use BCP-47 codes (en, de, ja, fr, etc.){RESET}"
                ),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        if let Some(parent) = self.rt.lang_state_path().parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(self.rt.lang_state_path(), &code)?;

        let name = lang_name(&code);
        let body = format!(
            "  {BOLD}{code}{RESET} — {name}\n\n\
             {DIM}All generative commands will now respond in{RESET}\n\
             {BOLD}{name}{RESET}{DIM}.{RESET}",
            BOLD = BOLD,
            DIM = DIM,
            RESET = RESET
        );
        let display = frame_box("LANGUAGE SWITCHED", &body, "🌍", GREEN);

        Ok(SkillOutput {
            display,
            feedback: vec![],
            inject_to_conversation: true,
        })
    }
}
