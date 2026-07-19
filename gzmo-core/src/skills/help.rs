//! # Help Skill — `/help`
//!
//! Lists all available slash commands with descriptions.
//! Shows both Rust-native and shell-based skills.

use anyhow::Result;
use async_trait::async_trait;

use super::skill_ccl::{ccl_for_skill, ChaosCouplingLevel};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const GOLD: &str = "\x1b[38;2;212;175;55m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub struct HelpSkill {
    /// Descriptions of all registered skills (populated at registration time)
    pub entries: Vec<(String, String, &'static str)>, // (name, description, type_label)
}

#[async_trait]
impl Skill for HelpSkill {
    fn name(&self) -> &str {
        "help"
    }
    fn description(&self) -> &str {
        "Display all available slash commands"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Info
    }

    async fn execute(&self, _ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let mut lines = vec![
            format!("\n{GOLD}┌─────────────────────────────────────────────────────────────────┐{RESET}"),
            format!("{BOLD}{CYAN}  ❓ GZMO SKILL REGISTRY — All Available Commands{RESET}"),
            format!("{GOLD}├─────────────────────────────────────────────────────────────────┤{RESET}"),
            String::new(),
            format!("{DIM}  CCL-4 ★ = autopoietic generative{RESET}"),
        ];

        // Built-in commands
        let builtins = [
            ("/quit", "Exit GZMO (auto-saves session)"),
            (
                "/status",
                "Ecosystem snapshot (paths, systemd, probes — no LLM)",
            ),
            ("/ecosystem", "Alias for /status"),
            ("/clear", "Clear conversation history"),
            ("/chaos", "Display chaos engine state dashboard"),
            ("/stats", "Show session statistics"),
            ("/vault", "Search semantic vault memory"),
            ("/remember", "Store a fact in semantic vault"),
            ("/sessions", "List saved sessions"),
            ("/resume", "Resume most recent session"),
        ];

        for (cmd, desc) in &builtins {
            lines.push(format!("  {DIM}{cmd:<16}{RESET} {desc}"));
        }

        lines.push(String::new());
        lines.push(format!(
            "{GOLD}├─ Chaos Skills (Rust) — type · CCL ─────────────────────────────────┤{RESET}"
        ));
        lines.push(String::new());

        for (name, desc, type_label) in &self.entries {
            let ccl = ccl_for_skill(name);
            let star = ccl.gold_star();
            let legendary = ccl.legendary_mark(name);
            let ccl_color = if ccl == ChaosCouplingLevel::Ccl4 {
                GREEN
            } else {
                DIM
            };
            let mut marks = ccl.badge().to_string();
            if !star.is_empty() {
                marks.push(' ');
                marks.push_str(star);
            }
            if !legendary.is_empty() {
                marks.push(' ');
                marks.push_str(legendary);
            }
            lines.push(format!(
                "  {BOLD}/{name:<14}{RESET} {DIM}[{type_label}]{RESET} {ccl_color}{marks}{RESET}  {desc}"
            ));
        }

        lines.push(String::new());
        lines.push(format!(
            "{GOLD}└─────────────────────────────────────────────────────────────────┘{RESET}"
        ));

        let display = lines.join("\n");

        Ok(SkillOutput {
            display,
            feedback: vec![],
            inject_to_conversation: false,
            evidence: None,
        })
    }
}
