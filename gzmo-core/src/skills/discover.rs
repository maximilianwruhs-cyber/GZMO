//! `/discover` — start or stop Pi ↔ mentor infrastructure discovery sessions.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::config::PedagogyConfig;

use super::generative::boxed_display;
use super::{Skill, SkillContext, SkillOutput, SkillType};

pub struct DiscoverSkill {
    pub pedagogy_config: PedagogyConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscoverSubcommand {
    Start,
    Stop,
    Status,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscoverStartArgs {
    size: Option<String>,
    pillar: Option<char>,
}

fn strip_flags(args: &str) -> Vec<&str> {
    args.split_whitespace()
        .filter(|t| *t != "--json")
        .collect()
}

fn is_size_token(s: &str) -> bool {
    matches!(s, "small" | "s" | "medium" | "m" | "large" | "l")
}

fn is_pillar_token(s: &str) -> bool {
    s.len() == 1 && matches!(s.chars().next(), Some('S' | 'A' | 'B' | 'C'))
}

fn parse_subcommand(args: &str) -> DiscoverSubcommand {
    let tokens = strip_flags(args);
    match tokens.first().map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("stop") => DiscoverSubcommand::Stop,
        Some("status") => DiscoverSubcommand::Status,
        None | Some("") | Some("start") => DiscoverSubcommand::Start,
        Some(first) if is_size_token(first) || is_pillar_token(first) => DiscoverSubcommand::Start,
        _ => DiscoverSubcommand::Unknown,
    }
}

fn normalize_size_token(token: &str) -> Option<String> {
    match token.to_ascii_lowercase().as_str() {
        "small" | "s" => Some("small".into()),
        "medium" | "m" => Some("medium".into()),
        "large" | "l" => Some("large".into()),
        _ => None,
    }
}

fn parse_start_args(args: &str) -> DiscoverStartArgs {
    let tokens = strip_flags(args);
    let rest: &[&str] = if tokens
        .first()
        .is_some_and(|t| t.eq_ignore_ascii_case("start"))
    {
        &tokens[1..]
    } else {
        &tokens
    };
    let mut size = None;
    let mut pillar = None;
    for token in rest {
        if token.len() == 1 {
            if let Some(p) = token.chars().next() {
                if matches!(p, 'S' | 'A' | 'B' | 'C') {
                    pillar = Some(p);
                    continue;
                }
            }
        }
        if let Some(s) = normalize_size_token(token) {
            size = Some(s);
        }
    }
    DiscoverStartArgs { size, pillar }
}

fn scripts_root(cfg: &PedagogyConfig) -> PathBuf {
    PathBuf::from(&cfg.discovery_scripts_root)
}

fn script_path(root: &Path, name: &str) -> PathBuf {
    root.join("scripts").join(name)
}

async fn run_discovery_script(
    root: &Path,
    script_name: &str,
    args: &[&str],
    pillar: Option<char>,
) -> Result<(i32, String)> {
    let script = script_path(root, script_name);
    if !script.is_file() {
        anyhow::bail!(
            "discovery script not found: {} (check pedagogy.discovery_scripts_root)",
            script.display()
        );
    }

    let mut cmd = tokio::process::Command::new(&script);
    cmd.args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(p) = pillar {
        cmd.env("DISCOVERY_PILLAR_FORCE", p.to_string());
    }
    if let Ok(gzmo_root) = std::env::var("GZMO_ROOT") {
        cmd.env("GZMO_ROOT", gzmo_root);
    }

    let output = cmd
        .output()
        .await
        .with_context(|| format!("run {}", script.display()))?;

    let mut combined = String::new();
    if !output.stdout.is_empty() {
        combined.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    Ok((output.status.code().unwrap_or(1), combined.trim().to_string()))
}

fn latest_report_path(root: &Path) -> Option<String> {
    let latest = root.join("data/pi-mentor-discovery/latest.md");
    if let Ok(target) = std::fs::read_link(&latest) {
        return Some(target.to_string_lossy().into_owned());
    }
    latest.is_file().then(|| latest.to_string_lossy().into_owned())
}

#[async_trait]
impl Skill for DiscoverSkill {
    fn name(&self) -> &str {
        "discover"
    }

    fn description(&self) -> &str {
        "Start or stop Pi mentor discovery (pillar probes + cycle reports)"
    }

    fn skill_type(&self) -> SkillType {
        SkillType::Mutation
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        match parse_subcommand(ctx.args) {
            DiscoverSubcommand::Start => self.start(ctx.args).await,
            DiscoverSubcommand::Stop => self.stop().await,
            DiscoverSubcommand::Status => self.status().await,
            DiscoverSubcommand::Unknown => Ok(usage_output()),
        }
    }
}

impl DiscoverSkill {
    async fn start(&self, args: &str) -> Result<SkillOutput> {
        let root = scripts_root(&self.pedagogy_config);
        let start_args = parse_start_args(args);
        let mut script_args: Vec<&str> = Vec::new();
        if let Some(ref size) = start_args.size {
            script_args.push(size.as_str());
        }

        let (code, output) = run_discovery_script(
            &root,
            "start-pi-mentor-discovery-session.sh",
            &script_args,
            start_args.pillar,
        )
        .await?;

        let body = if output.is_empty() {
            if code == 0 {
                "Discovery session started.".into()
            } else {
                format!("Discovery start failed (exit {code}).")
            }
        } else {
            output
        };

        let state_path = root.join("data/pi-mentor-discovery/state.json");
        let session_id = std::fs::read_to_string(&state_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|v| v.get("session_id").and_then(|s| s.as_str()).map(String::from));

        Ok(SkillOutput {
            display: boxed_display(
                if code == 0 {
                    "DISCOVERY START"
                } else {
                    "DISCOVERY ERROR"
                },
                "🔭",
                &body,
            ),
            feedback: vec![],
            inject_to_conversation: true,
            evidence: Some(serde_json::json!({
                "skill": "discover",
                "subcommand": "start",
                "exit_code": code,
                "session_id": session_id,
                "size": start_args.size,
                "pillar": start_args.pillar.map(|c| c.to_string()),
                "scripts_root": root,
            })),
        })
    }

    async fn stop(&self) -> Result<SkillOutput> {
        let root = scripts_root(&self.pedagogy_config);
        let (code, output) =
            run_discovery_script(&root, "stop-pi-mentor-discovery-session.sh", &[], None).await?;

        let body = if output.is_empty() {
            if code == 0 {
                "Discovery timer stopped (no final report).".into()
            } else {
                format!("Discovery stop failed (exit {code}).")
            }
        } else {
            output
        };

        Ok(SkillOutput {
            display: boxed_display(
                "DISCOVERY STOP",
                "🛑",
                &body,
            ),
            feedback: vec![],
            inject_to_conversation: true,
            evidence: Some(serde_json::json!({
                "skill": "discover",
                "subcommand": "stop",
                "exit_code": code,
                "last_report": latest_report_path(&root),
                "scripts_root": root,
            })),
        })
    }

    async fn status(&self) -> Result<SkillOutput> {
        let root = scripts_root(&self.pedagogy_config);
        let (code, output) =
            run_discovery_script(&root, "pi-mentor-discovery-status.sh", &[], None).await?;

        Ok(SkillOutput {
            display: boxed_display(
                "DISCOVERY STATUS",
                "🔭",
                &output,
            ),
            feedback: vec![],
            inject_to_conversation: false,
            evidence: Some(serde_json::json!({
                "skill": "discover",
                "subcommand": "status",
                "exit_code": code,
                "output": output,
                "scripts_root": root,
            })),
        })
    }
}

fn usage_output() -> SkillOutput {
    SkillOutput {
        display: boxed_display(
            "DISCOVER",
            "🔭",
            "Usage:\n  /discover              — start discovery session (config size)\n  /discover start small  — 5 min session\n  /discover start medium — 15 min\n  /discover start large  — 60 min\n  /discover start S      — force pillar S/A/B/C\n  /discover status       — show current session status\n  /discover stop         — stop timer early (no final report)",
        ),
        feedback: vec![],
        inject_to_conversation: false,
        evidence: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_start_and_stop() {
        assert_eq!(parse_subcommand(""), DiscoverSubcommand::Start);
        assert_eq!(parse_subcommand("start small"), DiscoverSubcommand::Start);
        assert_eq!(parse_subcommand("medium"), DiscoverSubcommand::Start);
        assert_eq!(parse_subcommand("stop"), DiscoverSubcommand::Stop);
        assert_eq!(parse_subcommand("status"), DiscoverSubcommand::Status);
        assert_eq!(parse_subcommand("pause"), DiscoverSubcommand::Unknown);
    }

    #[test]
    fn parse_start_size_and_pillar() {
        let a = parse_start_args("start small S");
        assert_eq!(a.size.as_deref(), Some("small"));
        assert_eq!(a.pillar, Some('S'));
        let b = parse_start_args("medium B");
        assert_eq!(b.size.as_deref(), Some("medium"));
        assert_eq!(b.pillar, Some('B'));
    }
}
