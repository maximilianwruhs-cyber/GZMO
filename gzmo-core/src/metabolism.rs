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

fn read_job_latest(dir: &Path, job: &str) -> Option<JobRunRecord> {
    let path = dir.join(format!("latest-{job}.json"));
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Markdown section: did last night's metabolism work?
pub fn format_overnight_metabolism(config: &GzmoConfig) -> String {
    let dir = runs_dir(config);
    let mut out = String::from("### Overnight metabolism\n\n");
    out.push_str(&format!("- **Runs dir:** `{}`\n", dir.display()));

    if let Ok(raw) = std::fs::read_to_string(dir.join("latest.json")) {
        if let Ok(v) = serde_json::from_str::<JobRunRecord>(&raw) {
            let mark = if v.ok { "OK" } else { "FAIL" };
            out.push_str(&format!(
                "- **Latest job:** `{mark}` `{}` at {} ({})\n",
                v.job,
                v.finished,
                v.runner.as_deref().unwrap_or(v.script.as_str())
            ));
            if let Some(err) = &v.error {
                out.push_str(&format!("  - error: {err}\n"));
            }
        }
    } else {
        out.push_str("- **Latest job:** none yet — start `gzmo serve`\n");
    }

    let jobs = ["distill", "promote", "embed", "dream", "spark"];
    out.push_str("\n| Job | Last run | Result |\n|---|---|---|\n");
    let mut ok_count = 0usize;
    let mut seen = 0usize;
    for job in jobs {
        match read_job_latest(&dir, job) {
            Some(r) => {
                seen += 1;
                if r.ok {
                    ok_count += 1;
                }
                let mark = if r.ok { "OK" } else { "FAIL" };
                out.push_str(&format!("| {job} | {} | {mark} |\n", r.finished));
            }
            None => {
                out.push_str(&format!("| {job} | — | missing |\n"));
            }
        }
    }

    // Vault / honeypot felt-artifact signal
    let vault = &config.memory.vault_db;
    let (honeypot_n, missing_embed) = vault_metabolism_counts(vault);
    out.push_str(&format!(
        "\n- **Honeypot rows:** {}\n- **Vault facts missing embeddings:** {}\n",
        honeypot_n
            .map(|n| n.to_string())
            .unwrap_or_else(|| "?".into()),
        missing_embed
            .map(|n| n.to_string())
            .unwrap_or_else(|| "?".into())
    ));

    let verdict = if seen == 0 {
        "RED — no metabolism runs recorded"
    } else if ok_count >= 3 && honeypot_n.unwrap_or(0) > 0 {
        "GREEN — core jobs ok and honeypot non-empty"
    } else if ok_count > 0 {
        "YELLOW — some jobs ran; check honeypot / embed seam"
    } else {
        "RED — recent jobs failed"
    };
    out.push_str(&format!("\n**Verdict:** {verdict}\n\n"));
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
