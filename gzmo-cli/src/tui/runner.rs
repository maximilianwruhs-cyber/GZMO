use std::sync::Arc;
use anyhow::Result;
use chrono::Utc;

use gzmo_core::config::GzmoConfig;
use gzmo_core::gateway::{TurboQuantGateway, VllmConfig};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::session::SessionManager;
use gzmo_core::mcp::{manager::McpManager, bridge::McpServerConfig};
use gzmo_core::tools::ToolRegistry;
use gzmo_core::tools::fs::{FileReadTool, FileWriteTool, DirListTool, FileSearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysMetricsTool, SysKillTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::web_browse::WebBrowseTool;
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::skills::{SkillRegistry, SkillType, NestedDispatch};
use gzmo_core::skills::registry::build_chaos_skill_registry;
use gzmo_chaos::triggers::{TriggerEngine, TriggerAction, NotifyLevel};

use crate::tui::action::Action;
use crate::tui::app::App;
use crate::tui::components::{
    input::InputComponent,
    transcript::TranscriptComponent,
    status_bar::StatusBarComponent,
    chaos_canvas::ChaosCanvasComponent,
    agent::AgentComponent,
    palette::PaletteComponent,
};

/// Boot the Knowledge Graph MCP and return a context string for injection.
async fn boot_knowledge_graph(tools: &ToolRegistry) -> Option<String> {
    let call = gzmo_core::gateway::ToolCall {
        id: "boot_kg_read".to_string(),
        function_name: "mcp__memory__read_graph".to_string(),
        arguments: serde_json::json!({}),
    };
    let result = tools.dispatch(&call).await;
    if !result.success || result.output.trim().is_empty() {
        return None;
    }

    gzmo_core::kg_reconcile::format_knowledge_graph_boot_context(&result.output)
}

/// Boot and run the full-screen TUI interface.
pub async fn run(config: &GzmoConfig, identity: &IdentityEngine) -> Result<()> {
    let config = config.clone();
    let config_arc = Arc::new(tokio::sync::RwLock::new(config.clone()));
    let config_path = std::env::current_dir()?.join("gzmo.toml");

    // ─── Vault ───────────────────────────────────────────────────
    let vault = match SqliteVault::open(&config.memory.vault_db) {
        Ok(v) => Some(Arc::new(v)),
        Err(_) => None,
    };

    // ─── Episodic Memory ─────────────────────────────────────────
    let episodic = Arc::new(FileEpisodicStore::new(&config.memory.directory));

    // ─── Session Manager ─────────────────────────────────────────
    let session_mgr = Arc::new(SessionManager::new("data/sessions"));
    let _ = session_mgr.ensure_dir().await;

    // ─── Gateway (RwLock-wrapped for /mode hot-swap) ─────────────
    let active_profile = config.engine.active_engine();
    let gateway = Arc::new(tokio::sync::RwLock::new(Arc::new(TurboQuantGateway::new(
        VllmConfig::from(active_profile.clone()),
    ))));

    // ─── Chaos Engine ────────────────────────────────────────────
    let chaos_runtime = crate::chaos_bootstrap::start_chaos_runtime(&config);
    let mut chaos_handle = chaos_runtime.handle;
    let chaos_snapshot_rx = chaos_handle.snapshot_rx.clone();
    let chaos_feedback_tx = chaos_runtime.feedback_tx.clone();

    // ─── Chaos Skills (full Rust pantheon) ─────────────────────
    let chaos_skills = Arc::new(build_chaos_skill_registry(&config.pedagogy));

    // ─── Tools ───────────────────────────────────────────────────
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(FileReadTool::default()));
    tools.register(Box::new(FileWriteTool));
    tools.register(Box::new(DirListTool));
    tools.register(Box::new(FileSearchTool::default()));
    tools.register(Box::new(ShellExecTool::default()));
    let serpapi_key = config.api_keys.serpapi_key();
    if serpapi_key.is_empty() {
        tools.register(Box::new(WebSearchTool::default()));
    } else {
        tools.register(Box::new(WebSearchTool::with_serpapi_key(serpapi_key)));
    }
    tools.register(Box::new(WebBrowseTool::default()));
    tools.register(Box::new(SysMetricsTool));
    tools.register(Box::new(SysKillTool));

    if let Some(ref v) = vault {
        tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(v) }));
        tools.register(Box::new(MemorySearchTool::new(Arc::clone(v))));
    }

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

    let tools = Arc::new(tools);

    // ─── Boot Knowledge Graph + Vault context ────────────────────
    let memory_context = boot_knowledge_graph(&tools).await;
    let vault_context: Option<String> = if let Some(ref v) = vault {
        match v.recent(10) {
            Ok(facts) if !facts.is_empty() => {
                let mut block = String::from("\n\n## Long-Term Memory (Vault)\n");
                for fact in &facts {
                    block.push_str(&format!("- {}\n", fact));
                }
                Some(block)
            }
            _ => None,
        }
    } else {
        None
    };

    // ─── System prompt ───────────────────────────────────────────
    let soul = identity.snapshot().await;
    let system_prompt = format!(
        "{}{}{}\n\n---\nYou are {}. Today is {}.\nAvailable tools: {}",
        soul.raw_markdown,
        memory_context.as_deref().unwrap_or(""),
        vault_context.as_deref().unwrap_or(""),
        soul.persona_name,
        Utc::now().format("%Y-%m-%d %H:%M UTC"),
        if tools.is_empty() {
            "none".to_string()
        } else {
            tools
                .definitions()
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );

    // ─── Build Components ────────────────────────────────────────
    let context_budget = active_profile.max_tokens as usize * 4;
    let max_iterations = config.agent.max_tool_iterations;
    // Share the inner soul Arc — IdentityEngine itself isn't Clone (contains watcher)
    let soul_arc = Arc::clone(&identity.soul);

    let input_cmp = InputComponent::new();
    let transcript_cmp = TranscriptComponent::new();
    let status_cmp = StatusBarComponent::new();
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
        chaos_snapshot_rx.clone(),
        Arc::clone(&chaos_skills),
        chaos_feedback_tx.clone(),
        Arc::clone(&config_arc),
        config_path,
    );
    let palette_cmp = PaletteComponent::new();

    let comps = crate::tui::app::AppComponents {
        input: Box::new(input_cmp),
        transcript: Box::new(transcript_cmp),
        status: Box::new(status_cmp),
        canvas: Box::new(canvas_cmp),
        agent: Box::new(agent_cmp),
        palette: Box::new(palette_cmp),
    };
    let (mut app, action_tx, action_rx) = App::new(comps);

    // ─── Background: Chaos → UI + Gateway + Trigger Engine ─────
    {
        let tx = action_tx.clone();
        let mut rx = chaos_snapshot_rx.clone();
        let gateway_ref = Arc::clone(&gateway);
        let feedback_tx_bg = chaos_feedback_tx.clone();
        let state_dir = config.memory.vault_db.parent()
            .unwrap_or(std::path::Path::new(".")).to_path_buf();
        tokio::spawn(async move {
            let mut triggers = TriggerEngine::with_defaults();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                let snap = rx.borrow_and_update().clone();

                // Update gateway LLM parameters from Lorenz coordinates
                let gw = gateway_ref.read().await;
                gw.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
                drop(gw);

                // Send snapshot to UI
                let _ = tx.send(Action::ChaosSnapshot(snap.clone()));

                // Write CHAOS_STATE.json (every 15 ticks for shell compat)
                if snap.tick % 15 == 0 {
                    let json = serde_json::to_string_pretty(&snap).unwrap_or_default();
                    let tmp_path = state_dir.join("CHAOS_STATE.json.tmp");
                    let target_path = state_dir.join("CHAOS_STATE.json");
                    if tokio::fs::write(&tmp_path, json.as_bytes()).await.is_ok() {
                        let _ = tokio::fs::rename(&tmp_path, &target_path).await;
                    }
                }

                // Evaluate autonomous triggers
                let fired = triggers.evaluate(&snap);
                for f in fired {
                    match &f.action {
                        TriggerAction::Notify { message, level } => {
                            let formatted = match level {
                                NotifyLevel::Whisper  => format!("[dim] {}", message),
                                NotifyLevel::Normal   => message.clone(),
                                NotifyLevel::Urgent   => format!("⚠ {}", message),
                                NotifyLevel::Critical => format!("⚠⚠ {}", message),
                            };
                            let _ = tx.send(Action::TriggerNotification(formatted));
                        }
                        TriggerAction::EmitEvent { tension_delta, energy_delta } => {
                            let _ = feedback_tx_bg.send(
                                gzmo_chaos::feedback::ChaosEvent::Custom {
                                    tension_delta: *tension_delta,
                                    energy_delta: *energy_delta,
                                    thought_seed: None,
                                }
                            ).await;
                        }
                        TriggerAction::RunSkill { skill_name, args } => {
                            let _ = tx.send(Action::TriggerSkill(
                                skill_name.clone(),
                                args.clone(),
                            ));
                        }
                        TriggerAction::InjectPrompt { prompt } => {
                            let _ = tx.send(Action::TriggerInject(prompt.clone()));
                        }
                    }
                }
            }
        });
    }

    // ─── Background: Lore → UI ──────────────────────────────────
    {
        let tx = action_tx.clone();
        tokio::spawn(async move {
            // chaos_handle is moved into this task, keeping it alive
            while let Some(lore) = chaos_handle.lore_rx.recv().await {
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

    // Initial system message
    let _ = action_tx.send(Action::AgentResponse(
        "⚙ Systems nominal. Terminal decoupled. Lorenz attractor online.".to_string(),
    ));

    // ─── Run the TUI mainloop ────────────────────────────────────
    app.run(action_tx, action_rx)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}
