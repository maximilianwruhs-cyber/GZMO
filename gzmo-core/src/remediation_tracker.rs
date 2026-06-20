//! Tracks discovery FAIL/GAP findings through fixer spawns until verify_gate passes.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::discovery_fixer::{ActionableFinding, DiscoveryFixVerification, FindingKind};
use crate::kurator_monitor::PendingRecommendation;
use crate::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};

const STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStatus {
    Open,
    InFlight,
    /// Deterministic probes finished; awaiting code implementer agent.
    Probed,
    Fixed,
    Failed,
}

impl RemediationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            RemediationStatus::Open => "open",
            RemediationStatus::InFlight => "in_flight",
            RemediationStatus::Probed => "probed",
            RemediationStatus::Fixed => "fixed",
            RemediationStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemediationFinding {
    pub finding_id: String,
    pub kind: String,
    pub title: String,
    pub excerpt: String,
    pub report_path: String,
    pub discovery_session_id: String,
    pub status: RemediationStatus,
    pub spawn_attempts: u32,
    #[serde(default)]
    pub task_ids: Vec<String>,
    #[serde(default)]
    pub artifact_paths: Vec<String>,
    #[serde(default)]
    pub last_verify_notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemediationTrackerState {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub findings: Vec<RemediationFinding>,
}

fn default_version() -> u32 {
    STATE_VERSION
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemediationSummary {
    pub open: usize,
    pub in_flight: usize,
    pub probed: usize,
    pub fixed: usize,
    pub failed: usize,
    pub total: usize,
}

impl RemediationTrackerState {
    pub fn summary(&self) -> RemediationSummary {
        let mut s = RemediationSummary::default();
        s.total = self.findings.len();
        for f in &self.findings {
            match f.status {
                RemediationStatus::Open => s.open += 1,
                RemediationStatus::InFlight => s.in_flight += 1,
                RemediationStatus::Probed => s.probed += 1,
                RemediationStatus::Fixed => s.fixed += 1,
                RemediationStatus::Failed => s.failed += 1,
            }
        }
        s
    }

    pub fn open_without_verified_fix(&self) -> Vec<&RemediationFinding> {
        self.findings
            .iter()
            .filter(|f| {
                matches!(
                    f.status,
                    RemediationStatus::Open
                        | RemediationStatus::InFlight
                        | RemediationStatus::Probed
                        | RemediationStatus::Failed
                )
            })
            .collect()
    }
}

pub fn default_tracker_path() -> PathBuf {
    if let Ok(data) = std::env::var("PI_MENTOR_DISCOVERY_DATA") {
        return PathBuf::from(data).join("remediation-tracker.json");
    }
    let skills = std::env::var("GZMO_SKILLS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("gzmo_skills")
        });
    skills
        .join("data/pi-mentor-discovery/remediation-tracker.json")
}

pub fn load(path: &Path) -> RemediationTrackerState {
    load_with_polling(path, true)
}

/// Load tracker state; retries briefly when `use_polling` to handle post-spawn flush races.
pub fn load_with_polling(path: &Path, use_polling: bool) -> RemediationTrackerState {
    if !path.is_file() {
        return RemediationTrackerState::default();
    }
    if use_polling && polling_enabled() {
        let config = crate::spawn_polling::PollConfig::default();
        return crate::spawn_polling::load_json_with_retry(path, &config)
            .unwrap_or_default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn polling_enabled() -> bool {
    std::env::var("SPAWN_LOAD_POLLING")
        .map(|v| !matches!(v.as_str(), "0" | "false" | "no"))
        .unwrap_or(true)
}

pub fn save(path: &Path, state: &RemediationTrackerState) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(state)?;
    std::fs::write(path, raw)?;
    Ok(())
}

fn finding_key(report_path: &str, finding_id: &str, kind: &str) -> String {
    format!("{report_path}::{finding_id}::{kind}")
}

/// Pending items (open, in-flight, or failed) for one report path.
pub fn count_pending_for_report(path: &Path, report_path: &Path) -> usize {
    let report_str = report_path.to_string_lossy();
    load(path)
        .findings
        .iter()
        .filter(|f| {
            f.report_path == report_str
                && matches!(
                    f.status,
                    RemediationStatus::Open | RemediationStatus::InFlight | RemediationStatus::Failed
                )
        })
        .count()
}

pub fn count_fixed_for_report(path: &Path, report_path: &Path) -> usize {
    let report_str = report_path.to_string_lossy();
    load(path)
        .findings
        .iter()
        .filter(|f| f.report_path == report_str && f.status == RemediationStatus::Fixed)
        .count()
}

pub fn count_probed_for_report(path: &Path, report_path: &Path) -> usize {
    let report_str = report_path.to_string_lossy();
    load(path)
        .findings
        .iter()
        .filter(|f| f.report_path == report_str && f.status == RemediationStatus::Probed)
        .count()
}

pub fn mark_all_probed_in_flight(path: &Path, report_path: &Path) -> anyhow::Result<()> {
    let report_str = report_path.to_string_lossy();
    let mut state = load(path);
    let now = Utc::now();
    let mut changed = false;
    for f in &mut state.findings {
        if f.report_path == report_str && f.status == RemediationStatus::Probed {
            f.status = RemediationStatus::InFlight;
            f.updated_at = now;
            changed = true;
        }
    }
    if changed {
        save(path, &state)?;
    }
    Ok(())
}

pub fn register_findings_from_report(
    path: &Path,
    report_path: &Path,
    discovery_session_id: &str,
    findings: &[ActionableFinding],
) -> anyhow::Result<usize> {
    let report_str = report_path.to_string_lossy().into_owned();
    let mut state = load(path);
    let mut added = 0usize;

    for f in findings {
        let kind = f.kind.as_str().to_string();
        let key = finding_key(&report_str, &f.finding_id, &kind);
        let exists = state.findings.iter().any(|existing| {
            finding_key(&existing.report_path, &existing.finding_id, &existing.kind) == key
        });
        if exists {
            continue;
        }
        state.findings.push(RemediationFinding {
            finding_id: f.finding_id.clone(),
            kind,
            title: f.title.clone(),
            excerpt: f.excerpt.clone(),
            report_path: report_str.clone(),
            discovery_session_id: discovery_session_id.to_string(),
            status: RemediationStatus::Open,
            spawn_attempts: 0,
            task_ids: Vec::new(),
            artifact_paths: Vec::new(),
            last_verify_notes: None,
            updated_at: Utc::now(),
        });
        added += 1;
    }

    if added > 0 {
        save(path, &state)?;
    }
    Ok(added)
}

pub fn mark_all_open_in_flight(path: &Path, report_path: &Path) -> anyhow::Result<()> {
    let report_str = report_path.to_string_lossy();
    let mut state = load(path);
    let now = Utc::now();
    let mut changed = false;
    for f in &mut state.findings {
        if f.report_path == report_str && f.status == RemediationStatus::Open {
            f.status = RemediationStatus::InFlight;
            f.updated_at = now;
            changed = true;
        }
    }
    if changed {
        save(path, &state)?;
    }
    Ok(())
}

pub fn mark_finding_in_flight(
    path: &Path,
    report_path: &Path,
    finding_id: &str,
    kind: FindingKind,
) -> anyhow::Result<()> {
    let report_str = report_path.to_string_lossy();
    let kind_str = kind.as_str();
    let mut state = load(path);
    let now = Utc::now();
    let mut changed = false;
    for f in &mut state.findings {
        if f.report_path == report_str && f.finding_id == finding_id && f.kind == kind_str {
            if f.status == RemediationStatus::Open
                || f.status == RemediationStatus::Failed
                || f.status == RemediationStatus::Probed
            {
                f.status = RemediationStatus::InFlight;
                f.updated_at = now;
                changed = true;
            }
        }
    }
    if changed {
        save(path, &state)?;
    }
    Ok(())
}

pub fn next_probed_finding(path: &Path, report_path: &Path) -> Option<ActionableFinding> {
    let report_str = report_path.to_string_lossy();
    let state = load(path);
    state
        .findings
        .iter()
        .filter(|f| f.report_path == report_str && f.status == RemediationStatus::Probed)
        .min_by_key(|f| &f.finding_id)
        .map(|f| ActionableFinding {
            finding_id: f.finding_id.clone(),
            title: f.title.clone(),
            kind: match f.kind.as_str() {
                "FAIL" => FindingKind::Fail,
                "GAP" => FindingKind::Gap,
                _ => FindingKind::Action,
            },
            excerpt: f.excerpt.clone(),
        })
}

pub fn next_open_finding(path: &Path, report_path: &Path) -> Option<ActionableFinding> {
    let report_str = report_path.to_string_lossy();
    let state = load(path);
    let mut candidates: Vec<&RemediationFinding> = state
        .findings
        .iter()
        .filter(|f| {
            f.report_path == report_str && f.status == RemediationStatus::Open
        })
        .collect();

    candidates.sort_by(|a, b| {
        let rank = |f: &RemediationFinding| match f.kind.as_str() {
            "FAIL" => 0,
            "GAP" => 1,
            _ => 2,
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.finding_id.cmp(&b.finding_id))
    });

    candidates.first().map(|f| ActionableFinding {
        finding_id: f.finding_id.clone(),
        title: f.title.clone(),
        kind: match f.kind.as_str() {
            "FAIL" => FindingKind::Fail,
            "GAP" => FindingKind::Gap,
            _ => FindingKind::Action,
        },
        excerpt: f.excerpt.clone(),
    })
}

/// When `finding_ids` is empty, all `in_flight` rows for the report are updated (fixer bulk mode).
pub fn record_spawn_outcome(
    path: &Path,
    report_path: &Path,
    task_id: &str,
    finding_ids: &[String],
    verification: &DiscoveryFixVerification,
    written_paths: &[String],
    max_retries: u32,
) -> anyhow::Result<()> {
    let report_str = report_path.to_string_lossy();
    let mut state = load(path);
    let now = Utc::now();
    let mut changed = false;

    let verified_artifacts: Vec<String> = written_paths.to_vec();
    let filter_ids = !finding_ids.is_empty();

    for f in &mut state.findings {
        if f.report_path != report_str || f.status != RemediationStatus::InFlight {
            continue;
        }
        if filter_ids && !finding_ids.iter().any(|id| id == &f.finding_id) {
            continue;
        }
        changed = true;
        if !f.task_ids.contains(&task_id.to_string()) {
            f.task_ids.push(task_id.to_string());
        }

        if verification.passed {
            f.status = RemediationStatus::Fixed;
            if !verified_artifacts.is_empty() {
                f.artifact_paths = verified_artifacts.clone();
            }
            f.last_verify_notes = if verification.notes.is_empty() {
                None
            } else {
                Some(verification.notes.clone())
            };
        } else {
            f.spawn_attempts += 1;
            f.last_verify_notes = Some(verification.notes.clone());
            if f.spawn_attempts > max_retries {
                f.status = RemediationStatus::Failed;
            } else if f.kind == "ACTION" {
                f.status = RemediationStatus::Probed;
            } else {
                f.status = RemediationStatus::Open;
            }
        }
        f.updated_at = now;
    }

    if changed {
        save(path, &state)?;
        write_spawn_snapshot(
            task_id,
            &state,
            report_path,
            finding_ids,
            verification,
            written_paths,
        );
    }
    Ok(())
}

/// Jules SessionSnapshot — persist timeline after each spawn outcome.
fn write_spawn_snapshot(
    task_id: &str,
    state: &RemediationTrackerState,
    report_path: &Path,
    finding_ids: &[String],
    verification: &DiscoveryFixVerification,
    written_paths: &[String],
) {
    let report_str = report_path.to_string_lossy();
    let session_id = state
        .findings
        .iter()
        .find(|f| {
            f.report_path == report_str
                && (finding_ids.is_empty() || finding_ids.iter().any(|id| id == &f.finding_id))
        })
        .map(|f| f.discovery_session_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let mut activities = Vec::new();
    let activity_type = if verification.passed {
        crate::remediation_snapshot::RemediationActivityType::SessionCompleted
    } else {
        crate::remediation_snapshot::RemediationActivityType::SessionFailed
    };
    crate::remediation_snapshot::append_activity(
        &mut activities,
        activity_type,
        verification.notes.clone(),
        crate::remediation_snapshot::extract_shell_exit_codes(&verification.notes)
            .last()
            .copied(),
    );

    let snapshot = crate::remediation_snapshot::build_snapshot(
        task_id,
        &session_id,
        report_path,
        verification,
        written_paths,
        activities,
    );
    if let Err(e) = crate::remediation_snapshot::write_snapshot(&snapshot) {
        tracing::warn!(error = %e, task_id = %task_id, "remediation snapshot write failed");
    }
}

pub fn emit_discovery_fix_closed(
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    task_id: &str,
    report_path: &Path,
    verification: &DiscoveryFixVerification,
    closed_finding_ids: &[String],
) -> Uuid {
    let corr = Uuid::parse_str(&rec.session_id).ok();
    let reply = Uuid::parse_str(&rec.event_id).ok();
    let event = SynapseEvent::with_envelope(
        EventType::DiscoveryFixClosed,
        EventSource::GzmoDaemon,
        corr,
        reply,
        Some(serde_json::json!({
            "recommendation_id": rec.event_id,
            "session_id": rec.session_id,
            "task_id": task_id,
            "report_path": report_path.display().to_string(),
            "finding_ids": closed_finding_ids,
            "verify_notes": verification.notes,
        })),
    );
    let id = event.id;
    let _ = bus.append(&event);
    id
}

pub fn emit_discovery_fix_failed(
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    task_id: &str,
    report_path: &Path,
    verification: &DiscoveryFixVerification,
    attempt: u32,
) -> Uuid {
    let corr = Uuid::parse_str(&rec.session_id).ok();
    let reply = Uuid::parse_str(&rec.event_id).ok();
    let event = SynapseEvent::with_envelope(
        EventType::DiscoveryFixFailed,
        EventSource::GzmoDaemon,
        corr,
        reply,
        Some(serde_json::json!({
            "recommendation_id": rec.event_id,
            "session_id": rec.session_id,
            "task_id": task_id,
            "report_path": report_path.display().to_string(),
            "attempt": attempt,
            "verify_notes": verification.notes,
            "missing_paths": verification.missing_paths,
            "hit_max_iterations": verification.hit_max_iterations,
        })),
    );
    let id = event.id;
    let _ = bus.append(&event);
    id
}

pub fn emit_remediation_escalated(
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    task_id: &str,
    report_path: &Path,
    finding_id: &str,
    verify_notes: &str,
) -> Uuid {
    let corr = Uuid::parse_str(&rec.session_id).ok();
    let reply = Uuid::parse_str(&rec.event_id).ok();
    let event = SynapseEvent::with_envelope(
        EventType::RemediationEscalated,
        EventSource::GzmoDaemon,
        corr,
        reply,
        Some(serde_json::json!({
            "recommendation_id": rec.event_id,
            "session_id": rec.session_id,
            "task_id": task_id,
            "report_path": report_path.display().to_string(),
            "finding_id": finding_id,
            "escalation_reason": "max_retries_exhausted",
            "verify_notes": verify_notes,
        })),
    );
    let id = event.id;
    let _ = bus.append(&event);
    id
}


/// Revert a single `in_flight` row after spawn gateway failure (before verify).
pub fn reset_in_flight_finding(
    path: &Path,
    report_path: &Path,
    finding_id: &str,
    kind: FindingKind,
) -> anyhow::Result<()> {
    let report_str = report_path.to_string_lossy();
    let kind_str = kind.as_str();
    let mut state = load(path);
    let now = Utc::now();
    let mut changed = false;
    for f in &mut state.findings {
        if f.report_path != report_str
            || f.finding_id != finding_id
            || f.kind != kind_str
            || f.status != RemediationStatus::InFlight
        {
            continue;
        }
        f.spawn_attempts += 1;
        f.status = if f.kind == "ACTION" {
            RemediationStatus::Probed
        } else {
            RemediationStatus::Open
        };
        f.last_verify_notes = Some("spawn gateway error — reverted from in_flight".into());
        f.updated_at = now;
        changed = true;
    }
    if changed {
        save(path, &state)?;
    }
    Ok(())
}

pub fn in_flight_finding_ids(path: &Path, report_path: &Path) -> Vec<String> {
    let report_str = report_path.to_string_lossy();
    load(path)
        .findings
        .iter()
        .filter(|f| f.report_path == report_str && f.status == RemediationStatus::InFlight)
        .map(|f| f.finding_id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery_fixer::FindingKind;

    fn temp_tracker(suffix: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "remediation-tracker-test-{}-{}",
            std::process::id(),
            suffix
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tracker.json");
        let report = dir.join("cycle-1.md");
        (path, report)
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn register_and_close_finding() {
        let (path, report) = temp_tracker("register");
        let dir = path.parent().unwrap().to_path_buf();

        let findings = vec![ActionableFinding {
            finding_id: "F2".into(),
            title: "Orphans".into(),
            kind: FindingKind::Fail,
            excerpt: "39 orphans".into(),
        }];
        register_findings_from_report(&path, &report, "sess-1", &findings).unwrap();
        assert_eq!(load(&path).findings.len(), 1);

        mark_all_open_in_flight(&path, &report).unwrap();
        assert_eq!(
            load(&path).findings[0].status,
            RemediationStatus::InFlight
        );

        let verification = DiscoveryFixVerification {
            passed: true,
            missing_paths: vec![],
            hit_max_iterations: false,
            notes: "verified file_write: scripts/fix.sh".into(),
            acceptance_failed: vec![],
        };
        record_spawn_outcome(
            &path,
            &report,
            "task-1",
            &[],
            &verification,
            &["scripts/fix.sh".into()],
            1,
        )
        .unwrap();
        let state = load(&path);
        assert_eq!(state.findings[0].status, RemediationStatus::Fixed);
        assert_eq!(state.summary().fixed, 1);
        cleanup(&dir);
    }

    #[test]
    fn failed_verify_reopens_until_max_retries() {
        let (path, report) = temp_tracker("retry");
        let dir = path.parent().unwrap().to_path_buf();
        let findings = vec![ActionableFinding {
            finding_id: "F1".into(),
            title: "Gap".into(),
            kind: FindingKind::Gap,
            excerpt: "missing".into(),
        }];
        register_findings_from_report(&path, &report, "s", &findings).unwrap();
        mark_all_open_in_flight(&path, &report).unwrap();

        let fail = DiscoveryFixVerification {
            passed: false,
            missing_paths: vec!["x.sh".into()],
            hit_max_iterations: true,
            notes: "missing".into(),
            acceptance_failed: vec![],
        };
        record_spawn_outcome(&path, &report, "t1", &[], &fail, &[], 1).unwrap();
        assert_eq!(load(&path).findings[0].status, RemediationStatus::Open);
        assert_eq!(load(&path).findings[0].spawn_attempts, 1);

        mark_finding_in_flight(&path, &report, "F1", FindingKind::Gap).unwrap();
        record_spawn_outcome(&path, &report, "t2", &[], &fail, &[], 1).unwrap();
        assert_eq!(load(&path).findings[0].status, RemediationStatus::Failed);
        cleanup(&dir);
    }

    #[test]
    fn record_spawn_outcome_scoped_to_finding_ids() {
        let (path, report) = temp_tracker("scoped");
        let dir = path.parent().unwrap().to_path_buf();
        register_findings_from_report(
            &path,
            &report,
            "s",
            &[
                ActionableFinding {
                    finding_id: "F1".into(),
                    title: "A".into(),
                    kind: FindingKind::Action,
                    excerpt: "a".into(),
                },
                ActionableFinding {
                    finding_id: "F2".into(),
                    title: "B".into(),
                    kind: FindingKind::Action,
                    excerpt: "b".into(),
                },
            ],
        )
        .unwrap();
        mark_finding_in_flight(&path, &report, "F1", FindingKind::Action).unwrap();
        mark_finding_in_flight(&path, &report, "F2", FindingKind::Action).unwrap();

        let verification = DiscoveryFixVerification {
            passed: true,
            missing_paths: vec![],
            hit_max_iterations: false,
            notes: "ok".into(),
            acceptance_failed: vec![],
        };
        record_spawn_outcome(
            &path,
            &report,
            "task-1",
            &["F1".into()],
            &verification,
            &["scripts/a.sh".into()],
            1,
        )
        .unwrap();

        let state = load(&path);
        let f1 = state.findings.iter().find(|f| f.finding_id == "F1").unwrap();
        let f2 = state.findings.iter().find(|f| f.finding_id == "F2").unwrap();
        assert_eq!(f1.status, RemediationStatus::Fixed);
        assert_eq!(f2.status, RemediationStatus::InFlight);
        cleanup(&dir);
    }

    #[test]
    fn next_open_prefers_fail_over_gap() {
        let (path, report) = temp_tracker("prefer-fail");
        let dir = path.parent().unwrap().to_path_buf();
        register_findings_from_report(
            &path,
            &report,
            "s",
            &[
                ActionableFinding {
                    finding_id: "F2".into(),
                    title: "G".into(),
                    kind: FindingKind::Gap,
                    excerpt: "g".into(),
                },
                ActionableFinding {
                    finding_id: "F1".into(),
                    title: "F".into(),
                    kind: FindingKind::Fail,
                    excerpt: "f".into(),
                },
            ],
        )
        .unwrap();
        let next = next_open_finding(&path, &report).unwrap();
        assert_eq!(next.finding_id, "F1");
        assert_eq!(next.kind, FindingKind::Fail);
        cleanup(&dir);
    }
}
