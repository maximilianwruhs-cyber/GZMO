use anyhow::Result;
use async_trait::async_trait;

use super::generative::boxed_display;
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct LanguageSkill;

#[async_trait]
impl Skill for LanguageSkill {
    fn name(&self) -> &str { "language" }
    fn description(&self) -> &str { "Switch output language for all generative commands" }
    fn skill_type(&self) -> SkillType { SkillType::Mutation }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let lang_path = ctx.skills_dir.join(".language");
        let code = ctx.args.trim();
        if code.is_empty() {
            let current = std::fs::read_to_string(&lang_path).unwrap_or_else(|_| "en".to_string());
            return Ok(SkillOutput {
                display: boxed_display("LANGUAGE", "🌍", &format!("Current language: {}", current.trim())),
                feedback: vec![],
                inject_to_conversation: false,
            evidence: None,
            });
        }
        std::fs::write(&lang_path, code)?;
        Ok(SkillOutput {
            display: boxed_display("LANGUAGE", "🌍", &format!("Output language set to: {code}")),
            feedback: vec![],
            inject_to_conversation: true,
            evidence: None,
        })
    }
}
