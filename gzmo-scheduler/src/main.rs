//! gzmo-scheduler — thin cron runner for GZMO-next.
//!
//! Loads the instance config, ticks every 60s, and spawns Little Tools Lab
//! recipe scripts. No inline engines, no vault, no LLM client, no MCP:
//! everything cognitive lives in the flat piece CLIs behind the recipes.

mod config;
mod cron;
mod jobs;
mod spawn;

use anyhow::{bail, Context, Result};
use chrono::{Datelike, NaiveDate, Utc};
use config::SchedulerConfig;
use serde_json::json;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, info};

const PID_FILE: &str = "/tmp/gzmo-scheduler.pid";

fn runs_dir(cfg: &SchedulerConfig) -> PathBuf {
    cfg.memory
        .vault_db
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("scheduler-runs")
}

fn write_job_result(
    cfg: &SchedulerConfig,
    job: &str,
    script: &str,
    args: &[String],
    started: chrono::DateTime<Utc>,
    ok: bool,
    error: Option<String>,
) {
    let dir = runs_dir(cfg);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        error!(%e, "scheduler-runs mkdir failed");
        return;
    }
    let finished = Utc::now();
    let stamp = finished.format("%Y%m%dT%H%M%SZ");
    let path = dir.join(format!("{job}-{stamp}.json"));
    let payload = json!({
        "job": job,
        "script": script,
        "args": args,
        "started": started.to_rfc3339(),
        "finished": finished.to_rfc3339(),
        "ok": ok,
        "error": error,
    });
    if let Err(e) = std::fs::write(
        &path,
        serde_json::to_string_pretty(&payload).unwrap_or_default() + "\n",
    ) {
        error!(%e, path = %path.display(), "scheduler-runs write failed");
        return;
    }
    let latest = dir.join("latest.json");
    let _ = std::fs::copy(&path, &latest);
    let job_latest = dir.join(format!("latest-{job}.json"));
    let _ = std::fs::copy(&path, &job_latest);
    info!(job, path = %path.display(), ok, "job result recorded");
}

async fn run_gzmo_job(
    cfg: &SchedulerConfig,
    job: &'static str,
    script: &'static str,
    args: Vec<String>,
) -> bool {
    let started = Utc::now();
    info!(job, script, "job starting");
    let (ok, err) = match spawn::run_gzmo_script(cfg, script, &args).await {
        Ok(()) => {
            info!(job, script, exit = 0, "job complete");
            (true, None)
        }
        Err(e) => {
            error!(job, script, "job failed: {e}");
            (false, Some(e.to_string()))
        }
    };
    write_job_result(cfg, job, script, &args, started, ok, err);
    ok
}

async fn run_job(
    cfg: &SchedulerConfig,
    job: &'static str,
    script: &'static str,
    args: Vec<String>,
) -> bool {
    let started = Utc::now();
    info!(job, script, "job starting");
    let (ok, err) = match spawn::run_lab_script(cfg, script, &args).await {
        Ok(()) => {
            info!(job, script, exit = 0, "job complete");
            (true, None)
        }
        Err(e) => {
            error!(job, script, "job failed: {e}");
            (false, Some(e.to_string()))
        }
    };
    write_job_result(cfg, job, script, &args, started, ok, err);
    ok
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    if std::env::var("GZMO_INSTANCE").as_deref() != Ok("next") {
        bail!("gzmo-scheduler is the GZMO-next runner: set GZMO_INSTANCE=next");
    }
    let (cfg, config_path) = SchedulerConfig::load()?;

    // Singleton lock — separate from legacy /tmp/gzmo_rust.pid so the
    // transitional `gzmo daemon` path can coexist on the same host.
    let pid_file = PathBuf::from(PID_FILE);
    if pid_file.exists() {
        if let Ok(old) = std::fs::read_to_string(&pid_file) {
            let old = old.trim();
            if std::path::Path::new(&format!("/proc/{old}/cmdline")).exists() {
                bail!("gzmo-scheduler already running (PID {old}, lockfile {PID_FILE})");
            }
            tracing::warn!(stale_pid = %old, "Reclaiming stale PID lockfile");
            let _ = std::fs::remove_file(&pid_file);
        }
    }
    let mut lock = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pid_file)
        .with_context(|| format!("acquire PID lockfile {PID_FILE}"))?;
    write!(lock, "{}", std::process::id())?;
    drop(lock);

    info!(
        config = %config_path.display(),
        lab = %spawn::lab_root().display(),
        vault = %cfg.memory.vault_db.display(),
        "gzmo-scheduler online — cron runner, spawns lab recipes only"
    );

    let result = run_loop(&cfg, &config_path).await;
    let _ = std::fs::remove_file(&pid_file);
    result
}

async fn run_loop(cfg: &SchedulerConfig, config_path: &std::path::Path) -> Result<()> {
    // Startup ops health — one shot, non-fatal.
    let (script, args) = jobs::ops_args();
    run_job(cfg, "ops_health", script, args).await;

    let mut last_dream: Option<NaiveDate> = None;
    let mut last_distill: Option<NaiveDate> = None;
    let mut last_qdrant_sync: Option<NaiveDate> = None;
    let mut last_handoff: Option<NaiveDate> = None;
    let mut last_recall: Option<NaiveDate> = None;
    let mut last_ingest: Option<NaiveDate> = None;
    let mut last_kg: Option<NaiveDate> = None;
    let mut last_wiki_push: Option<NaiveDate> = None;
    let mut last_pedagogy: Option<NaiveDate> = None;
    let mut last_cabinet: Option<NaiveDate> = None;
    let mut last_spark: HashSet<(u32, u32, NaiveDate)> = HashSet::new();

    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let now = Utc::now();
        let today = now.date_naive();

        if cfg.dreams.enabled
            && cron::cron_due_today(
                &now,
                cfg.dreams.cron_hour,
                cfg.dreams.cron_minute,
                last_dream,
            )
        {
            let (script, args) = jobs::dream_args(cfg);
            if run_job(cfg, "dream", script, args).await {
                last_dream = Some(today);
            }
        }

        if cfg.qdrant.enabled
            && cfg.qdrant.sync_enabled
            && cron::cron_due_today(
                &now,
                cfg.qdrant.sync_cron_hour,
                cfg.qdrant.sync_cron_minute,
                last_qdrant_sync,
            )
            && run_gzmo_job(cfg, "qdrant_sync", jobs::qdrant_sync_script(), vec![]).await
        {
            last_qdrant_sync = Some(today);
        }

        // Batch ingest before distill so new inbox files can enter tonight's distill.
        if cfg.ingest.batch_enabled
            && cron::cron_due_today(
                &now,
                cfg.ingest.cron_hour,
                cfg.ingest.cron_minute,
                last_ingest,
            )
        {
            let (script, args) = jobs::ingest_batch_args(cfg);
            if run_job(cfg, "ingest_batch", script, args).await {
                last_ingest = Some(today);
            }
        }

        if cfg.session_distill.enabled
            && cfg.session_distill.daemon_scheduled
            && cron::cron_due_today(
                &now,
                cfg.session_distill.cron_hour,
                cfg.session_distill.cron_minute,
                last_distill,
            )
        {
            let (script, args) = jobs::distill_args();
            if run_job(cfg, "distill", script, args).await {
                last_distill = Some(today);
            }
        }

        if cfg.spark.enabled {
            if let Some((h, m)) = cron::cron_slot_due(
                &now,
                &cfg.spark.cron_hours,
                cfg.spark.cron_minute,
                &last_spark,
            ) {
                let (script, args) = jobs::spark_args(cfg);
                if run_job(cfg, "spark", script, args).await {
                    last_spark.insert((h, m, today));
                }
            }
        }

        if cron::cron_due_today(
            &now,
            jobs::HANDOFF_CRON_HOUR,
            jobs::HANDOFF_CRON_MINUTE,
            last_handoff,
        ) {
            let (script, args) = jobs::handoff_args(config_path);
            // Gate-fail exits non-zero by design (hold previous config) — either
            // way the slot is consumed for today.
            run_job(cfg, "config_handoff", script, args).await;
            last_handoff = Some(today);
        }

        if cfg.kg_reconcile.enabled
            && cron::cron_due_today(
                &now,
                cfg.kg_reconcile.cron_hour,
                cfg.kg_reconcile.cron_minute,
                last_kg,
            )
        {
            let (script, args) = jobs::kg_reconcile_args(cfg);
            if run_job(cfg, "kg_reconcile", script, args).await {
                last_kg = Some(today);
            }
        }

        // Weekly recall floor (Sunday) → data-next/recall-report.json
        if now.weekday() == jobs::RECALL_CRON_WEEKDAY
            && cron::cron_due_today(
                &now,
                jobs::RECALL_CRON_HOUR,
                jobs::RECALL_CRON_MINUTE,
                last_recall,
            )
        {
            let args = jobs::recall_eval_args(cfg);
            if run_gzmo_job(cfg, "recall_eval", "recall-eval-weekly.sh", args).await {
                last_recall = Some(today);
            }
        }

        // OKForge wiki catch-up (recipe hooks are primary; this covers misses).
        if cfg.wiki.enabled
            && cfg.wiki.backend == "okforge"
            && cron::cron_due_today(
                &now,
                cfg.wiki.push_cron_hour,
                cfg.wiki.push_cron_minute,
                last_wiki_push,
            )
        {
            let (script, args) = jobs::wiki_okforge_push_args(cfg);
            if run_job(cfg, "wiki_okforge_push", script, args).await {
                last_wiki_push = Some(today);
            }
        }

        // Weekly pedagogy (Sunday) — ADR-0002 amended.
        if cfg.pedagogy.enabled
            && now.weekday().num_days_from_sunday() == cfg.pedagogy.cron_weekday
            && cron::cron_due_today(
                &now,
                cfg.pedagogy.cron_hour,
                cfg.pedagogy.cron_minute,
                last_pedagogy,
            )
        {
            let (script, args) = jobs::pedagogy_args(cfg);
            if run_job(cfg, "pedagogy", script, args).await {
                last_pedagogy = Some(today);
            }
        }

        // Weekly cabinet one-shot (Sunday) — not PulseLoop.
        if cfg.cabinet.enabled
            && now.weekday().num_days_from_sunday() == cfg.cabinet.cron_weekday
            && cron::cron_due_today(
                &now,
                cfg.cabinet.cron_hour,
                cfg.cabinet.cron_minute,
                last_cabinet,
            )
        {
            let (script, args) = jobs::cabinet_feed_args(cfg);
            if run_job(cfg, "cabinet_feed", script, args).await {
                last_cabinet = Some(today);
            }
        }
    }
}
