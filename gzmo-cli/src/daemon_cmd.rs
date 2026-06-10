//! Daemon mode — heartbeat + dreams + orchestrator.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Timelike, Utc};
use tracing::{error, info};

use gzmo_core::config::GzmoConfig;
use gzmo_core::daemon::{
    cron_due_today, cron_minutes, spark_cron_slot_due, FileChangeCheck, HeartbeatEngine,
    HealthPing,
};
use gzmo_core::dreams::DreamEngine;
use gzmo_core::dreams_md::write_dream_narrative;
use gzmo_core::ingest::IngestEngine;
use gzmo_core::spark::{append_spark_to_dreams, SparkEngine};
use gzmo_core::wiki::WikiEngine;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::config::TaskKind;
use gzmo_core::synapse::set_event_source;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::config::SparkScheduleMode;
use gzmo_core::health;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::scratch::{ScratchScope, ScratchService};
use gzmo_core::session_distill::{run_distill_worker, SessionDistillEngine};
use gzmo_core::synapse::SynapseBus;
use gzmo_core::memory::qdrant_sync::{self, sync_vault_to_qdrant};

use gzmo_core::spark_schedule;
use gzmo_core::mcp::{manager::McpManager, bridge::McpServerConfig};
use gzmo_core::tools::ToolRegistry;
use gzmo_core::tools::fs::{FileReadTool, FileWriteTool, DirListTool, FileSearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysMetricsTool, SysKillTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};

pub async fn run(config: &GzmoConfig, identity: IdentityEngine) -> Result<()> {
    let soul = identity.snapshot().await;

    info!("╔══════════════════════════════════════════════╗");
    info!("║            GZMO — Daemon Mode                ║");
    info!("║       100% Local · Air-Gapped · Rust         ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(persona = %soul.persona_name, "Identity loaded");

    // Set event source for this thread (daemon)
    set_event_source(gzmo_core::synapse::EventSource::GzmoDaemon);

    // Ensure directories
    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    embeddings::assert_vault_backend(&config.memory.vault_backend)?;

    // Heartbeat
    let heartbeat_interval = Duration::from_secs(config.agent.heartbeat_interval_secs);
    let mut heartbeat = HeartbeatEngine::new(heartbeat_interval);
    heartbeat.add_check(FileChangeCheck {
        watch_dir: config.memory.directory.to_string_lossy().to_string(),
        since: Duration::from_secs(config.agent.heartbeat_interval_secs),
    });
    heartbeat.add_check(HealthPing {
        url: format!("{}/models", config.engine.active_engine().url),
        service_name: "LLM Engine".to_string(),
    });

    // Gateway + Tools for dream cycle — use Obolus GatewayRouter
    let router = GatewayRouter::new(config);
    let dream_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamExtract));
    let dream_verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamVerify));
    let ingest_verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::IngestVerify));
    let spark_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::SparkHypothesis));
    let spark_verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::SparkVerify));
    let ingest_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::IngestExtract));

    let dream_vault = embeddings::open_vault_with_embeddings(
        &config.memory.vault_db,
        &config.embeddings,
        &config.redis,
        &config.rerank,
        &config.qdrant,
    )
    .await?;
    if let Err(e) = dream_vault.archive_stale_session_anchors(config.spark.max_session_anchor_age_days) {
        error!("Vault session-anchor cleanup failed: {e}");
    }
    let dream_vault = Arc::new(dream_vault);

    let scratch = Arc::new(
        ScratchService::from_config(&config.redis, &config.context_memory).await,
    );
    let memory_search_scope = Arc::new(std::sync::Mutex::new(ScratchScope::Orch {
        job: "init".to_string(),
        step: "init".to_string(),
    }));

    let mut dream_tools = ToolRegistry::new();
    dream_tools.register(Box::new(FileReadTool::default()));
    dream_tools.register(Box::new(FileWriteTool));
    dream_tools.register(Box::new(DirListTool));
    dream_tools.register(Box::new(FileSearchTool::default()));
    dream_tools.register(Box::new(ShellExecTool::default()));
    dream_tools.register(Box::new(WebSearchTool::default()));
    dream_tools.register(Box::new(SysMetricsTool));
    dream_tools.register(Box::new(SysKillTool));
    dream_tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(&dream_vault) }));
    dream_tools.register(Box::new(MemorySearchTool::with_orchestrator_scratch(
        Arc::clone(&dream_vault),
        Arc::clone(&scratch),
        Arc::clone(&memory_search_scope),
    )));

    // MCP for dreams
    let mut dream_mcp = McpManager::new();
    for server in config.active_mcp_servers() {
        match dream_mcp.connect(McpServerConfig {
            name: server.name.clone(),
            command: server.command.clone(),
            args: server.args.clone(),
            env: server.env.clone(),
        }).await {
            Ok(count) => info!(server = %server.name, tools = count, "Dream MCP connected"),
            Err(e) => error!(server = %server.name, "Dream MCP failed: {}", e),
        }
    }
    dream_mcp.register_all_tools(&mut dream_tools);
    let dream_tools = Arc::new(dream_tools);

    // Initialize Synapse event bus (append-only observability)
    let synapse = Arc::new(SynapseBus::new());
    info!(path = %synapse.path.display(), "SynapseBus initialized");

    let distill_engine = Arc::new(SessionDistillEngine::new(
        (*dream_vault).clone(),
        FileEpisodicStore::new(&config.memory.directory),
        &config.session_distill.sessions_dir,
        Arc::clone(router.gateway(TaskKind::DistillExtract)),
        Arc::clone(router.gateway(TaskKind::DistillVerify)),
        config
            .session_distill
            .librarian_summary
            .then(|| Arc::clone(router.gateway(TaskKind::DistillSummary))),
        Arc::clone(&dream_tools),
        config.session_distill.clone(),
        Some(Arc::clone(&synapse)),
    ));
    let distill_worker_handle = tokio::spawn(run_distill_worker(
        Arc::clone(&scratch),
        Arc::clone(&distill_engine),
    ));
    info!("Distill worker started (archive queue)");

    if let Err(e) = health::run_startup_probes(
        config,
        Some(dream_tools.as_ref()),
        config.health.strict_startup,
        Some(&synapse),
    )
    .await
    {
        error!("Startup health probes failed: {e}");
        if config.health.strict_startup {
            return Err(e);
        }
    }

    let dream_episodic = Arc::new(FileEpisodicStore::new(&config.memory.directory));
    let orch_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::Daemon));
    let dream_engine = Arc::new(DreamEngine::new_with_verify(
        FileEpisodicStore::new(&config.memory.directory),
        (*dream_vault).clone(),
        Arc::clone(&dream_gateway),
        Arc::clone(&dream_verify_gateway),
        Arc::clone(&dream_tools),
        config.dreams.clone(),
        Some(Arc::clone(&synapse)),
    ));
    let dream_engine_clone = Arc::clone(&dream_engine);
    let dreams_path = config.skills.dreams_path.clone();
    let dream_config = config.dreams.clone();

    let spark_engine = Arc::new(SparkEngine::new_with_verify(
        (*dream_vault).clone(),
        FileEpisodicStore::new(&config.memory.directory),
        Arc::clone(&spark_gateway),
        Arc::clone(&spark_verify_gateway),
        Arc::clone(&dream_tools),
        config.spark.clone(),
        Some(Arc::clone(&synapse)),
    ));
    let spark_engine_clone = Arc::clone(&spark_engine);
    let spark_config = config.spark.clone();
    let dreams_path_spark = config.skills.dreams_path.clone();

    let ingest_engine = Arc::new(
        IngestEngine::new_with_verify(
            (*dream_vault).clone(),
            FileEpisodicStore::new(&config.memory.directory),
            Arc::clone(&ingest_gateway),
            Arc::clone(&ingest_verify_gateway),
            Arc::clone(&dream_tools),
            config.ingest.clone(),
            Some(Arc::clone(&synapse)),
        )
        .with_wiki(config.wiki.clone()),
    );
    // ─── Chaos Engine ────────────────────────────────────────────
    // Keep PulseHandle alive for the full daemon lifetime — Drop aborts the loop.
    let chaos_runtime = crate::chaos_bootstrap::start_chaos_runtime(config);
    let chaos_pulse = chaos_runtime.handle;
    let gateway_rwlock = Arc::new(tokio::sync::RwLock::new(orch_gateway.clone()));
    let state_dir = config.memory.vault_db.parent().unwrap_or(std::path::Path::new("data")).to_path_buf();
    let _chaos_bridge = crate::chaos_bootstrap::spawn_snapshot_bridge(
        chaos_pulse.snapshot_rx.clone(),
        gateway_rwlock,
        chaos_pulse.feedback_tx.clone(),
        state_dir,
        None,
        Some(Arc::clone(&synapse)),
        gzmo_core::synapse::EventSource::GzmoDaemon,
        chaos_runtime.restore_policy.clone(),
    );
    info!("All subsystems online — entering daemon loop");
    let ccr = gzmo_core::context_compress::CcrStore::new(&config.redis, &config.context_compress);
    // Orchestrator (cron jobs) — spark is handled by SparkEngine, not headless prompt
    let orch_ctx = Arc::new(gzmo_core::orchestrator::OrchestratorContext {
        gateway: orch_gateway,
        tools: Arc::clone(&dream_tools),
        system_prompt: format!(
            "{}\n\n---\nYou are {} in BACKGROUND MODE.\nToday is {}.\nBe concise and action-oriented.",
            soul.raw_markdown, soul.persona_name, Utc::now().format("%Y-%m-%d %H:%M UTC"),
        ),
        vault: Some(Arc::clone(&dream_vault)),
        episodic: Some(Arc::clone(&dream_episodic)),
        chaos_feedback_tx: Some(chaos_pulse.feedback_tx.clone()),
        ingest_engine: if config.ingest.enabled {
            Some(Arc::clone(&ingest_engine))
        } else {
            None
        },
        synapse: Some(Arc::clone(&synapse)),
        scratch: Arc::clone(&scratch),
        memory_search_scope: Arc::clone(&memory_search_scope),
        context: gzmo_core::context::ContextConfig::from_memory_config(&config.context_memory),
        compress_config: config.context_compress.clone(),
        ccr,
    });

    let mut orch_jobs = config.orchestration.jobs.clone();
    orch_jobs.remove("spark");
    orch_jobs.remove("auto_dream");
    let _scheduler = match gzmo_core::orchestrator::start_orchestrator(orch_jobs, Arc::clone(&orch_ctx)).await {
        Ok(s) => { info!("Orchestrator online"); Some(s) }
        Err(e) => { error!("Orchestrator failed: {e}"); None }
    };

    // Watchers
    let orch_watchers = config.orchestration.watchers.clone();
    if let Err(e) = gzmo_core::watcher::start_watchers(&orch_watchers, orch_ctx).await {
        error!("Watchers failed: {e}");
    }

    // Heartbeat task
    let heartbeat_handle = tokio::spawn(async move {
        heartbeat.run(|anomalies| async move {
            info!(count = anomalies.len(), "Heartbeat triggered");
            for a in &anomalies { info!(anomaly = %a); }
        }).await
    });

    // Dream cycle task (DreamEngine — replaces headless auto_dream orchestrator job)
    let dream_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_consolidated: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !dream_config.enabled {
                continue;
            }
            let now = Utc::now();
            let yesterday = now.date_naive() - chrono::Duration::days(1);
            if now.hour() * 60 + now.minute() < cron_minutes(dream_config.cron_hour, dream_config.cron_minute)
            {
                continue;
            }
            if last_consolidated == Some(yesterday) {
                continue;
            }
            info!(date = %yesterday, "Dream consolidation starting");
            match dream_engine_clone.consolidate(yesterday).await {
                Ok(report) => {
                    last_consolidated = Some(yesterday);
                    info!(
                        entities = report.entities_extracted,
                        kg_relations = report.kg_relations_written,
                        "Dream cycle complete"
                    );
                    if let Err(e) = write_dream_narrative(&dreams_path, &report.narrative).await {
                        error!("Failed to write DREAMS.md: {e}");
                    }
                }
                Err(e) => error!("Dream failed: {e}"),
            }
        }
    });

    // Spark cycle task (chaos-free serendipity via SparkEngine)
    let chaos_seed = config
        .chaos
        .as_ref()
        .and_then(|c| c.get("seed"))
        .and_then(|v| v.as_float())
        .map(|f| (f * 1_000_000.0) as u64)
        .unwrap_or(506);
    let spark_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_run_key: Option<(u32, u32, NaiveDate)> = None;
        let mut next_dice_run: Option<chrono::DateTime<Utc>> = None;
        let mut dice_seed = spark_config.dice_seed.unwrap_or(chaos_seed);
        loop {
            interval.tick().await;
            if !spark_config.enabled {
                continue;
            }
            let now = Utc::now();
            let today = now.date_naive();
            let cron_slot = match spark_config.schedule_mode {
                SparkScheduleMode::Cron => spark_cron_slot_due(
                    &now,
                    &spark_config.cron_hours,
                    spark_config.cron_minute,
                    last_run_key,
                ),
                SparkScheduleMode::Dice => {
                    if next_dice_run.is_none() {
                        let (next, seed, roll, minutes) =
                            spark_schedule::next_spark_after(now, &spark_config, dice_seed);
                        dice_seed = seed;
                        next_dice_run = Some(next);
                        info!(roll, minutes, next = %next, "Spark dice scheduled next run");
                    }
                    if next_dice_run.is_some_and(|t| now >= t) {
                        Some((now.hour(), now.minute()))
                    } else {
                        None
                    }
                }
            };

            let Some((slot_hour, slot_minute)) = cron_slot else {
                continue;
            };

            info!(date = %today, mode = ?spark_config.schedule_mode, hour = slot_hour, minute = slot_minute, "Spark cycle starting");
            match spark_engine_clone.run(today).await {
                Ok(report) => {
                    if spark_config.schedule_mode == SparkScheduleMode::Cron {
                        last_run_key = Some((slot_hour, slot_minute, today));
                    } else {
                        next_dice_run = None;
                    }
                    if let Err(e) = append_spark_to_dreams(&dreams_path_spark, &report.section).await {
                        error!("Failed to append spark to DREAMS.md: {e}");
                    } else {
                        info!(
                            promoted = report.promoted,
                            kg = report.kg_relations_written,
                            "Spark cycle complete"
                        );
                    }
                }
                Err(e) => error!("Spark failed: {e}"),
            }
        }
    });

    // Qdrant mirror — daily sync after dream window (default 01:45 UTC)
    let qdrant_cfg = config.qdrant.clone();
    let vault_db_path = config.memory.vault_db.clone();
    let project_root = qdrant_sync::discover_project_root();
    let qdrant_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_sync_date: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !qdrant_cfg.sync_enabled {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(
                &now,
                qdrant_cfg.sync_cron_hour,
                qdrant_cfg.sync_cron_minute,
                last_sync_date,
            ) {
                continue;
            }
            let today = now.date_naive();
            info!(hour = qdrant_cfg.sync_cron_hour, minute = qdrant_cfg.sync_cron_minute, "Qdrant vault sync starting");
            if let Err(e) = sync_vault_to_qdrant(&project_root, &qdrant_cfg, &vault_db_path).await {
                error!("Qdrant vault sync failed: {e}");
            } else {
                last_sync_date = Some(today);
                info!("Qdrant vault sync complete");
            }
        }
    });

    let sd_config = config.session_distill.clone();
    let distill_root = qdrant_sync::discover_project_root();
    let distill_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_distill_date: Option<NaiveDate> = None;
        let bin = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                error!("Session distill cron: cannot resolve gzmo binary: {e}");
                return;
            }
        };
        loop {
            interval.tick().await;
            if !sd_config.enabled || !sd_config.daemon_scheduled {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(
                &now,
                sd_config.cron_hour,
                sd_config.cron_minute,
                last_distill_date,
            ) {
                continue;
            }
            let today = now.date_naive();
            info!(
                hour = sd_config.cron_hour,
                minute = sd_config.cron_minute,
                "Session distill starting"
            );
            match tokio::process::Command::new(&bin)
                .arg("distill")
                .current_dir(&distill_root)
                .output()
                .await
            {
                Ok(out) if out.status.success() => {
                    last_distill_date = Some(today);
                    let summary = String::from_utf8_lossy(&out.stdout);
                    info!(summary = %summary.trim(), "Session distill complete");
                }
                Ok(out) => {
                    error!(
                        code = ?out.status.code(),
                        stderr = %String::from_utf8_lossy(&out.stderr).trim(),
                        "Session distill failed"
                    );
                }
                Err(e) => error!("Session distill spawn failed: {e}"),
            }
        }
    });

    // Synapse pull — read-only Pi event tail → episodic (feeds Dream)
    let synapse_cfg = config.synapse_pull.clone();
    let synapse_episodic = FileEpisodicStore::new(&config.memory.directory);
    let synapse_root = qdrant_sync::discover_project_root();
    let synapse_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_pull_date: Option<NaiveDate> = None;
        let bus_path = synapse_root.join(&synapse_cfg.bus_path);
        let state_path = gzmo_core::synapse_reader::default_state_path(&synapse_root);
        loop {
            interval.tick().await;
            if !synapse_cfg.enabled {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(
                &now,
                synapse_cfg.cron_hour,
                synapse_cfg.cron_minute,
                last_pull_date,
            ) {
                continue;
            }
            let today = now.date_naive();
            info!("Synapse pull starting (read-only)");
            match gzmo_core::synapse_reader::pull_and_log_episodic(
                &bus_path,
                &state_path,
                &synapse_episodic,
                synapse_cfg.max_events,
            )
            .await
            {
                Ok(summary) => {
                    last_pull_date = Some(today);
                    info!(
                        events = summary.events_read,
                        quest = summary.quest_complete,
                        "Synapse pull complete"
                    );
                }
                Err(e) => error!("Synapse pull failed: {e}"),
            }
        }
    });

    // KG reconcile — canonicalize shared Neo4j ontology via MCP memory
    let kg_cfg = config.kg_reconcile.clone();
    let kg_tools = Arc::clone(&dream_tools);
    let kg_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_run_date: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !kg_cfg.enabled {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(&now, kg_cfg.cron_hour, kg_cfg.cron_minute, last_run_date) {
                continue;
            }
            let today = now.date_naive();
            info!(dry_run = kg_cfg.dry_run, "KG reconcile starting");
            match gzmo_core::kg_reconcile::run_kg_reconcile(kg_tools.as_ref(), &kg_cfg).await {
                Ok(report) => {
                    last_run_date = Some(today);
                    info!(
                        entities = report.entities_scanned,
                        relations_fixed = report.relations_recanonicalized,
                        dry_run = report.dry_run,
                        "KG reconcile complete"
                    );
                }
                Err(e) => error!("KG reconcile failed: {e}"),
            }
        }
    });

    // Wiki "Knowledge Gardener" — daily index sync (after Qdrant sync at 01:45).
    let wiki_sync_cfg = config.wiki.clone();
    let wiki_sync_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_sync_date: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !wiki_sync_cfg.enabled {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(
                &now,
                wiki_sync_cfg.sync_cron_hour,
                wiki_sync_cfg.sync_cron_minute,
                last_sync_date,
            ) {
                continue;
            }
            let today = now.date_naive();
            info!(
                hour = wiki_sync_cfg.sync_cron_hour,
                minute = wiki_sync_cfg.sync_cron_minute,
                "Wiki sync (Knowledge Gardener) starting"
            );
            match WikiEngine::new(wiki_sync_cfg.clone()).sync().await {
                Ok(r) => {
                    last_sync_date = Some(today);
                    info!(pages = r.pages, entries = r.index_entries, "Wiki sync complete");
                }
                Err(e) => error!("Wiki sync failed: {e}"),
            }
        }
    });

    // Wiki lint — weekly structural health report (default Sunday 06:00 UTC).
    let wiki_lint_cfg = config.wiki.clone();
    let wiki_lint_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_lint_date: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !wiki_lint_cfg.enabled {
                continue;
            }
            let now = Utc::now();
            let today = now.date_naive();
            if last_lint_date == Some(today) {
                continue;
            }
            if now.weekday().num_days_from_sunday() != wiki_lint_cfg.lint_cron_dow {
                continue;
            }
            if now.hour() < wiki_lint_cfg.lint_cron_hour {
                continue;
            }
            info!(weekday = wiki_lint_cfg.lint_cron_dow, hour = wiki_lint_cfg.lint_cron_hour, "Wiki lint starting");
            match WikiEngine::new(wiki_lint_cfg.clone()).lint().await {
                Ok(r) => {
                    last_lint_date = Some(today);
                    info!(
                        pages = r.pages,
                        orphans = r.orphans.len(),
                        broken = r.broken_links.len(),
                        "Wiki lint complete"
                    );
                }
                Err(e) => error!("Wiki lint failed: {e}"),
            }
        }
    });

    let _identity = identity;
    // Pin PulseLoop task — must not drop until daemon exits.
    let _chaos_pulse_keepalive = chaos_pulse;

    tokio::select! {
        _ = heartbeat_handle => error!("Heartbeat exited"),
        _ = dream_handle => error!("Dream cycle exited"),
        _ = spark_handle => error!("Spark cycle exited"),
        _ = qdrant_handle => error!("Qdrant sync loop exited"),
        _ = distill_handle => error!("Session distill loop exited"),
        _ = distill_worker_handle => error!("Distill archive worker exited"),
        _ = synapse_handle => error!("Synapse pull loop exited"),
        _ = kg_handle => error!("KG reconcile loop exited"),
        _ = wiki_sync_handle => error!("Wiki sync loop exited"),
        _ = wiki_lint_handle => error!("Wiki lint loop exited"),
    }

    Ok(())
}
