//! # Status Skill — `/status`, `/ecosystem`
//!
//! Deterministic ecosystem snapshot from loaded config (no LLM).

use anyhow::Result;
use async_trait::async_trait;

use crate::config::GzmoConfig;
use crate::ecosystem_status::format_ecosystem_status;

use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct StatusSkill {
    pub config: GzmoConfig,
}

#[async_trait]
impl Skill for StatusSkill {
    fn name(&self) -> &str {
        "status"
    }

    fn description(&self) -> &str {
        "Grounded GZMO-next ecosystem snapshot (paths, systemd, probes)"
    }

    fn skill_type(&self) -> SkillType {
        SkillType::Info
    }

    async fn execute(&self, _ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let display = format_ecosystem_status(&self.config).await;
        Ok(SkillOutput {
            display,
            feedback: vec![],
            inject_to_conversation: false,
            evidence: None,
        })
    }
}
