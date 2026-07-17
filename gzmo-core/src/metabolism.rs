//! Overnight memory metabolism — job run telemetry and status for `gzmo serve` / `gzmo status`.
//!
//! Product gate (ADR-0003): distill → promote → embed → dream/spark must leave felt artifacts.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::GzmoConfig;

/// One recorded overnight job result under `{vault_parent}/scheduler-runs/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRunRecord {
    pub job: String,
    #[serde(default)]
    pub script: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub started: String,
    pub finished: String,
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
    /// Typed runner marker (`rust` vs lab script name).
    #[serde(default)]
    pub runner: Option<String>,
}

pub fn runs_dir(config: &GzmoConfig) -> PathBuf {
    config
        .memory
        .vault_db
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("scheduler-runs")
}

pub fn write_job_run(
    config: &GzmoConfig,
    job: &str,
    runner: &str,
    started: DateTime<Utc>,
    ok: bool,
    error: Option<String>,
) -> PathBuf {
    let dir = runs_dir(config);
    let _ = std::fs::create_dir_all(&dir);
    let finished = Utc::now();
    let stamp = finished.format("%Y%m%dT%H%M%SZ");
    let path = dir.join(format!("{job}-{stamp}.json"));
    let payload = JobRunRecord {
        job: job.to_string(),
        script: runner.to_string(),
        args: vec![],
        started: started.to_rfc3339(),
        finished: finished.to_rfc3339(),
        ok,
        error,
        runner: Some(runner.to_string()),
    };
    let _ = std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload).unwrap_or_default() + "\n",
    );
    let latest = dir.join("latest.json");
    let _ = std::fs::copy(&path, &latest);
    // Also maintain per-job latest for status aggregation.
    let job_latest = dir.join(format!("latest-{job}.json"));
    let _ = std::fs::copy(&path, &job_latest);
    path
}

pub const METABOLISM_JOBS: &[&str] = &["distill", "promote", "embed", "dream", "spark"];

fn read_job_latest(dir: &Path, job: &str) -> Option<JobRunRecord> {
    let path = dir.join(format!("latest-{job}.json"));
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Newest metabolism job only — ignores lab wiki/handoff noise in `latest.json`.
pub fn newest_metabolism_job(dir: &Path) -> Option<JobRunRecord> {
    let mut best: Option<JobRunRecord> = None;
    for job in METABOLISM_JOBS {
        let Some(r) = read_job_latest(dir, job) else {
            continue;
        };
        match &best {
            None => best = Some(r),
            Some(prev) if r.finished > prev.finished => best = Some(r),
            _ => {}
        }
    }
    best
}

/// Per-job row for the metabolism TUI / status table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobRowStatus {
    Ok,
    Fail,
    Missing,
}

impl JobRowStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Fail => "FAIL",
            Self::Missing => "missing",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetabolismJobRow {
    pub job: String,
    pub finished: Option<String>,
    pub status: JobRowStatus,
    pub error: Option<String>,
    pub runner: Option<String>,
}

/// Wiki / OKForge plane summary from `wiki-push-latest.json`.
#[derive(Debug, Clone)]
pub struct WikiPlaneSummary {
    pub healthy: Option<bool>,
    pub concepts_written: u64,
    pub commit_sha: String,
    pub detail: String,
}

/// Overnight metabolism board for `gzmo metabolism`.
#[derive(Debug, Clone)]
pub struct MetabolismBoard {
    pub runs_dir: PathBuf,
    pub jobs: Vec<MetabolismJobRow>,
    pub newest: Option<JobRunRecord>,
    pub honeypot_rows: Option<usize>,
    pub missing_embeddings: Option<usize>,
    pub verdict: String,
    pub wiki: WikiPlaneSummary,
}

/// Collect structured overnight metabolism + wiki plane snapshot.
pub fn collect_metabolism_board(config: &GzmoConfig) -> MetabolismBoard {
    let dir = runs_dir(config);
    let mut jobs = Vec::with_capacity(METABOLISM_JOBS.len());
    let mut ok_count = 0usize;
    let mut seen = 0usize;

    for job in METABOLISM_JOBS {
        match read_job_latest(&dir, job) {
            Some(r) => {
                seen += 1;
                if r.ok {
                    ok_count += 1;
                }
                jobs.push(MetabolismJobRow {
                    job: (*job).to_string(),
                    finished: Some(r.finished.clone()),
                    status: if r.ok {
                        JobRowStatus::Ok
                    } else {
                        JobRowStatus::Fail
                    },
                    error: r.error.clone(),
                    runner: r.runner.clone().or(Some(r.script.clone())),
                });
            }
            None => {
                jobs.push(MetabolismJobRow {
                    job: (*job).to_string(),
                    finished: None,
                    status: JobRowStatus::Missing,
                    error: None,
                    runner: None,
                });
            }
        }
    }

    let (honeypot_n, missing_embed) = vault_metabolism_counts(&config.memory.vault_db);
    let verdict = if seen == 0 {
        "RED — no metabolism runs recorded".into()
    } else if ok_count >= 3 && honeypot_n.unwrap_or(0) > 0 {
        "GREEN — core jobs ok and honeypot non-empty".into()
    } else if ok_count > 0 {
        "YELLOW — some jobs ran; check honeypot / embed seam".into()
    } else {
        "RED — recent jobs failed".into()
    };

    MetabolismBoard {
        runs_dir: dir.clone(),
        jobs,
        newest: newest_metabolism_job(&dir),
        honeypot_rows: honeypot_n,
        missing_embeddings: missing_embed,
        verdict,
        wiki: read_wiki_plane_summary(config),
    }
}

fn read_wiki_plane_summary(config: &GzmoConfig) -> WikiPlaneSummary {
    let wiki_meta = config
        .memory
        .vault_db
        .parent()
        .map(|p| p.join("wiki-push-latest.json"))
        .unwrap_or_else(|| PathBuf::from("wiki-push-latest.json"));

    match std::fs::read_to_string(&wiki_meta) {
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(v) => {
                let healthy = v.get("healthy").and_then(|x| x.as_bool());
                let sha = v
                    .get("commit_sha")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .chars()
                    .take(12)
                    .collect::<String>();
                let n = v
                    .get("concepts_written")
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0);
                let detail = match healthy {
                    Some(true) => format!("healthy · {n} concepts · sha {sha}"),
                    Some(false) => format!(
                        "UNHEALTHY · {}",
                        v.get("error")
                            .and_then(|x| x.as_str())
                            .or_else(|| v.get("skipped_reason").and_then(|x| x.as_str()))
                            .unwrap_or("see wiki-push-latest.json")
                    ),
                    None => {
                        if sha.is_empty() {
                            "meta present (no healthy flag)".into()
                        } else {
                            format!("ok · {n} concepts · sha {sha}")
                        }
                    }
                };
                WikiPlaneSummary {
                    healthy,
                    concepts_written: n,
                    commit_sha: sha,
                    detail,
                }
            }
            Err(_) => WikiPlaneSummary {
                healthy: None,
                concepts_written: 0,
                commit_sha: String::new(),
                detail: "meta unreadable".into(),
            },
        },
        Err(_) => WikiPlaneSummary {
            healthy: None,
            concepts_written: 0,
            commit_sha: String::new(),
            detail: "no wiki-push-latest.json yet".into(),
        },
    }
}

/// Markdown section: did last night's metabolism work?
pub fn format_overnight_metabolism(config: &GzmoConfig) -> String {
    let board = collect_metabolism_board(config);
    let mut out = String::from("### Overnight metabolism\n\n");
    out.push_str(&format!("- **Runs dir:** `{}`\n", board.runs_dir.display()));

    if let Some(v) = &board.newest {
        let mark = if v.ok { "OK" } else { "FAIL" };
        out.push_str(&format!(
            "- **Latest metabolism job:** `{mark}` `{}` at {} ({})\n",
            v.job,
            v.finished,
            v.runner.as_deref().unwrap_or(v.script.as_str())
        ));
        if let Some(err) = &v.error {
            out.push_str(&format!("  - error: {err}\n"));
        }
    } else {
        out.push_str("- **Latest metabolism job:** none yet — start `gzmo serve`\n");
    }

    out.push_str("\n| Job | Last run | Result |\n|---|---|---|\n");
    for row in &board.jobs {
        let finished = row.finished.as_deref().unwrap_or("—");
        out.push_str(&format!(
            "| {} | {finished} | {} |\n",
            row.job,
            row.status.label()
        ));
    }

    out.push_str(&format!(
        "\n- **Honeypot rows:** {}\n- **Vault facts missing embeddings:** {}\n",
        board
            .honeypot_rows
            .map(|n| n.to_string())
            .unwrap_or_else(|| "?".into()),
        board
            .missing_embeddings
            .map(|n| n.to_string())
            .unwrap_or_else(|| "?".into())
    ));

    out.push_str(&format!("\n**Verdict:** {}\n\n", board.verdict));
    out
}

fn vault_metabolism_counts(path: &Path) -> (Option<usize>, Option<usize>) {
    let Ok(conn) = rusqlite::Connection::open(path) else {
        return (None, None);
    };
    let honeypot = conn
        .query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1",
            [],
            |r| r.get::<_, i64>(0),
        )
        .ok()
        .map(|n| n as usize);
    let missing = conn
        .query_row(
            "SELECT COUNT(*) FROM semantic_vault WHERE embedding IS NULL OR length(embedding) = 0",
            [],
            |r| r.get::<_, i64>(0),
        )
        .ok()
        .map(|n| n as usize);
    (honeypot, missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn metabolism_board_reads_job_fixtures() {
        let root = std::env::temp_dir().join(format!(
            "gzmo-metab-board-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&root);
        let runs = root.join("scheduler-runs");
        fs::create_dir_all(&runs).unwrap();
        let vault = root.join("vault.db");

        let record = JobRunRecord {
            job: "distill".into(),
            script: "rust".into(),
            args: vec![],
            started: "2026-07-16T00:00:00Z".into(),
            finished: "2026-07-16T00:01:00Z".into(),
            ok: true,
            error: None,
            runner: Some("rust".into()),
        };
        fs::write(
            runs.join("latest-distill.json"),
            serde_json::to_string(&record).unwrap(),
        )
        .unwrap();

        let mut config = GzmoConfig::default();
        config.memory.vault_db = vault;
        assert_eq!(runs_dir(&config), runs);

        let board = collect_metabolism_board(&config);
        let distill = board.jobs.iter().find(|j| j.job == "distill").unwrap();
        assert_eq!(distill.status, JobRowStatus::Ok);
        let promote = board.jobs.iter().find(|j| j.job == "promote").unwrap();
        assert_eq!(promote.status, JobRowStatus::Missing);
        assert!(board.newest.is_some());

        let _ = fs::remove_dir_all(&root);
    }
}
