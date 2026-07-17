//! # Help Skill — `/help`
//!
//! Lists all available slash commands with descriptions.
//! Shows both Rust-native and shell-based skills.

use anyhow::Result;
use async_trait::async_trait;

use super::{Skill, SkillContext, SkillOutput, SkillType};

const GOLD: &str = "\x1b[38;2;212;175;55m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub struct HelpSkill {
    /// Descriptions of all registered skills (populated at registration time)
    pub entries: Vec<(String, String, &'static str)>, // (name, description, type_label)
}

#[async_trait]
impl Skill for HelpSkill {
    fn name(&self) -> &str { "help" }
    fn description(&self) -> &str { "Display all available slash commands" }
    fn skill_type(&self) -> SkillType { SkillType::Info }

    async fn execute(&self, _ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let mut lines = vec![
            format!("\n{GOLD}┌─────────────────────────────────────────────────────────────────┐{RESET}"),
            format!("{BOLD}{CYAN}  ❓ GZMO SKILL REGISTRY — All Available Commands{RESET}"),
            format!("{GOLD}├─────────────────────────────────────────────────────────────────┤{RESET}"),
            String::new(),
        ];

        // Built-in commands
        let builtins = [
            ("/quit",    "Exit GZMO (auto-saves session)"),
            ("/status",  "Ecosystem snapshot (paths, systemd, probes — no LLM)"),
            ("/ecosystem","Alias for /status"),
            ("/clear",   "Clear conversation history"),
            ("/chaos",   "Display chaos engine state dashboard"),
            ("/stats",   "Show session statistics"),
            ("/vault",   "Search semantic vault memory"),
            ("/remember","Store a fact in semantic vault"),
            ("/sessions","List saved sessions"),
            ("/resume",  "Resume most recent session"),
        ];

        for (cmd, desc) in &builtins {
            lines.push(format!("  {DIM}{cmd:<16}{RESET} {desc}"));
        }

        lines.push(String::new());
        lines.push(format!("{GOLD}├─ Chaos Skills (Rust) ────────────────────────────────────────────┤{RESET}"));
        lines.push(String::new());

        for (name, desc, type_label) in &self.entries {
            lines.push(format!("  {BOLD}/{name:<15}{RESET} {DIM}[{type_label}]{RESET}  {desc}"));
        }

        lines.push(String::new());
        lines.push(format!("{GOLD}└─────────────────────────────────────────────────────────────────┘{RESET}"));

        let display = lines.join("\n");

        Ok(SkillOutput {
            display,
            feedback: vec![],
            inject_to_conversation: false,
        })
    }
}
