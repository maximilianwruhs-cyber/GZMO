//! Interactive chat REPL.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;

use gzmo_core::agent_loop::{run_agent_loop, AgentLoopConfig};
use gzmo_core::config::GzmoConfig;
use gzmo_core::context::ContextConfig;
use gzmo_core::gateway::{TurboQuantGateway, VllmConfig};
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::mcp::{manager::McpManager, bridge::McpServerConfig};
use gzmo_core::session::SessionManager;
use gzmo_core::tools::ToolRegistry;
use gzmo_core::tools::fs::{FileReadTool, FileWriteTool, DirListTool, FileSearchTool};
use gzmo_core::tools::shell::ShellExecTool;
use gzmo_core::tools::sysadmin::{SysMetricsTool, SysKillTool};
use gzmo_core::tools::web::WebSearchTool;
use gzmo_core::tools::memory::{MemoryRecordTool, MemorySearchTool};
use gzmo_core::types::{EpisodicEntry, EpisodicSource, Message, Role, SoulContext};
use gzmo_core::skills::{SkillRegistry as ChaosSkillRegistry, SkillContext, SkillType};
use gzmo_core::skills::{dice::DiceSkill, sound::SoundSkill, poker::PokerSkill, quote::QuoteSkill, calculate::CalculateSkill, help::HelpSkill, visual::VisualSkill};
use gzmo_chaos::triggers::{TriggerEngine, TriggerAction, NotifyLevel};



// ─── ANSI escape helpers ─────────────────────────────────────────────
const GOLD: &str = "\x1b[38;2;212;175;55m";
const COPPER: &str = "\x1b[38;2;184;115;51m";
const PARCHMENT: &str = "\x1b[38;2;253;246;227m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";

pub async fn run(config: &GzmoConfig, soul: &SoulContext) -> Result<()> {
    let mut config = config.clone();

    // Ensure directories
    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // ─── Vault ───────────────────────────────────────────────────
    let vault = match SqliteVault::open(&config.memory.vault_db) {
        Ok(v) => {
            let count = v.count().unwrap_or(0);
            if count > 0 {
                eprintln!("  {COPPER}⚙ Semantic vault: {count} facts loaded{RESET}");
            }
            Some(Arc::new(v))
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to open vault — continuing without it");
            None
        }
    };

    // ─── Gateway ─────────────────────────────────────────────────
    let gateway = Arc::new(TurboQuantGateway::new(VllmConfig {
        base_url: config.engine.url.clone(),
        model: config.engine.model.clone(),
        temperature: config.engine.temperature,
        top_p: config.engine.top_p,
        max_tokens: config.engine.max_tokens,
        api_key: config.engine.api_key.clone(),
    }));

    // ─── Chaos Engine ────────────────────────────────────────────
    let chaos_config = gzmo_chaos::pulse::ChaosConfig::default();
    let chaos_handle = gzmo_chaos::pulse::PulseLoop::start(chaos_config);
    let chaos_feedback_tx = chaos_handle.feedback_tx.clone();
    let chaos_snapshot_rx = chaos_handle.snapshot_rx.clone();
    eprintln!("  {COPPER}⚙ Chaos engine running — 174 BPM{RESET}");

    // Trigger notification channel: background → REPL
    let (trigger_notify_tx, mut trigger_notify_rx) = tokio::sync::mpsc::channel::<String>(32);

    // Spawn background task: chaos state → gateway + file + trigger evaluation
    {
        let mut snapshot_rx = chaos_handle.snapshot_rx.clone();
        let gateway_ref = Arc::clone(&gateway);
        let feedback_tx_bg = chaos_handle.feedback_tx.clone();
        let notify_tx = trigger_notify_tx.clone();
        tokio::spawn(async move {
            let mut triggers = TriggerEngine::with_defaults();
            loop {
                if snapshot_rx.changed().await.is_err() {
                    break; // PulseLoop dropped
                }
                let snap = snapshot_rx.borrow_and_update().clone();
                // Update gateway LLM parameters from Lorenz coordinates
                gateway_ref.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
                // Write snapshot file for shell skill backward compat
                if snap.tick % 15 == 0 {
                    let json = serde_json::to_string_pretty(&snap).unwrap_or_default();
                    let _ = tokio::fs::write("CHAOS_STATE.json", &json).await;
                }
                // Evaluate autonomous triggers
                let fired = triggers.evaluate(&snap);
                for f in fired {
                    match &f.action {
                        TriggerAction::Notify { message, level } => {
                            let prefix = match level {
                                NotifyLevel::Whisper  => format!("\x1b[2m  {message}\x1b[0m"),
                                NotifyLevel::Normal   => format!("  \x1b[36m{message}\x1b[0m"),
                                NotifyLevel::Urgent   => format!("  \x1b[1m\x1b[33m{message}\x1b[0m"),
                                NotifyLevel::Critical => format!("  \x1b[1m\x1b[31m⚠ {message}\x1b[0m"),
                            };
                            let _ = notify_tx.send(prefix).await;
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
                        TriggerAction::RunSkill { skill_name, .. } => {
                            // Log skill trigger (actual execution happens via notify for now)
                            let _ = notify_tx.send(
                                format!("  \x1b[38;2;212;175;55m⚡ AUTO: /{skill_name} triggered by chaos engine\x1b[0m")
                            ).await;
                        }
                        TriggerAction::InjectPrompt { prompt } => {
                            let _ = notify_tx.send(
                                format!("  \x1b[2m\x1b[35m🧠 {prompt}\x1b[0m")
                            ).await;
                        }
                    }
                }
            }
        });
    }

    // ─── Tools ───────────────────────────────────────────────────
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(FileReadTool));
    tools.register(Box::new(FileWriteTool));
    tools.register(Box::new(DirListTool));
    tools.register(Box::new(FileSearchTool));
    tools.register(Box::new(ShellExecTool::default()));
    tools.register(Box::new(WebSearchTool::default()));
    tools.register(Box::new(SysMetricsTool));
    tools.register(Box::new(SysKillTool));
    
    if let Some(ref v) = vault {
        tools.register(Box::new(MemoryRecordTool { vault: Arc::clone(v) }));
        tools.register(Box::new(MemorySearchTool { vault: Arc::clone(v) }));
    }

    // ─── Chaos Skills (Rust-native, priority over shell) ─────────
    let mut chaos_skills = ChaosSkillRegistry::new();
    chaos_skills.register(Arc::new(DiceSkill));
    chaos_skills.register(Arc::new(SoundSkill));
    chaos_skills.register(Arc::new(PokerSkill));
    chaos_skills.register(Arc::new(QuoteSkill));
    chaos_skills.register(Arc::new(CalculateSkill));
    chaos_skills.register(Arc::new(VisualSkill));
    // Build help entries from registered skills
    let help_entries: Vec<(String, String, &'static str)> = chaos_skills.all().iter().map(|s| {
        let type_label = match s.skill_type() {
            SkillType::Mechanical => "mechanical",
            SkillType::Generative => "generative",
            SkillType::Mutation => "mutation",
            SkillType::Info => "info",
        };
        (s.name().to_string(), s.description().to_string(), type_label)
    }).collect();
    chaos_skills.register(Arc::new(HelpSkill { entries: help_entries }));

    // ─── MCP ─────────────────────────────────────────────────────
    let mut mcp = McpManager::new();
    for server in config.active_mcp_servers() {
        match mcp.connect(McpServerConfig {
            name: server.name.clone(),
            command: server.command.clone(),
            args: server.args.clone(),
            env: server.env.clone(),
        }).await {
            Ok(count) => tracing::info!(server = %server.name, tools = count, "MCP connected"),
            Err(e) => tracing::error!(server = %server.name, "MCP failed: {}", e),
        }
    }
    mcp.register_all_tools(&mut tools);

    // ─── Memory context ──────────────────────────────────────────
    let episodic = FileEpisodicStore::new(&config.memory.directory);

    // Boot persistent memory from Knowledge Graph MCP
    let memory_context = boot_knowledge_graph(&tools).await;

    let vault_context: Option<String> = if let Some(ref v) = vault {
        match v.recent(10) {
            Ok(facts) if !facts.is_empty() => {
                let mut block = String::from("\n\n## Long-Term Memory (Vault)\n");
                for fact in &facts {
                    block.push_str(&format!("- {}\n", fact));
                }
                eprintln!("  {COPPER}⚙ {} vault memories injected{RESET}", facts.len());
                Some(block)
            }
            _ => None,
        }
    } else {
        None
    };

    // ─── System prompt ───────────────────────────────────────────
    let system_prompt = format!(
        "{}{}{}\\n\\n---\\nYou are {}. Today is {}.\\nAvailable tools: {}",
        soul.raw_markdown,
        memory_context.as_deref().unwrap_or(""),
        vault_context.as_deref().unwrap_or(""),
        soul.persona_name,
        Utc::now().format("%Y-%m-%d %H:%M UTC"),
        if tools.is_empty() { "none".to_string() }
        else { tools.definitions().iter().map(|d| d.name.clone()).collect::<Vec<_>>().join(", ") }
    );

    // ─── Session ─────────────────────────────────────────────────
    let session_mgr = SessionManager::new("data/sessions");
    session_mgr.ensure_dir().await?;

    let mut session_id = SessionManager::new_session_id();
    let mut session_name: Option<String> = None;
    let session_created_at = Utc::now();

    let mut messages: Vec<Message> = vec![Message {
        role: Role::System,
        content: system_prompt,
        is_meta: true, tool_calls: None, tool_call_id: None,
    }];

    // Offer resume
    if let Ok(Some(recent)) = session_mgr.most_recent().await {
        let age = Utc::now() - recent.last_active_at;
        if age.num_hours() < 24 && recent.messages.len() > 1 {
            let name_display = recent.name.as_deref().unwrap_or(&recent.id);
            eprintln!("  {DIM}⚙ Previous session: {} ({} msgs, {}){RESET}",
                name_display, recent.messages.len() - 1,
                recent.last_active_at.format("%H:%M %b %d"));
            eprintln!("  {DIM}  Type /resume to continue, or just start typing ›{RESET}");
        }
    }

    let loop_config = AgentLoopConfig {
        max_iterations: config.agent.max_tool_iterations,
        verbose_tool_output: true,
        context: ContextConfig::for_context_length(config.engine.max_tokens as usize * 4),
    };

    // ─── Engine health check ─────────────────────────────────────
    eprintln!("  {DIM}⚙ Pinging engine {}...{RESET}", config.engine.url);
    let (engine_status, engine_latency) = ping_engine(&config).await;

    if engine_status == "OFFLINE" {
        eprintln!("  {RED}⚠ Engine unreachable.{RESET}");
        eprintln!("  Enter a cloud API key for fallback, or press Enter to skip:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let key = input.trim();
        if !key.is_empty() {
            config.engine.api_key = key.to_string();
            config.engine.url = "https://api.groq.com/openai/v1".to_string();
            config.engine.model = "llama3-70b-8192".to_string();
            eprintln!("  \x1b[32m✔ Fallback: {} / {}{RESET}", config.engine.url, config.engine.model);
        }
    }

    // ─── Splash ──────────────────────────────────────────────────
    let vault_count = vault.as_ref().map(|v| v.count().unwrap_or(0)).unwrap_or(0);
    print_splash(&config, engine_status, &engine_latency, vault_count);

    // ─── REPL ────────────────────────────────────────────────────
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        // Drain any trigger notifications from the chaos engine
        while let Ok(notification) = trigger_notify_rx.try_recv() {
            eprintln!("{notification}");
        }

        eprint!("\n  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
        io::stderr().flush()?;

        let mut input = String::new();
        if reader.read_line(&mut input)? == 0 { break; }
        let input = input.trim();
        if input.is_empty() { continue; }

        // ─── Slash commands ─────────────────────────────────
        if input.starts_with('/') {
            if handle_slash_command(
                input, &mut messages, &session_mgr,
                &mut session_id, &mut session_name, session_created_at,
                &vault, &config, &chaos_snapshot_rx,
                &chaos_skills, &chaos_feedback_tx,
            ).await? {
                break; // /quit
            }
            continue;
        }

        // ─── Add user message ───────────────────────────────
        messages.push(Message {
            role: Role::User, content: input.to_string(),
            is_meta: false, tool_calls: None, tool_call_id: None,
        });

        // Log to episodic
        let _ = episodic.append(&EpisodicEntry {
            timestamp: Utc::now(), source: EpisodicSource::UserChat,
            content: input.to_string(), is_silent: false,
        }).await;

        // ─── Inject chaos context ────────────────────────────
        // Remove any previous chaos context message (is_meta system with "CHAOS_STATE" marker)
        messages.retain(|m| !(m.role == Role::System && m.is_meta && m.content.contains("[CHAOS_STATE]")));
        // Inject current state
        {
            let snap = chaos_snapshot_rx.borrow().clone();
            let valence_desc = if snap.llm_valence < -0.5 {
                "intense, restless, aggressive"
            } else if snap.llm_valence < 0.0 {
                "tense, focused, sharp"
            } else if snap.llm_valence < 0.5 {
                "calm, measured, reflective"
            } else {
                "serene, expansive, philosophical"
            };
            let phase_desc = match snap.phase {
                gzmo_chaos::chaos::Phase::Idle => "drifting in low-energy calm",
                gzmo_chaos::chaos::Phase::Build => "building tension, anticipation rising",
                gzmo_chaos::chaos::Phase::Drop => "in free-fall, maximum chaos",
            };
            let chaos_ctx = format!(
                "[CHAOS_STATE] Internal state: {} (valence {:.2}). You are {}. \
                Energy: {:.0}%, tension: {:.0}%. Lorenz orbit: ({:.1},{:.1},{:.1}). \
                Thoughts incubating: {}, crystallized: {}. \
                Let this emotional undertone subtly color your responses — \
                don't mention these numbers, just embody the mood.",
                valence_desc, snap.llm_valence, phase_desc,
                snap.energy, snap.tension, snap.x, snap.y, snap.z,
                snap.thoughts_incubating, snap.thoughts_crystallized,
            );
            // Insert after the main system prompt (position 1)
            let insert_pos = 1.min(messages.len());
            messages.insert(insert_pos, Message {
                role: Role::System,
                content: chaos_ctx,
                is_meta: true, tool_calls: None, tool_call_id: None,
            });
        }

        // ─── Agent loop ─────────────────────────────────────
        let start = std::time::Instant::now();
        let result = run_agent_loop(gateway.as_ref(), &tools, &mut messages, &loop_config).await;
        let elapsed = start.elapsed();

        match result {
            Ok(response) => {
                messages.push(Message {
                    role: Role::Assistant, content: response.text.clone(),
                    is_meta: false, tool_calls: None, tool_call_id: None,
                });
                eprintln!();
                eprintln!("  {DIM}⚙ {:.1}s | {} call(s) | {} tool(s){RESET}",
                    elapsed.as_secs_f64(), response.llm_calls, response.tool_results.len());
                eprintln!();

                let _ = episodic.append(&EpisodicEntry {
                    timestamp: Utc::now(), source: EpisodicSource::InternalMonologue,
                    content: response.text, is_silent: false,
                }).await;
            }
            Err(e) => {
                eprintln!("  {RED}⚙ Error: {e}{RESET}");
                eprintln!();
            }
        }
    }

    Ok(())
}

// ─── Helpers ────────────────────────────────────────────────────────────

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

    let graph: serde_json::Value = serde_json::from_str(&result.output).ok()?;
    let mut block = String::from("\n\n## Persistent Memory (Knowledge Graph)\n\n");
    let mut has_content = false;

    if let Some(entities) = graph.get("entities").and_then(|e| e.as_array()) {
        for entity in entities {
            let name = entity.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let etype = entity.get("entityType").and_then(|t| t.as_str()).unwrap_or("?");
            block.push_str(&format!("- **{}** ({})", name, etype));
            if let Some(obs) = entity.get("observations").and_then(|o| o.as_array()) {
                let obs_strs: Vec<&str> = obs.iter().filter_map(|o| o.as_str()).collect();
                if !obs_strs.is_empty() {
                    block.push_str(&format!(": {}", obs_strs.join("; ")));
                }
            }
            block.push('\n');
            has_content = true;
        }
    }

    if let Some(relations) = graph.get("relations").and_then(|r| r.as_array()) {
        if !relations.is_empty() {
            block.push_str("\nRelationships:\n");
            for rel in relations {
                let from = rel.get("from").and_then(|f| f.as_str()).unwrap_or("?");
                let to = rel.get("to").and_then(|t| t.as_str()).unwrap_or("?");
                let rtype = rel.get("relationType").and_then(|r| r.as_str()).unwrap_or("?");
                block.push_str(&format!("- {} -> ({}) -> {}\n", from, rtype, to));
                has_content = true;
            }
        }
    }

    if has_content {
        eprintln!("  {COPPER}⚙ Persistent memory loaded from Knowledge Graph{RESET}");
        Some(block)
    } else {
        None
    }
}

async fn ping_engine(config: &GzmoConfig) -> (&'static str, String) {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let health_url = format!("{}/models", config.engine.url);
    let start = std::time::Instant::now();

    for _ in 0..15 {
        let req = http.get(&health_url);
        let req = if !config.engine.api_key.is_empty() {
            req.bearer_auth(&config.engine.api_key)
        } else { req };

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                return ("ONLINE", format!("{}ms", start.elapsed().as_millis()));
            }
            _ => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
        }
    }
    ("OFFLINE", String::new())
}

#[allow(clippy::too_many_arguments)]
async fn handle_slash_command(
    input: &str,
    messages: &mut Vec<Message>,
    session_mgr: &SessionManager,
    session_id: &mut String,
    session_name: &mut Option<String>,
    session_created_at: chrono::DateTime<Utc>,
    vault: &Option<Arc<SqliteVault>>,
    config: &GzmoConfig,
    chaos_snapshot_rx: &tokio::sync::watch::Receiver<gzmo_chaos::pulse::ChaosSnapshot>,
    chaos_skills: &ChaosSkillRegistry,
    chaos_feedback_tx: &tokio::sync::mpsc::Sender<gzmo_chaos::feedback::ChaosEvent>,
) -> Result<bool> {
    match input {
        "/quit" | "/exit" | "/q" => {
            if messages.len() > 1 {
                if let Err(e) = session_mgr.save(session_id, session_name.as_deref(), messages, session_created_at).await {
                    eprintln!("  {RED}⚙ Save failed: {e}{RESET}");
                } else {
                    let display = session_name.as_deref().unwrap_or(session_id);
                    eprintln!("  {DIM}⚙ Session saved: {display}{RESET}");
                }
                // Store session summary in vault
                if let Some(ref v) = vault {
                    let topics: Vec<String> = messages.iter()
                        .filter(|m| m.role == Role::User && !m.content.is_empty())
                        .take(20)
                        .map(|m| if m.content.len() > 150 { m.content[..150].to_string() } else { m.content.clone() })
                        .collect();
                    if !topics.is_empty() {
                        let fact = format!("[Session {}] Topics: {}", Utc::now().format("%Y-%m-%d %H:%M"), topics.join(" | "));
                        let _ = v.store_text(&fact, "Episodic", 1.0);
                        eprintln!("  {COPPER}⚙ Session summary stored in vault{RESET}");
                    }
                }
            }
            eprintln!("  {DIM}⚙ GZMO shutting down{RESET}");
            return Ok(true);
        }
        "/clear" | "/reset" => {
            messages.truncate(1);
            *session_id = SessionManager::new_session_id();
            *session_name = None;
            eprintln!("  {DIM}⚙ Context cleared — new session {session_id}{RESET}");
        }
        "/new" => {
            if messages.len() > 1 {
                let _ = session_mgr.save(session_id, session_name.as_deref(), messages, session_created_at).await;
            }
            messages.truncate(1);
            *session_id = SessionManager::new_session_id();
            *session_name = None;
            eprintln!("  {DIM}⚙ New session: {session_id}{RESET}");
        }
        "/resume" => {
            match session_mgr.most_recent().await {
                Ok(Some(session)) => {
                    let count = session.messages.len().saturating_sub(1);
                    let display = session.name.clone().unwrap_or_else(|| session.id.clone());
                    *messages = session.messages;
                    *session_id = session.id;
                    *session_name = session.name;
                    eprintln!("  {DIM}⚙ Resumed: {display} ({count} messages){RESET}");
                }
                _ => eprintln!("  {DIM}⚙ No previous session found{RESET}"),
            }
        }
        "/system" => {
            eprintln!("  {DIM}--- System Prompt ---{RESET}");
            for line in messages[0].content.lines() {
                eprintln!("  {DIM}{line}{RESET}");
            }
            eprintln!("  {DIM}--- End ---{RESET}");
        }
        "/stats" => {
            let display = session_name.as_deref().unwrap_or(session_id);
            eprintln!("  {DIM}⚙ Session: {display} | Messages: {} | Model: {}{RESET}",
                messages.len(), config.engine.model);
        }
        "/chaos" => {
            let snap = chaos_snapshot_rx.borrow().clone();
            eprintln!("  {CYAN}╔══════════════════════════════════════╗{RESET}");
            eprintln!("  {CYAN}║  ⚡ Chaos Engine State               ║{RESET}");
            eprintln!("  {CYAN}╠══════════════════════════════════════╣{RESET}");
            eprintln!("  {CYAN}║{RESET}  Tick: {:<8}  Phase: {:<12}{CYAN}║{RESET}", snap.tick, format!("{}", snap.phase));
            eprintln!("  {CYAN}║{RESET}  Energy: {:<6.1}  Tension: {:<8.1}{CYAN}║{RESET}", snap.energy, snap.tension);
            eprintln!("  {CYAN}║{RESET}  Lorenz: ({:.2}, {:.2}, {:.2})    {CYAN}║{RESET}", snap.x, snap.y, snap.z);
            eprintln!("  {CYAN}║{RESET}  Alive: {:<7}  Deaths: {:<8}{CYAN}║{RESET}", snap.alive, snap.deaths);
            eprintln!("  {CYAN}╠══════════════════════════════════════╣{RESET}");
            eprintln!("  {CYAN}║  🧠 Thought Cabinet                  ║{RESET}");
            eprintln!("  {CYAN}║{RESET}  Incubating: {}  Crystallized: {:<5}{CYAN}║{RESET}", snap.thoughts_incubating, snap.thoughts_crystallized);
            eprintln!("  {CYAN}║{RESET}  Gravity mod:  {:<+8.2}           {CYAN}║{RESET}", snap.mutations.gravity_mod);
            eprintln!("  {CYAN}║{RESET}  Friction mod: {:<+8.2}           {CYAN}║{RESET}", snap.mutations.friction_mod);
            eprintln!("  {CYAN}║{RESET}  Lorenz ρ mod: {:<+8.2}           {CYAN}║{RESET}", snap.mutations.lorenz_rho_mod);
            eprintln!("  {CYAN}╠══════════════════════════════════════╣{RESET}");
            eprintln!("  {CYAN}║  🌡  LLM Parameters (Lorenz-derived) ║{RESET}");
            eprintln!("  {CYAN}║{RESET}  Temperature: {:.3}                {CYAN}║{RESET}", snap.llm_temperature);
            eprintln!("  {CYAN}║{RESET}  Max tokens:  {:<6}               {CYAN}║{RESET}", snap.llm_max_tokens);
            eprintln!("  {CYAN}║{RESET}  Valence:     {:<+.3}               {CYAN}║{RESET}", snap.llm_valence);
            eprintln!("  {CYAN}╚══════════════════════════════════════╝{RESET}");
        }
        "/sessions" => {
            match session_mgr.list().await {
                Ok(sessions) if sessions.is_empty() => eprintln!("  {DIM}⚙ No saved sessions{RESET}"),
                Ok(sessions) => {
                    eprintln!("  {DIM}--- Saved Sessions ---{RESET}");
                    for s in &sessions {
                        let name = s.name.as_deref().unwrap_or("(unnamed)");
                        eprintln!("  {DIM}  {} | {} | {} msgs | {}{RESET}",
                            s.id, name, s.message_count, s.last_active_at.format("%H:%M %b %d"));
                    }
                    eprintln!("  {DIM}--- /load <id or name> to resume ---{RESET}");
                }
                Err(e) => eprintln!("  {RED}⚙ List failed: {e}{RESET}"),
            }
        }
        "/vault" => {
            if let Some(ref v) = vault {
                let count = v.count().unwrap_or(0);
                eprintln!("  {COPPER}⚙ Vault: {count} facts{RESET}");
                if count > 0 {
                    if let Ok(recent) = v.recent(5) {
                        for (i, fact) in recent.iter().enumerate() {
                            let display = if fact.len() > 100 { &fact[..100] } else { fact };
                            eprintln!("  {DIM}  {}. {display}{RESET}", i + 1);
                        }
                    }
                }
            } else {
                eprintln!("  {RED}⚙ Vault not available{RESET}");
            }
        }
        _ if input.starts_with("/save") => {
            let name = input.strip_prefix("/save").map(|s| s.trim()).filter(|s| !s.is_empty());
            if let Some(n) = name { *session_name = Some(n.to_string()); }
            match session_mgr.save(session_id, session_name.as_deref(), messages, session_created_at).await {
                Ok(()) => {
                    let display = session_name.as_deref().unwrap_or(session_id);
                    eprintln!("  {DIM}⚙ Saved: {display}{RESET}");
                }
                Err(e) => eprintln!("  {RED}⚙ Save failed: {e}{RESET}"),
            }
        }
        _ if input.starts_with("/load") => {
            let target = input.strip_prefix("/load").map(|s| s.trim()).unwrap_or("");
            if target.is_empty() {
                eprintln!("  {DIM}⚙ Usage: /load <id or name>{RESET}");
            } else {
                let loaded = match session_mgr.load(target).await {
                    Ok(s) => Some(s),
                    Err(_) => session_mgr.load_by_name(target).await.unwrap_or(None),
                };
                match loaded {
                    Some(session) => {
                        let count = session.messages.len().saturating_sub(1);
                        let display = session.name.clone().unwrap_or_else(|| session.id.clone());
                        *messages = session.messages;
                        *session_id = session.id;
                        *session_name = session.name;
                        eprintln!("  {DIM}⚙ Loaded: {display} ({count} messages){RESET}");
                    }
                    None => eprintln!("  {RED}⚙ Not found: {target}{RESET}"),
                }
            }
        }
        _ if input.starts_with("/remember ") => {
            let fact = input.strip_prefix("/remember ").unwrap_or("").trim();
            if fact.is_empty() {
                eprintln!("  {DIM}⚙ Usage: /remember <fact>{RESET}");
            } else if let Some(ref v) = vault {
                match v.store_text(fact, "Semantic", 1.0) {
                    Ok(()) => eprintln!("  {COPPER}⚙ Stored: {fact}{RESET}"),
                    Err(e) => eprintln!("  {RED}⚙ Store failed: {e}{RESET}"),
                }
            } else {
                eprintln!("  {RED}⚙ Vault not available{RESET}");
            }
        }
        _ => {
            // Parse command and args
            let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
            let cmd = parts[0];
            let args = if parts.len() > 1 { parts[1] } else { "" };

            // ─── Rust skill dispatch (priority) ───────────────
            if chaos_skills.has(cmd) {
                let snap = chaos_snapshot_rx.borrow().clone();
                let ctx = SkillContext {
                    chaos: &snap,
                    feedback_tx: chaos_feedback_tx,
                    args,
                };
                match chaos_skills.get(cmd).unwrap().execute(ctx).await {
                    Ok(output) => {
                        eprint!("{}", output.display);
                        if output.inject_to_conversation {
                            messages.push(Message {
                                role: Role::System,
                                content: format!("[Skill /{}] {}", cmd, 
                                    output.display.replace('\x1b', "") // Strip ANSI for LLM
                                        .chars().take(200).collect::<String>()),
                                is_meta: true, tool_calls: None, tool_call_id: None,
                            });
                        }
                    }
                    Err(e) => eprintln!("  {RED}Skill error: {e}{RESET}"),
                }
            }
            // ─── Shell skill fallback ─────────────────────────
            else {
                let skills_dir = std::path::Path::new("skills");
                let dispatch = skills_dir.join("skill_dispatch.sh");

                if dispatch.exists() {
                    let mut child_args = vec![cmd.to_string()];
                    if !args.is_empty() {
                        child_args.extend(args.split_whitespace().map(String::from));
                    }
                    match std::process::Command::new(&dispatch)
                        .args(&child_args)
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .status()
                    {
                        Ok(status) if !status.success() => {
                            // skill_dispatch.sh already printed the error
                        }
                        Ok(_) => {}
                        Err(e) => eprintln!("  {RED}Skill dispatch error: {e}{RESET}"),
                    }
                } else {
                    eprintln!("  {RED}Unknown command: {input}{RESET}");
                }
            }
        }
    }
    Ok(false)
}

fn print_splash(config: &GzmoConfig, status: &str, latency: &str, vault_count: usize) {
    let ruby = "\x1b[38;2;155;17;30m";
    let gold = "\x1b[38;2;212;175;55m";
    let parchment = "\x1b[38;2;253;246;227m";
    let copper = "\x1b[38;2;184;115;51m";
    let dim = "\x1b[2m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    let bulb_g = format!("{gold}○{reset}");
    let bulb_r = format!("{ruby}●{reset}");

    let top: String = (0..26).map(|i| if i%2==0 { &bulb_g } else { &bulb_r }).cloned().collect::<Vec<_>>().join(" ");
    let bot: String = (0..26).map(|i| if i%2!=0 { &bulb_g } else { &bulb_r }).cloned().collect::<Vec<_>>().join(" ");

    let side_i = std::cell::Cell::new(0usize);
    let pl = |text: &str, color: &str| {
        let curr = side_i.get();
        let (l, r) = if curr.is_multiple_of(2) { (&bulb_r, &bulb_g) } else { (&bulb_g, &bulb_r) };
        side_i.set(curr+1);
        let w: usize = 47;
        let c = text.chars().count();
        let pl = w.saturating_sub(c)/2;
        let pr = w.saturating_sub(c).saturating_sub(pl);
        eprintln!("  {} {}{}{}{}{} {}", l, " ".repeat(pl), color, text, reset, " ".repeat(pr), r);
    };

    eprintln!();
    eprintln!("  {}", top);
    pl("", "");
    pl("★  S T E P   R I G H T   U P  ★", &format!("{ruby}{bold}"));
    pl("", "");
    pl("  ██████╗ ███████╗███╗   ███╗ ██████╗ ", &format!("{parchment}{bold}"));
    pl(" ██╔════╝ ╚══███╔╝████╗ ████║██╔═══██╗", &format!("{parchment}{bold}"));
    pl(" ██║  ███╗  ███╔╝ ██╔████╔██║██║   ██║", &format!("{parchment}{bold}"));
    pl(" ██║   ██║ ███╔╝  ██║╚██╔╝██║██║   ██║", &format!("{parchment}{bold}"));
    pl(" ╚██████╔╝███████╗██║ ╚═╝ ██║╚██████╔╝", &format!("{parchment}{bold}"));
    pl("  ╚═════╝ ╚══════╝╚═╝     ╚═╝ ╚═════╝ ", &format!("{parchment}{bold}"));
    pl("", "");
    pl("⚙  The Incredible Mechanical Marvel  ⚙", copper);
    pl("100% Local · Air-Gapped · Rust", dim);
    pl(&format!("Engine: {} ({})", status, latency), dim);
    pl(&format!("Host: {} | Vault: {} records", config.engine.model, vault_count), dim);
    pl("", "");

    let cmds = format!("{copper}/quit{dim} exit{reset} · {copper}/clear{dim} reset{reset} · {copper}/vault{dim} memory{reset} · {copper}/remember{dim} store{reset}");
    let curr = side_i.get();
    let (l, r) = if curr.is_multiple_of(2) { (&bulb_r, &bulb_g) } else { (&bulb_g, &bulb_r) };
    side_i.set(curr+1);
    eprintln!("  {}   {}   {}", l, cmds, r);

    pl("", "");
    eprintln!("  {}", bot);
    eprintln!();
}
