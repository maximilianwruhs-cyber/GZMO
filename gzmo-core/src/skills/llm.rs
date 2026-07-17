//! Shared LLM infrastructure for generative slash skills.
//! Mirrors `skills/_llm_helper.sh` — state files, prompts, API calls.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gzmo_chaos::pulse::ChaosSnapshot;
use reqwest::Client;
use serde_json::json;

use crate::config::GzmoConfig;

// ─── ANSI palette (matches shell helpers) ───────────────────────────

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[97m";

/// Runtime context shared by LLM-backed skills (constructed once at registration).
#[derive(Debug, Clone)]
pub struct SkillRuntime {
    pub skills_dir: PathBuf,
    pub llm_url: String,
    pub llm_model: String,
    pub stabilize_delta_rho: f64,
}

impl SkillRuntime {
    pub fn from_config(config: &GzmoConfig) -> Self {
        let active = config.engine.active_engine();
        let llm_url = std::env::var("GZMO_LLM_URL")
            .unwrap_or_else(|_| active.url.trim_end_matches('/').to_string());
        let llm_model = std::env::var("GZMO_LLM_MODEL")
            .unwrap_or_else(|_| active.model.clone());

        let stabilize_delta_rho = config
            .chaos
            .as_ref()
            .and_then(|v| v.clone().try_into().ok())
            .map(|c: gzmo_chaos::pulse::ChaosConfig| c.stabilize_delta_rho)
            .unwrap_or(-1.0);

        Self {
            skills_dir: resolve_skills_dir(config),
            llm_url,
            llm_model,
            stabilize_delta_rho,
        }
    }

    pub fn lang_state_path(&self) -> PathBuf {
        self.skills_dir.join(".language")
    }

    pub fn transform_state_path(&self) -> PathBuf {
        self.skills_dir.join(".transform_persona")
    }

    pub fn characters_path(&self) -> PathBuf {
        self.skills_dir.join("characters.toml")
    }

    pub fn cardforge_path(&self) -> PathBuf {
        self.skills_dir.join("cardforge.toml")
    }
}

/// Resolve skills directory: env `GZMO_SKILLS_DIR`, else config, else `./skills`.
pub fn resolve_skills_dir(config: &GzmoConfig) -> PathBuf {
    if let Ok(dir) = std::env::var("GZMO_SKILLS_DIR") {
        return PathBuf::from(dir);
    }
    if config.skills.directory.exists() {
        return config.skills.directory.clone();
    }
    PathBuf::from("skills")
}

pub fn get_language(skills_dir: &Path) -> String {
    let path = skills_dir.join(".language");
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "en".to_string())
}

pub fn get_persona_prompt(skills_dir: &Path) -> String {
    let path = skills_dir.join(".transform_persona");
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Inject active language + transform persona into a base system prompt.
pub fn build_system_prompt(base: &str, skills_dir: &Path, structured: bool) -> String {
    if structured || std::env::var("GZMO_SKILL_STRUCTURED").ok().as_deref() == Some("1") {
        return base.to_string();
    }

    let mut full = base.to_string();
    let lang = get_language(skills_dir);
    if lang != "en" {
        full.push_str(&format!(
            "\n\nCRITICAL: You MUST respond entirely in language code: {lang}. Do not use English."
        ));
    }

    let persona = get_persona_prompt(skills_dir);
    if !persona.trim().is_empty() {
        full.push_str("\n\nCHARACTER TRANSFORM ACTIVE:\n");
        full.push_str(persona.trim());
        full.push_str(
            "\nYou MUST adopt this character's speech patterns, vocabulary, and personality in your response.",
        );
    }

    full
}

/// Case-insensitive ASCII tag search that only returns char-boundary offsets.
/// Avoids the old byte-walk that panicked on UTF-8 (e.g. German `ä` in joke/poem output).
fn find_ascii_ci(hay: &str, needle: &str) -> Option<usize> {
    let hb = hay.as_bytes();
    let nb = needle.as_bytes();
    if nb.is_empty() || hb.len() < nb.len() {
        return None;
    }
    'outer: for i in 0..=(hb.len() - nb.len()) {
        if !hay.is_char_boundary(i) {
            continue;
        }
        for (j, &nc) in nb.iter().enumerate() {
            if hb[i + j].to_ascii_lowercase() != nc {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

pub fn strip_thinking_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = find_ascii_ci(rest, "<think") {
        out.push_str(&rest[..start]);
        let from_tag = &rest[start..];
        let Some(open_gt) = from_tag.find('>') else {
            // Malformed open tag — keep remainder and stop.
            out.push_str(from_tag);
            return out.trim().to_string();
        };
        let after_open = &from_tag[open_gt + 1..];
        if let Some(close) = find_ascii_ci(after_open, "</think") {
            let from_close = &after_open[close..];
            if let Some(close_gt) = from_close.find('>') {
                rest = &from_close[close_gt + 1..];
                continue;
            }
        }
        // No closing tag: drop the open tag body (same as skipping past `>`).
        rest = after_open;
    }
    out.push_str(rest);
    out.trim().to_string()
}

pub fn clean_llm_output(text: &str) -> String {
    let mut s = strip_thinking_tags(text);
    s = s.replace('`', "");
    for _ in 0..2 {
        s = s.trim().trim_matches('"').trim_matches('\'').to_string();
    }
    s
}

fn llm_api_base(url: &str) -> String {
    let base = url.trim_end_matches('/');
    let base = base.strip_suffix("/v1").unwrap_or(base);
    format!("{base}/v1")
}

pub async fn llm_chat(
    rt: &SkillRuntime,
    system: &str,
    user: &str,
    temp: f64,
    max_tokens: u32,
    structured: bool,
) -> Result<String> {
    let full_system = build_system_prompt(system, &rt.skills_dir, structured);
    let url = format!("{}/chat/completions", llm_api_base(&rt.llm_url));

    let payload = json!({
        "model": rt.llm_model,
        "messages": [
            {"role": "system", "content": full_system},
            {"role": "user", "content": user},
        ],
        "temperature": temp,
        "max_tokens": max_tokens,
        "stream": false,
    });

    let client = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("LLM request failed")?;

    let body: serde_json::Value = response.json().await.context("LLM response parse failed")?;
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| body["choices"][0]["message"]["reasoning_content"].as_str())
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        anyhow::bail!("LLM returned empty content");
    }

    Ok(clean_llm_output(&content))
}

pub async fn llm_available(rt: &SkillRuntime) -> bool {
    let url = format!("{}/models", llm_api_base(&rt.llm_url));
    let Ok(client) = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(2))
        .timeout(std::time::Duration::from_secs(3))
        .build()
    else {
        return false;
    };
    client
        .get(url)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

pub fn frame_box(title: &str, body: &str, emoji: &str, title_color: &str) -> String {
    format!(
        "\n{DIM}┌─────────────────────────────────────────────────┐{RESET}\n\
         {BOLD}{title_color}  {emoji} {title}{RESET}\n\
         {DIM}├─────────────────────────────────────────────────┤{RESET}\n\n\
         {body}\n\n\
         {DIM}└─────────────────────────────────────────────────┘{RESET}\n",
    )
}

pub fn chaos_index(snap: &ChaosSnapshot, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let combined = snap.chaos_val * 10000.0 + snap.x.abs() * 100.0 + snap.y.abs() * 10.0;
    (combined.fract() * n as f64).floor() as usize % n
}

pub fn char_count(s: &str) -> usize {
    s.chars().count()
}

pub fn quality_gate_poem(text: &str) -> bool {
    let lower = text.to_lowercase();
    if [
        "seele", "schicksal", "ewigkeit", "tränen", "tranen", "schatten", "flüstern",
        "flustern", " soul ", " fate ", " eternity ", " whisper ", " shadows ", " tears ",
        " dance ",
    ]
    .iter()
    .any(|p| lower.contains(p))
    {
        return false;
    }
    !(lower.contains("herz") && lower.contains("schmerz"))
}

pub fn quality_gate_joke(text: &str) -> bool {
    let lower = text.to_lowercase();
    ![
        "programmier", "programming bug", " coffee ", " kaffee ", "artificial intelligence",
        " chatgpt ", " openai ", " claude ", " deepseek ",
    ]
    .iter()
    .any(|p| lower.contains(p))
}

pub fn quality_gate_story(text: &str) -> bool {
    let lower = text.to_lowercase();
    ![
        "once upon a time", "happily ever after", "fairy tale", "and they lived",
        "es war einmal", "leben lang glücklich",
    ]
    .iter()
    .any(|p| lower.contains(p))
}

pub fn accept_creative_output(text: &str, max_chars: usize, gate: fn(&str) -> bool) -> bool {
    let count = char_count(text);
    if count < 10 || count > max_chars {
        return false;
    }
    gate(text)
}

pub fn fold_lines(text: &str, width: usize) -> String {
    let mut out = String::new();
    for word in text.split_whitespace() {
        if out.is_empty() {
            out.push_str(word);
        } else if out.len() + 1 + word.len() > width {
            out.push('\n');
            out.push_str(word);
        } else {
            out.push(' ');
            out.push_str(word);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_thinking_preserves_umlauts() {
        let s = "Käse und Öl — wunderbar";
        assert_eq!(strip_thinking_tags(s), s);
    }

    #[test]
    fn strip_thinking_removes_think_blocks_around_utf8() {
        let s = "Hallo <think>secret äöü</think> Welt — Größe";
        assert_eq!(strip_thinking_tags(s), "Hallo  Welt — Größe");
    }

    #[test]
    fn clean_llm_output_with_german_joke_does_not_panic() {
        let raw = "<think>planning</think>Der Optiker fragte, ob ich die Brille zum Überleben brauche.";
        let cleaned = clean_llm_output(raw);
        assert!(cleaned.contains("Überleben"));
        assert!(!cleaned.contains("<think"));
    }
}
