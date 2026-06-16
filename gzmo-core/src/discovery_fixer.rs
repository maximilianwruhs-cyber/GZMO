//! Parse pi-mentor-discovery reports for actionable items and build fixer spawn briefs.
//!
//! Actionable sources: FAIL/GAP markers in finding blocks, plus numbered items under
//! `## Recommended next actions` (session finals and cycle reports).

use std::path::{Path, PathBuf};

/// One actionable item extracted from a discovery report finding block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionableFinding {
    pub finding_id: String,
    pub title: String,
    pub kind: FindingKind,
    pub excerpt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    Fail,
    Gap,
    /// Numbered item from `## Recommended next actions`.
    Action,
}

impl FindingKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FindingKind::Fail => "FAIL",
            FindingKind::Gap => "GAP",
            FindingKind::Action => "ACTION",
        }
    }

    fn priority(self) -> u8 {
        match self {
            FindingKind::Fail => 0,
            FindingKind::Gap => 1,
            FindingKind::Action => 2,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryReportAnalysis {
    pub findings: Vec<ActionableFinding>,
    pub fail_count: usize,
    pub gap_count: usize,
    pub action_count: usize,
}

impl DiscoveryReportAnalysis {
    pub fn actionable_count(&self) -> usize {
        self.findings.len()
    }

    pub fn has_actionable(&self) -> bool {
        !self.findings.is_empty()
    }
}

/// Parse a published discovery markdown report for FAIL/GAP markers and recommended actions.
pub fn analyze_discovery_report(report_path: &Path) -> anyhow::Result<DiscoveryReportAnalysis> {
    let raw = std::fs::read_to_string(report_path)?;
    Ok(analyze_discovery_markdown(&raw))
}

pub fn analyze_discovery_markdown(raw: &str) -> DiscoveryReportAnalysis {
    let mut analysis = analyze_finding_blocks(raw);
    analysis
        .findings
        .extend(parse_recommended_actions(raw));
    analysis.action_count = analysis
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::Action)
        .count();
    analysis
}

fn analyze_finding_blocks(raw: &str) -> DiscoveryReportAnalysis {
    let mut analysis = DiscoveryReportAnalysis::default();

    let mut current_id = String::new();
    let mut current_title = String::new();
    let mut current_block = String::new();
    let mut in_block = false;

    let flush = |analysis: &mut DiscoveryReportAnalysis,
                 id: &str,
                 title: &str,
                 block: &str| {
        if id.is_empty() {
            return;
        }
        let risk = field_value(block, "Risk or opportunity");
        let observation = field_value(block, "Observation");
        let scan_text = format!("{risk} {observation}");

        for kind in [FindingKind::Fail, FindingKind::Gap] {
            if contains_marker(&scan_text, kind) {
                let excerpt = truncate_excerpt(&risk, 280);
                analysis.findings.push(ActionableFinding {
                    finding_id: id.to_string(),
                    title: title.to_string(),
                    kind,
                    excerpt,
                });
                match kind {
                    FindingKind::Fail => analysis.fail_count += 1,
                    FindingKind::Gap => analysis.gap_count += 1,
                    FindingKind::Action => {}
                }
            }
        }
    };

    for line in raw.lines() {
        if let Some((id, title)) = parse_finding_heading(line) {
            flush(
                &mut analysis,
                &current_id,
                &current_title,
                &current_block,
            );
            current_id = id;
            current_title = title;
            current_block.clear();
            in_block = true;
            continue;
        }
        if in_block {
            if line.starts_with("## ") {
                flush(
                    &mut analysis,
                    &current_id,
                    &current_title,
                    &current_block,
                );
                current_id.clear();
                current_title.clear();
                current_block.clear();
                in_block = false;
            } else {
                if !current_block.is_empty() {
                    current_block.push('\n');
                }
                current_block.push_str(line);
            }
        }
    }
    flush(
        &mut analysis,
        &current_id,
        &current_title,
        &current_block,
    );

    analysis
}

fn parse_recommended_actions(raw: &str) -> Vec<ActionableFinding> {
    let mut findings = Vec::new();
    let mut in_section = false;
    let mut index = 0usize;

    for line in raw.lines() {
        if line.starts_with("## Recommended next actions") {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.starts_with("## ") {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some(text) = parse_numbered_action_line(trimmed) else {
            continue;
        };
        index += 1;
        let title = action_title(&text);
        findings.push(ActionableFinding {
            finding_id: format!("R{index}"),
            title,
            kind: FindingKind::Action,
            excerpt: truncate_excerpt(&text, 280),
        });
    }

    findings
}

fn parse_numbered_action_line(line: &str) -> Option<&str> {
    let rest = line.strip_prefix(|c: char| c.is_ascii_digit())?;
    let rest = rest.strip_prefix('.')?.trim_start();
    if rest.is_empty() {
        return None;
    }
    Some(rest)
}

fn action_title(text: &str) -> String {
    let head = text
        .split_once(':')
        .map(|(prefix, _)| prefix.trim())
        .unwrap_or(text);
    truncate_excerpt(head, 80)
}

fn parse_finding_heading(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("### ")?;
    let (id, title) = rest.split_once(" — ")?;
    if !id.starts_with('F') || id[1..].chars().any(|c| !c.is_ascii_digit()) {
        return None;
    }
    Some((id.to_string(), title.to_string()))
}

fn contains_marker(text: &str, kind: FindingKind) -> bool {
    let lower = text.to_ascii_lowercase();
    match kind {
        FindingKind::Fail => {
            lower.contains("**fail**")
                || lower.contains("**fail:")
                || lower.contains("**fail risk**")
                || lower.contains("**fail risk:")
        }
        FindingKind::Gap => lower.contains("**gap**") || lower.contains("**gap:"),
        FindingKind::Action => false,
    }
}

fn field_value(block: &str, field: &str) -> String {
    let plain_prefix = format!("- {field}:");
    let bold_prefix = format!("- **{field}:**");
    let mut capture = false;
    let mut lines = Vec::new();
    for line in block.lines() {
        let rest = if let Some(r) = line.strip_prefix(&plain_prefix) {
            Some(r.trim_start())
        } else if let Some(r) = line.strip_prefix(&bold_prefix) {
            Some(r.trim_start())
        } else {
            None
        };
        if let Some(r) = rest {
            if !r.is_empty() {
                lines.push(r.to_string());
            }
            capture = true;
            continue;
        }
        if capture {
            if line.starts_with("- ") {
                break;
            }
            if !line.trim().is_empty() {
                lines.push(line.trim().to_string());
            }
        }
    }
    lines.join(" ")
}

fn truncate_excerpt(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed
        .chars()
        .take(max_chars.saturating_sub(1))
        .chain(std::iter::once('…'))
        .collect()
}

pub fn discovery_fix_session_id(discovery_session_id: &str, report_path: &Path) -> String {
    let stem = report_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("report");
    format!("discovery-fix:{discovery_session_id}:{stem}")
}

pub fn discovery_fix_reason(analysis: &DiscoveryReportAnalysis) -> String {
    format!(
        "discovery_actionable: {} FAIL, {} GAP, {} ACTION ({} actionable)",
        analysis.fail_count,
        analysis.gap_count,
        analysis.action_count,
        analysis.actionable_count()
    )
}

pub fn is_discovery_fix_recommendation(rec: &crate::kurator_monitor::PendingRecommendation) -> bool {
    rec.kind.as_deref() == Some("discovery_fix")
        || rec.reason.starts_with("discovery_actionable:")
        || rec.reason.starts_with("discovery_fail_gap:")
        || rec.session_id.starts_with("discovery-fix:")
}

/// Resolve report path from recommendation metadata or Synapse `spawn.recommended` payload.
pub fn resolve_discovery_report_path(
    rec: &crate::kurator_monitor::PendingRecommendation,
    bus_path: Option<&Path>,
) -> PathBuf {
    if let Some(path) = &rec.report_path {
        return PathBuf::from(path);
    }
    if let Some(bus_path) = bus_path {
        if let Ok(raw) = std::fs::read_to_string(bus_path) {
            for line in raw.lines().rev().take(5000) {
                if !line.contains(&rec.event_id) || !line.contains("spawn.recommended") {
                    continue;
                }
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                    if event.get("id").and_then(|v| v.as_str()) != Some(rec.event_id.as_str()) {
                        continue;
                    }
                    if let Some(path) = event
                        .pointer("/data/report_path")
                        .and_then(|v| v.as_str())
                    {
                        return PathBuf::from(path);
                    }
                }
            }
        }
    }
    if let Some(stem) = rec.session_id.rsplit(':').next() {
        let skills = std::env::var("GZMO_SKILLS_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("gzmo_skills")
            });
        let candidate = skills
            .join("data/pi-mentor-discovery/reports")
            .join(format!("{stem}.md"));
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from("data/pi-mentor-discovery/latest.md")
}

pub fn build_fixer_brief(
    report_path: &Path,
    discovery_session_id: &str,
    analysis: &DiscoveryReportAnalysis,
    max_chars: usize,
) -> String {
    build_fixer_brief_for_findings(
        report_path,
        discovery_session_id,
        &analysis.findings,
        false,
        max_chars,
    )
}

/// Narrow fixer brief for a single FAIL/GAP finding (retry pass).
pub fn build_fixer_brief_single(
    report_path: &Path,
    discovery_session_id: &str,
    finding: &ActionableFinding,
    max_chars: usize,
) -> String {
    build_fixer_brief_for_findings(
        report_path,
        discovery_session_id,
        std::slice::from_ref(finding),
        true,
        max_chars,
    )
}

pub fn build_fixer_brief_for_findings(
    report_path: &Path,
    discovery_session_id: &str,
    findings: &[ActionableFinding],
    retry_mode: bool,
    max_chars: usize,
) -> String {
    let fail_count = findings
        .iter()
        .filter(|f| f.kind == FindingKind::Fail)
        .count();
    let gap_count = findings
        .iter()
        .filter(|f| f.kind == FindingKind::Gap)
        .count();
    let action_count = findings.len() - fail_count - gap_count;

    let intro = if retry_mode {
        format!(
            "Discovery fixer RETRY for session `{discovery_session_id}` — single finding remediation."
        )
    } else {
        format!(
            "Discovery fixer for session `{discovery_session_id}` — remediation pass on published report."
        )
    };

    let mut lines = vec![
        intro,
        format!("Report: {}", report_path.display()),
        format!(
            "Actionable items in this pass: {} FAIL, {} GAP, {} ACTION.",
            fail_count, gap_count, action_count
        ),
        String::new(),
        "Items to address (FAIL first, then GAP, then recommended actions):".to_string(),
    ];

    let mut ordered: Vec<_> = findings.iter().collect();
    ordered.sort_by_key(|f| (f.kind.priority(), &f.finding_id));

    for f in ordered {
        lines.push(format!(
            "- {} {} — {}: {}",
            f.kind.as_str(),
            f.finding_id,
            f.title,
            f.excerpt
        ));
    }

    lines.push(String::new());
    lines.push("Task:".to_string());
    lines.push("1. Read the full discovery report at the path above.".to_string());
    if retry_mode {
        lines.push(
            "2. Focus ONLY on the finding listed above — one concrete fix with file_write or shell_exec.".to_string(),
        );
    } else {
        lines.push(
            "2. For each FAIL finding, attempt a concrete fix in-repo (scripts, config, docs, or small code changes).".to_string(),
        );
        lines.push(
            "3. For GAP findings, implement the smallest viable improvement or document a blocked fix with evidence.".to_string(),
        );
        lines.push(
            "4. For ACTION items (Recommended next actions), run or create the matching script under `gzmo_skills/scripts/discovery-probes/` (e.g. probe-a02-spark-distill.sh). One script per action; use file_write then shell_exec to verify.".to_string(),
        );
    }
    lines.push(
        "5. Prefer targeted commands (health checks, session cleanup, config tweaks) over broad exploration.".to_string(),
    );
    lines.push(
        "6. Return a concise summary: what you fixed, what you deferred, and commands run.".to_string(),
    );
    lines.push(
        "7. Use file_write or shell_exec to create or change files BEFORE your final summary — do not claim paths you did not write.".to_string(),
    );
    lines.push(
        "8. Do not prefix shell_exec commands with # — put comments in the brief only. First command token must be a real binary (find, cat, bash, …).".to_string(),
    );
    lines.push(String::new());
    lines.push(
        "Scope: GZMO project root and gzmo_skills only. Do NOT run broad recursive greps across /home, /data, or /var.".to_string(),
    );

    crate::text_util::truncate_chars(&lines.join("\n"), max_chars)
}

/// Search roots for verifying fixer artifact claims (GZMO repo + gzmo_skills).
pub fn discovery_fixer_search_roots(project_root: &Path) -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let skills = std::env::var("GZMO_SKILLS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&home).join("gzmo_skills"));
    vec![project_root.to_path_buf(), skills]
}

/// Default cwd for discovery fixer shell_exec (gzmo_skills tree).
pub fn discovery_fixer_working_dir() -> String {
    std::env::var("GZMO_SKILLS_ROOT").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home)
            .join("gzmo_skills")
            .to_string_lossy()
            .into_owned()
    })
}

/// Paths the fixer claims to have created or modified (parsed from summary text).
pub fn extract_claimed_artifact_paths(summary: &str) -> Vec<String> {
    const KEYWORDS: &[&str] = &[
        "created `",
        "modified `",
        "wrote `",
        "added `",
        "fixed `",
        "updated `",
        "created '",
        "modified '",
    ];

    let mut paths = Vec::new();
    for line in summary.lines() {
        let lower = line.to_lowercase();
        for kw in KEYWORDS {
            let Some(idx) = lower.find(kw) else {
                continue;
            };
            let rest = &line[idx + kw.len()..];
            let end = rest.find('`').or_else(|| rest.find('\'')).unwrap_or(rest.len());
            let path = rest[..end].trim();
            if looks_like_artifact_path(path) {
                paths.push(path.to_string());
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn looks_like_artifact_path(s: &str) -> bool {
    if s.is_empty() || s.contains(' ') {
        return false;
    }
    s.contains('/')
        || s.ends_with(".sh")
        || s.ends_with(".rs")
        || s.ends_with(".toml")
        || s.ends_with(".md")
        || s.ends_with(".json")
        || s.ends_with(".yaml")
        || s.ends_with(".yml")
}

fn is_remediation_artifact(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.ends_with("events.jsonl") || lower.ends_with("state.json") {
        return false;
    }
    lower.contains("/scripts/")
        || lower.contains("gzmo_skills/scripts")
        || lower.ends_with(".sh")
        || lower.ends_with(".toml")
}

fn summary_hallucinates_file_write(summary: &str, written_paths: &[String]) -> bool {
    written_paths.is_empty()
        && (summary.contains("<function=file_write>")
            || summary.contains("function=file_write")
            || summary.contains("<tool_call>"))
}

fn artifact_path_exists(claimed: &str, roots: &[PathBuf]) -> bool {
    let path = Path::new(claimed);
    if path.is_absolute() && path.exists() {
        return true;
    }
    roots.iter().any(|root| root.join(claimed).exists())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryFixVerification {
    pub passed: bool,
    pub missing_paths: Vec<String>,
    pub hit_max_iterations: bool,
    pub notes: String,
}

/// Post-spawn verify gate: proven file_write paths or claimed artifacts must exist on disk.
pub fn verify_discovery_fix_outcome(
    summary: &str,
    hit_max_iterations: bool,
    roots: &[PathBuf],
    written_paths: &[String],
) -> DiscoveryFixVerification {
    let claimed = extract_claimed_artifact_paths(summary);
    let missing_paths: Vec<String> = claimed
        .iter()
        .filter(|p| !artifact_path_exists(p, roots))
        .cloned()
        .collect();

    let verified_writes: Vec<String> = written_paths
        .iter()
        .filter(|p| artifact_path_exists(p, roots) && is_remediation_artifact(p))
        .cloned()
        .collect();

    let mut issues = Vec::new();
    if !missing_paths.is_empty() {
        issues.push(format!(
            "claimed artifacts missing on disk: {}",
            missing_paths.join(", ")
        ));
    }
    if hit_max_iterations {
        issues.push("agent hit max tool iterations before finishing".to_string());
    }
    if summary_hallucinates_file_write(summary, written_paths) {
        issues.push(
            "summary contains file_write tool markup but no file_write was executed".to_string(),
        );
    }
    if verified_writes.is_empty() && hit_max_iterations {
        issues.push("no remediation file_write on disk".to_string());
    }

    let passed = !verified_writes.is_empty()
        || (missing_paths.is_empty() && !claimed.is_empty() && !hit_max_iterations);

    let notes = if !passed {
        issues.join("; ")
    } else if !verified_writes.is_empty() {
        format!("verified file_write: {}", verified_writes.join(", "))
    } else {
        String::new()
    };

    DiscoveryFixVerification {
        passed,
        missing_paths,
        hit_max_iterations,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
## Findings

### F1 — Passive bus
- Observation: The bus is passive.
- Risk or opportunity: **OK**: fine.

### F2 — Orphan sessions
- Observation: 39 orphaned sessions.
- Risk or opportunity: **GAP**: cleanup missing. **FAIL**: 39 orphans remain.

### F3 — Pipeline
- Observation: distill rejects 96%.
- Risk or opportunity: **GAP**: dedup too strict.
"#;

    #[test]
    fn extracts_fail_and_gap() {
        let analysis = analyze_discovery_markdown(SAMPLE);
        assert_eq!(analysis.fail_count, 1);
        assert_eq!(analysis.gap_count, 2);
        assert_eq!(analysis.actionable_count(), 3);
        assert!(analysis
            .findings
            .iter()
            .any(|f| f.finding_id == "F2" && f.kind == FindingKind::Fail));
    }

    #[test]
    fn recognizes_fail_gap_with_bold_field_labels() {
        let raw = r#"
### F1 — VM200 SPOF
- **Observation:** Embeddings on VM200 only.
- **Risk or opportunity:** **FAIL risk** — VM200 is a single point of failure.

### F2 — Dedup gap
- **Observation:** Only 63 distill_dedup rows.
- **Risk or opportunity:** **GAP** — dedup mechanism unclear.
"#;
        let analysis = analyze_discovery_markdown(raw);
        assert_eq!(analysis.fail_count, 1);
        assert_eq!(analysis.gap_count, 1);
        assert_eq!(analysis.actionable_count(), 2);
    }

    #[test]
    fn recognizes_fail_risk_and_gap_title_case() {
        let raw = r#"
### F1 — Synapse lag
- Observation: Only distill_complete events.
- Risk or opportunity: **Gap**: invisible failures.

### F2 — Episodic debt
- Observation: Sessions never cleaned.
- Risk or opportunity: **FAIL risk**: unbounded growth.
"#;
        let analysis = analyze_discovery_markdown(raw);
        assert_eq!(analysis.gap_count, 1);
        assert_eq!(analysis.fail_count, 1);
        assert_eq!(analysis.actionable_count(), 2);
    }

    #[test]
    fn ignores_ok_only_findings() {
        let analysis = analyze_discovery_markdown(SAMPLE);
        assert!(!analysis.findings.iter().any(|f| f.finding_id == "F1"));
    }

    #[test]
    fn extracts_claimed_paths_from_summary() {
        let summary = "Created `gzmo_skills/scripts/cleanup_sessions.sh` and modified `gzmo.toml`.";
        let paths = extract_claimed_artifact_paths(summary);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"gzmo_skills/scripts/cleanup_sessions.sh".to_string()));
        assert!(paths.contains(&"gzmo.toml".to_string()));
    }

    #[test]
    fn verify_gate_fails_on_missing_claimed_file() {
        let summary = "Created `gzmo_skills/scripts/nonexistent_fix_12345.sh`.";
        let roots = vec![PathBuf::from("/tmp")];
        let v = verify_discovery_fix_outcome(summary, false, &roots, &[]);
        assert!(!v.passed);
        assert!(!v.missing_paths.is_empty());
    }

    #[test]
    fn verify_gate_passes_on_proven_file_write() {
        let roots = vec![PathBuf::from("/tmp")];
        let path = "/tmp/verify_gate_test_remediation.sh";
        std::fs::write(path, "#!/bin/sh\n").unwrap();
        let v = verify_discovery_fix_outcome(
            "Deferred summary claims.",
            true,
            &roots,
            &[path.to_string()],
        );
        assert!(v.passed);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn extracts_recommended_next_actions() {
        let raw = r#"
## Recommended next actions
1. Probe A02: Examine the spark_complete event structure.
2. Monitor vault_truths drift across the next 3 sessions.

## Mentor synthesis
Done.
"#;
        let analysis = analyze_discovery_markdown(raw);
        assert_eq!(analysis.action_count, 2);
        assert_eq!(analysis.actionable_count(), 2);
        assert!(analysis
            .findings
            .iter()
            .any(|f| f.finding_id == "R1" && f.kind == FindingKind::Action));
        assert!(analysis.findings.iter().any(|f| f.finding_id == "R2"));
    }

    #[test]
    fn verify_gate_fails_on_hallucinated_file_write_markup() {
        let summary = "Done.\n<tool_call>\n<function=file_write>\n";
        let v = verify_discovery_fix_outcome(summary, true, &[], &[]);
        assert!(!v.passed);
        assert!(v.notes.contains("tool markup"));
    }

    #[test]
    fn verify_gate_fails_when_iteration_cap_without_artifacts() {
        let v = verify_discovery_fix_outcome("Deferred all fixes.", true, &[], &[]);
        assert!(!v.passed);
        assert!(v.notes.contains("max tool iterations"));
    }
}
