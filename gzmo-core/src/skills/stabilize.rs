use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;

use super::generative::boxed_display;
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct StabilizeSkill;

#[async_trait]
impl Skill for StabilizeSkill {
    fn name(&self) -> &str { "stabilize" }
    fn description(&self) -> &str { "Stabilize the chaos engine attractor by decreasing rho" }
    fn skill_type(&self) -> SkillType { SkillType::Mechanical }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let delta = super::dispatch::stabilize_delta_rho(ctx.config);
        let event = ChaosEvent::Stabilize { delta_rho: delta };
        let _ = ctx.feedback_tx.send(event.clone()).await;
        let msg = if delta < 0.0 {
            format!("Attractor stabilized. Lorenz ρ mod decreased by {:.1}", -delta)
        } else {
            format!("Attractor stabilized. Lorenz ρ mod increased by {:.1}", delta)
        };
        Ok(SkillOutput {
            display: boxed_display("STABILIZE", "🌀", &msg),
            feedback: vec![event],
            inject_to_conversation: true,
            evidence: None,
        })
    }
}
