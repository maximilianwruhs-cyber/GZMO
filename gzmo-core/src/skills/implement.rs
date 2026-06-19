//! `/implement` — dequeue Forum-2 queue, run probes, spawn plan agent, eval gate.

use anyhow::Result;
use async_trait::async_trait;

use crate::config::PedagogyConfig;

use super::discovery_ops::{run_scripts_command, scripts_root};
use super::generative::boxed_display;
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct ImplementSkill {
    pub pedagogy_config: PedagogyConfig,
}

#[async_trait]
impl Skill for ImplementSkill {
    fn name(&self) -> &str {
        "implement"
    }

    fn description(&self) -> &str {
        "Run discovery implementation plan phase (Forum-2 queue → plan dossier)"
    }

    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let root = scripts_root(&self.pedagogy_config);
        let args: Vec<&str> = ctx.args.split_whitespace().filter(|t| *t != "--json").collect();
        let (code, output) = run_scripts_command(&root, "run-discovery-implement.sh", &args).await?;

        let body = if output.is_empty() {
            if code == 0 {
                "Discovery implement phase finished.".into()
            } else {
                format!("Discovery implement failed (exit {code}).")
            }
        } else {
            output
        };

        Ok(SkillOutput {
            display: boxed_display("IMPLEMENT", "📋", &body),
            feedback: vec![],
            inject_to_conversation: code == 0,
            evidence: Some(serde_json::json!({ "exit_code": code })),
        })
    }
}
