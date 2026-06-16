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
use gzmo_core::context_compress::CcrStore;
use gzmo_core::kurator_spawn;
use gzmo_core::subagent::SubagentRunner;
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

use crate::low_tension_dialogue;
use crate::mentor_ipc::{self, MentorServerState};

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
    let obolus_ledger = router.obolus_ledger().cloned();
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

    let kurator_runner: Option<Arc<SubagentRunner>> =
        if config.kurator.enabled
            && config.kurator.auto_spawn_on_recommend
            && config.kurator.approve_spawns_subagent
            && config.subagent.enabled
        {
            let gateway = Arc::clone(router.gateway(TaskKind::Chat));
            let system_prompt = std::fs::read_to_string(&config.identity.soul_path)
                .unwrap_or_else(|_| "You are a focused GZMO sub-agent.".to_string());
            let serpapi_key = std::env::var("SERPAPI_API_KEY").unwrap_or_default();
            let ccr = CcrStore::new(&config.redis, &config.context_compress);
            Some(Arc::new(SubagentRunner::new(
                config.subagent.clone(),
                config.context_compress.clone(),
                ccr,
                Arc::clone(&scratch),
                gateway,
                Some(Arc::clone(&dream_vault)),
                system_prompt,
                serpapi_key,
            )))
        } else {
            None
        };
    if kurator_runner.is_some() {
        info!("Kurator autospawn runner ready (phase 3)");
    }

    if config.obolus_analytics.enabled {
        if let Some(ledger) = obolus_ledger.clone() {
            let obolus_cfg = config.clone();
            tokio::spawn(async move {
                let secs = obolus_cfg.obolus_analytics.reconcile_interval_secs.max(10);
                let mut interval = tokio::time::interval(Duration::from_secs(secs));
                loop {
                    interval.tick().await;
                    if let Err(e) =
                        gzmo_core::obolus::reconcile::run_tick(&obolus_cfg, &ledger).await
                    {
                        tracing::warn!(error = %e, "obolus reconcile tick failed");
                    }
                }
            });
            info!("Obolus reconcile task started");
        }
    }

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
        config.bibliothek.min_dream_cycles,
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
    let dice_loop_data_dir = state_dir.clone();
    let dice_kurator_cfg = config.kurator.clone();
    let dice_redis_cfg = config.redis.clone();
    let dice_kurator_root = state_dir.clone();
    let dice_kurator_runner = kurator_runner.clone();
    let dice_subagent_enabled = config.subagent.enabled;
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

    // `/dice` autopoietic loop — fire follow-up rolls when scheduled
    let dice_loop_config = config.dice.r#loop.clone();
    let dice_loop_config_full = config.clone();
    let dice_feedback_tx = chaos_pulse.feedback_tx.clone();
    let dice_snapshot_rx = chaos_pulse.snapshot_rx.clone();
    let dice_synapse = Arc::clone(&synapse);
    let dice_obolus_ledger = obolus_ledger.clone();
    tokio::spawn(async move {
        let registry = gzmo_core::skills::registry::build_chaos_skill_registry(
            &dice_loop_config_full.pedagogy,
        );
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            if !dice_loop_config.enabled {
                continue;
            }
            if !gzmo_core::pedagogy::session::PedagogySession::load(&dice_loop_config_full.pedagogy)
                .await
                .map(|s| s.auto_triggers_enabled)
                .unwrap_or(true)
            {
                continue;
            }
            let Some(state) = gzmo_core::dice_loop::load_state(&dice_loop_data_dir) else {
                continue;
            };
            let now = Utc::now();
            if !gzmo_core::dice_loop::is_due(now, &state) {
                continue;
            }
            // Mark state as "in-flight" so the next 5s tick skips it.
            // schedule_from_roll (called inside the skill) will overwrite with a new fire_at.
            let _ = gzmo_core::dice_loop::mark_processing(&dice_loop_data_dir, &state);
            let snap = dice_snapshot_rx.borrow().clone();
            let die_arg = if state.die_max == 6 { "d6" } else { "d20" };
            let args = format!("--loop {die_arg}");
            let gateway = gzmo_core::skills::dispatch::headless_gateway(
                &dice_loop_config_full,
                &snap,
                dice_obolus_ledger.clone(),
            );
            info!(
                parent_inv = state.parent_inv,
                parent_roll = state.parent_roll,
                chain_depth = state.chain_depth,
                die = die_arg,
                "Dice loop: firing scheduled /dice"
            );
            match gzmo_core::skills::dispatch::run_registry_skill_with_gateway(
                &registry,
                &dice_loop_config_full,
                "dice",
                &args,
                &snap,
                &dice_feedback_tx,
                Some(gateway),
            )
            .await
            {
                Ok(output) => {
                    if let Some(evidence) = output.evidence {
                        info!(
                            roll = evidence.get("roll").and_then(|v| v.as_u64()),
                            inv = evidence.get("inv").and_then(|v| v.as_u64()),
                            "Dice loop: follow-up roll complete"
                        );
                        let mut data = evidence;
                        data["display_plain"] =
                            serde_json::Value::String(gzmo_core::text_util::pi_skill_display(
                                &output.display,
                            ));
                        data["headless"] = serde_json::json!(true);
                        data["source"] =
                            serde_json::json!(gzmo_core::bibliothek::WUERFEL_CRON_SOURCE);
                        dice_synapse.append(&gzmo_core::synapse::SynapseEvent::with_data(
                            gzmo_core::synapse::EventType::ChaosDiceLoop,
                            gzmo_core::synapse::EventSource::GzmoDaemon,
                            data,
                        ));
                        if dice_kurator_cfg.enabled {
                            let kpath = gzmo_core::kurator_monitor::default_state_path(
                                &dice_kurator_root,
                            );
                            match gzmo_core::kurator_monitor::record_dice_loop_fire(
                                &dice_synapse,
                                &kpath,
                                &dice_kurator_cfg,
                            ) {
                                Ok(new_recs) => {
                                    if let Some(runner) = dice_kurator_runner.as_ref() {
                                        kurator_spawn::autospawn_new_recommendations(
                                            Arc::clone(runner),
                                            Arc::clone(&dice_synapse),
                                            kpath,
                                            dice_kurator_cfg.clone(),
                                            dice_redis_cfg.clone(),
                                            dice_subagent_enabled,
                                            new_recs,
                                        );
                                    }
                                }
                                Err(e) => error!(error = %e, "Kurator dice-loop record failed"),
                            }
                        }
                    }
                }
                Err(e) => error!(error = %e, "Dice loop: follow-up roll failed"),
            }
        }
    });

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
        let mut completed_spark_slots: Vec<(u32, u32)> = Vec::new();
        let mut completed_spark_date: Option<NaiveDate> = None;
        let mut next_dice_run: Option<chrono::DateTime<Utc>> = None;
        let mut dice_seed = spark_config.dice_seed.unwrap_or(chaos_seed);
        loop {
            interval.tick().await;
            if !spark_config.enabled {
                continue;
            }
            let now = Utc::now();
            let today = now.date_naive();
            if completed_spark_date != Some(today) {
                completed_spark_slots.clear();
                completed_spark_date = Some(today);
            }
            let cron_slot = match spark_config.schedule_mode {
                SparkScheduleMode::Cron => spark_cron_slot_due(
                    &now,
                    &spark_config.cron_hours,
                    spark_config.cron_minute,
                    &completed_spark_slots,
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
                        if !completed_spark_slots.contains(&(slot_hour, slot_minute)) {
                            completed_spark_slots.push((slot_hour, slot_minute));
                        }
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

    // Synapse pull — Pi event tail → episodic + session_end → distill pi
    let synapse_cfg = config.synapse_pull.clone();
    let kurator_cfg = config.kurator.clone();
    let kurator_redis_cfg = config.redis.clone();
    let kurator_runner_poll = kurator_runner.clone();
    let subagent_enabled = config.subagent.enabled;
    let synapse_episodic = FileEpisodicStore::new(&config.memory.directory);
    let synapse_root = qdrant_sync::discover_project_root();
    let kurator_synapse = Arc::clone(&synapse);
    let synapse_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let bus_path = synapse_root.join(&synapse_cfg.bus_path);
        let state_path = gzmo_core::synapse_reader::default_state_path(&synapse_root);
        let distill_state_path =
            gzmo_core::synapse_reader::default_distill_state_path(&synapse_root);
        let gzmo_bin = std::env::current_exe().ok();
        loop {
            interval.tick().await;
            if !synapse_cfg.enabled {
                continue;
            }
            match gzmo_core::synapse_reader::poll_pi_synapse(
                &bus_path,
                &state_path,
                &synapse_episodic,
                synapse_cfg.max_events,
                true,
            )
            .await
            {
                Ok(result) => {
                    if result.summary.events_read > 0 {
                        info!(
                            events = result.summary.events_read,
                            quest = result.summary.quest_complete,
                            session_end = result.summary.session_end,
                            "Synapse poll complete"
                        );
                    }
                    if kurator_cfg.enabled {
                        let kurator_state =
                            gzmo_core::kurator_monitor::default_state_path(&synapse_root);
                        match gzmo_core::kurator_monitor::process_pi_poll(
                            &kurator_synapse,
                            &kurator_state,
                            &kurator_cfg,
                            &result.events,
                        ) {
                            Ok(new_recs) => {
                                if let Some(runner) = kurator_runner_poll.as_ref() {
                                    kurator_spawn::autospawn_new_recommendations(
                                        Arc::clone(runner),
                                        Arc::clone(&kurator_synapse),
                                        kurator_state,
                                        kurator_cfg.clone(),
                                        kurator_redis_cfg.clone(),
                                        subagent_enabled,
                                        new_recs,
                                    );
                                }
                            }
                            Err(e) => error!(error = %e, "Kurator monitor failed"),
                        }
                    }
                    if !synapse_cfg.distill_on_session_end {
                        continue;
                    }
                    let Some(bin) = gzmo_bin.as_ref() else {
                        continue;
                    };
                    for session_file in &result.session_end_files {
                        let path = std::path::Path::new(session_file);
                        match gzmo_core::synapse_reader::should_distill_pi_session(
                            path,
                            &distill_state_path,
                        ) {
                            Ok(true) => {
                                info!(
                                    path = %session_file,
                                    "Spawning Pi session distill (session_end)"
                                );
                                let bin = bin.clone();
                                let session_file = session_file.clone();
                                let distill_state_path = distill_state_path.clone();
                                let synapse_root = synapse_root.clone();
                                tokio::spawn(async move {
                                    match tokio::process::Command::new(&bin)
                                        .args(["distill", "pi", &session_file])
                                        .current_dir(&synapse_root)
                                        .output()
                                        .await
                                    {
                                        Ok(out) if out.status.success() => {
                                            if let Err(e) =
                                                gzmo_core::synapse_reader::mark_pi_session_distilled(
                                                    &session_file,
                                                    &distill_state_path,
                                                )
                                            {
                                                error!("Pi distill state update failed: {e}");
                                            } else {
                                                info!(
                                                    path = %session_file,
                                                    "Pi session distill complete"
                                                );
                                            }
                                        }
                                        Ok(out) => error!(
                                            path = %session_file,
                                            code = ?out.status.code(),
                                            stderr = %String::from_utf8_lossy(&out.stderr).trim(),
                                            "Pi session distill failed"
                                        ),
                                        Err(e) => error!(
                                            path = %session_file,
                                            "Pi session distill spawn failed: {e}"
                                        ),
                                    }
                                });
                            }
                            Ok(false) => {}
                            Err(e) => error!(
                                path = %session_file,
                                "Pi distill eligibility check failed: {e}"
                            ),
                        }
                    }
                }
                Err(e) => error!("Synapse poll failed: {e}"),
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
    let mentor_chaos_tx = chaos_pulse.feedback_tx.clone();
    let mentor_chaos_snap = chaos_pulse.snapshot_rx.clone();
    let low_tension_snap = chaos_pulse.snapshot_rx.clone();
    // Pin PulseLoop task — must not drop until daemon exits.
    let _chaos_pulse_keepalive = chaos_pulse;

    let mentor_handle = if config.pedagogy.enabled && config.pedagogy.mentor_api_enabled {
        let mentor_state = Arc::new(
            mentor_ipc::MentorServerState::boot_with_chaos(
                config,
                Some(mentor_chaos_tx),
                Some(mentor_chaos_snap),
            )
            .await?,
        );

        if config.pedagogy.low_tension_dialogue.enabled {
            let lt_state = Arc::clone(&mentor_state);
            let lt_cfg = config.pedagogy.low_tension_dialogue.clone();
            let lt_log = config
                .memory
                .vault_db
                .parent()
                .unwrap_or(std::path::Path::new("data"))
                .join("pedagogy/low_tension_dialogue.jsonl");
            let lt_gzmo_root = config
                .memory
                .vault_db
                .parent()
                .unwrap_or(std::path::Path::new("data"))
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            let lt_scripts_root = config.pedagogy.discovery_scripts_root.clone();
            let lt_tools = Some(Arc::clone(&dream_tools));
            tokio::spawn(async move {
                low_tension_dialogue::run_low_tension_watcher(
                    lt_state,
                    low_tension_snap,
                    lt_cfg,
                    lt_scripts_root,
                    lt_log,
                    lt_gzmo_root,
                    lt_tools,
                )
                .await;
            });
        }

        let mentor_socket = mentor_ipc::socket_path(config);
        info!(path = %mentor_socket.display(), "Starting mentor API");
        Some(tokio::spawn(async move {
            if let Err(e) = mentor_ipc::run_mentor_server(mentor_state, mentor_socket).await {
                error!("Mentor API exited: {e}");
            }
        }))
    } else {
        None
    };

    tokio::select! {
        _ = async {
            if let Some(h) = mentor_handle {
                h.await.ok();
            } else {
                std::future::pending::<()>().await;
            }
        } => error!("Mentor API exited"),
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
