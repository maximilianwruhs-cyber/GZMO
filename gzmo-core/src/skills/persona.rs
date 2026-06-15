//! `/transform` persona profiles — TOML load, fuzzy match, state file I/O.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

pub const PERSONA_FILE: &str = ".transform_persona";

#[derive(Debug, Clone, Deserialize)]
pub struct CharactersFile {
    #[serde(default)]
    pub characters: Vec<CharacterEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterEntry {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub alter_ego: String,
    #[serde(default)]
    pub universe: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub speech_style: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub catchphrases: Vec<String>,
    pub system_prompt: String,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub structural_constraints: String,
    #[serde(default)]
    pub banned_expressions: Vec<String>,
    #[serde(default)]
    pub mandatory_vocabulary: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PersonaState {
    pub name: String,
    pub icon: String,
    pub speech_style: String,
    pub personality: String,
    pub system_prompt: String,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub structural_constraints: String,
    pub banned_expressions: Vec<String>,
    pub mandatory_vocabulary: Vec<String>,
}

impl From<CharacterEntry> for PersonaState {
    fn from(c: CharacterEntry) -> Self {
        Self {
            name: c.name,
            icon: c.icon,
            speech_style: c.speech_style,
            personality: c.personality,
            system_prompt: c.system_prompt,
            temperature: c.temperature,
            top_p: c.top_p,
            structural_constraints: c.structural_constraints,
            banned_expressions: c.banned_expressions,
            mandatory_vocabulary: c.mandatory_vocabulary,
        }
    }
}

pub fn load_characters(path: &Path) -> Result<CharactersFile> {
    let file = std::fs::read_to_string(path)
        .with_context(|| format!("read characters.toml at {}", path.display()))?;
    toml::from_str(&file).context("parse characters.toml")
}

pub fn find_character<'a>(characters: &'a [CharacterEntry], query: &str) -> Option<&'a CharacterEntry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }

    characters
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(query))
        .or_else(|| {
            characters.iter().find(|c| {
                let name = c.name.to_lowercase();
                name.contains(&q) || q.contains(&name)
            })
        })
        .or_else(|| {
            characters.iter().find(|c| {
                let name = c.name.to_lowercase();
                name.split_whitespace().any(|word| q.contains(word) && word.len() > 3)
            })
        })
}

pub fn write_persona_state(path: &Path, persona: &PersonaState) -> Result<()> {
    let banned = persona.banned_expressions.join("|");
    let mandatory = persona.mandatory_vocabulary.join("|");
    let mut state = format!(
        "PERSONA: {}\nICON: {}\nSPEECH: {}\nPERSONALITY: {}\nSYSTEM_PROMPT: {}",
        persona.name, persona.icon, persona.speech_style, persona.personality, persona.system_prompt
    );
    if let Some(t) = persona.temperature {
        state.push_str(&format!("\nTEMPERATURE: {t}"));
    }
    if let Some(p) = persona.top_p {
        state.push_str(&format!("\nTOP_P: {p}"));
    }
    if !persona.structural_constraints.is_empty() {
        state.push_str(&format!("\nCONSTRAINTS: {}", persona.structural_constraints));
    }
    if !banned.is_empty() {
        state.push_str(&format!("\nBANNED: {banned}"));
    }
    if !mandatory.is_empty() {
        state.push_str(&format!("\nMANDATORY: {mandatory}"));
    }
    std::fs::write(path, state)?;
    Ok(())
}

pub fn read_persona_state(skills_dir: &Path) -> Option<PersonaState> {
    let raw = std::fs::read_to_string(skills_dir.join(PERSONA_FILE)).ok()?;
    let mut state = PersonaState::default();
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("PERSONA:") {
            state.name = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("ICON:") {
            state.icon = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("SPEECH:") {
            state.speech_style = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("PERSONALITY:") {
            state.personality = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("SYSTEM_PROMPT:") {
            state.system_prompt = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("TEMPERATURE:") {
            state.temperature = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("TOP_P:") {
            state.top_p = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("CONSTRAINTS:") {
            state.structural_constraints = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("BANNED:") {
            state.banned_expressions = rest
                .split('|')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        } else if let Some(rest) = line.strip_prefix("MANDATORY:") {
            state.mandatory_vocabulary = rest
                .split('|')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
    }
    if state.system_prompt.is_empty() {
        None
    } else {
        Some(state)
    }
}

pub fn list_characters_categorized(path: &Path) -> Result<String> {
    let parsed = load_characters(path)?;
    let mut sections: Vec<(String, Vec<String>)> = vec![
        ("polymath".into(), vec![]),
        ("comedic".into(), vec![]),
        ("hero".into(), vec![]),
    ];

    for c in &parsed.characters {
        let line = format!("{}  {}", c.icon, c.name);
        let cat = c.category.to_lowercase();
        if let Some((_, lines)) = sections.iter_mut().find(|(k, _)| *k == cat) {
            lines.push(line);
        } else {
            sections.push((cat.clone(), vec![line]));
        }
    }

    let mut out = String::new();
    for (cat, lines) in sections {
        if lines.is_empty() {
            continue;
        }
        let title = cat.to_uppercase();
        out.push_str(&format!("\n{title}\n"));
        for line in lines {
            out.push_str(&format!("  {line}\n"));
        }
    }
    out.push_str("\nUsage: /transform <name>\nReset: /transform\n");
    Ok(out.trim().to_string())
}

pub fn persona_state_path(skills_dir: &Path) -> PathBuf {
    skills_dir.join(PERSONA_FILE)
}

pub fn violates_persona_constraints(text: &str, persona: &PersonaState) -> bool {
    let lower = text.to_lowercase();
    for banned in &persona.banned_expressions {
        if !banned.is_empty() && lower.contains(&banned.to_lowercase()) {
            return true;
        }
    }
    for word in &persona.mandatory_vocabulary {
        if !word.is_empty() && !lower.contains(&word.to_lowercase()) {
            return true;
        }
    }
    false
}

pub fn parse_custom_profile(llm_output: &str, fallback_name: &str) -> Option<PersonaState> {
    let mut state = PersonaState {
        name: fallback_name.to_string(),
        icon: "🎭".into(),
        ..Default::default()
    };
    for line in llm_output.lines() {
        if let Some(rest) = line.strip_prefix("NAME:") {
            state.name = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("SPEECH:") {
            state.speech_style = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("PERSONALITY:") {
            state.personality = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("SYSTEM_PROMPT:") {
            state.system_prompt = rest.trim().to_string();
        }
    }
    if state.system_prompt.is_empty() {
        None
    } else {
        Some(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring_match_finds_heaviside() {
        let chars = vec![CharacterEntry {
            name: "Oliver Heaviside".into(),
            icon: "⚡".into(),
            alter_ego: String::new(),
            universe: String::new(),
            category: "polymath".into(),
            speech_style: String::new(),
            personality: String::new(),
            catchphrases: vec![],
            system_prompt: "test".into(),
            temperature: None,
            top_p: None,
            structural_constraints: String::new(),
            banned_expressions: vec![],
            mandatory_vocabulary: vec![],
        }];
        assert!(find_character(&chars, "Heaviside").is_some());
        assert!(find_character(&chars, "sherlock").is_none());
    }

    #[test]
    fn banned_expression_triggers_violation() {
        let persona = PersonaState {
            banned_expressions: vec!["hello".into()],
            ..Default::default()
        };
        assert!(violates_persona_constraints("Well hello there", &persona));
        assert!(!violates_persona_constraints("Observe the data.", &persona));
    }
}
