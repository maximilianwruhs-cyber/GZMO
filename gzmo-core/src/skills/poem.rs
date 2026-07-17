//! # Poem Skill — `/poem`
//!
//! Short poem generation (max 180 characters).

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;

use super::llm::{
    accept_creative_output, chaos_index, frame_box, llm_chat, quality_gate_poem, SkillRuntime,
    MAGENTA, WHITE,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const SYSTEM_PROMPT: &str = "You are a critically acclaimed contemporary German poet. Write a short, highly evocative poem.

CRITICAL CONSTRAINTS:
- STRICTLY BAN simple, predictable end-rhymes (e.g., Herz/Schmerz, Nacht/Lacht, Zeit/Weit). If you rhyme, use subtle slant rhymes (unreine Reime oder Binnenreime) or assonances.
- Avoid abstract words: eternity, soul, fate, whisper, dance, shadows, tears, Ewigkeit, Seele, Schicksal, Tränen.
- Focus on concrete, physical objects, textures, and sensory details.
- Maximum 180 characters total.
- Output ONLY the poem. No titles, no introduction, no markdown blockquotes, no commentary.";

const USER_PROMPT: &str = "Write a short, powerful poem.";

const FALLBACKS: [&str; 3] = [
    "Kupfer grünt, das Glas vergilbt langsam\nSand sinkt hinab, der Stahl bricht ab\nAsche legt sich, die Kälte bleibt",
    "Der Ruß auf der Kachel zerfällt leise\nKaltes Eisen gibt nach, dehnt sich aus\nKein Rad greift mehr ins andere",
    "Ein Tropfen Öl auf trockenem Schiefer\nEr glänzt im trüben Mittagslicht\nBevor der Stein den Glanz verschluckt",
];

pub struct PoemSkill {
    pub rt: Arc<SkillRuntime>,
}

#[async_trait]
impl Skill for PoemSkill {
    fn name(&self) -> &str {
        "poem"
    }
    fn description(&self) -> &str {
        "Short evocative poem via LLM (max 180 chars)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let mut poem = String::new();
        for _ in 0..3 {
            if let Ok(raw) = llm_chat(&self.rt, SYSTEM_PROMPT, USER_PROMPT, 0.85, 4096, false).await {
                if accept_creative_output(&raw, 180, quality_gate_poem) {
                    poem = raw;
                    break;
                }
            }
        }

        if poem.is_empty() {
            let idx = chaos_index(ctx.chaos, FALLBACKS.len());
            poem = FALLBACKS[idx].to_string();
        }

        let body = format!("  {WHITE}{poem}{RESET}", WHITE = WHITE, RESET = super::llm::RESET);
        let display = frame_box("POEM", &body, "🖋️ ", MAGENTA);

        let feedback_event = ChaosEvent::PoemGenerated { text: poem.clone() };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
        })
    }
}
