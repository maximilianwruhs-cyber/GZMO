use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;

use gzmo_core::agent_session::AgentSession;
use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::{GatewayRouter, LlmGateway, TurboQuantGateway, VllmConfig};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::session::SessionManager;
use gzmo_core::mcp::{bridge::McpServerConfig, manager::McpManager};
use gzmo_core::subagent::SubagentRunner;
use gzmo_core::tools::ToolRegistry;
use gzmo_core::tools::delegate::DelegateTaskTool;
use gzmo_core::tools::memory::MemorySearchTool;
use gzmo_core::tools::profile::{register_for_profile, CapabilityProfile, ToolRegisterOpts};
use gzmo_core::skills::{
    register_pantheon, SkillRegistry as ChaosSkillRegistry,
};

use crate::repl_shared::{
    boot_knowledge_graph, boot_workflow_skills, build_system_prompt_with_workflows,
    open_semantic_vault, ping_engine,
};
use crate::tui::action::Action;
use crate::tui::app::App;
use crate::tui::components::{
    agent::AgentComponent, chaos_canvas::ChaosCanvasComponent, input::InputComponent,
    instruments::InstrumentsComponent, palette::PaletteComponent, status_bar::StatusBarComponent,
    transcript::TranscriptComponent,
};

/// Boot and run the full-screen TUI interface.
pub async fn run(config: &GzmoConfig, identity: &IdentityEngine) -> Result<()> {
    let config = config.clone();
    let config_arc = Arc::new(tokio::sync::RwLock::new(config.clone()));
    let config_path = std::env::var("GZMO_CONFIG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("gzmo.toml")
        });

    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }

    // ─── Vault (semantic, shared with stdio) ─────────────────────
    let vault = open_semantic_vault(&config).await;

    // ─── Hot memory session ──────────────────────────────────────
    let agent_session = AgentSession::new_main(
        &config.redis,
        &config.context_memory,
        SessionManager::new_session_id(),
    )
    .await;
    let agent_session = Arc::new(tokio::sync::Mutex::new(agent_session));

    // ─── Episodic + sessions ─────────────────────────────────────
    let episodic = Arc::new(FileEpisodicStore::new(&config.memory.directory));
    let session_mgr = Arc::new(SessionManager::new(&config.session_distill.sessions_dir));
    let _ = session_mgr.ensure_dir().await;

    // ─── Gateway (primary still TurboQuant; delegate uses router) ─
    let active_profile = config.engine.active_engine();
    let gateway: Arc<tokio::sync::RwLock<Arc<dyn LlmGateway>>> = Arc::new(tokio::sync::RwLock::new(
        Arc::new(TurboQuantGateway::new(VllmConfig::from(active_profile.clone())))
            as Arc<dyn LlmGateway>,
    ));
    let router = GatewayRouter::new(&config);
    let chat_gateway_dyn = router.gateway(TaskKind::Chat);

    // ─── Chaos (always live in TUI — ADR-0003) ───────────────────
    let chaos_runtime = crate::chaos_bootstrap::start_chaos_runtime(&config);
    let chaos_enabled_in_config = crate::chaos_bootstrap::enabled_in_chat(&config);
    let mut chaos_handle = Some(chaos_runtime.handle);
    let chaos_snapshot_rx = chaos_handle
        .as_ref()
        .expect("chaos handle")
        .snapshot_rx
        .clone();
    let chaos_feedback_tx = chaos_runtime.feedback_tx.clone();
    let restore_policy = chaos_runtime.restore_policy.clone();

    // ─── Chaos Skills ────────────────────────────────────────────
    let mut chaos_skills = ChaosSkillRegistry::new();
    register_pantheon(&mut chaos_skills, &config);
    let chaos_skills = Arc::new(chaos_skills);

    // ─── Workflow skills ─────────────────────────────────────────
    let (workflow_index, workflow_session) = boot_workflow_skills(&config)?;

    // ─── Tools (capability profile + jail) ───────────────────────
    let profile = CapabilityProfile::parse(&config.tools.profile).unwrap_or(CapabilityProfile::Developer);
    let mut tools = ToolRegistry::new();
    register_for_profile(
        &mut tools,
        profile,
        &config.tools,
        ToolRegisterOpts {
            vault: vault.clone(),
            scratch: None,
            scratch_scope: None,
            serpapi_key: {
                let k = config.api_keys.serpapi_key();
                if k.is_empty() {
                    None
                } else {
                    Some(k)
                }
            },
            workflow: Some((Arc::clone(&workflow_index), Arc::clone(&workflow_session))),
            gzmo_config: Some(config.clone()),
        },
    )?;

    // ─── MCP ─────────────────────────────────────────────────────
    let mut mcp = McpManager::new();
    for server in config.active_mcp_servers() {
        match mcp
            .connect(McpServerConfig {
                name: server.name.clone(),
                command: server.command.clone(),
                args: server.args.clone(),
                env: server.env.clone(),
            })
            .await
        {
            Ok(count) => tracing::info!(server = %server.name, tools = count, "MCP connected"),
            Err(e) => tracing::error!(server = %server.name, "MCP failed: {}", e),
        }
    }
    mcp.register_all_tools(&mut tools);

    // ─── Memory context + system prompt (needs tool defs) ────────
    let soul = identity.snapshot().await;
    let memory_context = boot_knowledge_graph(&tools).await;
    let vault_context: Option<String> = if let Some(ref v) = vault {
        match v.recent(10) {
            Ok(facts) if !facts.is_empty() => {
                let mut block = String::from(
                    "\n\n## Long-Term Memory (Honeypot-first vault)\n\
                     Prefer these curated facts over raw episodic soup.\n",
                );
                for fact in &facts {
                    block.push_str(&format!("- {fact}\n"));
                }
                Some(block)
            }
            _ => None,
        }
    } else {
        None
    };

    let tool_names: Vec<String> = tools.definitions().iter().map(|d| d.name.clone()).collect();
    let last_handoff = workflow_index.latest_handoff();
    let system_prompt = build_system_prompt_with_workflows(
        &soul,
        memory_context.as_deref(),
        vault_context.as_deref(),
        &tool_names,
        &Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        Some(workflow_index.as_ref()),
        last_handoff.as_deref(),
    );

    // ─── Subagents (delegate_task uses GatewayRouter::Chat) ──────
    let scratch = {
        let session = agent_session.lock().await;
        session.scratch()
    };
    let subagent_runner = Arc::new(SubagentRunner::with_tools_config(
        config.subagent.clone(),
        config.tools.clone(),
        scratch,
        Arc::clone(&chat_gateway_dyn),
        vault.clone(),
        system_prompt.clone(),
        Some(config.clone()),
    ));
    {
        let mut session = agent_session.lock().await;
        session.attach_subagent_runner(Arc::clone(&subagent_runner));
    }
    let subagent_enabled = config.subagent.enabled;
    if subagent_enabled {
        tools.register(Box::new(DelegateTaskTool {
            runner: Arc::clone(&subagent_runner),
            session_id: agent_session.lock().await.session_id().to_string(),
            depth: 0,
        }));
    }

    if let Some(ref v) = vault {
        let session = agent_session.lock().await;
        tools.register(Box::new(MemorySearchTool {
            vault: Arc::clone(v),
            scratch: Some(session.scratch()),
            scope: Some(session.main_scope()),
            scope_cell: None,
        }));
    }

    let tools = Arc::new(tools);

    let context_budget = active_profile.max_tokens as usize * 4;
    let max_iterations = config.agent.max_tool_iterations;
    let soul_arc = Arc::clone(&identity.soul);

    let mode_str = match config.engine.active_mode {
        gzmo_core::config::EngineMode::Local => "LOCAL",
        gzmo_core::config::EngineMode::Cloud => "CLOUD",
        gzmo_core::config::EngineMode::Sovereign => "SOVEREIGN",
    };
    let model_name = active_profile.model.clone();

    let input_cmp = InputComponent::new();
    let transcript_cmp = TranscriptComponent::new();
    let status_cmp = StatusBarComponent::new(mode_str, model_name);
    let instruments_cmp = InstrumentsComponent::new(vault.clone());
    let canvas_cmp = ChaosCanvasComponent::new();
    let agent_cmp = AgentComponent::new(
        Arc::clone(&gateway),
        Arc::clone(&tools),
        system_prompt,
        max_iterations,
        context_budget,
        soul_arc,
        vault.clone(),
        Arc::clone(&episodic),
        Arc::clone(&session_mgr),
        Arc::clone(&agent_session),
        chaos_snapshot_rx.clone(),
        Arc::clone(&chaos_skills),
        chaos_feedback_tx.clone(),
        Arc::clone(&config_arc),
        config_path,
        Some(Arc::clone(&subagent_runner)),
        subagent_enabled,
        Arc::clone(&workflow_index),
        Arc::clone(&workflow_session),
    );
    let palette_cmp = PaletteComponent::new();

    let comps = crate::tui::app::AppComponents {
        input: Box::new(input_cmp),
        transcript: Box::new(transcript_cmp),
        status: Box::new(status_cmp),
        instruments: Box::new(instruments_cmp),
        canvas: Box::new(canvas_cmp),
        agent: Box::new(agent_cmp),
        palette: palette_cmp,
    };
    let (mut app, action_tx, action_rx) = App::new(comps);

    // ─── Unified chaos publisher (TUI adapters + HEARTBEAT.md) ───
    let state_dir = config
        .memory
        .vault_db
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    let _chaos_bridge = crate::chaos_bootstrap::spawn_snapshot_bridge(
        chaos_snapshot_rx.clone(),
        Arc::clone(&gateway),
        chaos_feedback_tx.clone(),
        state_dir,
        None,
        Some(action_tx.clone()),
        None,
        gzmo_core::synapse::EventSource::GzmoCli,
        restore_policy,
        true,
        crate::chaos_bootstrap::SnapshotBridgeOpts::TUI,
    );

    // ─── Background: Lore → UI ───────────────────────────────────
    if let Some(mut handle) = chaos_handle.take() {
        let tx = action_tx.clone();
        tokio::spawn(async move {
            while let Some(lore) = handle.lore_rx.recv().await {
                let author = lore.author.unwrap_or_default();
                let _ = tx.send(Action::LoreEvent(lore.category, author, lore.text));
            }
        });
    }

    // ─── Background: Hardware Telemetry ──────────────────────────
    {
        let tx = action_tx.clone();
        tokio::spawn(async move {
            let mut sys = sysinfo::System::new_all();
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(1500));
            loop {
                interval.tick().await;
                sys.refresh_cpu_all();
                sys.refresh_memory();

                let cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>()
                    / sys.cpus().len().max(1) as f32;
                let mem_usage: f32 =
                    (sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0) as f32;

                if tx.send(Action::Telemetry(cpu_usage, mem_usage)).is_err() {
                    break;
                }
            }
        });
    }

    // ─── Startup: engine health + session banner ───────────────────
    let (engine_status, engine_latency) = ping_engine(&config).await;
    let _ = action_tx.send(Action::EngineHealth(
        engine_status.to_string(),
        engine_latency,
    ));

    let boot_msg = if chaos_enabled_in_config {
        format!(
            "⚙ Systems nominal. Chaos pulse live — Lorenz instruments online. LLM {engine_status}."
        )
    } else {
        format!(
            "⚙ Systems nominal. Chaos pulse forced for TUI (config enabled_in_chat=false). LLM {engine_status}."
        )
    };
    let _ = action_tx.send(Action::AgentResponse(boot_msg));

    if let Ok(Some(recent)) = session_mgr.most_recent().await {
        let age = Utc::now() - recent.last_active_at;
        if age.num_hours() < 24 && recent.messages.len() > 1 {
            let name_display = recent.name.as_deref().unwrap_or(&recent.id);
            let _ = action_tx.send(Action::AgentResponse(format!(
                "⚙ Previous session: {name_display} ({} msgs, {}). Type /resume to continue.",
                recent.messages.len().saturating_sub(1),
                recent.last_active_at.format("%H:%M %b %d")
            )));
        }
    }

    if engine_status == "OFFLINE" {
        let active = config.engine.active_engine();
        let _ = action_tx.send(Action::AgentResponse(format!(
            "⚠ Engine unreachable at {}. /mode cloud may help if configured.",
            active.url
        )));
    }

    app.run(action_tx, action_rx)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}
