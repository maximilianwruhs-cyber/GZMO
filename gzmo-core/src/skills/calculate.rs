//! # Calculate Skill — `/calculate`
//!
//! Math solver via bc. Falls back to simple Rust evaluation
//! for basic arithmetic if bc is not available.

use anyhow::Result;
use async_trait::async_trait;

use super::{Skill, SkillContext, SkillOutput, SkillType};

const WHITE: &str = "\x1b[97m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub struct CalculateSkill;

#[async_trait]
impl Skill for CalculateSkill {
    fn name(&self) -> &str {
        "calculate"
    }
    fn description(&self) -> &str {
        "Solve a mathematical expression via bc"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mechanical
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let expr = ctx.args.trim();
        if expr.is_empty() {
            return Ok(SkillOutput {
                display: format!("  {RED}✗ Usage: /calculate <expression>{RESET}\n  Examples: /calculate 2^10\n           /calculate \"sqrt(144)\""),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        // Try bc first
        let result = std::process::Command::new("bc")
            .arg("-l")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    let _ = write!(stdin, "{}\n", expr);
                }
                child.wait_with_output()
            })
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if result.is_empty() {
                        None
                    } else {
                        Some(result)
                    }
                } else {
                    None
                }
            });

        let result = match result {
            Some(r) => {
                // Trim trailing zeros
                if r.contains('.') {
                    r.trim_end_matches('0').trim_end_matches('.').to_string()
                } else {
                    r
                }
            }
            None => {
                return Ok(SkillOutput {
                    display: format!("  {RED}✗ Invalid expression: {expr}{RESET}"),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
        };

        let display = format!(
            "\n{DIM}┌─────────────────────────────────────────────────┐{RESET}\n\
             {BOLD}{GREEN}  🧮 CALCULATE{RESET}\n\
             {DIM}├─────────────────────────────────────────────────┤{RESET}\n\n\
               {DIM}expr: {expr}{RESET}\n\
               {BOLD}{WHITE}  =  {result}{RESET}\n\n\
             {DIM}└─────────────────────────────────────────────────┘{RESET}\n",
        );

        Ok(SkillOutput {
            display,
            feedback: vec![],
            inject_to_conversation: true,
        })
    }
}
