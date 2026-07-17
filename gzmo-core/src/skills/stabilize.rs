//! # Stabilize Skill — `/stabilize`
//!
//! Stabilizes the chaos engine attractor by adjusting Lorenz ρ mod.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;

use super::llm::{frame_box, SkillRuntime, GREEN, WHITE};
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct StabilizeSkill {
    pub rt: Arc<SkillRuntime>,
}

#[async_trait]
impl Skill for StabilizeSkill {
    fn name(&self) -> &str {
        "stabilize"
    }
    fn description(&self) -> &str {
        "Stabilize attractor parameter (decreases rho)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mechanical
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let delta = self.rt.stabilize_delta_rho;
        let feedback = ChaosEvent::Stabilize { delta_rho: delta };
        let _ = ctx.feedback_tx.send(feedback.clone()).await;

        let message = if delta < 0.0 {
            format!(
                "Attractor stabilized. Lorenz ρ mod decreased by {:.1}",
                -delta
            )
        } else {
            format!(
                "Attractor stabilized. Lorenz ρ mod increased by {:.1}",
                delta
            )
        };

        let body = format!(
            "  {WHITE}{message}{RESET}",
            WHITE = WHITE,
            RESET = super::llm::RESET
        );
        let display = frame_box("STABILIZE", &body, "🌀", GREEN);

        Ok(SkillOutput {
            display,
            feedback: vec![feedback],
            inject_to_conversation: true,
        })
    }
}
