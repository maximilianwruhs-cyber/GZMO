//! # Transform Skill — `/transform [character]`
//!
//! Activate a character persona overlay for generative skills.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;
use serde::Deserialize;

use super::llm::{
    frame_box, llm_available, llm_chat, SkillRuntime, BOLD, CYAN, DIM, GREEN, MAGENTA, RED, RESET,
    WHITE, YELLOW,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

#[derive(Debug, Deserialize)]
struct CharactersFile {
    #[serde(default)]
    characters: Vec<CharacterEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct CharacterEntry {
    name: String,
    #[serde(default)]
    alter_ego: String,
    #[serde(default)]
    universe: String,
    #[serde(default)]
    icon: String,
    #[serde(default)]
    speech_style: String,
    #[serde(default)]
    personality: String,
    #[serde(default)]
    system_prompt: String,
}

pub struct TransformSkill {
    pub rt: Arc<SkillRuntime>,
}

fn find_character<'a>(entries: &'a [CharacterEntry], query: &str) -> Option<&'a CharacterEntry> {
    let q = query.to_lowercase();
    entries.iter().find(|c| c.name.to_lowercase().contains(&q))
}

fn write_transform_state(rt: &SkillRuntime, ch: &CharacterEntry) -> Result<()> {
    let content = format!(
        "PERSONA: {}\nICON: {}\nSPEECH: {}\nPERSONALITY: {}\nSYSTEM_PROMPT: {}\n",
        ch.name, ch.icon, ch.speech_style, ch.personality, ch.system_prompt
    );
    let path = rt.transform_state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, content)?;
    Ok(())
}

#[async_trait]
impl Skill for TransformSkill {
    fn name(&self) -> &str {
        "transform"
    }
    fn description(&self) -> &str {
        "Activate a character persona overlay for generative skills"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let character = ctx.args.trim();

        if character.is_empty() {
            let path = self.rt.transform_state_path();
            if path.exists() {
                std::fs::remove_file(&path)?;
                let body = format!(
                    "  {WHITE}Persona cleared. Back to default GZMO voice.{RESET}",
                    WHITE = WHITE,
                    RESET = RESET
                );
                let display = frame_box("TRANSFORM RESET", &body, "🎭", GREEN);
                let feedback = ChaosEvent::PersonaCleared;
                let _ = ctx.feedback_tx.send(feedback.clone()).await;
                return Ok(SkillOutput {
                    display,
                    feedback: vec![feedback],
                    inject_to_conversation: true,
                });
            }

            let chars_path = self.rt.characters_path();
            if !chars_path.exists() {
                return Ok(SkillOutput {
                    display: format!(
                        "  {RED}✗ characters.toml not found at {}{RESET}",
                        chars_path.display()
                    ),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }

            let content = std::fs::read_to_string(&chars_path)?;
            let file: CharactersFile =
                toml::from_str(&content).unwrap_or(CharactersFile { characters: vec![] });

            let mut body = String::new();
            for ch in &file.characters {
                body.push_str(&format!(
                    "  {BOLD}{}{RESET}  {BOLD}{}{RESET}\n",
                    ch.icon, ch.name
                ));
            }
            body.push_str(&format!(
                "\n  {DIM}Usage: /transform <name>{RESET}\n  {DIM}Reset: /transform{RESET}",
                DIM = DIM,
                RESET = RESET
            ));

            let display = frame_box("AVAILABLE PERSONAS", &body, "🎭", MAGENTA);
            return Ok(SkillOutput {
                display,
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        let chars_path = self.rt.characters_path();
        if !chars_path.exists() {
            return Ok(SkillOutput {
                display: format!(
                    "  {RED}✗ characters.toml not found at {}{RESET}",
                    chars_path.display()
                ),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        let content = std::fs::read_to_string(&chars_path)?;
        let file: CharactersFile =
            toml::from_str(&content).unwrap_or(CharactersFile { characters: vec![] });

        let mut ch = find_character(&file.characters, character).cloned();

        if ch.is_none() && llm_available(&self.rt).await {
            let custom_prompt = "You are a character analyst. Given a fictional or real character name, create a persona profile.
Output EXACTLY in this format (5 lines, no other text):
NAME: [character name]
SPEECH: [2 sentences describing their unique speech patterns and vocabulary]
PERSONALITY: [1 sentence describing core personality trait]
CATCHPHRASE: [one iconic quote or saying]
SYSTEM_PROMPT: [A 2-sentence instruction for an AI to roleplay as this character]";

            if let Ok(raw) = llm_chat(
                &self.rt,
                custom_prompt,
                &format!("Create a persona profile for: {character}"),
                0.7,
                384,
                true,
            )
            .await
            {
                let mut entry = CharacterEntry {
                    name: character.to_string(),
                    icon: "🎭".to_string(),
                    ..Default::default()
                };
                for line in raw.lines() {
                    let line = line.trim();
                    if let Some(v) = line.strip_prefix("SPEECH:") {
                        entry.speech_style = v.trim().to_string();
                    } else if let Some(v) = line.strip_prefix("PERSONALITY:") {
                        entry.personality = v.trim().to_string();
                    } else if let Some(v) = line.strip_prefix("SYSTEM_PROMPT:") {
                        entry.system_prompt = v.trim().to_string();
                    }
                }
                if !entry.system_prompt.is_empty() {
                    ch = Some(entry);
                }
            }
        }

        let Some(ch) = ch else {
            return Ok(SkillOutput {
                display: format!(
                    "  {RED}✗ Character '{character}' not found in the Pantheon and LLM offline.{RESET}\n\
                     {DIM}  Available characters: Superman, Batman, Spider-Man, Wonder Woman,{RESET}\n\
                     {DIM}  Wolverine, Captain America, Iron Man, The Flash, Hulk, Thor{RESET}",
                    DIM = DIM
                ),
                feedback: vec![],
                inject_to_conversation: false,
            });
        };

        write_transform_state(&self.rt, &ch)?;

        let mut body = format!(
            "  {BOLD}{icon} {name}{RESET}\n",
            BOLD = BOLD,
            icon = ch.icon,
            name = ch.name,
            RESET = RESET
        );
        if !ch.alter_ego.is_empty() {
            body.push_str(&format!(
                "  {DIM}aka {} ({}){RESET}\n",
                ch.alter_ego, ch.universe
            ));
        }
        body.push_str(&format!(
            "\n  {CYAN}Speech:{RESET} {}\n\n  {YELLOW}Personality:{RESET} {}\n\n\
             {DIM}All generative commands will now channel this{RESET}\n\
             {DIM}persona until you run /transform again.{RESET}",
            ch.speech_style, ch.personality
        ));

        let mut display = frame_box("TRANSFORM ACTIVATED", &body, "🎭", MAGENTA);

        if llm_available(&self.rt).await {
            if let Ok(intro) = llm_chat(
                &self.rt,
                &ch.system_prompt,
                "Introduce yourself in one dramatic sentence. Stay in character.",
                0.9,
                128,
                false,
            )
            .await
            {
                display.push_str(&format!(
                    "\n  {BOLD}{}{RESET} {WHITE}{}{RESET}\n",
                    ch.icon, intro
                ));
            }
        }

        let feedback = ChaosEvent::PersonaShift {
            persona: ch.name.clone(),
        };
        let _ = ctx.feedback_tx.send(feedback.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback],
            inject_to_conversation: true,
        })
    }
}

impl Default for CharacterEntry {
    fn default() -> Self {
        Self {
            name: String::new(),
            alter_ego: String::new(),
            universe: String::new(),
            icon: String::new(),
            speech_style: String::new(),
            personality: String::new(),
            system_prompt: String::new(),
        }
    }
}
