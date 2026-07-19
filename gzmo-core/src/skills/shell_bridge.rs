//! Shell skill bridge — runs `skill_dispatch.sh` and wires `ChaosEvent` feedback.
//!
//! Generative slash commands still live as shell scripts. This module closes the
//! autopoietic loop by parsing structured events from stderr (`GZMO_CHAOS_EVENT:`)
//! and falling back to command-aware inference when scripts omit explicit emits.

use std::path::Path;

use anyhow::{bail, Context, Result};
use gzmo_chaos::feedback::ChaosEvent;
use serde_json::Value;
use tokio::process::Command;

pub const EVENT_PREFIX: &str = "GZMO_CHAOS_EVENT:";

/// Result of executing a shell skill through the bridge.
pub struct ShellSkillResult {
    pub display: String,
    pub events: Vec<ChaosEvent>,
    pub success: bool,
}

/// Options for a shell skill invocation.
pub struct ShellSkillOptions<'a> {
    pub skills_dir: &'a Path,
    pub cmd: &'a str,
    pub args: &'a str,
    pub llm_url: Option<String>,
    pub llm_model: Option<String>,
    /// Used when `/stabilize` runs via shell and emits no explicit event.
    pub stabilize_delta_rho: f64,
}

/// Build the OpenAI-compatible completions URL from an engine profile base.
pub fn llm_completions_url(base_url: &str) -> String {
    let mut url = base_url.trim_end_matches('/').to_string();
    if !url.ends_with("/chat/completions") {
        url = format!("{url}/chat/completions");
    }
    url
}

/// Run a shell skill and collect chaos feedback events.
pub async fn run_shell_skill(opts: &ShellSkillOptions<'_>) -> Result<ShellSkillResult> {
    let dispatch = opts.skills_dir.join("skill_dispatch.sh");
    if !dispatch.exists() {
        bail!(
            "skill_dispatch.sh not found in {}",
            opts.skills_dir.display()
        );
    }

    let dispatch_canon = std::fs::canonicalize(&dispatch)
        .with_context(|| format!("canonicalize {}", dispatch.display()))?;
    let base_canon = std::fs::canonicalize(opts.skills_dir)
        .with_context(|| format!("canonicalize {}", opts.skills_dir.display()))?;
    if !dispatch_canon.starts_with(&base_canon) {
        bail!("skill_dispatch.sh escapes skills directory");
    }

    let mut child = Command::new(&dispatch_canon);
    child.arg(opts.cmd);
    if !opts.args.is_empty() {
        child.arg(opts.args);
    }
    child.current_dir(opts.skills_dir.parent().unwrap_or_else(|| Path::new(".")));
    if let Some(url) = &opts.llm_url {
        child.env("GZMO_LLM_URL", url);
    }
    if let Some(model) = &opts.llm_model {
        child.env("GZMO_LLM_MODEL", model);
    }
    child.env("GZMO_CHAOS_SKILL_INNER", "1");

    let output = child
        .output()
        .await
        .with_context(|| format!("execute shell skill /{}", opts.cmd))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let success = output.status.success();

    let mut events = parse_stderr_events(&stderr);
    if events.is_empty() && success {
        events = infer_events(opts.cmd, &stdout, opts.stabilize_delta_rho);
    }

    Ok(ShellSkillResult {
        display: stdout,
        events,
        success,
    })
}

/// Parse `GZMO_CHAOS_EVENT:{json}` lines from shell stderr.
pub fn parse_stderr_events(stderr: &str) -> Vec<ChaosEvent> {
    stderr
        .lines()
        .filter_map(|line| {
            let json = line.strip_prefix(EVENT_PREFIX)?.trim();
            parse_event_json(json)
        })
        .collect()
}

fn parse_event_json(json: &str) -> Option<ChaosEvent> {
    let value: Value = serde_json::from_str(json).ok()?;
    let kind = value.get("type").and_then(Value::as_str)?;

    match kind {
        "JokeGenerated" => {
            let text = value.get("text").and_then(Value::as_str)?.to_string();
            Some(ChaosEvent::JokeGenerated { text })
        }
        "PoemGenerated" => {
            let text = value.get("text").and_then(Value::as_str)?.to_string();
            Some(ChaosEvent::PoemGenerated { text })
        }
        "StoryGenerated" => {
            let text = value.get("text").and_then(Value::as_str)?.to_string();
            Some(ChaosEvent::StoryGenerated { text })
        }
        "CardForged" => {
            let name = value.get("name").and_then(Value::as_str)?.to_string();
            let card_type = value
                .get("card_type")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            Some(ChaosEvent::CardForged { name, card_type })
        }
        "WordGenerated" => {
            let word = value.get("word").and_then(Value::as_str)?.to_string();
            let definition = value
                .get("definition")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Some(ChaosEvent::WordGenerated { word, definition })
        }
        "PersonaShift" => {
            let persona = value.get("persona").and_then(Value::as_str)?.to_string();
            Some(ChaosEvent::PersonaShift { persona })
        }
        "PersonaCleared" => Some(ChaosEvent::PersonaCleared),
        "Stabilize" => {
            let delta_rho = value
                .get("delta_rho")
                .and_then(Value::as_f64)
                .unwrap_or(-1.0);
            Some(ChaosEvent::Stabilize { delta_rho })
        }
        _ => None,
    }
}

/// Fallback mapping when shell scripts have not yet been updated to emit events.
fn infer_events(cmd: &str, stdout: &str, stabilize_delta_rho: f64) -> Vec<ChaosEvent> {
    match cmd {
        "stabilize" => vec![ChaosEvent::Stabilize {
            delta_rho: stabilize_delta_rho,
        }],
        "joke" | "poem" | "story" => {
            if let Some(text) = extract_boxed_content(stdout) {
                let event = match cmd {
                    "joke" => ChaosEvent::JokeGenerated { text },
                    "poem" => ChaosEvent::PoemGenerated { text },
                    "story" => ChaosEvent::StoryGenerated { text },
                    _ => unreachable!(),
                };
                vec![event]
            } else {
                vec![]
            }
        }
        _ => vec![],
    }
}

/// Extract the main text block from a boxed skill display (stdout).
fn extract_boxed_content(stdout: &str) -> Option<String> {
    let mut in_body = false;
    let mut lines: Vec<String> = Vec::new();

    for line in stdout.lines() {
        let stripped = strip_ansi(line).trim().to_string();
        if stripped.starts_with("├") {
            in_body = true;
            continue;
        }
        if stripped.starts_with("└") {
            break;
        }
        if !in_body {
            continue;
        }
        if stripped.is_empty() {
            continue;
        }
        // Skip title lines like "😂 JOKE" or "📖 STORY — seed: ..."
        if stripped
            .chars()
            .any(|c| matches!(c, '😂' | '🖋' | '📖' | '🃏' | '🔤' | '🎭'))
        {
            continue;
        }
        lines.push(stripped);
    }

    let text = lines.join(" ").trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_joke_event_from_stderr() {
        let stderr = r#"some log
GZMO_CHAOS_EVENT:{"type":"JokeGenerated","text":"Why did the attractor cross?"}
"#;
        let events = parse_stderr_events(stderr);
        assert_eq!(events.len(), 1);
        match &events[0] {
            ChaosEvent::JokeGenerated { text } => {
                assert!(text.contains("attractor"));
            }
            other => panic!("expected JokeGenerated, got {other:?}"),
        }
    }

    #[test]
    fn parse_stabilize_event() {
        let stderr = "GZMO_CHAOS_EVENT:{\"type\":\"Stabilize\",\"delta_rho\":-1.0}\n";
        let events = parse_stderr_events(stderr);
        match &events[0] {
            ChaosEvent::Stabilize { delta_rho } => assert!((*delta_rho + 1.0).abs() < f64::EPSILON),
            other => panic!("expected Stabilize, got {other:?}"),
        }
    }

    #[test]
    fn infer_story_from_attractor_fiction_stdout() {
        let stdout = "\n┌─────────────────────────────────────────────────┐\n\
                      │  📖 ATTRACTOR FICTION                          \n\
                      │    keyword: chaos · inv #3 · tick 42           \n\
                      ├─────────────────────────────────────────────────┤\n\
                      \n\
                        The lighthouse keeper counted waves.\n\
                      \n\
                      └─────────────────────────────────────────────────┘\n";
        let events = infer_events("story", stdout, -1.0);
        assert_eq!(events.len(), 1);
        match &events[0] {
            ChaosEvent::StoryGenerated { text } => {
                assert!(text.contains("lighthouse"));
            }
            other => panic!("expected StoryGenerated, got {other:?}"),
        }
    }

    #[test]
    fn infer_story_from_legacy_boxed_stdout() {
        let stdout = "\n┌─────────────────────────────────────────────────┐\n\
                      │  📖 STORY — seed: \"chaos\"                      \n\
                      ├─────────────────────────────────────────────────┤\n\
                      \n\
                        The lighthouse keeper counted waves.\n\
                      \n\
                      └─────────────────────────────────────────────────┘\n";
        let events = infer_events("story", stdout, -1.0);
        assert_eq!(events.len(), 1);
        match &events[0] {
            ChaosEvent::StoryGenerated { text } => {
                assert!(text.contains("lighthouse"));
            }
            other => panic!("expected StoryGenerated, got {other:?}"),
        }
    }

    #[test]
    fn llm_url_appends_completions() {
        assert_eq!(
            llm_completions_url("http://localhost:8000/v1"),
            "http://localhost:8000/v1/chat/completions"
        );
        assert_eq!(
            llm_completions_url("http://localhost:8000/v1/chat/completions"),
            "http://localhost:8000/v1/chat/completions"
        );
    }
}
