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
use chrono::{NaiveDate, Utc};
use config::SchedulerConfig;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info};

const PID_FILE: &str = "/tmp/gzmo-scheduler.pid";

async fn run_job(job: &'static str, script: &'static str, args: Vec<String>) -> bool {
    info!(job, script, "job starting");
    match spawn::run_lab_script(script, &args).await {
        Ok(()) => {
            info!(job, script, exit = 0, "job complete");
            true
        }
        Err(e) => {
            error!(job, script, "job failed: {e}");
            false
        }
    }
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
    run_job("ops_health", script, args).await;

    let mut last_dream: Option<NaiveDate> = None;
    let mut last_distill: Option<NaiveDate> = None;
    let mut last_handoff: Option<NaiveDate> = None;
    let mut last_spark: Option<(u32, u32, NaiveDate)> = None;

    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let now = Utc::now();
        let today = now.date_naive();

        if cfg.dreams.enabled
            && cron::cron_due_today(&now, cfg.dreams.cron_hour, cfg.dreams.cron_minute, last_dream)
        {
            let (script, args) = jobs::dream_args(cfg);
            if run_job("dream", script, args).await {
                last_dream = Some(today);
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
            if run_job("distill", script, args).await {
                last_distill = Some(today);
            }
        }

        if cfg.spark.enabled {
            if let Some((h, m)) =
                cron::cron_slot_due(&now, &cfg.spark.cron_hours, cfg.spark.cron_minute, last_spark)
            {
                let (script, args) = jobs::spark_args(cfg);
                if run_job("spark", script, args).await {
                    last_spark = Some((h, m, today));
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
            run_job("config_handoff", script, args).await;
            last_handoff = Some(today);
        }
    }
}
