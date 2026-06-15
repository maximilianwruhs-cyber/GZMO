//! `/ops` — toggle execution-first mode or autonomous trigger arm.

use anyhow::Result;
use async_trait::async_trait;

use crate::config::PedagogyConfig;
use crate::pedagogy::session::PedagogySession;

use super::generative::boxed_display;
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct OpsSkill {
    pub pedagogy_config: PedagogyConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpsSubcommand {
    ToggleOps,
    ToggleAuto,
    Unknown,
}

fn parse_ops_subcommand(args: &str) -> OpsSubcommand {
    let token = args
        .split_whitespace()
        .find(|t| *t != "--json")
        .unwrap_or("")
        .to_ascii_uppercase();
    match token.as_str() {
        "" => OpsSubcommand::ToggleOps,
        "AUTO" => OpsSubcommand::ToggleAuto,
        _ => OpsSubcommand::Unknown,
    }
}

#[async_trait]
impl Skill for OpsSkill {
    fn name(&self) -> &str {
        "ops"
    }

    fn description(&self) -> &str {
        "Toggle ops mode or /ops AUTO for daemon autotriggers"
    }

    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        match parse_ops_subcommand(ctx.args) {
            OpsSubcommand::ToggleAuto => self.toggle_auto(ctx).await,
            OpsSubcommand::ToggleOps => self.toggle_ops_mode().await,
            OpsSubcommand::Unknown => Ok(SkillOutput {
                display: boxed_display(
                    "OPS",
                    "⚙️",
                    "Usage:\n  /ops       — toggle execution-first ops mode\n  /ops AUTO  — toggle daemon autotriggers\n             (low-tension discovery cycle + /dice loop)",
                ),
                feedback: vec![],
                inject_to_conversation: false,
                evidence: None,
            }),
        }
    }
}

impl OpsSkill {
    async fn toggle_ops_mode(&self) -> Result<SkillOutput> {
        let mut session = PedagogySession::load(&self.pedagogy_config).await?;
        let active = session.toggle_ops();
        session.save(&self.pedagogy_config).await?;

        let body = if active {
            format!(
                "Ops mode ON — execution-first. Pedagogy orchestrator bypassed.\n\
                 Run /ops again to return to mentor mode.\n\
                 AUTO triggers: {}",
                if session.auto_triggers_enabled {
                    "ON"
                } else {
                    "OFF"
                }
            )
        } else {
            format!(
                "Ops mode OFF — Friendly Linux Mentor restored.\n\
                 Socratic scaffolding active.\n\
                 AUTO triggers: {}",
                if session.auto_triggers_enabled {
                    "ON"
                } else {
                    "OFF"
                }
            )
        };

        Ok(SkillOutput {
            display: boxed_display(
                if active { "OPS MODE" } else { "MENTOR MODE" },
                "⚙️",
                &body,
            ),
            feedback: vec![],
            inject_to_conversation: true,
            evidence: Some(serde_json::json!({
                "skill": "ops",
                "subcommand": "toggle",
                "ops_mode": active,
                "auto_triggers": session.auto_triggers_enabled,
            })),
        })
    }

    async fn toggle_auto(&self, _ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let mut session = PedagogySession::load(&self.pedagogy_config).await?;
        let active = session.toggle_auto_triggers();
        session.save(&self.pedagogy_config).await?;

        let body = if active {
            "AUTO triggers ON — low-tension discovery cycle (pillar probe + report) and /dice follow-up loop armed.\n\
             Run /ops AUTO again to disable."
        } else {
            "AUTO triggers OFF — no autonomous discovery cycles or scheduled /dice rolls.\n\
             Run /ops AUTO again to re-enable."
        };

        Ok(SkillOutput {
            display: boxed_display(
                if active { "AUTO ON" } else { "AUTO OFF" },
                "⏱",
                body,
            ),
            feedback: vec![],
            inject_to_conversation: true,
            evidence: Some(serde_json::json!({
                "skill": "ops",
                "subcommand": "auto",
                "auto_triggers": active,
                "ops_mode": session.ops_mode,
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_auto_subcommand() {
        assert_eq!(parse_ops_subcommand(""), OpsSubcommand::ToggleOps);
        assert_eq!(parse_ops_subcommand("AUTO"), OpsSubcommand::ToggleAuto);
        assert_eq!(parse_ops_subcommand("auto"), OpsSubcommand::ToggleAuto);
        assert_eq!(parse_ops_subcommand("--json AUTO"), OpsSubcommand::ToggleAuto);
    }
}
