//! `/fixer` — execute planned workstreams from Forum-2 plan.json.

use anyhow::Result;
use async_trait::async_trait;

use crate::config::PedagogyConfig;

use super::discovery_ops::{run_scripts_command, scripts_root};
use super::generative::boxed_display;
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct FixerSkill {
    pub pedagogy_config: PedagogyConfig,
}

#[async_trait]
impl Skill for FixerSkill {
    fn name(&self) -> &str {
        "fixer"
    }

    fn description(&self) -> &str {
        "Run discovery fixer execute phase (workstreams from approved plan)"
    }

    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let root = scripts_root(&self.pedagogy_config);
        let args: Vec<&str> = ctx.args.split_whitespace().filter(|t| *t != "--json").collect();
        let (code, output) = run_scripts_command(&root, "run-discovery-fixer.sh", &args).await?;

        let body = if output.is_empty() {
            if code == 0 {
                "Discovery fixer phase finished.".into()
            } else {
                format!("Discovery fixer failed (exit {code}).")
            }
        } else {
            output
        };

        Ok(SkillOutput {
            display: boxed_display("FIXER", "🔧", &body),
            feedback: vec![],
            inject_to_conversation: code == 0,
            evidence: Some(serde_json::json!({ "exit_code": code })),
        })
    }
}
