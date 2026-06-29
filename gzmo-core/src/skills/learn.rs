//! `/learn` — flipped classroom prep for a topic.

use anyhow::Result;
use async_trait::async_trait;

use crate::config::{PedagogyConfig, TaskKind};
use crate::pedagogy::SimplifiedOrchestrator;
use crate::pedagogy::session::PedagogySession;

use super::generative::boxed_display;
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct LearnSkill {
    pub pedagogy_config: PedagogyConfig,
}

#[async_trait]
impl Skill for LearnSkill {
    fn name(&self) -> &str {
        "learn"
    }

    fn description(&self) -> &str {
        "Start flipped-classroom prep for a topic, then Socratic sync session"
    }

    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let topic = ctx.args.trim();
        if topic.is_empty() {
            return Ok(SkillOutput {
                display: boxed_display(
                    "LEARN",
                    "📚",
                    "Usage: /learn <topic>\nExample: /learn systemd units",
                ),
                feedback: vec![],
                inject_to_conversation: false,
            evidence: None,
            });
        }

        let orchestrator = SimplifiedOrchestrator::new(self.pedagogy_config.clone(), None);
        let prep = if let Some(router) = ctx.router {
            let gw = router.gateway(TaskKind::PedagogyInternal);
            orchestrator.run_learn_prep(gw.as_ref(), topic).await?
        } else if let Some(gateway) = ctx.gateway {
            orchestrator.run_learn_prep(gateway, topic).await?
        } else {
            return Ok(SkillOutput {
                display: boxed_display(
                    "LEARN",
                    "📚",
                    "LLM offline — cannot prep flipped-classroom materials.",
                ),
                feedback: vec![],
                inject_to_conversation: false,
            evidence: None,
            });
        };

        let mut session = PedagogySession::load(&self.pedagogy_config).await?;
        session.set_learn_prep(topic);
        session.learn_prep_notes = Some(prep.clone());
        session.save(&self.pedagogy_config).await?;

        let body = format!(
            "Flipped classroom prep ready for: {topic}\n\n{prep}\n\n\
             Ask your first question — Socratic sync session is live."
        );

        Ok(SkillOutput {
            display: boxed_display("LEARN PREP", "📚", &body),
            feedback: vec![],
            inject_to_conversation: true,
            evidence: None,
        })
    }
}
