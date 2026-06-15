//! Shared LLM + quality-gate infrastructure for generative Rust skills.

use std::path::Path;

use anyhow::{bail, Result};

use crate::gateway::{LlmGateway, LlmResponse};
use crate::types::{Message, Role};

use super::persona::{read_persona_state, violates_persona_constraints};

const LANG_FILE: &str = ".language";

pub fn read_language(skills_dir: &Path) -> String {
    std::fs::read_to_string(skills_dir.join(LANG_FILE))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "en".to_string())
}

pub fn read_persona_prompt(skills_dir: &Path) -> String {
    read_persona_state(skills_dir)
        .map(|p| p.system_prompt)
        .unwrap_or_default()
}

pub fn build_system_prompt(base: &str, skills_dir: &Path) -> String {
    let mut full = base.to_string();
    let lang = read_language(skills_dir);
    if lang != "en" {
        full.push_str(&format!(
            "\n\nCRITICAL: You MUST respond entirely in language code: {lang}. Do not use English."
        ));
    }
    if let Some(persona) = read_persona_state(skills_dir) {
        full.push_str(&format!(
            "\n\nCHARACTER TRANSFORM ACTIVE:\n{}\nYou MUST adopt this character's speech patterns, vocabulary, and personality in your response.",
            persona.system_prompt
        ));
        if !persona.structural_constraints.is_empty() {
            full.push_str(&format!(
                "\n\nSTRUCTURAL CONSTRAINTS:\n{}",
                persona.structural_constraints
            ));
        }
        if !persona.banned_expressions.is_empty() {
            full.push_str(&format!(
                "\n\nNEVER use these expressions: {}",
                persona.banned_expressions.join(", ")
            ));
        }
        if !persona.mandatory_vocabulary.is_empty() {
            full.push_str(&format!(
                "\n\nPrefer these vocabulary anchors: {}",
                persona.mandatory_vocabulary.join(", ")
            ));
        }
    }
    full
}

pub async fn llm_complete(
    gateway: &dyn LlmGateway,
    skills_dir: &Path,
    system: &str,
    user: &str,
) -> Result<String> {
    let persona = read_persona_state(skills_dir);
    let system = build_system_prompt(system, skills_dir);
    let messages = vec![
        Message {
            role: Role::System,
            content: system,
            is_meta: true,
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: user.to_string(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let sampling = persona.as_ref().map(|p| (p.temperature, p.top_p));
    let response = match sampling {
        Some((temp, top_p)) if temp.is_some() || top_p.is_some() => {
            gateway
                .complete_with_persona(&messages, &[], temp, top_p)
                .await?
        }
        _ => gateway.complete(&messages, &[]).await?,
    };

    match response {
        LlmResponse::Text(t) => Ok(t),
        LlmResponse::ToolCalls(_) => bail!("unexpected tool calls from generative skill"),
    }
}

const THINKING_CHANNEL_MARKERS: &[&str] = &["<|channel>thought", "<channel>thought"];

/// Strip Gemma/Qwen thinking-channel wrappers so structured fields like `NAME:` stay parseable.
pub fn strip_thinking_channels(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if THINKING_CHANNEL_MARKERS
                .iter()
                .any(|marker| trimmed.eq_ignore_ascii_case(marker))
            {
                return None;
            }
            let stripped = trimmed
                .strip_prefix("<|channel|>")
                .or_else(|| trimmed.strip_prefix("<channel|>"))
                .unwrap_or(trimmed)
                .trim();
            if stripped.is_empty() {
                None
            } else {
                Some(stripped.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn clean_llm_output(text: &str) -> String {
    strip_thinking_channels(text)
        .replace('`', "")
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

pub fn char_count(text: &str) -> usize {
    text.chars().count()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    let lower = haystack.to_lowercase();
    needles.iter().any(|n| lower.contains(&n.to_lowercase()))
}

pub fn quality_gate_poem(text: &str) -> bool {
    !contains_any(
        text,
        &[
            "seele", "schicksal", "ewigkeit", "tränen", "tranen", "schatten", "flüstern",
            "flustern", " soul", "fate", "eternity", "whisper", "shadows", "tears", " dance",
        ],
    )
}

pub fn quality_gate_joke(text: &str) -> bool {
    !contains_any(
        text,
        &[
            "programmier", "programming bug", "coffee", "kaffee", "artificial intelligence",
            "chatgpt", "dad joke", "flachwitz", "montagmorgen", " wlan", " wifi", " bug",
        ],
    )
}

pub fn quality_gate_story(text: &str) -> bool {
    !contains_any(
        text,
        &[
            "once upon a time", "es war einmal", "happily ever after", "und sie lebten",
            "moral of the story", "lehre des", "märchen", "marchen",
        ],
    )
}

pub fn quality_gate_word(text: &str) -> bool {
    text.lines().any(|l| l.starts_with("WORD:"))
        && !contains_any(
            text,
            &["wordsmith", "neologism of the day", "made-up word:", "fake word:", "lorem ipsum"],
        )
}

pub fn quality_gate_card(text: &str) -> bool {
    super::card_forge::validate_forged_card(text, false)
}

pub fn quality_gate_define(text: &str) -> bool {
    text.lines().any(|l| l.starts_with("DEFINITION:"))
        && !contains_any(
            text,
            &["as an ai", "i don't know", "cannot define", "no definition", "lorem ipsum"],
        )
}

pub fn persona_constraint_gate(skills_dir: &Path) -> impl Fn(&str) -> bool + '_ {
    move |text: &str| {
        read_persona_state(skills_dir)
            .map(|p| !violates_persona_constraints(text, &p))
            .unwrap_or(true)
    }
}

pub fn accept_creative(text: &str, max_chars: usize, gate: fn(&str) -> bool) -> bool {
    let count = char_count(text);
    if count == 0 || count > max_chars {
        return false;
    }
    gate(text)
}

pub async fn try_generative(
    gateway: &dyn LlmGateway,
    skills_dir: &Path,
    system: &str,
    user: &str,
    max_chars: usize,
    gate: fn(&str) -> bool,
    max_attempts: u32,
) -> Option<String> {
    let persona_gate = persona_constraint_gate(skills_dir);
    for _ in 0..max_attempts {
        let raw = llm_complete(gateway, skills_dir, system, user).await.ok()?;
        let cleaned = clean_llm_output(&raw);
        if accept_creative(&cleaned, max_chars, gate) && persona_gate(&cleaned) {
            return Some(cleaned);
        }
    }
    None
}

pub fn chaos_index(snap: &gzmo_chaos::pulse::ChaosSnapshot, modulo: usize) -> usize {
    if modulo == 0 {
        return 0;
    }
    let combined = snap.chaos_val * 10000.0 + snap.x.abs() * 100.0 + snap.y.abs() * 10.0;
    (combined as usize) % modulo
}

pub fn line_value<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix).map(str::trim))
}

pub fn boxed_display(title: &str, icon: &str, body: &str) -> String {
    format!(
        "\n┌─────────────────────────────────────────────────┐\n  {icon} {title}\n├─────────────────────────────────────────────────┤\n\n{body}\n\n└─────────────────────────────────────────────────┘\n"
    )
}

pub fn require_gateway<'a>(ctx: &super::SkillContext<'a>) -> Result<&'a dyn LlmGateway> {
    ctx.gateway
        .ok_or_else(|| anyhow::anyhow!("LLM gateway unavailable"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_thinking_channels_unwraps_inline_name() {
        let raw = "<|channel>thought\n<channel|>NAME: Contingency Plan\nCOST: {1}{W}\nTYPE: Instant\nRARITY: Common\nRULES: Draw a card.\nPT: NONE";
        let cleaned = clean_llm_output(raw);
        assert!(cleaned.starts_with("NAME: Contingency Plan"));
        assert!(quality_gate_card(&cleaned));
    }

    #[test]
    fn strip_thinking_channels_preserves_multiline_name() {
        let raw = "<|channel>thought\n<channel|>Target identified.\n\nNAME: Shadow of the Alley\nCOST: {1}{B}\nTYPE: Creature — Rogue\nRARITY: Uncommon\nRULES: Menace.\nPT: 2/1";
        let cleaned = clean_llm_output(raw);
        assert!(quality_gate_card(&cleaned));
        assert_eq!(line_value(&cleaned, "NAME:"), Some("Shadow of the Alley"));
    }
}
