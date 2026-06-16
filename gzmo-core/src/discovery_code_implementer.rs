//! Discovery code implementer — Epimetheus sub-agent pass after deterministic probes.
//!
//! Reads probe JSON + report actions and writes concrete code/config under GZMO + gzmo_skills.

use std::path::{Path, PathBuf};

use crate::discovery_fixer::{self, ActionableFinding};

pub fn discovery_implement_session_id(discovery_session_id: &str, report_path: &Path) -> String {
    let stem = report_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("report");
    format!("discovery-implement:{discovery_session_id}:{stem}")
}

pub fn is_discovery_code_implement_recommendation(
    rec: &crate::kurator_monitor::PendingRecommendation,
) -> bool {
    rec.kind.as_deref() == Some("discovery_code_implement")
        || rec.reason.starts_with("discovery_code_implement:")
        || rec.session_id.starts_with("discovery-implement:")
}

pub fn resolve_implement_manifest_path(session_id: &str) -> PathBuf {
    let skills = std::env::var("GZMO_SKILLS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("gzmo_skills")
        });
    skills
        .join("data/pi-mentor-discovery/implementations")
        .join(session_id)
        .join("manifest.json")
}

pub fn resolve_probe_results_dir() -> PathBuf {
    let skills = std::env::var("GZMO_SKILLS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("gzmo_skills")
        });
    skills.join("data/pi-mentor-discovery/probe-results")
}

pub fn discovery_code_implement_reason(probed_count: usize) -> String {
    format!("discovery_code_implement: {probed_count} probed finding(s) awaiting code")
}

pub fn build_code_implementer_brief(
    report_path: &Path,
    discovery_session_id: &str,
    manifest_path: &Path,
    findings: &[ActionableFinding],
    max_chars: usize,
) -> String {
    build_code_implementer_brief_for_findings(
        report_path,
        discovery_session_id,
        manifest_path,
        findings,
        false,
        max_chars,
    )
}

pub fn build_code_implementer_brief_single(
    report_path: &Path,
    discovery_session_id: &str,
    manifest_path: &Path,
    finding: &ActionableFinding,
    max_chars: usize,
) -> String {
    build_code_implementer_brief_for_findings(
        report_path,
        discovery_session_id,
        manifest_path,
        std::slice::from_ref(finding),
        true,
        max_chars,
    )
}

fn build_code_implementer_brief_for_findings(
    report_path: &Path,
    discovery_session_id: &str,
    manifest_path: &Path,
    findings: &[ActionableFinding],
    retry_mode: bool,
    max_chars: usize,
) -> String {
    let probe_dir = resolve_probe_results_dir();
    let remediation_dir = format!(
        "gzmo_skills/scripts/discovery-remediations/{discovery_session_id}"
    );

    let intro = if retry_mode {
        format!(
            "Discovery code implementer RETRY for session `{discovery_session_id}` — one finding, one code patch."
        )
    } else {
        format!(
            "Discovery code implementer for session `{discovery_session_id}` — implement probe findings in-repo."
        )
    };

    let mut lines = vec![
        intro,
        format!("Report: {}", report_path.display()),
        format!("Probe manifest: {}", manifest_path.display()),
        format!("Probe results dir: {}", probe_dir.display()),
        format!(
            "Write session remediations under: {remediation_dir}/ (create dir via file_write)"
        ),
        String::new(),
        "Context: deterministic probes already ran. JSON probe outputs contain measured facts.".to_string(),
        String::new(),
        "Findings to implement in code/config:".to_string(),
    ];

    for f in findings {
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
    lines.push("1. Read the probe manifest and matching probe-*.json files for this session.".to_string());
    if retry_mode {
        lines.push(
            "2. Focus ONLY on the single finding above — one concrete code/config change with file_write.".to_string(),
        );
    } else {
        lines.push(
            "2. For each finding, read its probe JSON verdict and implement the smallest viable code/config fix.".to_string(),
        );
        lines.push(
            "3. Put scripts under gzmo_skills/scripts/discovery-remediations/{session}/; patch gzmo-core/ or gzmo.toml when warranted.".to_string(),
        );
    }
    lines.push(
        "4. Examples: memory_index empty → sync/populate script; friction gaps → chaos event helper; vault_truths drift → monitor script + systemd timer snippet.".to_string(),
    );
    lines.push(
        "5. MUST file_write at least one .rs, .sh, or .toml remediation file before finishing.".to_string(),
    );
    lines.push(
        "6. Run one short shell_exec to verify (bash -n script, cargo check -p gzmo-core, etc.).".to_string(),
    );
    lines.push(
        "7. Summary must list every path you created or modified — only claim paths you wrote.".to_string(),
    );
    lines.push(String::new());
    lines.push(
        "Scope: survey_GZMO/ (gzmo-core, gzmo.toml, systemd/) and gzmo_skills/ only. No greps across /home or /var.".to_string(),
    );

    crate::text_util::truncate_chars(&lines.join("\n"), max_chars)
}

pub fn is_discovery_agent_brief(brief: &str) -> bool {
    brief.contains("Discovery fixer") || brief.contains("Discovery code implementer")
}

/// Extend remediation artifact detection for code-implementer writes.
pub fn verify_code_implement_outcome(
    summary: &str,
    hit_max_iterations: bool,
    roots: &[PathBuf],
    written_paths: &[String],
) -> discovery_fixer::DiscoveryFixVerification {
    let mut v = discovery_fixer::verify_discovery_fix_outcome(
        summary,
        hit_max_iterations,
        roots,
        written_paths,
    );
    if v.passed {
        return v;
    }
    let code_writes: Vec<String> = written_paths
        .iter()
        .filter(|p| is_code_remediation_artifact(p))
        .filter(|p| {
            let path = Path::new(p.as_str());
            path.is_absolute() && path.exists()
                || roots.iter().any(|root| root.join(p).exists())
        })
        .cloned()
        .collect();
    if !code_writes.is_empty() {
        v.passed = true;
        v.notes = format!("verified code file_write: {}", code_writes.join(", "));
    }
    v
}

pub fn is_code_remediation_artifact(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.ends_with("events.jsonl") || lower.ends_with("probe-results/") {
        return false;
    }
    lower.contains("discovery-remediations/")
        || lower.contains("gzmo-core/src/")
        || lower.contains("gzmo-core/")
        || lower.ends_with(".rs")
        || lower.ends_with(".toml")
        || lower.ends_with(".service")
        || (lower.ends_with(".sh") && lower.contains("discovery-remediations"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery_fixer::FindingKind;

    #[test]
    fn brief_mentions_code_implementer() {
        let brief = build_code_implementer_brief(
            Path::new("/tmp/report.md"),
            "sess-1",
            Path::new("/tmp/manifest.json"),
            &[ActionableFinding {
                finding_id: "R1".into(),
                title: "Probe A02".into(),
                kind: FindingKind::Action,
                excerpt: "check spark".into(),
            }],
            8000,
        );
        assert!(brief.contains("Discovery code implementer"));
        assert!(brief.contains("discovery-remediations/sess-1"));
    }

    #[test]
    fn detects_code_remediation_paths() {
        assert!(is_code_remediation_artifact(
            "gzmo_skills/scripts/discovery-remediations/s1/sync-memory-index.sh"
        ));
        assert!(is_code_remediation_artifact("gzmo-core/src/foo.rs"));
    }
}
