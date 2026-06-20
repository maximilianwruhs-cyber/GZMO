//! Point-in-time remediation spawn snapshots (Jules SessionSnapshot pattern).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::discovery_fixer::DiscoveryFixVerification;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationActivityType {
    PlanGenerated,
    PlanApproved,
    ProgressUpdated,
    SessionCompleted,
    SessionFailed,
}

impl RemediationActivityType {
    pub fn as_str(self) -> &'static str {
        match self {
            RemediationActivityType::PlanGenerated => "plan_generated",
            RemediationActivityType::PlanApproved => "plan_approved",
            RemediationActivityType::ProgressUpdated => "progress_updated",
            RemediationActivityType::SessionCompleted => "session_completed",
            RemediationActivityType::SessionFailed => "session_failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationActivity {
    pub at: DateTime<Utc>,
    pub activity_type: String,
    pub summary: String,
    #[serde(default)]
    pub shell_exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationInsights {
    pub completion_attempts: u32,
    pub plan_regenerations: u32,
    pub failed_commands: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSnapshot {
    pub task_id: String,
    pub session_id: String,
    pub report_path: String,
    pub passed: bool,
    pub created_at: DateTime<Utc>,
    pub activities: Vec<RemediationActivity>,
    pub activity_counts: BTreeMap<String, u32>,
    pub insights: RemediationInsights,
    pub verify_notes: String,
    #[serde(default)]
    pub written_paths: Vec<String>,
}

pub fn snapshots_dir() -> PathBuf {
    crate::discovery_plan_agent::discovery_implementation_data_root().join("snapshots")
}

pub fn snapshot_path(task_id: &str) -> PathBuf {
    let safe = task_id.replace(['/', ':'], "_");
    snapshots_dir().join(format!("{safe}.json"))
}

pub fn append_activity(
    activities: &mut Vec<RemediationActivity>,
    activity_type: RemediationActivityType,
    summary: impl Into<String>,
    shell_exit_code: Option<i32>,
) {
    activities.push(RemediationActivity {
        at: Utc::now(),
        activity_type: activity_type.as_str().to_string(),
        summary: summary.into(),
        shell_exit_code,
    });
}

pub fn build_snapshot(
    task_id: &str,
    session_id: &str,
    report_path: &Path,
    verification: &DiscoveryFixVerification,
    written_paths: &[String],
    activities: Vec<RemediationActivity>,
) -> RemediationSnapshot {
    let mut activity_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut completion_attempts = 0u32;
    let mut plan_regenerations = 0u32;
    let mut failed_commands = 0u32;

    for a in &activities {
        *activity_counts.entry(a.activity_type.clone()).or_insert(0) += 1;
        match a.activity_type.as_str() {
            "session_completed" => completion_attempts += 1,
            "plan_generated" => plan_regenerations += 1,
            _ => {}
        }
        if a.shell_exit_code.unwrap_or(0) != 0 {
            failed_commands += 1;
        }
    }

    RemediationSnapshot {
        task_id: task_id.to_string(),
        session_id: session_id.to_string(),
        report_path: report_path.display().to_string(),
        passed: verification.passed,
        created_at: Utc::now(),
        activities,
        activity_counts,
        insights: RemediationInsights {
            completion_attempts,
            plan_regenerations,
            failed_commands,
        },
        verify_notes: verification.notes.clone(),
        written_paths: written_paths.to_vec(),
    }
}

pub fn write_snapshot(snapshot: &RemediationSnapshot) -> std::io::Result<PathBuf> {
    let dir = snapshots_dir();
    std::fs::create_dir_all(&dir)?;
    let json_path = snapshot_path(&snapshot.task_id);
    let md_path = json_path.with_extension("md");
    std::fs::write(&json_path, serde_json::to_string_pretty(snapshot).unwrap_or_default())?;
    std::fs::write(&md_path, snapshot_to_markdown(snapshot))?;
    Ok(json_path)
}

pub fn snapshot_to_markdown(s: &RemediationSnapshot) -> String {
    let mut lines = vec![
        "# Remediation snapshot".to_string(),
        String::new(),
        "## Overview".to_string(),
        format!("- task_id: `{}`", s.task_id),
        format!("- session_id: `{}`", s.session_id),
        format!("- report: `{}`", s.report_path),
        format!("- passed: {}", s.passed),
        format!("- created_at: {}", s.created_at.to_rfc3339()),
        String::new(),
        "## Insights".to_string(),
        format!(
            "- completion_attempts: {}",
            s.insights.completion_attempts
        ),
        format!(
            "- plan_regenerations: {}",
            s.insights.plan_regenerations
        ),
        format!("- failed_commands: {}", s.insights.failed_commands),
        String::new(),
        "## Verify notes".to_string(),
        s.verify_notes.clone(),
        String::new(),
        "## Timeline".to_string(),
    ];

    for a in &s.activities {
        let exit = a
            .shell_exit_code
            .map(|c| format!(" exit={c}"))
            .unwrap_or_default();
        lines.push(format!(
            "- {} `{}` — {}{}",
            a.at.to_rfc3339(),
            a.activity_type,
            a.summary,
            exit
        ));
    }

    if !s.written_paths.is_empty() {
        lines.push(String::new());
        lines.push("## Written paths".to_string());
        for p in &s.written_paths {
            lines.push(format!("- `{p}`"));
        }
    }

    lines.join("\n")
}

/// Parse shell exit codes from verify notes like `exit_code=1` or `exit 1`.
pub fn extract_shell_exit_codes(notes: &str) -> Vec<i32> {
    let mut codes = Vec::new();
    for token in notes.split_whitespace() {
        if let Some(rest) = token.strip_prefix("exit_code=") {
            if let Ok(n) = rest.trim_matches(|c: char| !c.is_ascii_digit() && c != '-').parse() {
                codes.push(n);
            }
        }
    }
    codes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_includes_timeline() {
        let snap = RemediationSnapshot {
            task_id: "t1".into(),
            session_id: "s1".into(),
            report_path: "/tmp/r.md".into(),
            passed: true,
            created_at: Utc::now(),
            activities: vec![RemediationActivity {
                at: Utc::now(),
                activity_type: "session_completed".into(),
                summary: "verify_gate PASS".into(),
                shell_exit_code: Some(0),
            }],
            activity_counts: BTreeMap::from([("session_completed".into(), 1)]),
            insights: RemediationInsights {
                completion_attempts: 1,
                plan_regenerations: 0,
                failed_commands: 0,
            },
            verify_notes: "ok".into(),
            written_paths: vec!["/tmp/a.sh".into()],
        };
        let md = snapshot_to_markdown(&snap);
        assert!(md.contains("## Timeline"));
        assert!(md.contains("session_completed"));
    }
}
