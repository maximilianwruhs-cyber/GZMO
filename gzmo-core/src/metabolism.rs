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

/// Jobs that gate the missed-run watchdog (soft-fail; not the GREEN count alone).
const WATCHDOG_JOBS: &[&str] = &["distill", "dream"];

/// Default: 26h — one missed overnight window with slack.
pub const DEFAULT_METABOLISM_STALE_SECS: u64 = 26 * 3600;

/// Override with `GZMO_METABOLISM_STALE_SECS` (seconds) for burst tests.
pub fn metabolism_stale_threshold_secs() -> u64 {
    std::env::var("GZMO_METABOLISM_STALE_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_METABOLISM_STALE_SECS)
}

/// Soft-fail missed-run / stale metabolism signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogRecord {
    pub job: String,
    pub ok: bool,
    pub stale: bool,
    pub threshold_secs: u64,
    pub checked_at: String,
    pub detail: String,
    #[serde(default)]
    pub ages_secs: serde_json::Map<String, serde_json::Value>,
}

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
    /// Soft-fail stale signal (does not flip GREEN core-job math into RED).
    pub watchdog: WatchdogRecord,
}

/// Evaluate distill/dream age against stale threshold; write `latest-watchdog.json`.
pub fn evaluate_and_write_watchdog(config: &GzmoConfig) -> WatchdogRecord {
    let record = evaluate_missed_run_watchdog(config);
    let dir = runs_dir(config);
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("latest-watchdog.json");
    let _ = std::fs::write(
        &path,
        serde_json::to_string_pretty(&record).unwrap_or_default() + "\n",
    );
    record
}

/// Soft-fail: distill or dream missing / older than threshold → stale.
pub fn evaluate_missed_run_watchdog(config: &GzmoConfig) -> WatchdogRecord {
    let dir = runs_dir(config);
    let threshold = metabolism_stale_threshold_secs();
    let now = Utc::now();
    let mut ages = serde_json::Map::new();
    let mut stale_reasons: Vec<String> = Vec::new();

    for job in WATCHDOG_JOBS {
        match read_job_latest(&dir, job) {
            None => {
                ages.insert((*job).into(), serde_json::Value::Null);
                stale_reasons.push(format!("{job}: missing latest-{job}.json"));
            }
            Some(r) => match DateTime::parse_from_rfc3339(&r.finished) {
                Ok(finished) => {
                    let age = (now - finished.with_timezone(&Utc)).num_seconds().max(0) as u64;
                    ages.insert((*job).into(), serde_json::json!(age));
                    if age > threshold {
                        stale_reasons.push(format!("{job}: {age}s old (threshold {threshold}s)"));
                    }
                }
                Err(_) => {
                    ages.insert((*job).into(), serde_json::Value::Null);
                    stale_reasons.push(format!("{job}: unparseable finished timestamp"));
                }
            },
        }
    }

    let stale = !stale_reasons.is_empty();
    WatchdogRecord {
        job: "watchdog".into(),
        ok: !stale,
        stale,
        threshold_secs: threshold,
        checked_at: now.to_rfc3339(),
        detail: if stale {
            format!("metabolism stale — {}", stale_reasons.join("; "))
        } else {
            "metabolism fresh within threshold".into()
        },
        ages_secs: ages,
    }
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
    let watchdog = evaluate_and_write_watchdog(config);

    // Core GREEN math ignores watchdog; stale only demotes display verdict to YELLOW.
    let mut verdict: String = if seen == 0 {
        "RED — no metabolism runs recorded".into()
    } else if ok_count >= 3 && honeypot_n.unwrap_or(0) > 0 {
        "GREEN — core jobs ok and honeypot non-empty".into()
    } else if ok_count > 0 {
        "YELLOW — some jobs ran; check honeypot / embed seam".into()
    } else {
        "RED — recent jobs failed".into()
    };

    if watchdog.stale && !verdict.starts_with("RED") {
        verdict = format!("YELLOW — {}", watchdog.detail);
    }

    MetabolismBoard {
        runs_dir: dir.clone(),
        jobs,
        newest: newest_metabolism_job(&dir),
        honeypot_rows: honeypot_n,
        missing_embeddings: missing_embed,
        verdict,
        wiki: read_wiki_plane_summary(config),
        watchdog,
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

    out.push_str(&format!(
        "- **Missed-run watchdog:** {} (threshold {}s)\n",
        if board.watchdog.stale {
            format!("STALE — {}", board.watchdog.detail)
        } else {
            "fresh".into()
        },
        board.watchdog.threshold_secs
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

/// Soft Awattar shift advice for distill/dream (sibling note; cron not overwritten).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceShiftAdvice {
    pub job: String,
    pub note: String,
    pub suggested_start_utc: Option<String>,
    pub shift_hours: Option<f64>,
    /// When true and advice says delay, serve should skip firing this minute.
    pub delay_now: bool,
}

fn price_shift_opt_in() -> bool {
    match std::env::var("GZMO_PRICE_SHIFT") {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            t == "1" || t == "true" || t == "yes" || t == "on"
        }
        Err(_) => false,
    }
}

/// Read `price-window/latest.json`, write `scheduler-runs/latest-price-shift.json`,
/// and return whether `job` should be soft-delayed (only when `GZMO_PRICE_SHIFT=1`).
pub fn evaluate_price_shift(
    config: &GzmoConfig,
    job: &str,
    now: DateTime<Utc>,
) -> PriceShiftAdvice {
    let parent = config
        .memory
        .vault_db
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let pw_path = parent.join("price-window").join("latest.json");
    let runs = runs_dir(config);
    let _ = std::fs::create_dir_all(&runs);
    let opt_in = price_shift_opt_in();

    let mut advice = PriceShiftAdvice {
        job: job.to_string(),
        note: "no price-window/latest.json".into(),
        suggested_start_utc: None,
        shift_hours: None,
        delay_now: false,
    };

    let Ok(raw) = std::fs::read_to_string(&pw_path) else {
        let payload = serde_json::json!({
            "schema": "gzmo.price.shift/v1",
            "generated_at": now.to_rfc3339(),
            "ok": false,
            "opt_in": opt_in,
            "detail": advice.note,
            "job": job,
        });
        let _ = std::fs::write(
            runs.join("latest-price-shift.json"),
            payload.to_string() + "\n",
        );
        return advice;
    };

    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        advice.note = "price-window JSON unreadable".into();
        return advice;
    };

    let sug = v
        .pointer(&format!("/suggestions/{job}"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let suggested = sug
        .get("suggested_start_utc")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let nominal = sug
        .get("nominal_utc")
        .and_then(|x| x.as_str())
        .unwrap_or("?");
    let shift_h = sug.get("shift_hours").and_then(|x| x.as_f64());
    let savings = sug.get("savings_c_kwh").and_then(|x| x.as_f64());

    let mut delay_now = false;
    if let Some(ref start) = suggested {
        if let Ok(suggested_dt) = DateTime::parse_from_rfc3339(start) {
            let suggested_utc = suggested_dt.with_timezone(&Utc);
            if opt_in && now < suggested_utc {
                delay_now = true;
            }
        }
        advice.note = format!(
            "would shift {job} from {nominal} → {start} (Δ{shift_h:?}h, save {savings:?} ¢/kWh); metabolism still wins"
        );
    } else {
        advice.note = format!("no suggested_start_utc for {job}");
    }

    advice.suggested_start_utc = suggested.clone();
    advice.shift_hours = shift_h;
    advice.delay_now = delay_now;

    let payload = serde_json::json!({
        "schema": "gzmo.price.shift/v1",
        "generated_at": now.to_rfc3339(),
        "ok": true,
        "opt_in": opt_in,
        "job": job,
        "delay_now": delay_now,
        "suggested_start_utc": suggested,
        "shift_hours": shift_h,
        "note": advice.note,
        "apply": if opt_in { "delay_until_suggested" } else { "log_only" },
    });
    let _ = std::fs::write(
        runs.join("latest-price-shift.json"),
        serde_json::to_string_pretty(&payload).unwrap_or_default() + "\n",
    );
    advice
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
        // dream missing → watchdog stale (soft-fail YELLOW, not RED from watchdog alone)
        assert!(board.watchdog.stale);
        assert!(board.verdict.starts_with("YELLOW"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn watchdog_stale_with_short_threshold() {
        let root = std::env::temp_dir().join(format!(
            "gzmo-metab-watch-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&root);
        let runs = root.join("scheduler-runs");
        fs::create_dir_all(&runs).unwrap();

        let old = "2020-01-01T00:00:00+00:00";
        for job in ["distill", "dream", "promote", "embed", "spark"] {
            let record = JobRunRecord {
                job: job.into(),
                script: "rust".into(),
                args: vec![],
                started: old.into(),
                finished: old.into(),
                ok: true,
                error: None,
                runner: Some("rust".into()),
            };
            fs::write(
                runs.join(format!("latest-{job}.json")),
                serde_json::to_string(&record).unwrap(),
            )
            .unwrap();
        }

        let mut config = GzmoConfig::default();
        config.memory.vault_db = root.join("vault.db");

        std::env::set_var("GZMO_METABOLISM_STALE_SECS", "60");
        let wd = evaluate_missed_run_watchdog(&config);
        std::env::remove_var("GZMO_METABOLISM_STALE_SECS");
        assert!(wd.stale);
        assert!(wd.detail.contains("distill"));

        let _ = fs::remove_dir_all(&root);
    }
}
