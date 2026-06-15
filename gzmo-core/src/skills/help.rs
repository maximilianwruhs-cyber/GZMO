//! # Help Skill — `/help`
//!
//! Lists all available slash commands with descriptions and CCL badges.

use anyhow::Result;
use async_trait::async_trait;

use super::skill_ccl::ChaosCouplingLevel;
use super::{Skill, SkillContext, SkillOutput, SkillType};

const GOLD: &str = "\x1b[38;2;212;175;55m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub struct HelpSkill {
    /// (name, description, type_label, ccl)
    pub entries: Vec<(String, String, &'static str, ChaosCouplingLevel)>,
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
            format!("  {DIM}CCL = Chaos Coupling Level (see docs/SKILL_GOLDEN_STANDARD.md){RESET}"),
            format!("  {DIM}★ = meets generative gold standard (CCL-4){RESET}"),
            format!("  {DIM}◆ = legendary skill (/dice, /card — autopoietic proof + corpus){RESET}"),
            String::new(),
        ];

        let builtins = [
            ("/quit", "Exit GZMO (auto-saves session)"),
            ("/clear", "Clear conversation history"),
            ("/ops", "Toggle ops mode; /ops AUTO toggles daemon autotriggers"),
            ("/discover", "Start/stop Pi mentor discovery (pillar probes + reports)"),
            ("/learn", "Flipped-classroom prep for a topic"),
            ("/chaos", "Display chaos engine state dashboard"),
            ("/stabilize", "Stabilize attractor parameter (decreases rho)"),
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

        for (name, desc, type_label, ccl) in &self.entries {
            let star = ccl.gold_star();
            let legendary = ccl.legendary_mark(name);
            let ccl_color = if *ccl == ChaosCouplingLevel::Ccl4 {
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
                marks.push_str(" LEGENDARY");
            }
            lines.push(format!(
                "  {BOLD}/{name:<14}{RESET} {DIM}[{type_label}]{RESET} {ccl_color}{marks}{RESET}  {desc}",
            ));
        }

        lines.push(String::new());
        lines.push(format!("{GOLD}└─────────────────────────────────────────────────────────────────┘{RESET}"));

        Ok(SkillOutput {
            display: lines.join("\n"),
            inject_to_conversation: false,
            evidence: None,
            feedback: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_entry_includes_ccl4_badge() {
        let skill = HelpSkill {
            entries: vec![(
                "story".to_string(),
                "Attractor Fiction".to_string(),
                "generative",
                ChaosCouplingLevel::Ccl4,
            )],
        };
        assert_eq!(skill.entries[0].3.badge(), "CCL-4");
        assert_eq!(skill.entries[0].3.gold_star(), "★");
    }
}
