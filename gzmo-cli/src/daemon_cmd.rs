//! Daemon mode — heartbeat + dreams + orchestrator.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::idle_evolve::{idle_evolve_due, IDLE_EVOLVE_COOLDOWN};

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Timelike, Utc};
use tracing::{error, info};

use gzmo_core::config::EngineMode;
use gzmo_core::config::GzmoConfig;
use gzmo_core::config::SparkScheduleMode;
use gzmo_core::config::TaskKind;
use gzmo_core::daemon::{
    cron_due_today, cron_minutes, spark_cron_slot_due, write_cheapcheck_section,
    CognitionBlackoutCheck, EmbedHealthPing, FileChangeCheck, HealthPing, HeartbeatEngine,
};
use gzmo_core::dreams::DreamEngine;
use gzmo_core::dreams_md::write_dream_narrative;
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::gateway::LlmGateway;
use gzmo_core::health;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::ingest::IngestEngine;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::qdrant_sync::{self, sync_vault_to_qdrant};
use gzmo_core::memory::scratch::{ScratchScope, ScratchService};
use gzmo_core::metabolism;
use gzmo_core::session_distill::{run_distill_worker, SessionDistillEngine};
use gzmo_core::spark::{append_spark_to_dreams, SparkEngine};
use gzmo_core::synapse::SynapseBus;
use gzmo_core::synapse::{append_cognition_schedule, set_event_source};
use gzmo_core::wiki::WikiEngine;

use gzmo_core::mcp::{bridge::McpServerConfig, manager::McpManager};
use gzmo_core::spark_schedule;
use gzmo_core::tools::fs::{DirListTool, FileReadTool, FileSearchTool, FileWriteTool};
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysKillTool, SysMetricsTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::ToolRegistry;

use crate::mentor_ipc::{self, MentorServerState};

/// Run a lab recipe off the async runtime (recipes can take minutes).
async fn run_lab_script_blocking(script: &'static str, args: Vec<String>) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        gzmo_core::assembly::run_lab_script(script, &arg_refs)
    })
    .await?
}

pub async fn run(config: &GzmoConfig, identity: IdentityEngine) -> Result<()> {
    let soul = identity.snapshot().await;

    info!("╔══════════════════════════════════════════════╗");
    info!("║            GZMO — Daemon Mode                ║");
    info!("║       100% Local · Air-Gapped · Rust         ║");
    info!("╚══════════════════════════════════════════════╝");
    info!(persona = %soul.persona_name, "Identity loaded");

    // Assembly backends — lab recipes only activate under GZMO_INSTANCE=next
    // (AssemblyConfig::effective forces Inline otherwise; CT101-safe).
    let asm = &config.assembly;
    let distill_backend = asm.effective(asm.distill);
    let dream_backend = asm.effective(asm.dream);
    let spark_backend = asm.effective(asm.spark);
    let ops_backend = asm.effective(asm.ops_health);
    let handoff_backend = asm.effective(asm.config_handoff);
    info!(
        instance = %std::env::var("GZMO_INSTANCE").unwrap_or_else(|_| "legacy".into()),
        distill = distill_backend.label(),
        dream = dream_backend.label(),
        spark = spark_backend.label(),
        ops_health = ops_backend.label(),
        config_handoff = handoff_backend.label(),
        "Assembly backends resolved"
    );

    // Set event source for this thread (daemon)
    set_event_source(gzmo_core::synapse::EventSource::GzmoDaemon);

    // Ensure directories
    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    embeddings::assert_vault_backend(&config.memory.vault_backend)?;

    let _owner = gzmo_core::control_plane::claim_owner(config).await?;

    // Heartbeat
    let heartbeat_interval = Duration::from_secs(config.agent.heartbeat_interval_secs);
    let mut heartbeat = HeartbeatEngine::new(heartbeat_interval);
    heartbeat.add_check(FileChangeCheck {
        watch_dir: config.memory.directory.to_string_lossy().to_string(),
        since: Duration::from_secs(config.agent.heartbeat_interval_secs),
    });
    let active = config.engine.active_engine();
    heartbeat.add_check(HealthPing {
        url: format!("{}/models", active.url.trim_end_matches('/')),
        service_name: if config.engine.active_mode == EngineMode::Cloud {
            "Cloud LLM".to_string()
        } else {
            "LLM Engine".to_string()
        },
    });
    let prime = config.engine.active_engine_for_mode(EngineMode::Local);
    heartbeat.add_check(HealthPing {
        url: format!("{}/models", prime.url.trim_end_matches('/')),
        service_name: "Prime Fallback".to_string(),
    });
    let cloud = config.engine.active_engine_for_mode(EngineMode::Cloud);
    heartbeat.add_check(CognitionBlackoutCheck {
        cloud_models_url: format!("{}/models", cloud.url.trim_end_matches('/')),
        cloud_api_key: cloud.api_key.clone(),
        prime_models_url: format!("{}/models", prime.url.trim_end_matches('/')),
        prime_api_key: prime.api_key.clone(),
        cloud_primary: config.engine.active_mode == EngineMode::Cloud,
    });
    if config.embeddings.enabled {
        heartbeat.add_check(EmbedHealthPing {
            url: config.embeddings.url.clone(),
            model: config.embeddings.model.clone(),
            api_key: config.embeddings.api_key.clone(),
            expected_dims: 1024,
        });
    }

    // Gateway + Tools for dream cycle — use Obolus GatewayRouter
    let router = GatewayRouter::new(config);
    let dream_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamExtract));
    let dream_verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::DreamVerify));
    let ingest_verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::IngestVerify));
    let spark_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::SparkHypothesis));
    let spark_verify_gateway: Arc<dyn LlmGateway> =
        Arc::clone(router.gateway(TaskKind::SparkVerify));
    let ingest_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::IngestExtract));

    let dream_vault = embeddings::open_vault_with_embeddings(
        &config.memory.vault_db,
        &config.embeddings,
        &config.redis,
        &config.rerank,
        &config.qdrant,
    )
    .await?;
    if let Err(e) =
        dream_vault.archive_stale_session_anchors(config.spark.max_session_anchor_age_days)
    {
        error!("Vault session-anchor cleanup failed: {e}");
    }
    let dream_vault = Arc::new(dream_vault);

    let scratch =
        Arc::new(ScratchService::from_config(&config.redis, &config.context_memory).await);
    let memory_search_scope = Arc::new(std::sync::Mutex::new(ScratchScope::Orch {
        job: "init".to_string(),
        step: "init".to_string(),
    }));

    let mut dream_tools = ToolRegistry::new();
    dream_tools.register(Box::new(FileReadTool::default()));
    dream_tools.register(Box::new(FileWriteTool::default()));
    dream_tools.register(Box::new(DirListTool::default()));
    dream_tools.register(Box::new(FileSearchTool::default()));
    dream_tools.register(Box::new(ShellExecTool::default()));
    dream_tools.register(Box::new(WebSearchTool::default()));
    dream_tools.register(Box::new(SysMetricsTool));
    dream_tools.register(Box::new(SysKillTool));
    dream_tools.register(Box::new(MemoryRecordTool {
        vault: Arc::clone(&dream_vault),
    }));
    dream_tools.register(Box::new(MemorySearchTool::with_orchestrator_scratch(
        Arc::clone(&dream_vault),
        Arc::clone(&scratch),
        Arc::clone(&memory_search_scope),
    )));

    // MCP for dreams
    let mut dream_mcp = McpManager::new();
    for server in config.active_mcp_servers() {
        match dream_mcp
            .connect(McpServerConfig {
                name: server.name.clone(),
                command: server.command.clone(),
                args: server.args.clone(),
                env: server.env.clone(),
            })
            .await
        {
            Ok(count) => info!(server = %server.name, tools = count, "Dream MCP connected"),
            Err(e) => error!(server = %server.name, "Dream MCP failed: {}", e),
        }
    }
    dream_mcp.register_all_tools(&mut dream_tools);
    let mcp_manager = Arc::new(tokio::sync::Mutex::new(dream_mcp));
    let mcp_watch = Arc::clone(&mcp_manager);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            let mut mgr = mcp_watch.lock().await;
            if let Err(e) = mgr.ensure_healthy().await {
                tracing::warn!(error = %e, "MCP watchdog reconnect failed");
            }
        }
    });
    let _mcp_keepalive = mcp_manager;
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
            .then(|| Arc::clone(router.gateway(TaskKind::DistillSummary)))
            .filter(|_| config.librarian.enabled),
        Arc::clone(&dream_tools),
        config.session_distill.clone(),
        Some(Arc::clone(&synapse)),
    ));
    let distill_worker_handle = tokio::spawn(run_distill_worker(
        Arc::clone(&scratch),
        Arc::clone(&distill_engine),
    ));
    info!("Distill worker started (archive queue)");

    if ops_backend.is_lab() {
        // Lab recipe: endpoint-scan → synapse-health → plan-gate
        info!(assembly_backend = "lab", "Startup health via ops-smoke.sh");
        if let Err(e) = run_lab_script_blocking("ops-smoke.sh", vec!["--live".to_string()]).await {
            error!("Ops lab recipe failed: {e}");
            if config.health.strict_startup {
                return Err(e);
            }
        }
    } else if let Err(e) = health::run_startup_probes(
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
    let meta_config_dream = config.clone();

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
    let spark_vault_path = config.memory.vault_db.clone();
    let meta_config_spark = config.clone();
    let meta_config_distill = config.clone();
    let meta_config_watchdog = config.clone();

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
    // ─── Chaos quarantined on daemon path (ADR-0003) ─────────────
    // PulseLoop is chat-opt-in only. Prefer `gzmo serve` for overnight metabolism.
    let state_dir = config
        .memory
        .vault_db
        .parent()
        .unwrap_or(std::path::Path::new("data"))
        .to_path_buf();
    info!("Chaos engine skipped on daemon — use [chaos].enabled_in_chat for chat only");

    // ─── Mentor Unix socket (chaos-free) ─────────────────────────
    // Living discovery / Pi teach path. Do not wire PulseLoop here.
    // Dedicated OS thread + runtime: CT101 often has 1 tokio worker; dream/spark
    // sync work would otherwise starve the accept loop (connect ok, never reply).
    let mentor_thread = if config.pedagogy.enabled && config.pedagogy.mentor_api_enabled {
        let mut mentor_config = config.clone();
        mentor_config.pedagogy.active_learner_id =
            Some(gzmo_core::config::PedagogyConfig::resolve_learner_id(None));
        match MentorServerState::boot(&mentor_config).await {
            Ok(state) => {
                let mentor_socket = mentor_ipc::socket_path(&mentor_config);
                info!(
                    path = %mentor_socket.display(),
                    "Starting mentor API (chaos-free, dedicated thread)"
                );
                let state = Arc::new(state);
                match std::thread::Builder::new()
                    .name("gzmo-mentor".into())
                    .spawn(move || {
                        let rt = match tokio::runtime::Builder::new_multi_thread()
                            .worker_threads(2)
                            .thread_name("gzmo-mentor-rt")
                            .enable_all()
                            .build()
                        {
                            Ok(rt) => rt,
                            Err(e) => {
                                error!("Mentor runtime build failed: {e}");
                                return;
                            }
                        };
                        rt.block_on(async move {
                            if let Err(e) =
                                mentor_ipc::run_mentor_server(state, mentor_socket).await
                            {
                                error!("Mentor API exited: {e}");
                            }
                        });
                    }) {
                    Ok(join) => Some(join),
                    Err(e) => {
                        error!("Mentor thread spawn failed: {e}");
                        None
                    }
                }
            }
            Err(e) => {
                error!("Mentor API failed to boot: {e}");
                None
            }
        }
    } else {
        info!("Mentor API disabled in [pedagogy] config");
        None
    };

    info!("All subsystems online — entering daemon loop");

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
        chaos_feedback_tx: None,
        ingest_engine: if config.ingest.enabled {
            Some(Arc::clone(&ingest_engine))
        } else {
            None
        },
        synapse: Some(Arc::clone(&synapse)),
        scratch: Arc::clone(&scratch),
        memory_search_scope: Arc::clone(&memory_search_scope),
        context: gzmo_core::context::ContextConfig::from_memory_config(&config.context_memory),
    });

    let mut orch_jobs = config.orchestration.jobs.clone();
    orch_jobs.remove("spark");
    orch_jobs.remove("auto_dream");
    let _scheduler =
        match gzmo_core::orchestrator::start_orchestrator(orch_jobs, Arc::clone(&orch_ctx)).await {
            Ok(s) => {
                info!("Orchestrator online");
                Some(s)
            }
            Err(e) => {
                error!("Orchestrator failed: {e}");
                None
            }
        };

    // Watchers
    let orch_watchers = config.orchestration.watchers.clone();
    if let Err(e) = gzmo_core::watcher::start_watchers(&orch_watchers, orch_ctx).await {
        error!("Watchers failed: {e}");
    }

    // Heartbeat task — writes CheapCheck rows into HEARTBEAT.md
    let hb_dir = state_dir.join("HEARTBEAT.md");
    let idle_stamp = match std::env::var("GZMO_LIVING_HOME") {
        Ok(h) if !h.is_empty() => {
            std::path::PathBuf::from(h).join("data/research-intel/last-idle-evolve")
        }
        _ => state_dir.join("research-intel/last-idle-evolve"),
    };
    let evolve_script =
        qdrant_sync::discover_project_root().join("scripts/living-research-intel.sh");
    let heartbeat_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(heartbeat.interval);
        loop {
            interval.tick().await;
            info!("Heartbeat tick");
            let results = heartbeat.tick_with_results().await;
            if let Err(e) = write_cheapcheck_section(&hb_dir, &results).await {
                error!("Failed to write HEARTBEAT CheapCheck section: {e}");
            }
            let anomalies: Vec<String> = results
                .iter()
                .filter(|r| r.status == "WARN")
                .map(|r| format!("[{}] {}", r.name, r.detail))
                .collect();
            if !anomalies.is_empty() {
                info!(count = anomalies.len(), "Heartbeat anomalies");
                for a in &anomalies {
                    info!(anomaly = %a);
                }
                continue;
            }
            // Silent (all CheapChecks OK): at most one living-research evolve / 6h.
            let stamp_mtime = std::fs::metadata(&idle_stamp)
                .and_then(|m| m.modified())
                .ok();
            if !idle_evolve_due(stamp_mtime, SystemTime::now(), IDLE_EVOLVE_COOLDOWN) {
                continue;
            }
            if !evolve_script.is_file() {
                continue;
            }
            if let Some(parent) = idle_stamp.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&idle_stamp, []);
            let script = evolve_script.clone();
            info!(script = %script.display(), "idle evolve starting");
            tokio::spawn(async move {
                match tokio::process::Command::new("nice")
                    .args(["-n", "19", "bash"])
                    .arg(&script)
                    .status()
                    .await
                {
                    Ok(s) if s.code() == Some(2) => {
                        info!("idle evolve: no LLM (exit 2), continuing");
                    }
                    Ok(s) if s.success() => info!("idle evolve complete"),
                    Ok(s) => error!(code = ?s.code(), "idle evolve failed"),
                    Err(e) => error!(error = %e, "idle evolve spawn failed"),
                }
            });
        }
    });

    // Dream cycle task (DreamEngine — replaces headless auto_dream orchestrator job)
    let synapse_dream = Arc::clone(&synapse);
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
            if now.hour() * 60 + now.minute()
                < cron_minutes(dream_config.cron_hour, dream_config.cron_minute)
            {
                continue;
            }
            if last_consolidated == Some(yesterday) {
                continue;
            }
            info!(date = %yesterday, assembly_backend = dream_backend.label(), "Dream consolidation starting");
            let dream_started = Utc::now();
            append_cognition_schedule(
                synapse_dream.as_ref(),
                "dream",
                "tick",
                serde_json::json!({ "date": yesterday.to_string() }),
            );
            if dream_backend.is_lab() {
                // Lab recipe: session-distill → neural-finesse → DREAMS.md
                let args = vec![
                    "--live".to_string(),
                    "--output".to_string(),
                    dreams_path.to_string_lossy().into_owned(),
                ];
                match run_lab_script_blocking("session-to-dream.sh", args).await {
                    Ok(()) => {
                        last_consolidated = Some(yesterday);
                        metabolism::write_job_run(
                            &meta_config_dream,
                            "dream",
                            "lab",
                            dream_started,
                            true,
                            None,
                        );
                        append_cognition_schedule(
                            synapse_dream.as_ref(),
                            "dream",
                            "complete",
                            serde_json::json!({ "mode": "lab" }),
                        );
                        info!("Dream cycle complete (lab recipe)");
                    }
                    Err(e) => {
                        metabolism::write_job_run(
                            &meta_config_dream,
                            "dream",
                            "lab",
                            dream_started,
                            false,
                            Some(e.to_string()),
                        );
                        append_cognition_schedule(
                            synapse_dream.as_ref(),
                            "dream",
                            "fail",
                            serde_json::json!({ "error": e.to_string() }),
                        );
                        error!("Dream lab recipe failed: {e}");
                    }
                }
                continue;
            }
            match dream_engine_clone.consolidate(yesterday).await {
                Ok(report) => {
                    last_consolidated = Some(yesterday);
                    metabolism::write_job_run(
                        &meta_config_dream,
                        "dream",
                        "rust",
                        dream_started,
                        true,
                        None,
                    );
                    append_cognition_schedule(
                        synapse_dream.as_ref(),
                        "dream",
                        "complete",
                        serde_json::json!({
                            "entities": report.entities_extracted,
                            "kg_relations": report.kg_relations_written,
                        }),
                    );
                    info!(
                        entities = report.entities_extracted,
                        kg_relations = report.kg_relations_written,
                        "Dream cycle complete"
                    );
                    if let Err(e) = write_dream_narrative(&dreams_path, &report.narrative).await {
                        error!("Failed to write DREAMS.md: {e}");
                    }
                }
                Err(e) => {
                    metabolism::write_job_run(
                        &meta_config_dream,
                        "dream",
                        "rust",
                        dream_started,
                        false,
                        Some(e.to_string()),
                    );
                    append_cognition_schedule(
                        synapse_dream.as_ref(),
                        "dream",
                        "fail",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    error!("Dream failed: {e}");
                }
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
    let synapse_spark = Arc::clone(&synapse);
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

            info!(date = %today, mode = ?spark_config.schedule_mode, hour = slot_hour, minute = slot_minute, assembly_backend = spark_backend.label(), "Spark cycle starting");
            let spark_started = Utc::now();
            append_cognition_schedule(
                synapse_spark.as_ref(),
                "spark",
                "tick",
                serde_json::json!({ "hour": slot_hour, "minute": slot_minute }),
            );
            if spark_backend.is_lab() {
                // Lab recipe: cognition chain (distill → gate → spark-link → recall)
                let args = vec![
                    "--live".to_string(),
                    "--vault".to_string(),
                    spark_vault_path.to_string_lossy().into_owned(),
                    "--spark-run".to_string(),
                ];
                match run_lab_script_blocking("cognition-smoke.sh", args).await {
                    Ok(()) => {
                        if spark_config.schedule_mode == SparkScheduleMode::Cron {
                            last_run_key = Some((slot_hour, slot_minute, today));
                        } else {
                            next_dice_run = None;
                        }
                        metabolism::write_job_run(
                            &meta_config_spark,
                            "spark",
                            "lab",
                            spark_started,
                            true,
                            None,
                        );
                        append_cognition_schedule(
                            synapse_spark.as_ref(),
                            "spark",
                            "complete",
                            serde_json::json!({ "mode": "lab" }),
                        );
                        info!("Spark cycle complete (lab recipe)");
                    }
                    Err(e) => {
                        // Advance slot so Err does not retry every 60s in the same cron window.
                        if spark_config.schedule_mode == SparkScheduleMode::Cron {
                            last_run_key = Some((slot_hour, slot_minute, today));
                        } else {
                            next_dice_run = None;
                        }
                        metabolism::write_job_run(
                            &meta_config_spark,
                            "spark",
                            "lab",
                            spark_started,
                            false,
                            Some(e.to_string()),
                        );
                        append_cognition_schedule(
                            synapse_spark.as_ref(),
                            "spark",
                            "fail",
                            serde_json::json!({ "error": e.to_string() }),
                        );
                        error!("Spark lab recipe failed: {e}");
                    }
                }
                continue;
            }
            match spark_engine_clone.run(today).await {
                Ok(report) => {
                    if spark_config.schedule_mode == SparkScheduleMode::Cron {
                        last_run_key = Some((slot_hour, slot_minute, today));
                    } else {
                        next_dice_run = None;
                    }
                    metabolism::write_job_run(
                        &meta_config_spark,
                        "spark",
                        "rust",
                        spark_started,
                        true,
                        None,
                    );
                    if let Err(e) =
                        append_spark_to_dreams(&dreams_path_spark, &report.section).await
                    {
                        error!("Failed to append spark to DREAMS.md: {e}");
                    } else {
                        append_cognition_schedule(
                            synapse_spark.as_ref(),
                            "spark",
                            "complete",
                            serde_json::json!({
                                "promoted": report.promoted,
                                "kg_relations": report.kg_relations_written,
                            }),
                        );
                        info!(
                            promoted = report.promoted,
                            kg = report.kg_relations_written,
                            "Spark cycle complete"
                        );
                    }
                }
                Err(e) => {
                    // Advance slot / re-roll dice so hard errors do not spin every minute.
                    if spark_config.schedule_mode == SparkScheduleMode::Cron {
                        last_run_key = Some((slot_hour, slot_minute, today));
                    } else {
                        next_dice_run = None;
                    }
                    metabolism::write_job_run(
                        &meta_config_spark,
                        "spark",
                        "rust",
                        spark_started,
                        false,
                        Some(e.to_string()),
                    );
                    append_cognition_schedule(
                        synapse_spark.as_ref(),
                        "spark",
                        "fail",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    error!("Spark failed: {e}");
                }
            }
        }
    });

    // Qdrant mirror — daily sync after dream window (default 01:45 UTC)
    let qdrant_cfg = config.qdrant.clone();
    let vault_db_path = config.memory.vault_db.clone();
    let project_root = qdrant_sync::discover_project_root();
    let synapse_qdrant = Arc::clone(&synapse);
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
            info!(
                hour = qdrant_cfg.sync_cron_hour,
                minute = qdrant_cfg.sync_cron_minute,
                "Qdrant vault sync starting"
            );
            append_cognition_schedule(
                synapse_qdrant.as_ref(),
                "qdrant_sync",
                "tick",
                serde_json::json!({
                    "cron_hour": qdrant_cfg.sync_cron_hour,
                    "cron_minute": qdrant_cfg.sync_cron_minute,
                }),
            );
            if let Err(e) = sync_vault_to_qdrant(&project_root, &qdrant_cfg, &vault_db_path).await {
                append_cognition_schedule(
                    synapse_qdrant.as_ref(),
                    "qdrant_sync",
                    "fail",
                    serde_json::json!({ "error": e.to_string() }),
                );
                error!("Qdrant vault sync failed: {e}");
            } else {
                last_sync_date = Some(today);
                append_cognition_schedule(
                    synapse_qdrant.as_ref(),
                    "qdrant_sync",
                    "complete",
                    serde_json::Value::Null,
                );
                info!("Qdrant vault sync complete");
            }
        }
    });

    let sd_config = config.session_distill.clone();
    let distill_root = qdrant_sync::discover_project_root();
    let synapse_distill = Arc::clone(&synapse);
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
                assembly_backend = distill_backend.label(),
                "Session distill starting"
            );
            let distill_started = Utc::now();
            append_cognition_schedule(
                synapse_distill.as_ref(),
                "session_distill",
                "tick",
                serde_json::json!({
                    "cron_hour": sd_config.cron_hour,
                    "cron_minute": sd_config.cron_minute,
                }),
            );
            if distill_backend.is_lab() {
                // Lab recipe: synapse session_end → session-distill handoff
                match run_lab_script_blocking(
                    "synapse-distill-handoff.sh",
                    vec!["--live".to_string()],
                )
                .await
                {
                    Ok(()) => {
                        last_distill_date = Some(today);
                        metabolism::write_job_run(
                            &meta_config_distill,
                            "distill",
                            "lab",
                            distill_started,
                            true,
                            None,
                        );
                        append_cognition_schedule(
                            synapse_distill.as_ref(),
                            "session_distill",
                            "complete",
                            serde_json::json!({ "mode": "lab" }),
                        );
                        info!("Session distill complete (lab recipe)");
                    }
                    Err(e) => {
                        metabolism::write_job_run(
                            &meta_config_distill,
                            "distill",
                            "lab",
                            distill_started,
                            false,
                            Some(e.to_string()),
                        );
                        append_cognition_schedule(
                            synapse_distill.as_ref(),
                            "session_distill",
                            "fail",
                            serde_json::json!({ "error": e.to_string() }),
                        );
                        error!("Session distill lab recipe failed: {e}");
                    }
                }
                continue;
            }
            match tokio::process::Command::new(&bin)
                .arg("distill")
                .current_dir(&distill_root)
                .output()
                .await
            {
                Ok(out) if out.status.success() => {
                    last_distill_date = Some(today);
                    metabolism::write_job_run(
                        &meta_config_distill,
                        "distill",
                        "rust",
                        distill_started,
                        true,
                        None,
                    );
                    let summary = String::from_utf8_lossy(&out.stdout);
                    append_cognition_schedule(
                        synapse_distill.as_ref(),
                        "session_distill",
                        "complete",
                        serde_json::json!({ "summary": summary.trim() }),
                    );
                    info!(summary = %summary.trim(), "Session distill complete");
                }
                Ok(out) => {
                    metabolism::write_job_run(
                        &meta_config_distill,
                        "distill",
                        "rust",
                        distill_started,
                        false,
                        Some(format!(
                            "exit {:?} {}",
                            out.status.code(),
                            String::from_utf8_lossy(&out.stderr).trim()
                        )),
                    );
                    append_cognition_schedule(
                        synapse_distill.as_ref(),
                        "session_distill",
                        "fail",
                        serde_json::json!({
                            "code": out.status.code(),
                            "stderr": String::from_utf8_lossy(&out.stderr).trim(),
                        }),
                    );
                    error!(
                        code = ?out.status.code(),
                        stderr = %String::from_utf8_lossy(&out.stderr).trim(),
                        "Session distill failed"
                    );
                }
                Err(e) => {
                    metabolism::write_job_run(
                        &meta_config_distill,
                        "distill",
                        "rust",
                        distill_started,
                        false,
                        Some(e.to_string()),
                    );
                    error!("Session distill spawn failed: {e}");
                }
            }
        }
    });

    // Synapse pull — read-only Pi event tail → episodic (feeds Dream)
    let synapse_cfg = config.synapse_pull.clone();
    let synapse_episodic = FileEpisodicStore::new(&config.memory.directory);
    let synapse_root = qdrant_sync::discover_project_root();
    let synapse_scratch = Arc::clone(&scratch);
    let synapse_pull_bus = Arc::clone(&synapse);
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
            append_cognition_schedule(
                synapse_pull_bus.as_ref(),
                "synapse_pull",
                "tick",
                serde_json::Value::Null,
            );
            match gzmo_core::synapse_reader::pull_and_log_episodic(
                &bus_path,
                &state_path,
                &synapse_episodic,
                synapse_cfg.max_events,
                Some(synapse_scratch.as_ref()),
            )
            .await
            {
                Ok(summary) => {
                    last_pull_date = Some(today);
                    append_cognition_schedule(
                        synapse_pull_bus.as_ref(),
                        "synapse_pull",
                        "complete",
                        serde_json::json!({
                            "events": summary.events_read,
                            "quest": summary.quest_complete,
                            "session_end": summary.session_end,
                            "distill_enqueued": summary.distill_enqueued,
                        }),
                    );
                    info!(
                        events = summary.events_read,
                        quest = summary.quest_complete,
                        session_end = summary.session_end,
                        distill_enqueued = summary.distill_enqueued,
                        "Synapse pull complete"
                    );
                }
                Err(e) => {
                    append_cognition_schedule(
                        synapse_pull_bus.as_ref(),
                        "synapse_pull",
                        "fail",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    error!("Synapse pull failed: {e}");
                }
            }
        }
    });

    // KG reconcile — canonicalize shared Neo4j ontology via MCP memory
    let kg_cfg = config.kg_reconcile.clone();
    let kg_tools = Arc::clone(&dream_tools);
    let synapse_kg = Arc::clone(&synapse);
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
            append_cognition_schedule(
                synapse_kg.as_ref(),
                "kg_reconcile",
                "tick",
                serde_json::json!({ "dry_run": kg_cfg.dry_run }),
            );
            match gzmo_core::kg_reconcile::run_kg_reconcile(kg_tools.as_ref(), &kg_cfg).await {
                Ok(report) => {
                    last_run_date = Some(today);
                    append_cognition_schedule(
                        synapse_kg.as_ref(),
                        "kg_reconcile",
                        "complete",
                        serde_json::json!({
                            "entities": report.entities_scanned,
                            "relations_fixed": report.relations_recanonicalized,
                            "dry_run": report.dry_run,
                        }),
                    );
                    info!(
                        entities = report.entities_scanned,
                        relations_fixed = report.relations_recanonicalized,
                        dry_run = report.dry_run,
                        "KG reconcile complete"
                    );
                }
                Err(e) => {
                    append_cognition_schedule(
                        synapse_kg.as_ref(),
                        "kg_reconcile",
                        "fail",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    error!("KG reconcile failed: {e}");
                }
            }
        }
    });

    // Wiki "Knowledge Gardener" — daily index sync (after Qdrant sync at 01:45).
    let wiki_sync_cfg = config.wiki.clone();
    let synapse_wiki = Arc::clone(&synapse);
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
            append_cognition_schedule(
                synapse_wiki.as_ref(),
                "wiki_sync",
                "tick",
                serde_json::json!({
                    "cron_hour": wiki_sync_cfg.sync_cron_hour,
                    "cron_minute": wiki_sync_cfg.sync_cron_minute,
                }),
            );
            match WikiEngine::new(wiki_sync_cfg.clone()).sync().await {
                Ok(r) => {
                    last_sync_date = Some(today);
                    append_cognition_schedule(
                        synapse_wiki.as_ref(),
                        "wiki_sync",
                        "complete",
                        serde_json::json!({ "pages": r.pages, "entries": r.index_entries }),
                    );
                    info!(
                        pages = r.pages,
                        entries = r.index_entries,
                        "Wiki sync complete"
                    );
                }
                Err(e) => {
                    append_cognition_schedule(
                        synapse_wiki.as_ref(),
                        "wiki_sync",
                        "fail",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    error!("Wiki sync failed: {e}");
                }
            }
        }
    });

    // Wiki lint — weekly structural health report (default Sunday 06:00 UTC).
    let wiki_lint_cfg = config.wiki.clone();
    let synapse_wiki_lint = Arc::clone(&synapse);
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
            info!(
                weekday = wiki_lint_cfg.lint_cron_dow,
                hour = wiki_lint_cfg.lint_cron_hour,
                "Wiki lint starting"
            );
            append_cognition_schedule(
                synapse_wiki_lint.as_ref(),
                "wiki_lint",
                "tick",
                serde_json::json!({
                    "dow": wiki_lint_cfg.lint_cron_dow,
                    "hour": wiki_lint_cfg.lint_cron_hour,
                }),
            );
            match WikiEngine::new(wiki_lint_cfg.clone()).lint().await {
                Ok(r) => {
                    last_lint_date = Some(today);
                    append_cognition_schedule(
                        synapse_wiki_lint.as_ref(),
                        "wiki_lint",
                        "complete",
                        serde_json::json!({
                            "pages": r.pages,
                            "orphans": r.orphans.len(),
                            "broken": r.broken_links.len(),
                        }),
                    );
                    info!(
                        pages = r.pages,
                        orphans = r.orphans.len(),
                        broken = r.broken_links.len(),
                        "Wiki lint complete"
                    );
                }
                Err(e) => {
                    append_cognition_schedule(
                        synapse_wiki_lint.as_ref(),
                        "wiki_lint",
                        "fail",
                        serde_json::json!({ "error": e.to_string() }),
                    );
                    error!("Wiki lint failed: {e}");
                }
            }
        }
    });

    // Config handoff — lab-only calibration loop (bench → fuse → apply on gate pass).
    // Inline has no equivalent (manual `gzmo assemble handoff`); daily 04:00 UTC,
    // after dream (01:00), distill (02:15) and spark (03:30) windows.
    const HANDOFF_CRON_HOUR: u32 = 4;
    const HANDOFF_CRON_MINUTE: u32 = 0;
    let handoff_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_handoff_date: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !handoff_backend.is_lab() {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(
                &now,
                HANDOFF_CRON_HOUR,
                HANDOFF_CRON_MINUTE,
                last_handoff_date,
            ) {
                continue;
            }
            let today = now.date_naive();
            info!(
                assembly_backend = "lab",
                "Config handoff starting (gzmo-handoff.sh)"
            );
            // Script only applies the fused config when the benchmark gate passes.
            // Apply target is the sibling *-fused.toml — never the live instance
            // config, which config-fuse output would clobber wholesale.
            let mut args = vec!["--live".to_string(), "--apply".to_string()];
            if let Some(target) = gzmo_core::assembly::handoff_apply_target() {
                args.push("--gzmo-config".to_string());
                args.push(target.to_string_lossy().into_owned());
            }
            match run_lab_script_blocking("gzmo-handoff.sh", args).await {
                Ok(()) => {
                    last_handoff_date = Some(today);
                    info!("Config handoff complete (gate passed)");
                }
                Err(e) => {
                    // Gate-fail exits non-zero by design — hold previous config.
                    last_handoff_date = Some(today);
                    error!("Config handoff held or failed: {e}");
                }
            }
        }
    });

    // Missed-run watchdog — same ledger path as `gzmo serve` / `gzmo metabolism watchdog`.
    let watchdog_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            let record = metabolism::evaluate_and_write_watchdog(&meta_config_watchdog);
            if record.stale {
                info!(
                    detail = %record.detail,
                    "Metabolism watchdog stale (daemon ledger path)"
                );
            }
        }
    });

    // Promote + embed (metabolism triad) — same receipts as `gzmo serve` / oneshot.
    // Living CT101 runs daemon; without these jobs organ-trace soft-missed promote/embed.
    // Use spawn_blocking + thread runtime: embed/promote touch rusqlite (not Send for spawn).
    let meta_config_promote = config.clone();
    let meta_config_embed = config.clone();
    let vault_db_embed = config.memory.vault_db.clone();
    let qdrant_for_embed = config.qdrant.clone();
    let project_root_embed = qdrant_sync::discover_project_root();
    let promote_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_promote: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !meta_config_promote.metabolism.enabled {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(
                &now,
                meta_config_promote.metabolism.promote_cron_hour,
                meta_config_promote.metabolism.promote_cron_minute,
                last_promote,
            ) {
                continue;
            }
            let cfg = meta_config_promote.clone();
            info!("Daemon metabolism promote starting");
            let result = tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("promote runtime");
                rt.block_on(crate::promote_cmd::run(&cfg, None))
            })
            .await;
            match result {
                Ok(Ok(())) => {
                    last_promote = Some(now.date_naive());
                    info!("Daemon metabolism promote complete");
                }
                Ok(Err(e)) => {
                    last_promote = Some(now.date_naive());
                    error!("Daemon promote failed: {e}");
                }
                Err(e) => {
                    last_promote = Some(now.date_naive());
                    error!("Daemon promote join failed: {e}");
                }
            }
        }
    });
    let embed_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        let mut last_embed: Option<NaiveDate> = None;
        loop {
            interval.tick().await;
            if !meta_config_embed.metabolism.enabled {
                continue;
            }
            let now = Utc::now();
            if !cron_due_today(
                &now,
                meta_config_embed.metabolism.embed_cron_hour,
                meta_config_embed.metabolism.embed_cron_minute,
                last_embed,
            ) {
                continue;
            }
            let cfg = meta_config_embed.clone();
            let qcfg = qdrant_for_embed.clone();
            let vdb = vault_db_embed.clone();
            let root = project_root_embed.clone();
            info!("Daemon metabolism embed starting");
            let result = tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("embed runtime");
                rt.block_on(async {
                    crate::embed_cmd::run(&cfg, None).await?;
                    if qcfg.enabled && qcfg.sync_enabled {
                        sync_vault_to_qdrant(&root, &qcfg, &vdb).await?;
                    }
                    anyhow::Ok(())
                })
            })
            .await;
            match result {
                Ok(Ok(())) => {
                    last_embed = Some(now.date_naive());
                    info!("Daemon metabolism embed complete");
                }
                Ok(Err(e)) => {
                    last_embed = Some(now.date_naive());
                    error!("Daemon embed failed: {e}");
                }
                Err(e) => {
                    last_embed = Some(now.date_naive());
                    error!("Daemon embed join failed: {e}");
                }
            }
        }
    });

    let _identity = identity;
    let _mentor_thread = mentor_thread;

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
        _ = handoff_handle => error!("Config handoff loop exited"),
        _ = promote_handle => error!("Metabolism promote loop exited"),
        _ = embed_handle => error!("Metabolism embed loop exited"),
        _ = watchdog_handle => error!("Metabolism watchdog loop exited"),
    }

    Ok(())
}
