//! `gzmo workflow-skill curate` — ACE playbook deltas on workflow `SKILL.md`.
//!
//! Dry-run by default. Writes require `ACE_PIN_APPLY=1` (same shape as
//! `IMMUNE_APPLY=1`). Never mutates `SOUL.md` or living engine toml.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::workflow_skills::{
    curate_workflow_skill, pin_apply_from_env, AceDelta, AceDeltaBatch,
};

const USAGE: &str = "\
Usage:
  gzmo workflow-skill curate --skill <name> --add-rule <text> [--section Rules]
  gzmo workflow-skill curate --skill <name> --update-rule <old> --with <new>
  gzmo workflow-skill curate --skill <name> --remove-rule <text>
  gzmo workflow-skill curate --delta-json <path>

Dry-run prints a unified diff and does not write. To pin:
  ACE_PIN_APPLY=1 gzmo workflow-skill curate --skill <name> --add-rule <text> --apply

ACE curator only mutates skills/workflows/<name>/SKILL.md. SOUL.md is refused.
";

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args.first().map(String::as_str) != Some("curate") {
        eprint!("{USAGE}");
        std::process::exit(2);
    }
    let rest = &args[1..];
    if rest.is_empty()
        || rest
            .iter()
            .any(|a| matches!(a.as_str(), "--help" | "-h" | "help"))
    {
        print!("{USAGE}");
        return Ok(());
    }

    let mut skill_name = String::new();
    let mut section = "Rules".to_string();
    let mut add_rule: Option<String> = None;
    let mut update_target: Option<String> = None;
    let mut update_with: Option<String> = None;
    let mut remove_rule: Option<String> = None;
    let mut delta_json: Option<PathBuf> = None;
    let mut apply_flag = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--skill" => {
                skill_name = rest.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--section" => {
                section = rest.get(i + 1).cloned().unwrap_or_default();
                i += 2;
            }
            "--add-rule" => {
                add_rule = rest.get(i + 1).cloned();
                i += 2;
            }
            "--update-rule" => {
                update_target = rest.get(i + 1).cloned();
                i += 2;
            }
            "--with" => {
                update_with = rest.get(i + 1).cloned();
                i += 2;
            }
            "--remove-rule" => {
                remove_rule = rest.get(i + 1).cloned();
                i += 2;
            }
            "--delta-json" => {
                delta_json = rest.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            "--apply" => {
                apply_flag = true;
                i += 1;
            }
            other => {
                bail!("Unknown argument: {other}\n{USAGE}");
            }
        }
    }

    let batch = if let Some(path) = delta_json {
        if add_rule.is_some() || update_target.is_some() || remove_rule.is_some() {
            bail!(
                "REFUSE: --delta-json cannot be mixed with --add-rule/--update-rule/--remove-rule"
            );
        }
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let batch: AceDeltaBatch = serde_json::from_str(&raw).context("parse ACE delta JSON")?;
        if !skill_name.is_empty() && batch.skill_name != skill_name {
            bail!(
                "REFUSE: --skill {skill_name} does not match JSON skill_name {}",
                batch.skill_name
            );
        }
        batch
    } else {
        if skill_name.is_empty() {
            bail!("REFUSE: --skill is required\n{USAGE}");
        }
        let delta = if let Some(rule) = add_rule {
            AceDelta::AddRule { section, rule }
        } else if let Some(target) = update_target {
            let Some(replacement) = update_with else {
                bail!("REFUSE: --update-rule requires --with <new>");
            };
            AceDelta::UpdateRule {
                target,
                replacement,
            }
        } else if let Some(target) = remove_rule {
            AceDelta::RemoveRule { target }
        } else {
            bail!("REFUSE: one of --add-rule / --update-rule / --remove-rule / --delta-json is required\n{USAGE}");
        };
        AceDeltaBatch {
            skill_name,
            deltas: vec![delta],
            evidence_source: None,
        }
    };

    let pin = pin_apply_from_env();
    if apply_flag && !pin {
        bail!("REFUSE: --apply requires ACE_PIN_APPLY=1 (human pin; same shape as IMMUNE_APPLY)");
    }
    let pin_apply = apply_flag && pin;

    let result = curate_workflow_skill(&config.workflow_skills.dir, &batch, pin_apply)?;
    let mode = if result.wrote {
        "PIN APPLIED"
    } else {
        "DRY RUN"
    };
    println!("=== ACE curate {} ({mode}) ===", result.skill_name);
    println!("path={}", result.path.display());
    if let Some(src) = &batch.evidence_source {
        println!("evidence={src}");
    }
    if !result.changed() {
        println!("No changes.");
        return Ok(());
    }

    print_unified_diff(&result.original, &result.proposed, &result.path)?;
    if result.wrote {
        println!("\n[ACE PIN APPLIED] wrote {}", result.path.display());
    } else {
        println!("\n[DRY RUN] Set ACE_PIN_APPLY=1 and pass --apply to pin this mutation.");
    }
    Ok(())
}

fn print_unified_diff(original: &str, proposed: &str, path: &std::path::Path) -> Result<()> {
    let dir = std::env::temp_dir();
    let a = dir.join(format!("gzmo-ace-a-{}", std::process::id()));
    let b = dir.join(format!("gzmo-ace-b-{}", std::process::id()));
    std::fs::write(&a, original)?;
    std::fs::write(&b, proposed)?;
    let status = Command::new("diff").args(["-u"]).arg(&a).arg(&b).status();
    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) if s.code() == Some(1) => Ok(()), // diff found differences
        Ok(s) => bail!("diff exited {s} for {}", path.display()),
        Err(e) => {
            eprintln!("(diff unavailable: {e})");
            println!("--- original\n{original}\n+++ proposed\n{proposed}");
            Ok(())
        }
    }
}
