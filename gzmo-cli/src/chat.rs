//! Operator REPL (`gzmo` / `gzmo chat`). Hot-memory lifecycle via [`AgentSession`].
//! Canonical operator frontend — see `docs/OPERATOR_FRONTEND_DECISION.md`.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;

use gzmo_core::agent_loop::run_agent_loop;
use gzmo_core::agent_session::AgentSession;
use gzmo_core::config::{EngineMode, GzmoConfig, TaskKind};
use gzmo_core::control_plane::{attach_memory, MemoryAttach};
use gzmo_core::gateway::{GatewayRouter, LlmGateway, TurboQuantGateway, VllmConfig};
use gzmo_core::identity::IdentityEngine;
use gzmo_core::mcp::{bridge::McpServerConfig, manager::McpManager};
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::scratch::{messages_to_transcript, DistillJob, DistillSource};
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::session::SessionManager;
use gzmo_core::skills::dispatch;
use gzmo_core::skills::register_pantheon;
use gzmo_core::skills::{NestedDispatch, SkillRegistry as ChaosSkillRegistry};
use gzmo_core::subagent::SubagentRunner;
use gzmo_core::tools::delegate::DelegateTaskTool;
use gzmo_core::tools::learner::{LearnerRecallTool, LearnerUpdateTool};
use gzmo_core::tools::memory::OwnerMemorySearchTool;
use gzmo_core::tools::profile::{register_for_profile, CapabilityProfile, ToolRegisterOpts};
use gzmo_core::tools::ToolRegistry;
use gzmo_core::types::{EpisodicEntry, EpisodicSource, Message, Role};
use gzmo_core::workflow_skills::{SharedWorkflowSession, WorkflowSkillIndex};

use crate::pedagogy_bridge::{should_delegate_exec, PedagogyRuntime};

// ─── ANSI escape helpers ─────────────────────────────────────────────
const GOLD: &str = "\x1b[38;2;212;175;55m";
const COPPER: &str = "\x1b[38;2;184;115;51m";
const PARCHMENT: &str = "\x1b[38;2;253;246;227m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";

pub async fn run(config: &GzmoConfig, identity: &IdentityEngine) -> Result<()> {
    let mut config = config.clone();
    let config_path = std::env::var("GZMO_CONFIG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("gzmo.toml")
        });
    let soul = identity.snapshot().await;

    // Ensure directories
    tokio::fs::create_dir_all(&config.memory.directory).await?;
    if let Some(parent) = config.memory.vault_db.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // ─── Vault: owner socket, or in-process only for lite / explicit escape ─
    let owner_client = match attach_memory(&config, None, false).await? {
        MemoryAttach::Owner(c) => {
            eprintln!("  {COPPER}⚙ Memory via owner socket (no local vault handle){RESET}");
            Some(c)
        }
        MemoryAttach::Local => None,
    };

    let vault = if owner_client.is_some() {
        None
    } else {
        match crate::repl_shared::open_semantic_vault(&config).await {
            Some(v) => {
                let count = v.count().unwrap_or(0);
                if count > 0 {
                    eprintln!("  {COPPER}⚙ Semantic vault: {count} facts loaded{RESET}");
                }
                Some(v)
            }
            None => None,
        }
    };

    let mut agent_session = AgentSession::new_main(
        &config.redis,
        &config.context_memory,
        SessionManager::new_session_id(),
    )
    .await;
    if agent_session.uses_redis() {
        if agent_session.scratch().redis_live().await {
            eprintln!("  {COPPER}⚙ Scratch cache: Redis (LXC101){RESET}");
        } else {
            eprintln!(
                "  {COPPER}⚙ Scratch cache: Redis configured but unreachable — \
                 in-memory buffer, retrying{RESET}"
            );
        }
    } else {
        eprintln!("  {COPPER}⚙ Scratch cache: in-memory (Redis disabled){RESET}");
    }
    let scratch = agent_session.scratch();

    // ─── Gateway ─────────────────────────────────────────────────
    let active_profile = config.engine.active_engine();
    let gateway: Arc<tokio::sync::RwLock<Arc<dyn LlmGateway>>> =
        Arc::new(tokio::sync::RwLock::new(
            Arc::new(TurboQuantGateway::new(VllmConfig::from(active_profile)))
                as Arc<dyn LlmGateway>,
        ));

    // ─── Chaos Engine (opt-in; ADR-0003 quarantined by default) ──
    let chaos_boot = crate::chaos_bootstrap::boot_chat_chaos(&config);
    let chaos_enabled = chaos_boot.enabled;
    let mut chaos_handle = chaos_boot.runtime.map(|r| r.handle);
    let chaos_feedback_tx = chaos_boot.feedback_tx.clone();
    let chaos_snapshot_rx = chaos_boot.snapshot_rx.clone();
    // When chaos is off, lore select branch parks forever (no PulseHandle).
    if chaos_enabled {
        eprintln!("  {COPPER}⚙ Chaos engine running — 174 BPM (HW telemetry active){RESET}");
    } else {
        eprintln!(
            "  {DIM}⚙ Chaos quarantined — set [chaos].enabled_in_chat = true to enable{RESET}"
        );
    }

    // Trigger notification channel: background → REPL
    let (trigger_notify_tx, mut trigger_notify_rx) = tokio::sync::mpsc::channel::<String>(32);

    // Spawn background task: chaos state → gateway + file + trigger evaluation
    let _chaos_bridge = if chaos_enabled {
        let gateway_ref = gateway.clone();
        let state_dir = config
            .memory
            .vault_db
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();
        Some(crate::chaos_bootstrap::spawn_snapshot_bridge(
            chaos_snapshot_rx.clone(),
            gateway_ref,
            chaos_feedback_tx.clone(),
            state_dir,
            Some(trigger_notify_tx),
            None,
            None,
            gzmo_core::synapse::EventSource::GzmoCli,
            chaos_boot.restore_policy.clone(),
            true, // interactive REPL — no periodic autonomous monologue injects
            crate::chaos_bootstrap::SnapshotBridgeOpts::STDIO,
        ))
    } else {
        None
    };

    // ─── Workflow skills ─────────────────────────────────────────
    let (workflow_index, workflow_session) = crate::repl_shared::boot_workflow_skills(&config)?;
    if !workflow_index.is_empty() {
        eprintln!(
            "  {COPPER}⚙ Workflow skills: {}{RESET}",
            workflow_index.names().join(", ")
        );
    }

    // ─── Tools (capability profile + jail) ───────────────────────
    let profile =
        CapabilityProfile::parse(&config.tools.profile).unwrap_or(CapabilityProfile::Developer);
    let mut tools = ToolRegistry::new();
    register_for_profile(
        &mut tools,
        profile,
        &config.tools,
        ToolRegisterOpts {
            vault: vault.clone(),
            scratch: None, // wired after AgentSession
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
    eprintln!("  {COPPER}⚙ Tool profile: {}{RESET}", profile.as_str());

    let router = GatewayRouter::new(&config);
    let chat_gateway_dyn = router.gateway(TaskKind::Chat);

    // ─── Pedagogy (Wave 2b — mentor path before agent loop) ──────
    let mut pedagogy_runtime = PedagogyRuntime::boot(&config).await?;
    if config.pedagogy.enabled {
        eprintln!("  {COPPER}⚙ Pedagogy: mentor path on (maybe_teach before agent loop){RESET}");
    }

    // ─── Chaos Skills (Rust-native, priority over shell) ─────────
    let mut chaos_skills = ChaosSkillRegistry::new();
    register_pantheon(&mut chaos_skills, &config);

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

    // ─── Memory context ──────────────────────────────────────────
    let episodic = FileEpisodicStore::new(&config.memory.directory);

    // Boot persistent memory from Knowledge Graph MCP
    let memory_context = crate::repl_shared::boot_knowledge_graph(&tools).await;
    if memory_context.is_some() {
        eprintln!("  {COPPER}⚙ Persistent memory loaded from Knowledge Graph{RESET}");
    }

    let vault_context: Option<String> = if let Some(ref v) = vault {
        match v.recent(10) {
            Ok(facts) if !facts.is_empty() => {
                // Prefer curated honeypot when M3 cognition path is live (vault.recent already does).
                let mut block = String::from(
                    "\n\n## Long-Term Memory (Honeypot-first vault)\n\
                     Prefer these curated facts over raw episodic soup.\n",
                );
                for fact in &facts {
                    block.push_str(&format!("- {}\n", fact));
                }
                eprintln!(
                    "  {COPPER}⚙ {} vault memories injected (honeypot-first){RESET}",
                    facts.len()
                );
                Some(block)
            }
            _ => None,
        }
    } else {
        None
    };

    // ─── System prompt ───────────────────────────────────────────
    let last_handoff = workflow_session
        .lock()
        .ok()
        .and_then(|s| s.last_handoff.clone())
        .or_else(|| workflow_index.latest_handoff());
    let mut system_prompt = crate::repl_shared::build_system_prompt_with_workflows(
        &soul,
        memory_context.as_deref(),
        vault_context.as_deref(),
        &if tools.is_empty() {
            vec![]
        } else {
            tools.definitions().iter().map(|d| d.name.clone()).collect()
        },
        &Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        Some(workflow_index.as_ref()),
        last_handoff.as_deref(),
    );
    if config.pedagogy.enabled {
        let suffix = pedagogy_runtime.learner_prompt_suffix();
        if !suffix.is_empty() {
            system_prompt.push_str(&suffix);
        }
    }

    // ─── Session ─────────────────────────────────────────────────
    let session_mgr = SessionManager::new(&config.session_distill.sessions_dir);
    session_mgr.ensure_dir().await?;

    let mut session_name: Option<String> = None;
    let session_created_at = Utc::now();

    let subagent_runner = Arc::new(SubagentRunner::with_tools_config(
        config.subagent.clone(),
        config.tools.clone(),
        Arc::clone(&scratch),
        Arc::clone(&chat_gateway_dyn),
        vault.clone(),
        system_prompt.clone(),
        Some(config.clone()),
    ));
    agent_session.attach_subagent_runner(Arc::clone(&subagent_runner));
    if config.subagent.enabled {
        tools.register(Box::new(DelegateTaskTool {
            runner: Arc::clone(&subagent_runner),
            session_id: agent_session.session_id().to_string(),
            depth: 0,
        }));
        eprintln!(
            "  {COPPER}⚙ SubagentRunner: max {} parallel, {}k ctx budget{RESET}",
            config.subagent.max_concurrent,
            config.subagent.context_budget_tokens / 1024
        );
    }

    if let Some(ref v) = vault {
        tools.register(Box::new(gzmo_core::tools::memory::MemorySearchTool {
            vault: Arc::clone(v),
            scratch: Some(agent_session.scratch()),
            scope: Some(agent_session.main_scope()),
            scope_cell: None,
        }));
    } else if let Some(client) = owner_client.clone() {
        tools.register(Box::new(OwnerMemorySearchTool { client }));
    }

    if config.pedagogy.enabled {
        tools.register(Box::new(LearnerRecallTool::new(&config.pedagogy)));
        tools.register(Box::new(LearnerUpdateTool::new(&config.pedagogy)));
    }

    let mut messages: Vec<Message> = vec![Message {
        role: Role::System,
        content: system_prompt,
        is_meta: true,
        tool_calls: None,
        tool_call_id: None,
    }];

    // Offer resume
    if let Ok(Some(recent)) = session_mgr.most_recent().await {
        let age = Utc::now() - recent.last_active_at;
        if age.num_hours() < 24 && recent.messages.len() > 1 {
            let name_display = recent.name.as_deref().unwrap_or(&recent.id);
            eprintln!(
                "  {DIM}⚙ Previous session: {} ({} msgs, {}){RESET}",
                name_display,
                recent.messages.len() - 1,
                recent.last_active_at.format("%H:%M %b %d")
            );
            eprintln!("  {DIM}  Type /resume to continue, or just start typing ›{RESET}");
        }
    }

    let mut loop_config = agent_session.loop_config(config.agent.max_tool_iterations, true, None);

    // ─── Engine health check ─────────────────────────────────────
    let active = config.engine.active_engine();
    eprintln!("  {DIM}⚙ Pinging engine {}...{RESET}", active.url);
    let (engine_status, engine_latency) = crate::repl_shared::ping_engine(&config).await;

    if engine_status == "OFFLINE" {
        eprintln!("  {RED}⚠ Engine unreachable at {}{RESET}", active.url);
        if config.engine.cloud.is_some() {
            eprintln!("  {DIM}Tip: type /mode cloud to switch to cloud inference{RESET}");
        } else {
            eprintln!("  {DIM}No cloud profile configured — start local engine or add [engine.cloud] to gzmo.toml{RESET}");
        }
    }

    // ─── Splash ──────────────────────────────────────────────────
    let vault_count = vault.as_ref().map(|v| v.count().unwrap_or(0)).unwrap_or(0);
    print_splash(&config, engine_status, &engine_latency, vault_count);

    // ─── REPL (async stdin + chaos notifications) ─────────────
    // Use a channel to receive stdin lines asynchronously so we can
    // display chaos notifications in real-time while waiting for input.
    let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(4);
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let reader = stdin.lock();
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if stdin_tx.blocking_send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Print initial prompt
    eprint!("\n  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
    io::stderr().flush()?;
    let mut prompt_dirty = false; // Track if we need to reprint the prompt

    loop {
        // Race between user input and chaos notifications
        tokio::select! {
            // ─── User input ─────────────────────────────────
            line = stdin_rx.recv() => {
                let Some(raw) = line else { break; }; // EOF
                let input = raw.trim().to_string();
                prompt_dirty = false;

                if input.is_empty() {
                    eprint!("\n  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
                    io::stderr().flush()?;
                    continue;
                }

        // ─── Slash commands ─────────────────────────────────
        if input.starts_with('/') {
            let msg_count_before = messages.len();
            let subagent_enabled = config.subagent.enabled;
            if handle_slash_command(
                &input, &mut messages, &mut tools, &session_mgr,
                &mut agent_session, &mut session_name, session_created_at,
                &vault, &mut config, &chaos_snapshot_rx,
                &chaos_skills, &chaos_feedback_tx,
                &gateway, &config_path,
                &subagent_runner, subagent_enabled,
                &workflow_index, &workflow_session,
            ).await? {
                break; // /quit
            }

            // Workflow inject → full agent loop (with tools)
            if messages.len() > msg_count_before {
                let last = &messages[messages.len() - 1];
                if last.role == Role::System && last.is_meta && last.content.starts_with("[Workflow") {
                    agent_session.turn_start().await;
                    loop_config = agent_session.loop_config(config.agent.max_tool_iterations, true, None);
                    messages.push(Message {
                        role: Role::User,
                        content: "Begin following the activated workflow skill now.".into(),
                        is_meta: true,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    eprintln!("  {DIM}⚙ workflow...{RESET}");
                    let start = std::time::Instant::now();
                    match run_agent_loop(gateway.read().await.as_ref(), &tools, &mut messages, &loop_config).await {
                        Ok(response) => {
                            messages.push(Message {
                                role: Role::Assistant,
                                content: response.text.clone(),
                                is_meta: false,
                                tool_calls: None,
                                tool_call_id: None,
                            });
                            eprintln!("  {DIM}⚙ {:.1}s{RESET}", start.elapsed().as_secs_f64());
                        }
                        Err(e) => eprintln!("  {RED}⚙ workflow turn failed: {e}{RESET}"),
                    }
                    eprint!("\n  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
                    io::stderr().flush()?;
                    continue;
                }
            }

            // Auto-narration: if a skill injected a message, run one agent loop
            // so the LLM reacts to the skill output in character
            if messages.len() > msg_count_before {
                let last = &messages[messages.len() - 1];
                if last.role == Role::System && last.is_meta && last.content.starts_with("[Skill") {
                    // Remove stale chaos/narration context
                    messages.retain(|m| !(m.role == Role::System && m.is_meta &&
                        (m.content.contains("[CHAOS_STATE]") || m.content.contains("[NARRATION]"))));
                    // Append narration instruction at the END (right after skill output)
                    // so the LLM sees it immediately before generating its response
                    {
                        let snap = chaos_snapshot_rx.borrow().clone();
                        let valence_desc = if snap.llm_valence < -0.5 { "intense, restless" }
                            else if snap.llm_valence < 0.0 { "tense, focused" }
                            else if snap.llm_valence < 0.5 { "calm, reflective" }
                            else { "serene, philosophical" };
                        let narration_prompt = format!(
                            "React to the dice event above in ONE short atmospheric sentence. \
                            Your current mood is {}. Be dramatic and mystical, like a fortune teller. \
                            Examples of good reactions: \
                            \"The cosmos held its breath — and exhaled fire.\" \
                            \"Twenty. The attractor sings. I feel it in every register.\" \
                            \"A shadow-roll. The engine grows hungry.\" \
                            Now write YOUR reaction (one sentence, no formatting):",
                            valence_desc,
                        );
                        // Push as a User message at the end so it's the last thing the LLM sees
                        messages.push(Message {
                            role: Role::User, content: narration_prompt,
                            is_meta: true, tool_calls: None, tool_call_id: None,
                        });
                    }

                    eprintln!("  {DIM}⚙ narrating...{RESET}");
                    let start = std::time::Instant::now();
                    // Use empty tool set for narration — prevents the LLM from
                    // calling tools (dir_list, etc.) instead of narrating the skill result.
                    let narration_tools = ToolRegistry::new();
                    match run_agent_loop(gateway.read().await.as_ref(), &narration_tools, &mut messages, &loop_config).await {
                        Ok(response) => {
                            messages.push(Message {
                                role: Role::Assistant, content: response.text.clone(),
                                is_meta: false, tool_calls: None, tool_call_id: None,
                            });
                            let elapsed = start.elapsed();
                            eprintln!("  {DIM}⚙ {:.1}s{RESET}", elapsed.as_secs_f64());
                        }
                        Err(e) => eprintln!("  {DIM}⚙ narration failed: {e}{RESET}"),
                    }
                }
            }
            // Keep pedagogy session in sync after /ops, /learn, etc.
            if config.pedagogy.enabled {
                if let Err(e) = pedagogy_runtime.reload_from_disk().await {
                    eprintln!("  {DIM}⚙ pedagogy reload: {e}{RESET}");
                }
            }
            // Reprint prompt after slash command
            eprint!("\n  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
            io::stderr().flush()?;
            continue;
        }

        // ─── New turn: hot memory lifecycle (platform) ─────────
        agent_session.turn_start().await;
        loop_config = agent_session.loop_config(config.agent.max_tool_iterations, true, None);

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

        // ─── SOUL.md hot-reload: refresh system prompt if identity changed ─
        {
            let live_soul = identity.snapshot().await;
            if live_soul.loaded_at != soul.loaded_at {
                eprintln!("  {COPPER}⚙ SOUL.md hot-reloaded — persona: {}{RESET}", live_soul.persona_name);
                let mut new_prompt = format!(
                    "{}\n\n---\nYou are {}. Today is {}.\nAvailable tools: {}",
                    live_soul.raw_markdown,
                    live_soul.persona_name,
                    Utc::now().format("%Y-%m-%d %H:%M UTC"),
                    if tools.is_empty() { "none".to_string() }
                    else { tools.definitions().iter().map(|d| d.name.clone()).collect::<Vec<_>>().join(", ") }
                );
                if config.pedagogy.enabled {
                    let suffix = pedagogy_runtime.learner_prompt_suffix();
                    if !suffix.is_empty() {
                        new_prompt.push_str(&suffix);
                    }
                }
                if !messages.is_empty() {
                    messages[0].content = new_prompt;
                }
            }
        }

        // ─── Inject chaos context (only when pulse is live) ──
        messages.retain(|m| !(m.role == Role::System && m.is_meta && m.content.contains("[CHAOS_STATE]")));
        if chaos_enabled {
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
            let insert_pos = 1.min(messages.len());
            messages.insert(insert_pos, Message {
                role: Role::System,
                content: chaos_ctx,
                is_meta: true, tool_calls: None, tool_call_id: None,
            });
        }

        // ─── Pedagogy mentor path (Wave 2b) ──────────────────
        // Socratic turns skip the tool agent loop; ops intent falls through.
        if config.pedagogy.enabled && !should_delegate_exec(&pedagogy_runtime.session, &input)
        {
            let chaos_context = if chaos_enabled {
                let snap = chaos_snapshot_rx.borrow().clone();
                Some(format!(
                    "valence={:.2} energy={:.0} tension={:.0}",
                    snap.llm_valence, snap.energy, snap.tension
                ))
            } else {
                None
            };
            let tutor = router.gateway(TaskKind::Chat);
            let start = std::time::Instant::now();
            match pedagogy_runtime
                .maybe_teach(
                    &config,
                    &router,
                    tutor.as_ref(),
                    &input,
                    &messages,
                    chaos_context.as_deref(),
                    None,
                    if chaos_enabled {
                        Some(&chaos_snapshot_rx)
                    } else {
                        None
                    },
                )
                .await
            {
                Ok(Some(turn)) => {
                    eprintln!("  {DIM}⚙ pedagogy orchestrator | mentor mode{RESET}");
                    eprint!("{}", turn.response);
                    eprintln!();
                    messages.push(Message {
                        role: Role::Assistant,
                        content: turn.response.clone(),
                        is_meta: false,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    eprintln!(
                        "  {DIM}⚙ {:.1}s | mentor (no agent loop){RESET}",
                        start.elapsed().as_secs_f64()
                    );
                    let _ = episodic
                        .append(&EpisodicEntry {
                            timestamp: Utc::now(),
                            source: EpisodicSource::InternalMonologue,
                            content: turn.response,
                            is_silent: false,
                        })
                        .await;
                    eprint!("\n  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
                    io::stderr().flush()?;
                    continue;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!(
                        "  {DIM}⚙ pedagogy failed ({e}); falling through to agent loop{RESET}"
                    );
                }
            }
        }

        // ─── Agent loop ─────────────────────────────────────
        let start = std::time::Instant::now();
        let result = run_agent_loop(gateway.read().await.as_ref(), &tools, &mut messages, &loop_config).await;
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
        // Reprint prompt after response
        eprint!("\n  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
        io::stderr().flush()?;
            } // end stdin arm

            // ─── Trigger notifications (real-time) ──────────────
            notification = trigger_notify_rx.recv() => {
                if let Some(notification) = notification {
                    if prompt_dirty {
                        eprintln!(); // Newline before notification if prompt is active
                    }
                    if let Some(skill_cmd) = notification.strip_prefix("__TRIGGER_SKILL__:") {
                        let parts: Vec<&str> = skill_cmd.splitn(2, ' ').collect();
                        let cmd = parts[0];
                        let args = if parts.len() > 1 { parts[1] } else { "" };
                        if chaos_skills.has(cmd) {
                            eprintln!("  {GOLD}⚡ AUTO: /{cmd} triggered by chaos engine{RESET}");
                            let snap = chaos_snapshot_rx.borrow().clone();
                            let profile = config.engine.active_engine();
                            let gw = gateway.read().await;
                            let ctx = dispatch::skill_context(
                                &snap,
                                &chaos_feedback_tx,
                                args,
                                Some(gw.as_ref()),
                                Some(&router),
                                &config,
                                NestedDispatch {
                                    registry: Some(&chaos_skills),
                                    profile: Some(&profile),
                                    depth: 0,
                                },
                            );
                            match chaos_skills.get(cmd).unwrap().execute(ctx).await {
                                Ok(output) => eprint!("{}", output.display),
                                Err(e) => eprintln!("  {RED}Auto-skill error: {e}{RESET}"),
                            }
                        }
                    } else if let Some(_prompt) = notification.strip_prefix("__TRIGGER_INJECT__:") {
                        // InjectPrompt is disabled in interactive mode; ignore if one slips through.
                    } else {
                        eprintln!("{notification}");
                    }
                    // Reprint prompt after notification
                    eprint!("  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
                    io::stderr().flush()?;
                    prompt_dirty = true;
                }
            }

            // ─── Lore notifications (real-time; idle when chaos off) ─
            lore = async {
                match chaos_handle.as_mut() {
                    Some(h) => h.lore_rx.recv().await,
                    None => futures::future::pending().await,
                }
            } => {
                if let Some(lore_notif) = lore {
                    if prompt_dirty {
                        eprintln!(); // Newline before lore if prompt is active
                    }
                    let author_str = lore_notif.author.as_deref().unwrap_or("");
                    let prefix = match lore_notif.category.as_str() {
                        "joke" => "🃏",
                        "quote" => "📜",
                        "fact" => "📡",
                        _ => "📖",
                    };
                    if author_str.is_empty() {
                        eprintln!("  {DIM}{prefix} {}{RESET}", lore_notif.text);
                    } else {
                        eprintln!("  {DIM}{prefix} \"{}\" — {author_str}{RESET}", lore_notif.text);
                    }
                    // Reprint prompt after lore
                    eprint!("  {GOLD}★{RESET} {PARCHMENT}{BOLD}you ›{RESET} ");
                    io::stderr().flush()?;
                    prompt_dirty = true;
                }
            }
        } // end select!
    } // end loop

    Ok(())
}

fn strip_autonomous_monologue(messages: &mut Vec<Message>) {
    messages.retain(|m| {
        !(m.role == Role::System && m.is_meta && m.content.starts_with("[AUTONOMOUS MONOLOGUE]"))
    });
}

// ─── Helpers ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn reregister_delegate_tool(
    tools: &mut ToolRegistry,
    runner: &Arc<SubagentRunner>,
    session_id: &str,
) {
    tools.register(Box::new(DelegateTaskTool {
        runner: Arc::clone(runner),
        session_id: session_id.to_string(),
        depth: 0,
    }));
}

async fn handle_slash_command(
    input: &str,
    messages: &mut Vec<Message>,
    tools: &mut ToolRegistry,
    session_mgr: &SessionManager,
    agent_session: &mut AgentSession,
    session_name: &mut Option<String>,
    session_created_at: chrono::DateTime<Utc>,
    vault: &Option<Arc<SqliteVault>>,
    config: &mut GzmoConfig,
    chaos_snapshot_rx: &tokio::sync::watch::Receiver<gzmo_chaos::pulse::ChaosSnapshot>,
    chaos_skills: &ChaosSkillRegistry,
    chaos_feedback_tx: &tokio::sync::mpsc::Sender<gzmo_chaos::feedback::ChaosEvent>,
    gateway: &Arc<tokio::sync::RwLock<Arc<dyn LlmGateway>>>,
    config_path: &std::path::Path,
    subagent_runner: &Arc<SubagentRunner>,
    subagent_enabled: bool,
    workflow_index: &WorkflowSkillIndex,
    workflow_session: &SharedWorkflowSession,
) -> Result<bool> {
    let session_id = agent_session.session_id();
    match input {
        "/quit" | "/exit" | "/q" => {
            if messages.len() > 1 {
                let transcript = messages_to_transcript(messages);
                let job = DistillJob {
                    session_id: session_id.to_string(),
                    transcript,
                    source: DistillSource::MainArchive,
                };
                if let Err(e) = agent_session.scratch().enqueue_distill(job).await {
                    eprintln!("  {RED}⚙ Distill enqueue failed: {e}{RESET}");
                } else {
                    eprintln!("  {COPPER}⚙ Session distill enqueued{RESET}");
                }
                if let Err(e) = session_mgr
                    .save(
                        session_id,
                        session_name.as_deref(),
                        messages,
                        session_created_at,
                    )
                    .await
                {
                    eprintln!("  {RED}⚙ Save failed: {e}{RESET}");
                } else {
                    let display = session_name.as_deref().unwrap_or(session_id);
                    eprintln!("  {DIM}⚙ Session saved: {display}{RESET}");
                }
                // Store session summary in vault
                if let Some(ref v) = vault {
                    let topics: Vec<String> = messages
                        .iter()
                        .filter(|m| m.role == Role::User && !m.content.is_empty())
                        .take(20)
                        .map(|m| gzmo_core::text_util::truncate_chars(&m.content, 150))
                        .collect();
                    if !topics.is_empty() {
                        let fact = format!(
                            "[Session {}] Topics: {}",
                            Utc::now().format("%Y-%m-%d %H:%M"),
                            topics.join(" | ")
                        );
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
            let new_id = SessionManager::new_session_id();
            agent_session.set_session_id(new_id.clone());
            *session_name = None;
            if subagent_enabled {
                reregister_delegate_tool(tools, subagent_runner, agent_session.session_id());
            }
            eprintln!("  {DIM}⚙ Context cleared — new session {new_id}{RESET}");
        }
        "/new" => {
            if messages.len() > 1 {
                let _ = session_mgr
                    .save(
                        session_id,
                        session_name.as_deref(),
                        messages,
                        session_created_at,
                    )
                    .await;
            }
            messages.truncate(1);
            let new_id = SessionManager::new_session_id();
            agent_session.set_session_id(new_id.clone());
            *session_name = None;
            if subagent_enabled {
                reregister_delegate_tool(tools, subagent_runner, agent_session.session_id());
            }
            eprintln!("  {DIM}⚙ New session: {new_id}{RESET}");
        }
        "/resume" => match session_mgr.most_recent().await {
            Ok(Some(session)) => {
                let count = session.messages.len().saturating_sub(1);
                let display = session.name.clone().unwrap_or_else(|| session.id.clone());
                *messages = session.messages;
                strip_autonomous_monologue(messages);
                agent_session.set_session_id(session.id.clone());
                *session_name = session.name;
                if subagent_enabled {
                    reregister_delegate_tool(tools, subagent_runner, agent_session.session_id());
                }
                eprintln!("  {DIM}⚙ Resumed: {display} ({count} messages){RESET}");
            }
            _ => eprintln!("  {DIM}⚙ No previous session found{RESET}"),
        },
        "/system" => {
            eprintln!("  {DIM}--- System Prompt ---{RESET}");
            for line in messages[0].content.lines() {
                eprintln!("  {DIM}{line}{RESET}");
            }
            eprintln!("  {DIM}--- End ---{RESET}");
        }
        "/stats" => {
            let display = session_name.as_deref().unwrap_or(session_id);
            let active = config.engine.active_engine();
            let mode_str = match config.engine.active_mode {
                EngineMode::Local => format!("{COPPER}LOCAL{RESET}"),
                EngineMode::Cloud => format!("\x1b[38;2;100;200;255mCLOUD{RESET}"),
                EngineMode::Sovereign => format!("\x1b[38;2;180;140;255mSOVEREIGN{RESET}"),
            };
            eprintln!(
                "  {DIM}⚙ Session: {display} | Messages: {} | Mode: {} | Model: {}{RESET}",
                messages.len(),
                mode_str,
                active.model
            );
        }
        _ if input.starts_with("/mode") => {
            let arg = input.strip_prefix("/mode").unwrap_or("").trim();
            if arg.is_empty() {
                // Show current mode
                let active = config.engine.active_engine();
                let mode_str = match config.engine.active_mode {
                    EngineMode::Local => format!("{COPPER}⚙ LOCAL{RESET}"),
                    EngineMode::Cloud => format!("\x1b[38;2;100;200;255m☁ CLOUD{RESET}"),
                    EngineMode::Sovereign => format!("\x1b[38;2;180;140;255m⚡ SOVEREIGN{RESET}"),
                };
                eprintln!("  Mode: {mode_str}");
                eprintln!("  {DIM}Engine: {} → {}{RESET}", active.provider, active.url);
                eprintln!("  {DIM}Model: {}{RESET}", active.model);
                eprintln!("  {DIM}Usage: /mode local | /mode cloud | /mode sovereign{RESET}");
            } else {
                match arg.parse::<EngineMode>() {
                    Ok(new_mode) => {
                        if new_mode == config.engine.active_mode {
                            eprintln!("  {DIM}⚙ Already in {} mode{RESET}", new_mode);
                        } else {
                            let profile = config.engine.active_engine_for_mode(new_mode);
                            let test_url = format!("{}/models", profile.url.trim_end_matches('/'));
                            eprint!("  {DIM}⚙ Pinging {}...{RESET}", profile.url);

                            let ping_ok = match reqwest::Client::new()
                                .get(&test_url)
                                .timeout(std::time::Duration::from_secs(5))
                                .send()
                                .await
                            {
                                Ok(r) if r.status().is_success() || r.status().as_u16() == 401 => {
                                    true
                                }
                                _ => false,
                            };

                            if !ping_ok
                                && matches!(new_mode, EngineMode::Local | EngineMode::Sovereign)
                            {
                                eprintln!(" ✗");
                                eprintln!(
                                    "  {RED}⚙ Engine not reachable at {}{RESET}",
                                    profile.url
                                );
                                eprintln!(
                                    "  {DIM}  Prime: :8000 | Sovereign: scripts in llama.cpp/prime-bench | embed: scripts/start-embed.sh{RESET}"
                                );
                            } else {
                                eprintln!(" ✓");
                                // Build new gateway
                                let new_gw: Arc<dyn LlmGateway> = Arc::new(TurboQuantGateway::new(
                                    VllmConfig::from(profile.clone()),
                                ));
                                // Swap gateway
                                {
                                    let mut gw = gateway.write().await;
                                    *gw = new_gw;
                                }
                                // Update config
                                config.engine.active_mode = new_mode;
                                // Persist to disk
                                if let Err(e) = config.persist_active_mode(config_path, new_mode) {
                                    eprintln!("  {RED}⚙ Failed to persist mode: {e}{RESET}");
                                }
                                let mode_str = match new_mode {
                                    EngineMode::Local => format!("{COPPER}⚙ LOCAL{RESET}"),
                                    EngineMode::Cloud => {
                                        format!("\x1b[38;2;100;200;255m☁ CLOUD{RESET}")
                                    }
                                    EngineMode::Sovereign => {
                                        format!("\x1b[38;2;180;140;255m⚡ SOVEREIGN{RESET}")
                                    }
                                };
                                eprintln!("  Switched to: {mode_str}");
                                eprintln!(
                                    "  {DIM}Model: {} → {}{RESET}",
                                    profile.model, profile.url
                                );
                            }
                        }
                    }
                    Err(e) => eprintln!("  {RED}⚙ {e}{RESET}"),
                }
            }
        }
        "/calibrate" => {
            eprintln!("  {DIM}⚙ Running calibration assembly (fixture)...{RESET}");
            if let Err(e) = crate::assemble_cmd::run(config, "calibration", true, false).await {
                eprintln!("  {RED}⚙ {e}{RESET}");
            } else {
                eprintln!("  {GREEN}⚙ Calibration assembly complete{RESET}");
            }
        }
        "/cognition-smoke" | "/cognition" => {
            eprintln!("  {DIM}⚙ Running cognition assembly (fixture)...{RESET}");
            if let Err(e) = crate::assemble_cmd::run(config, "cognition", true, false).await {
                eprintln!("  {RED}⚙ {e}{RESET}");
            } else {
                eprintln!("  {GREEN}⚙ Cognition assembly complete{RESET}");
            }
        }
        "/ops-smoke" | "/ops" => {
            eprintln!("  {DIM}⚙ Running ops assembly (fixture)...{RESET}");
            if let Err(e) = crate::assemble_cmd::run(config, "ops", true, false).await {
                eprintln!("  {RED}⚙ {e}{RESET}");
            } else {
                eprintln!("  {GREEN}⚙ Ops assembly complete{RESET}");
            }
        }
        "/status" | "/ecosystem" => {
            let report = gzmo_core::ecosystem_status::format_ecosystem_status(config).await;
            for line in report.lines() {
                eprintln!("  {DIM}{line}{RESET}");
            }
            for line in crate::repl_shared::workflow_status_lines(workflow_index, workflow_session)
            {
                eprintln!("  {DIM}{line}{RESET}");
            }
        }
        "/chaos" => {
            let snap = chaos_snapshot_rx.borrow().clone();
            eprintln!("  {CYAN}╔══════════════════════════════════════╗{RESET}");
            eprintln!("  {CYAN}║  ⚡ Chaos Engine State               ║{RESET}");
            eprintln!("  {CYAN}╠══════════════════════════════════════╣{RESET}");
            eprintln!(
                "  {CYAN}║{RESET}  Tick: {:<8}  Phase: {:<12}{CYAN}║{RESET}",
                snap.tick,
                format!("{}", snap.phase)
            );
            eprintln!(
                "  {CYAN}║{RESET}  Energy: {:<6.1}  Tension: {:<8.1}{CYAN}║{RESET}",
                snap.energy, snap.tension
            );
            eprintln!(
                "  {CYAN}║{RESET}  Lorenz: ({:.2}, {:.2}, {:.2})    {CYAN}║{RESET}",
                snap.x, snap.y, snap.z
            );
            eprintln!(
                "  {CYAN}║{RESET}  Alive: {:<7}  Deaths: {:<8}{CYAN}║{RESET}",
                snap.alive, snap.deaths
            );
            eprintln!("  {CYAN}╠══════════════════════════════════════╣{RESET}");
            eprintln!("  {CYAN}║  🧠 Thought Cabinet                  ║{RESET}");
            eprintln!(
                "  {CYAN}║{RESET}  Incubating: {}  Crystallized: {:<5}{CYAN}║{RESET}",
                snap.thoughts_incubating, snap.thoughts_crystallized
            );
            eprintln!(
                "  {CYAN}║{RESET}  Gravity mod:  {:<+8.2}           {CYAN}║{RESET}",
                snap.mutations.gravity_mod
            );
            eprintln!(
                "  {CYAN}║{RESET}  Friction mod: {:<+8.2}           {CYAN}║{RESET}",
                snap.mutations.friction_mod
            );
            eprintln!(
                "  {CYAN}║{RESET}  Lorenz ρ mod: {:<+8.2}  Δ{:+.3}     {CYAN}║{RESET}",
                snap.mutations.lorenz_rho_mod, snap.rho_mod_delta
            );
            eprintln!(
                "  {CYAN}║{RESET}  ρ_eff: {:.2}  forcing: {:+}  breath: {:<2} {CYAN}║{RESET}",
                snap.rho_effective, snap.rho_forcing_sign, snap.rho_breath_phase
            );
            let chaos_config: gzmo_chaos::pulse::ChaosConfig = config
                .chaos
                .as_ref()
                .and_then(|v| v.clone().try_into().ok())
                .unwrap_or_default();
            let policy_str = if chaos_config.rho_restore_alpha > 0.0 {
                format!(
                    "tanh (α={:.2}, β={:.2})",
                    chaos_config.rho_restore_alpha, chaos_config.rho_restore_beta
                )
            } else {
                format!("linear (k={:.4})", chaos_config.rho_decay_k)
            };
            eprintln!(
                "  {CYAN}║{RESET}  Restore policy: {:<21}{CYAN}║{RESET}",
                policy_str
            );
            eprintln!("  {CYAN}╠══════════════════════════════════════╣{RESET}");
            eprintln!("  {CYAN}║  🌡  LLM Parameters (Lorenz-derived) ║{RESET}");
            eprintln!(
                "  {CYAN}║{RESET}  Temperature: {:.3}                {CYAN}║{RESET}",
                snap.llm_temperature
            );
            eprintln!(
                "  {CYAN}║{RESET}  Max tokens:  {:<6}               {CYAN}║{RESET}",
                snap.llm_max_tokens
            );
            eprintln!(
                "  {CYAN}║{RESET}  Valence:     {:<+.3}               {CYAN}║{RESET}",
                snap.llm_valence
            );
            eprintln!("  {CYAN}╚══════════════════════════════════════╝{RESET}");
        }
        "/sessions" => match session_mgr.list().await {
            Ok(sessions) if sessions.is_empty() => eprintln!("  {DIM}⚙ No saved sessions{RESET}"),
            Ok(sessions) => {
                eprintln!("  {DIM}--- Saved Sessions ---{RESET}");
                for s in &sessions {
                    let name = s.name.as_deref().unwrap_or("(unnamed)");
                    eprintln!(
                        "  {DIM}  {} | {} | {} msgs | {}{RESET}",
                        s.id,
                        name,
                        s.message_count,
                        s.last_active_at.format("%H:%M %b %d")
                    );
                }
                eprintln!("  {DIM}--- /load <id or name> to resume ---{RESET}");
            }
            Err(e) => eprintln!("  {RED}⚙ List failed: {e}{RESET}"),
        },
        "/vault" => {
            if let Some(ref v) = vault {
                let count = v.count().unwrap_or(0);
                eprintln!("  {COPPER}⚙ Vault: {count} facts{RESET}");
                if count > 0 {
                    if let Ok(recent) = v.recent(5) {
                        for (i, fact) in recent.iter().enumerate() {
                            let display = gzmo_core::text_util::truncate_chars(fact, 100);
                            eprintln!("  {DIM}  {}. {display}{RESET}", i + 1);
                        }
                    }
                }
            } else {
                eprintln!("  {RED}⚙ Vault not available{RESET}");
            }
        }
        _ if input.starts_with("/save") => {
            let name = input
                .strip_prefix("/save")
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
            if let Some(n) = name {
                *session_name = Some(n.to_string());
            }
            match session_mgr
                .save(
                    session_id,
                    session_name.as_deref(),
                    messages,
                    session_created_at,
                )
                .await
            {
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
                        strip_autonomous_monologue(messages);
                        agent_session.set_session_id(session.id.clone());
                        *session_name = session.name;
                        if subagent_enabled {
                            reregister_delegate_tool(
                                tools,
                                subagent_runner,
                                agent_session.session_id(),
                            );
                        }
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

            // ecosystem alias → status skill
            let cmd = if cmd == "ecosystem" { "status" } else { cmd };

            // Aliases for workflow skills
            let wf_name = match cmd {
                "grill-me" => "grill",
                "debug" | "diagnosing-bugs" => "diagnose",
                "code-review" => "review",
                other => other,
            };

            // ─── Workflow skill dispatch ──────────────────────
            if workflow_index.has(wf_name) {
                match crate::repl_shared::activate_workflow_slash(
                    workflow_index,
                    workflow_session,
                    wf_name,
                    args,
                    messages,
                ) {
                    Ok(true) => {
                        eprintln!("  {COPPER}⚙ Workflow activated: /{wf_name}{RESET}");
                        if wf_name == "handoff" {
                            // Seed a stub handoff file; agent will flesh it out.
                            let session_id = agent_session.session_id();
                            let stub = format!(
                                "# Handoff\n\n**Session:** {session_id}\n\n**Topic:** {}\n\n(Agent will complete sections.)\n",
                                if args.is_empty() { "(unspecified)" } else { args }
                            );
                            match workflow_index.write_handoff(workflow_session, session_id, &stub)
                            {
                                Ok(path) => {
                                    eprintln!(
                                        "  {COPPER}⚙ Handoff stub: {}{RESET}",
                                        path.display()
                                    );
                                    if config.workflow_skills.handoff_to_vault {
                                        if let Some(ref v) = vault {
                                            let _ = v.store_text(
                                                &format!("[Handoff] {}", path.display()),
                                                "Episodic",
                                                1.0,
                                            );
                                        }
                                    }
                                }
                                Err(e) => eprintln!("  {RED}⚙ Handoff write failed: {e}{RESET}"),
                            }
                        }
                    }
                    Ok(false) => {}
                    Err(e) => eprintln!("  {RED}⚙ Workflow error: {e}{RESET}"),
                }
            }
            // ─── Rust skill dispatch (priority) ───────────────
            else if chaos_skills.has(cmd) {
                let snap = chaos_snapshot_rx.borrow().clone();
                let profile = config.engine.active_engine();
                let gw = gateway.read().await;
                let ctx = dispatch::skill_context(
                    &snap,
                    chaos_feedback_tx,
                    args,
                    Some(gw.as_ref()),
                    None,
                    &config,
                    NestedDispatch {
                        registry: Some(&chaos_skills),
                        profile: Some(&profile),
                        depth: 0,
                    },
                );
                match chaos_skills.get(cmd).unwrap().execute(ctx).await {
                    Ok(output) => {
                        eprint!("{}", output.display);
                        if output.inject_to_conversation {
                            // Strip ANSI escapes, box-drawing chars, and excessive whitespace
                            // to give the LLM clean semantic text, not garbled ASCII art
                            let clean: String = output
                                .display
                                .replace('\x1b', "")
                                .chars()
                                .filter(|c| {
                                    !matches!(
                                        c,
                                        '┌' | '┐'
                                            | '└'
                                            | '┘'
                                            | '├'
                                            | '┤'
                                            | '─'
                                            | '│'
                                            | '╔'
                                            | '╗'
                                            | '╚'
                                            | '╝'
                                            | '║'
                                            | '═'
                                            | '╠'
                                            | '╣'
                                            | '/'
                                            | '\\'
                                            | '_'
                                    )
                                })
                                .collect::<String>()
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ");
                            let clean = clean.chars().take(300).collect::<String>();
                            messages.push(Message {
                                role: Role::System,
                                content: format!("[Skill /{}] {}", cmd, clean),
                                is_meta: true,
                                tool_calls: None,
                                tool_call_id: None,
                            });
                        }
                    }
                    Err(e) => eprintln!("  {RED}Skill error: {e}{RESET}"),
                }
            }
            // ─── Shell skill fallback ─────────────────────────
            else {
                let skills_dir = config.skills.directory.clone();
                let dispatch = skills_dir.join("skill_dispatch.sh");

                if dispatch.exists() {
                    if let Ok(dispatch_canon) = std::fs::canonicalize(&dispatch) {
                        let ok_starts_with = std::fs::canonicalize(&config.skills.directory)
                            .map(|base_canon| dispatch_canon.starts_with(&base_canon))
                            .unwrap_or(false);

                        if ok_starts_with {
                            let mut child_args = vec![cmd.to_string()];
                            if !args.is_empty() {
                                child_args.extend(args.split_whitespace().map(String::from));
                            }
                            match tokio::process::Command::new(&dispatch_canon)
                                .args(&child_args)
                                .stdin(std::process::Stdio::inherit())
                                .stdout(std::process::Stdio::inherit())
                                .stderr(std::process::Stdio::inherit())
                                .status()
                                .await
                            {
                                Ok(status) if !status.success() => {
                                    // skill_dispatch.sh already printed the error
                                }
                                Ok(_) => {}
                                Err(e) => eprintln!("  {RED}Skill dispatch error: {e}{RESET}"),
                            }
                        } else {
                            eprintln!(
                                "  {RED}Security alert: skill script outside base directory{RESET}"
                            );
                        }
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

    let top: String = (0..26)
        .map(|i| if i % 2 == 0 { &bulb_g } else { &bulb_r })
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let bot: String = (0..26)
        .map(|i| if i % 2 != 0 { &bulb_g } else { &bulb_r })
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    let side_i = std::cell::Cell::new(0usize);
    let pl = |text: &str, color: &str| {
        let curr = side_i.get();
        let (l, r) = if curr % 2 == 0 {
            (&bulb_r, &bulb_g)
        } else {
            (&bulb_g, &bulb_r)
        };
        side_i.set(curr + 1);
        let w: usize = 47;
        let c = text.chars().count();
        let pl = w.saturating_sub(c) / 2;
        let pr = w.saturating_sub(c).saturating_sub(pl);
        eprintln!(
            "  {} {}{}{}{}{} {}",
            l,
            " ".repeat(pl),
            color,
            text,
            reset,
            " ".repeat(pr),
            r
        );
    };

    eprintln!();
    eprintln!("  {}", top);
    pl("", "");
    pl("★  S T E P   R I G H T   U P  ★", &format!("{ruby}{bold}"));
    pl("", "");
    pl(
        "  ██████╗ ███████╗███╗   ███╗ ██████╗ ",
        &format!("{parchment}{bold}"),
    );
    pl(
        " ██╔════╝ ╚══███╔╝████╗ ████║██╔═══██╗",
        &format!("{parchment}{bold}"),
    );
    pl(
        " ██║  ███╗  ███╔╝ ██╔████╔██║██║   ██║",
        &format!("{parchment}{bold}"),
    );
    pl(
        " ██║   ██║ ███╔╝  ██║╚██╔╝██║██║   ██║",
        &format!("{parchment}{bold}"),
    );
    pl(
        " ╚██████╔╝███████╗██║ ╚═╝ ██║╚██████╔╝",
        &format!("{parchment}{bold}"),
    );
    pl(
        "  ╚═════╝ ╚══════╝╚═╝     ╚═╝ ╚═════╝ ",
        &format!("{parchment}{bold}"),
    );
    pl("", "");
    let active = config.engine.active_engine();
    let mode_tag = match config.engine.active_mode {
        gzmo_core::config::EngineMode::Local => "LOCAL",
        gzmo_core::config::EngineMode::Cloud => "CLOUD",
        gzmo_core::config::EngineMode::Sovereign => "SOVEREIGN",
    };
    pl("⚙  The Incredible Mechanical Marvel  ⚙", copper);
    pl(&format!("Mode: {} · Rust · Sovereign", mode_tag), dim);
    pl(&format!("Engine: {} ({})", status, latency), dim);
    pl(
        &format!("Model: {} | Vault: {} records", active.model, vault_count),
        dim,
    );
    pl("", "");

    let cmds = format!("{copper}/quit{dim} exit{reset} · {copper}/status{dim} ecosystem{reset} · {copper}/clear{dim} reset{reset} · {copper}/mode{dim} switch{reset} · {copper}/vault{dim} memory{reset}");
    let curr = side_i.get();
    let (l, r) = if curr % 2 == 0 {
        (&bulb_r, &bulb_g)
    } else {
        (&bulb_g, &bulb_r)
    };
    side_i.set(curr + 1);
    eprintln!("  {}   {}   {}", l, cmds, r);

    pl("", "");
    eprintln!("  {}", bot);
    eprintln!();
}
