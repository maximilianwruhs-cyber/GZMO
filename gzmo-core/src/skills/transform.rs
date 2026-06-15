use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;

use super::generative::{boxed_display, llm_complete};
use super::persona::{
    find_character, list_characters_categorized, load_characters, parse_custom_profile,
    persona_state_path, write_persona_state, PersonaState,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const CUSTOM_PROFILE_SYSTEM: &str = "You are a character analyst. Given a fictional or real character name, create a persona profile.
Output EXACTLY in this format (5 lines, no other text):
NAME: [character name]
SPEECH: [2 sentences describing their unique speech patterns and vocabulary]
PERSONALITY: [1 sentence describing core personality trait]
CATCHPHRASE: [one iconic quote or saying]
SYSTEM_PROMPT: [A 2-sentence instruction for an AI to roleplay as this character]";

pub struct TransformSkill;

#[async_trait]
impl Skill for TransformSkill {
    fn name(&self) -> &str {
        "transform"
    }
    fn description(&self) -> &str {
        "Activate a character persona that alters all subsequent output"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let state_path: PathBuf = persona_state_path(ctx.skills_dir);
        let characters_path = ctx.skills_dir.join("characters.toml");

        if ctx.args.trim().is_empty() {
            if state_path.exists() {
                std::fs::remove_file(&state_path)?;
                let event = ChaosEvent::PersonaCleared;
                let _ = ctx.feedback_tx.send(event.clone()).await;
                return Ok(SkillOutput {
                    display: boxed_display(
                        "TRANSFORM RESET",
                        "🎭",
                        "Persona cleared. Back to default GZMO voice.",
                    ),
                    feedback: vec![event],
                    inject_to_conversation: true,
            evidence: None,
                });
            }
            let list = list_characters_categorized(&characters_path)?;
            return Ok(SkillOutput {
                display: boxed_display("AVAILABLE PERSONAS", "🎭", &list),
                feedback: vec![],
                inject_to_conversation: false,
            evidence: None,
            });
        }

        let name_query = ctx.args.trim();
        let parsed = load_characters(&characters_path)?;
        let persona = if let Some(character) = find_character(&parsed.characters, name_query) {
            PersonaState::from(character.clone())
        } else if let Some(gateway) = ctx.gateway {
            generate_custom_persona(gateway, ctx.skills_dir, name_query).await?
        } else {
            anyhow::bail!("Character '{name_query}' not found in Pantheon and LLM offline")
        };

        write_persona_state(&state_path, &persona)?;

        let mut body = format!(
            "{} {}\n\nSpeech: {}\nPersonality: {}",
            persona.icon, persona.name, persona.speech_style, persona.personality
        );
        if !persona.structural_constraints.is_empty() {
            body.push_str(&format!("\n\nConstraints: {}", persona.structural_constraints));
        }

        if let Some(gateway) = ctx.gateway {
            if let Ok(intro) = llm_complete(
                gateway,
                ctx.skills_dir,
                &persona.system_prompt,
                "Introduce yourself in one dramatic sentence. Stay in character.",
            )
            .await
            {
                let intro = intro.trim();
                if !intro.is_empty() {
                    body.push_str(&format!("\n\n{} {}", persona.icon, intro));
                }
            }
        }

        let event = ChaosEvent::PersonaShift {
            persona: persona.name.clone(),
        };
        let _ = ctx.feedback_tx.send(event.clone()).await;

        Ok(SkillOutput {
            display: boxed_display("TRANSFORM ACTIVATED", "🎭", &body),
            feedback: vec![event],
            inject_to_conversation: true,
            evidence: None,
        })
    }
}

async fn generate_custom_persona(
    gateway: &dyn crate::gateway::LlmGateway,
    skills_dir: &std::path::Path,
    name: &str,
) -> Result<PersonaState> {
    let user = format!("Create a persona profile for: {name}");
    let raw = llm_complete(gateway, skills_dir, CUSTOM_PROFILE_SYSTEM, &user).await?;
    parse_custom_profile(&raw, name)
        .ok_or_else(|| anyhow::anyhow!("LLM failed to generate a valid profile for '{name}'"))
}
