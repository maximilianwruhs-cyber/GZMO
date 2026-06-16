//! Parse pi-mentor-discovery reports for FAIL/GAP findings and build fixer spawn briefs.

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
}

impl FindingKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FindingKind::Fail => "FAIL",
            FindingKind::Gap => "GAP",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryReportAnalysis {
    pub findings: Vec<ActionableFinding>,
    pub fail_count: usize,
    pub gap_count: usize,
}

impl DiscoveryReportAnalysis {
    pub fn actionable_count(&self) -> usize {
        self.findings.len()
    }

    pub fn has_actionable(&self) -> bool {
        !self.findings.is_empty()
    }
}

/// Parse a published discovery markdown report for `**FAIL**` / `**GAP**` markers.
pub fn analyze_discovery_report(report_path: &Path) -> anyhow::Result<DiscoveryReportAnalysis> {
    let raw = std::fs::read_to_string(report_path)?;
    Ok(analyze_discovery_markdown(&raw))
}

pub fn analyze_discovery_markdown(raw: &str) -> DiscoveryReportAnalysis {
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

fn parse_finding_heading(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("### ")?;
    let (id, title) = rest.split_once(" — ")?;
    if !id.starts_with('F') || id[1..].chars().any(|c| !c.is_ascii_digit()) {
        return None;
    }
    Some((id.to_string(), title.to_string()))
}

fn contains_marker(text: &str, kind: FindingKind) -> bool {
    let marker = match kind {
        FindingKind::Fail => "**FAIL**",
        FindingKind::Gap => "**GAP**",
    };
    text.contains(marker)
}

fn field_value(block: &str, field: &str) -> String {
    let prefix = format!("- {field}:");
    let mut capture = false;
    let mut lines = Vec::new();
    for line in block.lines() {
        if line.starts_with(&prefix) {
            let rest = line[prefix.len()..].trim_start();
            if !rest.is_empty() {
                lines.push(rest.to_string());
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
        "discovery_fail_gap: {} FAIL, {} GAP ({} actionable)",
        analysis.fail_count,
        analysis.gap_count,
        analysis.actionable_count()
    )
}

pub fn is_discovery_fix_recommendation(rec: &crate::kurator_monitor::PendingRecommendation) -> bool {
    rec.kind.as_deref() == Some("discovery_fix")
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
    let mut lines = vec![
        format!(
            "Discovery fixer for session `{discovery_session_id}` — remediation pass on published report."
        ),
        format!("Report: {}", report_path.display()),
        format!(
            "Actionable findings: {} FAIL, {} GAP.",
            analysis.fail_count, analysis.gap_count
        ),
        String::new(),
        "Findings to address (FAIL first, then GAP):".to_string(),
    ];

    let mut fails: Vec<_> = analysis
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::Fail)
        .collect();
    let mut gaps: Vec<_> = analysis
        .findings
        .iter()
        .filter(|f| f.kind == FindingKind::Gap)
        .collect();
    fails.sort_by_key(|f| &f.finding_id);
    gaps.sort_by_key(|f| &f.finding_id);

    for f in fails.into_iter().chain(gaps) {
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
    lines.push(
        "2. For each FAIL finding, attempt a concrete fix in-repo (scripts, config, docs, or small code changes).".to_string(),
    );
    lines.push(
        "3. For GAP findings, implement the smallest viable improvement or document a blocked fix with evidence.".to_string(),
    );
    lines.push(
        "4. Prefer targeted commands (health checks, session cleanup, config tweaks) over broad exploration.".to_string(),
    );
    lines.push(
        "5. Return a concise summary: what you fixed, what you deferred, and commands run.".to_string(),
    );
    lines.push(String::new());
    lines.push(
        "Scope: GZMO project root and gzmo_skills only. Do NOT run broad recursive greps across /home, /data, or /var.".to_string(),
    );

    crate::text_util::truncate_chars(&lines.join("\n"), max_chars)
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
    fn ignores_ok_only_findings() {
        let analysis = analyze_discovery_markdown(SAMPLE);
        assert!(!analysis.findings.iter().any(|f| f.finding_id == "F1"));
    }
}
