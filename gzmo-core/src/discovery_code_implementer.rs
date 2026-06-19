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
    single_finding_mode: bool,
    max_chars: usize,
) -> String {
    let probe_dir = resolve_probe_results_dir();
    let remediation_dir = format!(
        "gzmo_skills/scripts/discovery-remediations/{discovery_session_id}"
    );

    let intro = if single_finding_mode {
        format!(
            "Discovery code implementer for session `{discovery_session_id}` — ONE finding, ONE code patch."
        )
    } else {
        format!(
            "Discovery code implementer for session `{discovery_session_id}` — implement probe findings in-repo."
        )
    };

    let mut lines = vec![
        intro,
        format!("Report path (file_read): {}", report_path.display()),
        format!("Probe manifest (file_read): {}", manifest_path.display()),
        format!("Probe results dir (file_read): {}", probe_dir.display()),
        format!(
            "Write session remediations under: {remediation_dir}/ (create dir via file_write)"
        ),
        String::new(),
        "Context: deterministic probes already ran. Use file_read on probe JSON — do not assume report text in this brief.".to_string(),
        String::new(),
        "Finding to implement:".to_string(),
    ];

    for f in findings {
        let excerpt = crate::text_util::truncate_chars(&f.excerpt, 500);
        lines.push(format!(
            "- {} {} — {}: {}",
            f.kind.as_str(),
            f.finding_id,
            f.title,
            excerpt
        ));
    }

    lines.push(String::new());
    lines.push("Task:".to_string());
    lines.push("1. file_read the probe manifest and the probe-*.json for this finding's session.".to_string());
    if single_finding_mode {
        lines.push(
            "2. file_read the report section for this finding_id only — implement one concrete code/config change.".to_string(),
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
    brief.contains("Discovery fixer")
        || brief.contains("Discovery code implementer")
        || crate::discovery_plan_agent::is_discovery_agent_brief(brief)
}

/// Write-phase tuning for discovery sub-agents (plan, fixer, code implementer).
pub struct DiscoveryAgentWriteConfig {
    pub write_phase_at: usize,
    pub write_phase_message: String,
    pub require_file_write_prompt: String,
}

pub fn discovery_agent_write_config(brief: &str, max_iterations: usize) -> Option<DiscoveryAgentWriteConfig> {
    if !is_discovery_agent_brief(brief) {
        return None;
    }
    if crate::discovery_plan_agent::is_discovery_agent_brief(brief) {
        let write_phase_at = 8.min(max_iterations.saturating_sub(3));
        return Some(DiscoveryAgentWriteConfig {
            write_phase_at,
            write_phase_message: crate::discovery_plan_agent::plan_agent_write_phase_message(),
            require_file_write_prompt: crate::discovery_plan_agent::plan_agent_require_file_write_prompt(brief),
        });
    }
    let write_phase_at = max_iterations.saturating_sub(10).max(5);
    let fixer_msg = "WRITE PHASE — stop exploration now. \
Use file_write to create or patch remediation scripts/config under gzmo_skills/ or the GZMO repo. \
Run at most one short shell_exec to verify. Do not finish until file_write succeeded for at least one fix. \
Do not prefix shell_exec commands with # comment lines. \
Never output tool-call XML or pseudo file_write blocks — call the file_write tool.";
    Some(DiscoveryAgentWriteConfig {
        write_phase_at,
        write_phase_message: fixer_msg.to_string(),
        require_file_write_prompt: "STOP — you have not called file_write yet. Use file_write now to create at least one remediation script or config under gzmo_skills/scripts/ (or patch an existing file). Do not reply with text only or tool-call markup.".to_string(),
    })
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
