//! `gzmo serve` — thin overnight metabolism runner (ADR-0003).
//!
//! Core jobs (GREEN gate): distill → promote → embed → dream/spark.
//! Soft-fail satellite: OKForge wiki push (`[wiki]` cron) — not on GREEN verdict.
//! No chaos / KG on this path. Writes `scheduler-runs/`.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::{Datelike, NaiveDate, Timelike, Utc, Weekday};
use gzmo_core::cron;
use tracing::{error, info, warn};

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::{GatewayRouter, LlmGateway};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::qdrant_sync;
use gzmo_core::memory::scratch::ScratchService;
use gzmo_core::metabolism;
use gzmo_core::session_distill::{run_distill_worker, SessionDistillEngine};
use gzmo_core::synapse::{set_event_source, SynapseBus};
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;
use gzmo_core::wiki_okf;

use crate::{cron_cmd, distill_cmd, dream_cmd, embed_cmd, promote_cmd, spark_cmd};

const PID_FILE: &str = "/tmp/gzmo-serve.pid";

fn cron_due_today(
    now: &chrono::DateTime<Utc>,
    hour: u32,
    minute: u32,
    last: Option<NaiveDate>,
) -> bool {
    let today = now.date_naive();
    if last == Some(today) {
        return false;
    }
    let now_mins = now.hour() * 60 + now.minute();
    now_mins >= hour * 60 + minute
}

fn spark_slot_due(
    now: &chrono::DateTime<Utc>,
    hours: &[u32],
    minute: u32,
    done: &HashSet<(u32, u32, NaiveDate)>,
) -> Option<(u32, u32)> {
    let today = now.date_naive();
    let now_mins = now.hour() * 60 + now.minute();
    hours
        .iter()
        .copied()
        .filter(|&h| now_mins >= h * 60 + minute && !done.contains(&(h, minute, today)))
        .min()
        .map(|h| (h, minute))
}

async fn run_named_job<F, Fut>(config: &GzmoConfig, job: &str, f: F) -> bool
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let started = Utc::now();
    info!(job, "metabolism job starting");
    match f().await {
        Ok(()) => {
            metabolism::write_job_run(config, job, "rust", started, true, None);
            info!(job, "metabolism job complete");
            true
        }
        Err(e) => {
            error!(job, error = %e, "metabolism job failed");
            metabolism::write_job_run(config, job, "rust", started, false, Some(e.to_string()));
            false
        }
    }
}

fn project_root() -> PathBuf {
    if let Ok(root) = std::env::var("GZMO_CLONE_ROOT") {
        return PathBuf::from(root).join("GZMO");
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

async fn spawn_distill_worker(config: &GzmoConfig) -> Result<()> {
    set_event_source(gzmo_core::synapse::EventSource::GzmoDaemon);

    let router = GatewayRouter::new(config);
    let extract_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DistillExtract));
    let verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DistillVerify));
    let summary_gateway: Option<Arc<dyn LlmGateway>> = config
        .session_distill
        .librarian_summary
        .then(|| Arc::clone(router.gateway(TaskKind::DistillSummary)))
        .filter(|_| config.librarian.enabled);

    let vault = Arc::new(
        embeddings::open_vault_with_embeddings(
            &config.memory.vault_db,
            &config.embeddings,
            &config.redis,
            &config.rerank,
            &config.qdrant,
        )
        .await?,
    );
    let scratch =
        Arc::new(ScratchService::from_config(&config.redis, &config.context_memory).await);

    let mut tools = ToolRegistry::new();
    tools.register(Box::new(FileReadTool::default()));
    tools.register(Box::new(FileWriteTool::default()));
    tools.register(Box::new(DirListTool::default()));
    tools.register(Box::new(FileSearchTool::default()));
    tools.register(Box::new(ShellExecTool::default()));
    tools.register(Box::new(WebSearchTool::default()));
    tools.register(Box::new(SysMetricsTool));
    tools.register(Box::new(SysKillTool));
    tools.register(Box::new(MemoryRecordTool {
        vault: Arc::clone(&vault),
    }));
    tools.register(Box::new(MemorySearchTool::new(Arc::clone(&vault))));
    let tools = Arc::new(tools);

    let synapse = Arc::new(SynapseBus::new());
    let engine = Arc::new(SessionDistillEngine::new(
        (*vault).clone(),
        FileEpisodicStore::new(&config.memory.directory),
        &config.session_distill.sessions_dir,
        extract_gateway,
        verify_gateway,
        summary_gateway,
        tools,
        config.session_distill.clone(),
        Some(synapse),
    ));

    tokio::spawn(run_distill_worker(scratch, engine));
    info!("distill BRPOP worker spawned");
    Ok(())
}

pub async fn run(config: &GzmoConfig, identity: &IdentityEngine) -> Result<()> {
    if !config.metabolism.enabled {
        bail!("[metabolism].enabled = false — refusing to start gzmo serve");
    }

    let _owner = gzmo_core::control_plane::claim_owner(config).await?;

    let pid_file = PathBuf::from(PID_FILE);
    if pid_file.exists() {
        if let Ok(old) = std::fs::read_to_string(&pid_file) {
            let old = old.trim();
            if std::path::Path::new(&format!("/proc/{old}/cmdline")).exists() {
                bail!("gzmo serve already running (PID {old}, lockfile {PID_FILE})");
            }
            warn!(stale_pid = %old, "Reclaiming stale serve PID lockfile");
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
        vault = %config.memory.vault_db.display(),
        runs = %metabolism::runs_dir(config).display(),
        "gzmo serve online — overnight metabolism (typed Rust jobs)"
    );

    if let Err(e) = spawn_distill_worker(config).await {
        warn!(error = %e, "distill worker not started (cron distill still runs)");
    }

    let result = run_loop(config, identity).await;
    let _ = std::fs::remove_file(&pid_file);
    result
}

async fn run_wiki_push_satellite(config: &GzmoConfig) -> Result<()> {
    let meta_path = config
        .memory
        .vault_db
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("wiki-push-latest.json");

    // Soft hold: concept-gate HOLD skips OKForge push (does not trip GREEN).
    if let Some(reason) = wiki_okf::concept_gate_hold_reason(&config.memory.vault_db) {
        let report = wiki_okf::WikiPushReport {
            mode: "gated".into(),
            origin: "serve-catchup".into(),
            concepts_written: 0,
            commit_sha: String::new(),
            branch: String::new(),
            skipped_reason: reason.clone(),
            paths: vec![],
            healthy: true, // soft-fail satellite — hold is intentional, not unhealthy
            timestamp: chrono::Utc::now().to_rfc3339(),
            error: String::new(),
        };
        wiki_okf::write_push_report(&meta_path, &report)?;
        warn!(%reason, "OKForge wiki satellite held by concept-gate");
        return Ok(());
    }

    let report = wiki_okf::push_from_vault(
        &config.wiki,
        &config.memory.vault_db,
        "serve-catchup",
        40,
        false,
    )
    .await;
    let report = wiki_okf::record_push_result(&meta_path, "serve-catchup", report)?;
    info!(
        concepts = report.concepts_written,
        sha = %report.commit_sha,
        healthy = report.healthy,
        skipped = %report.skipped_reason,
        "OKForge wiki satellite push"
    );
    if !report.healthy {
        bail!(
            "wiki push unhealthy: skipped={} sha={}",
            report.skipped_reason,
            report.commit_sha
        );
    }
    Ok(())
}

async fn run_loop(config: &GzmoConfig, identity: &IdentityEngine) -> Result<()> {
    let mut last_dream: Option<NaiveDate> = None;
    let mut last_distill: Option<NaiveDate> = None;
    let mut last_promote: Option<NaiveDate> = None;
    let mut last_embed: Option<NaiveDate> = None;
    let mut last_wiki: Option<NaiveDate> = None;
    let mut last_dream_compact: Option<NaiveDate> = None;
    let mut last_spark: HashSet<(u32, u32, NaiveDate)> = HashSet::new();
    // Last fire slot per custom job: (date, hour, minute)
    let mut last_custom: HashMap<String, (NaiveDate, u32, u32)> = HashMap::new();

    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let now = Utc::now();
        let today = now.date_naive();

        // Soft-fail missed-run watchdog (YELLOW in status; never on GREEN job math).
        let wd = metabolism::evaluate_and_write_watchdog(config);
        if wd.stale {
            warn!(detail = %wd.detail, threshold_secs = wd.threshold_secs, "metabolism watchdog stale");
        }

        if config.dreams.enabled
            && cron_due_today(
                &now,
                config.dreams.cron_hour,
                config.dreams.cron_minute,
                last_dream,
            )
        {
            let shift = metabolism::evaluate_price_shift(config, "dream", now);
            info!(note = %shift.note, delay = shift.delay_now, "price-shift soft advice (dream)");
            if shift.delay_now {
                warn!(
                    suggested = ?shift.suggested_start_utc,
                    "GZMO_PRICE_SHIFT=1 — delaying dream until suggested UTC"
                );
            } else if run_named_job(config, "dream", || async {
                dream_cmd::run(config, identity, Some(today)).await
            })
            .await
            {
                last_dream = Some(today);
            }
        }

        if config.session_distill.enabled
            && config.session_distill.daemon_scheduled
            && cron_due_today(
                &now,
                config.session_distill.cron_hour,
                config.session_distill.cron_minute,
                last_distill,
            )
        {
            let shift = metabolism::evaluate_price_shift(config, "distill", now);
            info!(note = %shift.note, delay = shift.delay_now, "price-shift soft advice (distill)");
            if shift.delay_now {
                warn!(
                    suggested = ?shift.suggested_start_utc,
                    "GZMO_PRICE_SHIFT=1 — delaying distill until suggested UTC"
                );
            } else if run_named_job(config, "distill", || async {
                distill_cmd::run(config, None).await
            })
            .await
            {
                last_distill = Some(today);
            }
        }

        if cron_due_today(
            &now,
            config.metabolism.promote_cron_hour,
            config.metabolism.promote_cron_minute,
            last_promote,
        ) {
            if run_named_job(config, "promote", || async {
                promote_cmd::run(config, None).await
            })
            .await
            {
                last_promote = Some(today);
            }
        }

        if cron_due_today(
            &now,
            config.metabolism.embed_cron_hour,
            config.metabolism.embed_cron_minute,
            last_embed,
        ) {
            let root = project_root();
            if run_named_job(config, "embed", || async {
                embed_cmd::run(config, None).await?;
                if config.qdrant.enabled && config.qdrant.sync_enabled {
                    qdrant_sync::sync_vault_to_qdrant(
                        &root,
                        &config.qdrant,
                        &config.memory.vault_db,
                    )
                    .await?;
                }
                Ok(())
            })
            .await
            {
                last_embed = Some(today);
            }
        }

        if config.spark.enabled {
            if let Some((h, m)) = spark_slot_due(
                &now,
                &config.spark.cron_hours,
                config.spark.cron_minute,
                &last_spark,
            ) {
                if run_named_job(config, "spark", || async {
                    spark_cmd::run(config, identity, Some(today)).await
                })
                .await
                {
                    last_spark.insert((h, m, today));
                }
            }
        }

        // Soft-fail: Sunday dream compaction (not on GREEN metabolism gate).
        if now.weekday() == Weekday::Sun && cron_due_today(&now, 3, 0, last_dream_compact) {
            let ok = run_named_job(config, "dream-compact", || async {
                dream_cmd::run_compact(config, None, Some(30), false).await
            })
            .await;
            if !ok {
                warn!("dream-compact soft-fail — metabolism GREEN unaffected");
            }
            last_dream_compact = Some(today);
        }

        // Soft-fail satellite: OKForge wiki feed (not on GREEN metabolism verdict).
        if config.wiki.enabled
            && config.wiki.backend == "okforge"
            && cron_due_today(
                &now,
                config.wiki.push_cron_hour,
                config.wiki.push_cron_minute,
                last_wiki,
            )
        {
            let ok = run_named_job(config, "wiki", || async {
                run_wiki_push_satellite(config).await
            })
            .await;
            if !ok {
                warn!("OKForge wiki satellite failed — metabolism GREEN unaffected");
            }
            // Always consume the slot so a flaky forge does not retry every minute.
            last_wiki = Some(today);
        }

        // Operator custom jobs (`[cron.jobs.*]`) — shell or prompt.
        let custom: Vec<_> = config
            .cron
            .jobs
            .iter()
            .map(|(id, job)| (id.clone(), job.clone()))
            .collect();
        for (id, job) in custom {
            let prev = last_custom.get(&id).copied();
            if !cron::custom_due(&job, now, prev) {
                continue;
            }
            let job_name = format!("custom/{id}");
            let ok = run_named_job(config, &job_name, || async {
                cron_cmd::run_custom_job(config, identity, &id, &job).await
            })
            .await;
            if ok {
                last_custom.insert(id, (today, now.hour(), now.minute()));
            } else {
                // Consume slot on failure to avoid retry storm every minute.
                last_custom.insert(id, (today, now.hour(), now.minute()));
            }
        }
    }
}
