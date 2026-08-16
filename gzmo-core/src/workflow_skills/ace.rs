//! ACE (Agentic Context Engineering) playbook curator.
//!
//! Literature: Stanford / SambaNova / Berkeley (arXiv:2510.04618, ICLR 2026).
//!
//! Deterministic `ADD` / `UPDATE` / `REMOVE` on workflow `SKILL.md` bodies.
//! The curator never LLM-rewrites a playbook. Living `SOUL.md` / engine toml
//! are out of bounds. Writes require an explicit pin at the call site
//! (`ACE_PIN_APPLY=1` on the CLI / script).

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// A single atomic playbook delta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AceDelta {
    /// Add a new rule or item under a specific section header (e.g. "Rules").
    AddRule { section: String, rule: String },
    /// Update an existing rule matching `target` substring with `replacement`.
    UpdateRule { target: String, replacement: String },
    /// Remove an obsolete or harmful rule matching `target` substring.
    RemoveRule { target: String },
}

/// A batch of ACE deltas targeting one workflow skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AceDeltaBatch {
    pub skill_name: String,
    pub deltas: Vec<AceDelta>,
    #[serde(default)]
    pub evidence_source: Option<String>,
}

/// Preview / apply result. `wrote` is true only when the caller passed `pin_apply`.
#[derive(Debug, Clone)]
pub struct AceCurateResult {
    pub skill_name: String,
    pub path: PathBuf,
    pub original: String,
    pub proposed: String,
    pub wrote: bool,
}

impl AceCurateResult {
    pub fn changed(&self) -> bool {
        self.original != self.proposed
    }
}

/// Apply a sequence of ACE deltas deterministically to raw `SKILL.md` markdown.
///
/// YAML frontmatter (bytes before and including the closing `---` fence) is
/// preserved exactly. This function does not write files and does not pin.
pub fn apply_ace_deltas(raw_markdown: &str, deltas: &[AceDelta]) -> Result<String> {
    if deltas.is_empty() {
        return Ok(raw_markdown.to_string());
    }

    let (prefix, body) = split_frontmatter_prefix_and_body(raw_markdown);
    let mut body = body.to_string();

    for delta in deltas {
        match delta {
            AceDelta::AddRule { section, rule } => {
                body = apply_add_rule(&body, section, rule)?;
            }
            AceDelta::UpdateRule {
                target,
                replacement,
            } => {
                body = apply_update_rule(&body, target, replacement)?;
            }
            AceDelta::RemoveRule { target } => {
                body = apply_remove_rule(&body, target)?;
            }
        }
    }

    Ok(format!("{prefix}{body}"))
}

/// Resolve, preview, and optionally write deltas onto `workflows_dir/<name>/SKILL.md`.
///
/// `pin_apply = false` never writes. `pin_apply = true` writes only when the
/// proposed text differs. Paths outside the workflow dir, `SOUL.md`, and
/// engine toml are refused.
pub fn curate_workflow_skill(
    workflows_dir: &Path,
    batch: &AceDeltaBatch,
    pin_apply: bool,
) -> Result<AceCurateResult> {
    let path = resolve_workflow_skill_path(workflows_dir, &batch.skill_name)?;
    let original =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let proposed = apply_ace_deltas(&original, &batch.deltas)?;
    let mut wrote = false;
    if pin_apply && proposed != original {
        let tmp = path.with_extension("md.ace-tmp");
        std::fs::write(&tmp, &proposed).with_context(|| format!("write {}", tmp.display()))?;
        std::fs::rename(&tmp, &path)
            .with_context(|| format!("rename {} → {}", tmp.display(), path.display()))?;
        wrote = true;
    }
    Ok(AceCurateResult {
        skill_name: batch.skill_name.clone(),
        path,
        original,
        proposed,
        wrote,
    })
}

pub fn pin_apply_from_env() -> bool {
    std::env::var("ACE_PIN_APPLY").ok().as_deref() == Some("1")
}

pub fn validate_skill_name(name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        bail!("REFUSE: skill name is empty");
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!("REFUSE: skill name must be a single path segment (got {name})");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("REFUSE: skill name must match [a-z0-9-]+ (got {name})");
    }
    if name == "soul" {
        bail!("REFUSE: ACE curator does not mutate SOUL.md");
    }
    Ok(())
}

fn resolve_workflow_skill_path(workflows_dir: &Path, name: &str) -> Result<PathBuf> {
    validate_skill_name(name)?;
    let workflows = workflows_dir
        .canonicalize()
        .with_context(|| format!("canonicalize workflow dir {}", workflows_dir.display()))?;
    let path = workflows.join(name).join("SKILL.md");
    let canon = path
        .canonicalize()
        .with_context(|| format!("canonicalize {}", path.display()))?;
    if !canon.starts_with(&workflows) {
        bail!(
            "REFUSE: skill path escapes workflow dir ({})",
            canon.display()
        );
    }
    if canon.file_name().and_then(|s| s.to_str()) != Some("SKILL.md") {
        bail!("REFUSE: ACE curator only mutates workflow SKILL.md");
    }
    Ok(canon)
}

/// Prefix includes opening `---`, YAML, and the closing `---` fence.
fn split_frontmatter_prefix_and_body(raw: &str) -> (&str, &str) {
    if raw.starts_with("---") {
        let search_from = if raw.starts_with("---\n") { 4 } else { 3 };
        if let Some(rel) = raw[search_from..].find("\n---") {
            let after_fence = search_from + rel + 4;
            return (&raw[..after_fence], &raw[after_fence..]);
        }
    }
    ("", raw)
}

fn apply_add_rule(body: &str, section: &str, rule: &str) -> Result<String> {
    let section = section.trim();
    let rule = rule.trim();
    if section.is_empty() {
        bail!("ACE add section is empty");
    }
    if rule.is_empty() {
        bail!("ACE add rule is empty");
    }

    let header = format!("## {section}");
    let lines: Vec<&str> = body.lines().collect();
    let header_idx = lines
        .iter()
        .position(|l| l.trim().eq_ignore_ascii_case(&header));

    match header_idx {
        None => {
            let formatted = format_new_item(&[], rule);
            let mut out = body.trim_end().to_string();
            if !out.is_empty() {
                out.push('\n');
            }
            out.push('\n');
            out.push_str(&header);
            out.push('\n');
            out.push('\n');
            out.push_str(&formatted);
            out.push('\n');
            Ok(out)
        }
        Some(idx) => {
            let end = section_end(&lines, idx);
            let section_lines = &lines[idx + 1..end];
            if section_contains_rule(section_lines, rule) {
                return Ok(restore_trailing_newline(body, &lines.join("\n")));
            }
            let formatted = format_new_item(section_lines, rule);
            let mut out: Vec<String> = lines[..end].iter().map(|s| (*s).to_string()).collect();
            while out.last().is_some_and(|l| l.trim().is_empty()) {
                out.pop();
            }
            out.push(formatted);
            out.extend(lines[end..].iter().map(|s| (*s).to_string()));
            Ok(restore_trailing_newline(body, &out.join("\n")))
        }
    }
}

fn section_end(lines: &[&str], header_idx: usize) -> usize {
    lines
        .iter()
        .enumerate()
        .skip(header_idx + 1)
        .find(|(_, l)| is_h2(l))
        .map(|(i, _)| i)
        .unwrap_or(lines.len())
}

fn is_h2(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("## ") && !t.starts_with("###")
}

fn section_contains_rule(section_lines: &[&str], rule: &str) -> bool {
    let needle = rule.trim();
    section_lines.iter().any(|l| l.contains(needle))
}

fn format_new_item(section_lines: &[&str], rule: &str) -> String {
    let rule = rule.trim();
    if looks_preformatted(rule) {
        return rule.to_string();
    }
    let next_n = section_lines.iter().filter_map(|l| numbered_item(l)).max();
    if let Some(n) = next_n {
        format!("{}. {rule}", n + 1)
    } else {
        format!("- {rule}")
    }
}

fn looks_preformatted(rule: &str) -> bool {
    let t = rule.trim();
    t.starts_with("- ") || t.starts_with("* ") || numbered_item(t).is_some()
}

fn numbered_item(line: &str) -> Option<u32> {
    let t = line.trim();
    let dot = t.find('.')?;
    if dot == 0 {
        return None;
    }
    let n = t[..dot].parse::<u32>().ok()?;
    if t[dot + 1..].starts_with(' ') || t.len() == dot + 1 {
        Some(n)
    } else {
        None
    }
}

fn apply_update_rule(body: &str, target: &str, replacement: &str) -> Result<String> {
    let target = target.trim();
    let replacement = replacement.trim();
    if target.is_empty() {
        bail!("ACE update target is empty");
    }
    if replacement.is_empty() {
        bail!("ACE update replacement is empty");
    }
    let lines: Vec<&str> = body.lines().collect();
    let hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains(target))
        .map(|(i, _)| i)
        .collect();
    match hits.as_slice() {
        [] => bail!("ACE update target not found in playbook: '{target}'"),
        [i] => {
            let new_line = if looks_preformatted(replacement) {
                replacement.to_string()
            } else {
                lines[*i].replacen(target, replacement, 1)
            };
            let mut out: Vec<String> = lines.iter().map(|s| (*s).to_string()).collect();
            out[*i] = new_line;
            Ok(restore_trailing_newline(body, &out.join("\n")))
        }
        _ => bail!(
            "ACE update target is ambiguous ({} matching lines): '{target}'",
            hits.len()
        ),
    }
}

fn apply_remove_rule(body: &str, target: &str) -> Result<String> {
    let target = target.trim();
    if target.is_empty() {
        bail!("ACE remove target is empty");
    }
    let lines: Vec<&str> = body.lines().collect();
    let hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains(target))
        .map(|(i, _)| i)
        .collect();
    match hits.as_slice() {
        [] => bail!("ACE remove target not found in playbook: '{target}'"),
        [i] => {
            let mut out: Vec<String> = Vec::with_capacity(lines.len().saturating_sub(1));
            for (idx, line) in lines.iter().enumerate() {
                if idx == *i {
                    continue;
                }
                out.push((*line).to_string());
            }
            Ok(restore_trailing_newline(body, &out.join("\n")))
        }
        _ => bail!(
            "ACE remove target is ambiguous ({} matching lines): '{target}'",
            hits.len()
        ),
    }
}

fn restore_trailing_newline(original: &str, joined: &str) -> String {
    let mut result = joined.to_string();
    if original.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SAMPLE_SKILL: &str = r#"---
name: diagnose
description: Disciplined diagnosis for hard bugs.
requires_evidence: true
---

# Diagnose

Hard bugs need discipline.

## Loop

1. **Reproduce** — Get a failing test.
2. **Fix** — Apply the fix.

## Rules

- Do not ship without regression evidence.
- Prefer cargo test over refactors.
"#;

    #[test]
    fn test_add_rule_to_existing_section() {
        let deltas = vec![AceDelta::AddRule {
            section: "Rules".to_string(),
            rule: "Never bypass clippy warnings.".to_string(),
        }];

        let result = apply_ace_deltas(SAMPLE_SKILL, &deltas).expect("apply");
        assert!(result.contains("name: diagnose"));
        assert!(result.contains("- Never bypass clippy warnings."));
        assert!(result.contains("- Do not ship without regression evidence."));
        let rules_at = result.find("## Rules").unwrap();
        let loop_at = result.find("## Loop").unwrap();
        let added_at = result.find("- Never bypass clippy warnings.").unwrap();
        assert!(loop_at < rules_at);
        assert!(added_at > rules_at);
    }

    #[test]
    fn test_update_rule_in_body() {
        let deltas = vec![AceDelta::UpdateRule {
            target: "- Prefer cargo test over refactors.".to_string(),
            replacement: "- Prefer cargo test and clippy over large speculative refactors."
                .to_string(),
        }];

        let result = apply_ace_deltas(SAMPLE_SKILL, &deltas).expect("apply");
        assert!(result.contains("- Prefer cargo test and clippy over large speculative refactors."));
        assert!(!result.contains("- Prefer cargo test over refactors."));
        assert!(result.contains("name: diagnose"));
    }

    #[test]
    fn test_remove_rule_from_body() {
        let deltas = vec![AceDelta::RemoveRule {
            target: "Do not ship without regression evidence.".to_string(),
        }];

        let result = apply_ace_deltas(SAMPLE_SKILL, &deltas).expect("apply");
        assert!(!result.contains("Do not ship without regression evidence."));
        assert!(result.contains("Prefer cargo test over refactors."));
        assert!(result.contains("name: diagnose"));
    }

    #[test]
    fn test_add_new_section() {
        let deltas = vec![AceDelta::AddRule {
            section: "Checklist".to_string(),
            rule: "Confirm all tests pass before handoff.".to_string(),
        }];

        let result = apply_ace_deltas(SAMPLE_SKILL, &deltas).expect("apply");
        assert!(result.contains("## Checklist"));
        assert!(result.contains("- Confirm all tests pass before handoff."));
    }

    #[test]
    fn test_add_to_numbered_section_continues_numbering() {
        let deltas = vec![AceDelta::AddRule {
            section: "Loop".to_string(),
            rule: "**Cite** — Quote the failing command.".to_string(),
        }];
        let result = apply_ace_deltas(SAMPLE_SKILL, &deltas).expect("apply");
        assert!(result.contains("3. **Cite** — Quote the failing command."));
        let cite = result.find("3. **Cite**").unwrap();
        let rules = result.find("## Rules").unwrap();
        assert!(cite < rules);
    }

    #[test]
    fn test_add_is_idempotent_when_rule_already_present() {
        let deltas = vec![AceDelta::AddRule {
            section: "Rules".to_string(),
            rule: "Do not ship without regression evidence.".to_string(),
        }];
        let result = apply_ace_deltas(SAMPLE_SKILL, &deltas).expect("apply");
        assert_eq!(result, SAMPLE_SKILL);
    }

    #[test]
    fn test_update_refuses_ambiguous_target() {
        let raw = "---\nname: x\ndescription: d\n---\n\n## Rules\n\n- alpha foo\n- beta foo\n";
        let err = apply_ace_deltas(
            raw,
            &[AceDelta::UpdateRule {
                target: "foo".to_string(),
                replacement: "bar".to_string(),
            }],
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("ambiguous"), "{err}");
    }

    #[test]
    fn test_empty_add_refused() {
        let err = apply_ace_deltas(
            SAMPLE_SKILL,
            &[AceDelta::AddRule {
                section: "Rules".to_string(),
                rule: "   ".to_string(),
            }],
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("empty"), "{err}");
    }

    #[test]
    fn test_curate_dry_run_does_not_write() {
        let dir = tempfile_workflows();
        write_skill(&dir, "diagnose", SAMPLE_SKILL);
        let batch = AceDeltaBatch {
            skill_name: "diagnose".into(),
            deltas: vec![AceDelta::AddRule {
                section: "Rules".into(),
                rule: "Cite tool output.".into(),
            }],
            evidence_source: Some("gate refuse".into()),
        };
        let out = curate_workflow_skill(&dir, &batch, false).expect("curate");
        assert!(out.changed());
        assert!(!out.wrote);
        let on_disk = std::fs::read_to_string(dir.join("diagnose/SKILL.md")).unwrap();
        assert_eq!(on_disk, SAMPLE_SKILL);
        assert!(out.proposed.contains("- Cite tool output."));
    }

    #[test]
    fn test_curate_pin_writes() {
        let dir = tempfile_workflows();
        write_skill(&dir, "diagnose", SAMPLE_SKILL);
        let batch = AceDeltaBatch {
            skill_name: "diagnose".into(),
            deltas: vec![AceDelta::AddRule {
                section: "Rules".into(),
                rule: "Cite tool output.".into(),
            }],
            evidence_source: None,
        };
        let out = curate_workflow_skill(&dir, &batch, true).expect("curate");
        assert!(out.wrote);
        let on_disk = std::fs::read_to_string(dir.join("diagnose/SKILL.md")).unwrap();
        assert!(on_disk.contains("- Cite tool output."));
        assert!(on_disk.contains("name: diagnose"));
    }

    #[test]
    fn test_refuse_path_traversal_and_soul() {
        assert!(validate_skill_name("../soul").is_err());
        assert!(validate_skill_name("soul").is_err());
        assert!(validate_skill_name("SOUL").is_err());
        assert!(validate_skill_name("diagnose").is_ok());
    }

    #[test]
    fn test_curate_real_diagnose_fixture_dry_run() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let src = repo.join("skills/workflows/diagnose/SKILL.md");
        let raw = std::fs::read_to_string(&src).expect("in-tree diagnose SKILL.md");
        let dir = tempfile_workflows();
        write_skill(&dir, "diagnose", &raw);
        let batch = AceDeltaBatch {
            skill_name: "diagnose".into(),
            deltas: vec![AceDelta::AddRule {
                section: "Rules".into(),
                rule: "Cite exact tool output before claiming a bug is resolved.".into(),
            }],
            evidence_source: None,
        };
        let out = curate_workflow_skill(&dir, &batch, false).expect("curate");
        assert!(out.changed());
        assert!(!out.wrote);
        assert!(out
            .proposed
            .contains("Cite exact tool output before claiming a bug is resolved."));
        let live = std::fs::read_to_string(&src).unwrap();
        assert_eq!(live, raw, "in-tree diagnose SKILL.md must stay unpinned");
    }

    fn tempfile_workflows() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gzmo-ace-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_skill(root: &Path, name: &str, content: &str) {
        let d = root.join(name);
        std::fs::create_dir_all(&d).unwrap();
        let mut f = std::fs::File::create(d.join("SKILL.md")).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }
}
