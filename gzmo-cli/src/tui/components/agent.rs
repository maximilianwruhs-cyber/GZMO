use chrono::Utc;
use color_eyre::Result;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use gzmo_core::agent_loop::{run_agent_loop, AgentLoopConfig};
use gzmo_core::context::ContextConfig;
use gzmo_core::gateway::{TurboQuantGateway, VllmConfig};
use gzmo_core::types::{EpisodicEntry, EpisodicSource, Message, Role, SoulContext};

use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::pulse::ChaosSnapshot;
use gzmo_core::memory::episodic::FileEpisodicStore;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::session::SessionManager;
use gzmo_core::skills::{SkillContext, SkillRegistry as ChaosSkillRegistry};
use gzmo_core::tools::ToolRegistry;

use crate::tui::action::Action;
use crate::tui::component::Component;

/// Headless agent orchestrator component — no render, pure logic.
/// Handles chaos context injection, SOUL.md hot-reload, session management,
/// episodic logging, and streaming token dispatch.
pub struct AgentComponent {
    pub action_tx: Option<UnboundedSender<Action>>,
    pub gateway: Arc<tokio::sync::RwLock<Arc<TurboQuantGateway>>>,
    pub tools: Arc<ToolRegistry>,
    pub messages: Vec<Message>,
    pub max_iterations: usize,
    pub context_budget: usize,
    pub soul: Arc<tokio::sync::RwLock<SoulContext>>,
    pub vault: Option<Arc<SqliteVault>>,
    pub episodic: Arc<FileEpisodicStore>,
    pub session_mgr: Arc<SessionManager>,
    pub session_id: String,
    pub session_name: Option<String>,
    pub session_created_at: chrono::DateTime<Utc>,
    pub chaos_snapshot_rx: tokio::sync::watch::Receiver<ChaosSnapshot>,
    pub chaos_skills: Arc<ChaosSkillRegistry>,
    pub chaos_feedback_tx: tokio::sync::mpsc::Sender<ChaosEvent>,
    pub config: Arc<tokio::sync::RwLock<gzmo_core::config::GzmoConfig>>,
    pub config_path: std::path::PathBuf,
    /// Cached soul loaded_at for hot-reload detection
    pub soul_loaded_at: Option<chrono::DateTime<Utc>>,
}

impl AgentComponent {
    pub fn new(
        gateway: Arc<tokio::sync::RwLock<Arc<TurboQuantGateway>>>,
        tools: Arc<ToolRegistry>,
        system_prompt: String,
        max_iterations: usize,
        context_budget: usize,
        soul: Arc<tokio::sync::RwLock<SoulContext>>,
        vault: Option<Arc<SqliteVault>>,
        episodic: Arc<FileEpisodicStore>,
        session_mgr: Arc<SessionManager>,
        chaos_snapshot_rx: tokio::sync::watch::Receiver<ChaosSnapshot>,
        chaos_skills: Arc<ChaosSkillRegistry>,
        chaos_feedback_tx: tokio::sync::mpsc::Sender<ChaosEvent>,
        config: Arc<tokio::sync::RwLock<gzmo_core::config::GzmoConfig>>,
        config_path: std::path::PathBuf,
    ) -> Self {
        let messages = vec![Message {
            role: Role::System,
            content: system_prompt,
            is_meta: true,
            tool_calls: None,
            tool_call_id: None,
        }];

        Self {
            action_tx: None,
            gateway,
            tools,
            messages,
            max_iterations,
            context_budget,
            soul,
            vault,
            episodic,
            session_mgr,
            session_id: SessionManager::new_session_id(),
            session_name: None,
            session_created_at: Utc::now(),
            chaos_snapshot_rx,
            chaos_skills,
            chaos_feedback_tx,
            config,
            config_path,
            soul_loaded_at: None,
        }
    }

    /// Inject [CHAOS_STATE] context message at position 1, replacing any existing one.
    fn inject_chaos_context(messages: &mut Vec<Message>, snap: &ChaosSnapshot) {
        // Remove any previous chaos context
        messages.retain(|m| {
            !(m.role == Role::System && m.is_meta && m.content.contains("[CHAOS_STATE]"))
        });

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
            valence_desc,
            snap.llm_valence,
            phase_desc,
            snap.energy,
            snap.tension,
            snap.x,
            snap.y,
            snap.z,
            snap.thoughts_incubating,
            snap.thoughts_crystallized,
        );

        let insert_pos = 1.min(messages.len());
        messages.insert(
            insert_pos,
            Message {
                role: Role::System,
                content: chaos_ctx,
                is_meta: true,
                tool_calls: None,
                tool_call_id: None,
            },
        );
    }
}

impl Component for AgentComponent {
    fn init(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(action_tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        // ─── Sync mutated messages from background tasks ─────
        if let Action::AgentMessagesSync(synced_msgs) = action {
            self.messages = synced_msgs;
            return Ok(None);
        }

        // ─── Trigger: inject autonomous prompt ───────────────
        if let Action::TriggerInject(prompt) = action {
            self.messages.push(Message {
                role: Role::System,
                content: format!("[AUTONOMOUS MONOLOGUE] {}", prompt),
                is_meta: true,
                tool_calls: None,
                tool_call_id: None,
            });
            return Ok(None);
        }

        // ─── Trigger: auto-run a chaos skill ─────────────────
        if let Action::TriggerSkill(skill_name, args) = action {
            let skills = Arc::clone(&self.chaos_skills);
            let snap = self.chaos_snapshot_rx.borrow().clone();
            let feedback_tx = self.chaos_feedback_tx.clone();
            let action_tx = self.action_tx.as_ref().unwrap().clone();

            tokio::spawn(async move {
                if skills.has(&skill_name) {
                    let ctx = SkillContext {
                        chaos: &snap,
                        feedback_tx: &feedback_tx,
                        args: &args,
                    };
                    match skills.get(&skill_name).unwrap().execute(ctx).await {
                        Ok(output) => {
                            // Strip ANSI for transcript display
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
                                    )
                                })
                                .collect::<String>()
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" ");
                            let _ = action_tx.send(Action::AgentResponse(format!(
                                "⚡ AUTO /{}: {}",
                                skill_name,
                                clean.chars().take(300).collect::<String>()
                            )));
                        }
                        Err(e) => {
                            let _ = action_tx.send(Action::AgentResponse(format!(
                                "⚡ Auto-skill /{} error: {}",
                                skill_name, e
                            )));
                        }
                    }
                }
            });
            return Ok(None);
        }

        if let Action::SubmitInput(text) = action {
            // ─── Slash commands ────────────────────────────────
            if text.starts_with('/') {
                let action_tx = self.action_tx.as_ref().unwrap().clone();
                let cmd = text.clone();

                // We need to handle slash commands async — but Component::update is sync.
                let messages = std::mem::take(&mut self.messages);
                let session_mgr = Arc::clone(&self.session_mgr);
                let vault = self.vault.clone();
                let session_id = self.session_id.clone();
                let session_name = self.session_name.clone();
                let session_created_at = self.session_created_at;
                let chaos_snapshot_rx = self.chaos_snapshot_rx.clone();
                let chaos_skills = Arc::clone(&self.chaos_skills);
                let chaos_feedback_tx = self.chaos_feedback_tx.clone();
                let config = Arc::clone(&self.config);
                let config_path = self.config_path.clone();
                let gateway = Arc::clone(&self.gateway);

                // Restore messages immediately (slash commands will get a copy)
                self.messages = messages.clone();

                tokio::spawn(async move {
                    let mut ctx = SlashCommandContext {
                        action_tx: action_tx.clone(),
                        messages,
                        session_mgr,
                        vault,
                        session_id,
                        session_name,
                        session_created_at,
                        chaos_snapshot_rx,
                        chaos_skills,
                        chaos_feedback_tx,
                        config,
                        config_path,
                        gateway,
                    };
                    ctx.handle(&cmd).await;

                    // Sync messages back
                    let _ = action_tx.send(Action::AgentMessagesSync(ctx.messages));
                });

                return Ok(None);
            }

            // ─── SOUL.md hot-reload handled in spawned agent task ─

            // ─── Add user message ──────────────────────────────
            self.messages.push(Message {
                role: Role::User,
                content: text.clone(),
                is_meta: false,
                tool_calls: None,
                tool_call_id: None,
            });

            // ─── Log to episodic ───────────────────────────────
            {
                let episodic = Arc::clone(&self.episodic);
                let text_clone = text.clone();
                tokio::spawn(async move {
                    let _ = episodic
                        .append(&EpisodicEntry {
                            timestamp: Utc::now(),
                            source: EpisodicSource::UserChat,
                            content: text_clone,
                            is_silent: false,
                        })
                        .await;
                });
            }

            // ─── Inject chaos context ──────────────────────────
            let snap = self.chaos_snapshot_rx.borrow().clone();
            Self::inject_chaos_context(&mut self.messages, &snap);

            // ─── Spawn agent loop ──────────────────────────────
            let action_tx = self.action_tx.as_ref().unwrap().clone();
            let gateway = Arc::clone(&self.gateway);
            let tools = Arc::clone(&self.tools);
            let mut msgs = self.messages.clone();
            let max_iter = self.max_iterations;
            let ctx_budget = self.context_budget;
            let episodic = Arc::clone(&self.episodic);
            let soul = Arc::clone(&self.soul);
            let tool_defs = self.tools.definitions();

            tokio::spawn(async move {
                // SOUL.md hot-reload check
                {
                    let live_soul = soul.read().await.clone();
                    if !msgs.is_empty() {
                        let tool_names = if tool_defs.is_empty() {
                            "none".to_string()
                        } else {
                            tool_defs
                                .iter()
                                .map(|d| d.name.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        // Rebuild system prompt with current soul
                        let new_prompt = format!(
                            "{}\n\n---\nYou are {}. Today is {}.\nAvailable tools: {}",
                            live_soul.raw_markdown,
                            live_soul.persona_name,
                            Utc::now().format("%Y-%m-%d %H:%M UTC"),
                            tool_names,
                        );
                        msgs[0].content = new_prompt;
                    }
                }

                // Build streaming callback
                let stream_tx = action_tx.clone();
                let on_chunk = Arc::new(move |token: String| {
                    let _ = stream_tx.send(Action::AgentTokenStream(token));
                });

                let config = AgentLoopConfig {
                    max_iterations: max_iter,
                    verbose_tool_output: false,
                    context: ContextConfig::for_context_length(ctx_budget),
                    on_chunk: Some(on_chunk),
                };

                let gw = gateway.read().await;
                let res = run_agent_loop(gw.as_ref(), tools.as_ref(), &mut msgs, &config).await;

                // Sync mutated conversation history back
                let _ = action_tx.send(Action::AgentMessagesSync(msgs));

                match res {
                    Ok(resp) => {
                        // Log agent response to episodic
                        let resp_text = resp.text.clone();
                        let ep = episodic.clone();
                        tokio::spawn(async move {
                            let _ = ep
                                .append(&EpisodicEntry {
                                    timestamp: Utc::now(),
                                    source: EpisodicSource::InternalMonologue,
                                    content: resp_text,
                                    is_silent: false,
                                })
                                .await;
                        });
                        let _ = action_tx.send(Action::AgentResponse(resp.text));
                    }
                    Err(e) => {
                        let _ = action_tx.send(Action::AgentResponse(format!("[ERROR] {}", e)));
                    }
                }
            });
        }
        Ok(None)
    }
}

/// Lightweight context for async slash command execution.
struct SlashCommandContext {
    action_tx: UnboundedSender<Action>,
    messages: Vec<Message>,
    session_mgr: Arc<SessionManager>,
    vault: Option<Arc<SqliteVault>>,
    session_id: String,
    session_name: Option<String>,
    session_created_at: chrono::DateTime<Utc>,
    chaos_snapshot_rx: tokio::sync::watch::Receiver<ChaosSnapshot>,
    chaos_skills: Arc<ChaosSkillRegistry>,
    chaos_feedback_tx: tokio::sync::mpsc::Sender<ChaosEvent>,
    config: Arc<tokio::sync::RwLock<gzmo_core::config::GzmoConfig>>,
    config_path: std::path::PathBuf,
    gateway: Arc<tokio::sync::RwLock<Arc<TurboQuantGateway>>>,
}

impl SlashCommandContext {
    async fn handle(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
        let raw_cmd = parts[0].to_lowercase();
        let args = if parts.len() > 1 { parts[1] } else { "" };

        match raw_cmd.as_str() {
            "/quit" | "/exit" | "/q" => {
                if self.messages.len() > 1 {
                    let _ = self
                        .session_mgr
                        .save(
                            &self.session_id,
                            self.session_name.as_deref(),
                            &self.messages,
                            self.session_created_at,
                        )
                        .await;
                }
                let _ = self.action_tx.send(Action::Quit);
            }
            "/clear" | "/reset" => {
                self.messages.truncate(1);
                self.session_id = SessionManager::new_session_id();
                self.session_name = None;
                let _ = self.action_tx.send(Action::TranscriptClear);
                let _ = self.action_tx.send(Action::AgentResponse(
                    "⚙ Context cleared — new session.".to_string(),
                ));
            }
            "/resume" => {
                if let Ok(Some(session)) = self.session_mgr.most_recent().await {
                    let count = session.messages.len().saturating_sub(1);
                    let display = session.name.clone().unwrap_or_else(|| session.id.clone());
                    self.messages = session.messages.clone();
                    self.session_id = session.id;
                    self.session_name = session.name;
                    let _ = self
                        .action_tx
                        .send(Action::TranscriptRestore(self.messages.clone()));
                    let _ = self.action_tx.send(Action::AgentResponse(format!(
                        "⚙ Resumed: {} ({} messages)",
                        display, count
                    )));
                } else {
                    let _ = self.action_tx.send(Action::AgentResponse(
                        "⚙ No previous session found.".to_string(),
                    ));
                }
            }
            "/new" => {
                if self.messages.len() > 1 {
                    let _ = self
                        .session_mgr
                        .save(
                            &self.session_id,
                            self.session_name.as_deref(),
                            &self.messages,
                            self.session_created_at,
                        )
                        .await;
                }
                self.messages.truncate(1);
                self.session_id = SessionManager::new_session_id();
                self.session_name = None;
                self.session_created_at = Utc::now();
                let _ = self.action_tx.send(Action::TranscriptClear);
                let _ = self.action_tx.send(Action::AgentResponse(format!(
                    "⚙ New session: {}",
                    self.session_id
                )));
            }
            "/system" => {
                let prompt = self
                    .messages
                    .first()
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                let _ = self.action_tx.send(Action::AgentResponse(format!(
                    "--- System Prompt ---\n{}\n--- End ---",
                    prompt
                )));
            }
            "/stats" => {
                let display = self.session_name.as_deref().unwrap_or(&self.session_id);
                let config = self.config.read().await;
                let active = config.engine.active_engine();
                let mode_str = match config.engine.active_mode {
                    gzmo_core::config::EngineMode::Local => "LOCAL",
                    gzmo_core::config::EngineMode::Cloud => "CLOUD",
                };
                let _ = self.action_tx.send(Action::AgentResponse(format!(
                    "⚙ Session: {} | Messages: {} | Mode: {} | Model: {}",
                    display,
                    self.messages.len(),
                    mode_str,
                    active.model
                )));
            }
            "/chaos" => {
                let snap = self.chaos_snapshot_rx.borrow().clone();
                let _ = self.action_tx.send(Action::AgentResponse(format!(
                    "⚡ Chaos Engine\n  Phase: {:?} | Energy: {:.1}% | Tension: {:.1}%\n  \
                     Lorenz: ({:.2}, {:.2}, {:.2})\n  Alive: {} | Deaths: {}\n  \
                     Thoughts: {} incubating, {} crystallized\n  \
                     LLM Temp: {:.3} | Tokens: {} | Valence: {:+.3}",
                    snap.phase,
                    snap.energy,
                    snap.tension,
                    snap.x,
                    snap.y,
                    snap.z,
                    snap.alive,
                    snap.deaths,
                    snap.thoughts_incubating,
                    snap.thoughts_crystallized,
                    snap.llm_temperature,
                    snap.llm_max_tokens,
                    snap.llm_valence,
                )));
            }
            "/vault" => {
                if let Some(ref v) = self.vault {
                    let count = v.count().unwrap_or(0);
                    let mut text = format!("⚙ Vault: {} facts", count);
                    if count > 0 {
                        if let Ok(recent) = v.recent(5) {
                            for (i, fact) in recent.iter().enumerate() {
                                let display = if fact.len() > 100 {
                                    &fact[..100]
                                } else {
                                    fact.as_str()
                                };
                                text.push_str(&format!("\n  {}. {}", i + 1, display));
                            }
                        }
                    }
                    let _ = self.action_tx.send(Action::AgentResponse(text));
                } else {
                    let _ = self
                        .action_tx
                        .send(Action::AgentResponse("⚙ Vault not available".to_string()));
                }
            }
            "/save" => {
                let name = args.trim();
                let name_opt = if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                };
                if name_opt.is_some() {
                    self.session_name = name_opt.clone();
                }
                match self
                    .session_mgr
                    .save(
                        &self.session_id,
                        self.session_name.as_deref(),
                        &self.messages,
                        self.session_created_at,
                    )
                    .await
                {
                    Ok(()) => {
                        let display = self.session_name.as_deref().unwrap_or(&self.session_id);
                        let _ = self
                            .action_tx
                            .send(Action::AgentResponse(format!("⚙ Saved: {}", display)));
                    }
                    Err(e) => {
                        let _ = self
                            .action_tx
                            .send(Action::AgentResponse(format!("⚙ Save failed: {}", e)));
                    }
                }
            }
            "/load" => {
                let target = args.trim();
                if target.is_empty() {
                    let _ = self.action_tx.send(Action::AgentResponse(
                        "⚙ Usage: /load <id or name>".to_string(),
                    ));
                } else {
                    let loaded = match self.session_mgr.load(target).await {
                        Ok(s) => Some(s),
                        Err(_) => self.session_mgr.load_by_name(target).await.unwrap_or(None),
                    };
                    match loaded {
                        Some(session) => {
                            let count = session.messages.len().saturating_sub(1);
                            let display =
                                session.name.clone().unwrap_or_else(|| session.id.clone());
                            self.messages = session.messages.clone();
                            self.session_id = session.id;
                            self.session_name = session.name;
                            let _ = self
                                .action_tx
                                .send(Action::TranscriptRestore(self.messages.clone()));
                            let _ = self.action_tx.send(Action::AgentResponse(format!(
                                "⚙ Loaded: {} ({} messages)",
                                display, count
                            )));
                        }
                        None => {
                            let _ = self
                                .action_tx
                                .send(Action::AgentResponse(format!("⚙ Not found: {}", target)));
                        }
                    }
                }
            }
            "/remember" => {
                let fact = args.trim();
                if fact.is_empty() {
                    let _ = self.action_tx.send(Action::AgentResponse(
                        "⚙ Usage: /remember <fact>".to_string(),
                    ));
                } else if let Some(ref v) = self.vault {
                    match v.store_text(fact, "Semantic", 1.0) {
                        Ok(()) => {
                            let _ = self
                                .action_tx
                                .send(Action::AgentResponse(format!("⚙ Stored: {}", fact)));
                        }
                        Err(e) => {
                            let _ = self
                                .action_tx
                                .send(Action::AgentResponse(format!("⚙ Store failed: {}", e)));
                        }
                    }
                } else {
                    let _ = self
                        .action_tx
                        .send(Action::AgentResponse("⚙ Vault not available".to_string()));
                }
            }
            "/mode" => {
                let arg = args.trim().to_lowercase();
                if arg.is_empty() {
                    let config = self.config.read().await;
                    let active = config.engine.active_engine();
                    let mode_str = match config.engine.active_mode {
                        gzmo_core::config::EngineMode::Local => "LOCAL",
                        gzmo_core::config::EngineMode::Cloud => "CLOUD",
                    };
                    let _ = self.action_tx.send(Action::AgentResponse(format!(
                        "⚙ Mode: {}\n  Engine: {} → {}\n  Model: {}",
                        mode_str, active.provider, active.url, active.model
                    )));
                } else {
                    match arg.parse::<gzmo_core::config::EngineMode>() {
                        Ok(new_mode) => {
                            let mut config = self.config.write().await;
                            if new_mode == config.engine.active_mode {
                                let _ = self.action_tx.send(Action::AgentResponse(format!(
                                    "⚙ Already in {} mode",
                                    new_mode
                                )));
                            } else {
                                let profile = config.engine.active_engine_for_mode(new_mode);
                                let test_url =
                                    format!("{}/models", profile.url.trim_end_matches('/'));

                                let req_client = reqwest::Client::new();
                                let ping_ok = match req_client
                                    .get(&test_url)
                                    .timeout(std::time::Duration::from_secs(5))
                                    .send()
                                    .await
                                {
                                    Ok(r)
                                        if r.status().is_success()
                                            || r.status().as_u16() == 401 =>
                                    {
                                        true
                                    }
                                    _ => false,
                                };

                                if !ping_ok && new_mode == gzmo_core::config::EngineMode::Local {
                                    let _ = self.action_tx.send(Action::AgentResponse(format!(
                                        "✗ Local engine not reachable at {}\n  Start llama-server or LM Studio first.",
                                        profile.url
                                    )));
                                } else {
                                    let new_gw = Arc::new(TurboQuantGateway::new(
                                        VllmConfig::from(profile.clone()),
                                    ));
                                    {
                                        let mut gw = self.gateway.write().await;
                                        *gw = new_gw;
                                    }
                                    config.engine.active_mode = new_mode;
                                    if let Err(e) =
                                        config.persist_active_mode(&self.config_path, new_mode)
                                    {
                                        let _ = self.action_tx.send(Action::AgentResponse(
                                            format!("⚙ Failed to persist config: {}", e),
                                        ));
                                    }
                                    let mode_str = match new_mode {
                                        gzmo_core::config::EngineMode::Local => "LOCAL",
                                        gzmo_core::config::EngineMode::Cloud => "CLOUD",
                                    };
                                    let _ = self.action_tx.send(Action::AgentResponse(format!(
                                        "⚙ Switched to: {}\n  Model: {} → {}",
                                        mode_str, profile.model, profile.url
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = self
                                .action_tx
                                .send(Action::AgentResponse(format!("⚙ {}", e)));
                        }
                    }
                }
            }
            _ => {
                if raw_cmd.len() < 2 {
                    let _ = self
                        .action_tx
                        .send(Action::AgentResponse(format!("⚙ Unknown command: {}", cmd)));
                    return;
                }
                let raw_skill = &raw_cmd[1..]; // remove the slash
                let skill_cmd = match raw_skill {
                    "card" | "cards" | "hand" => "poker",
                    "roll" => "dice",
                    "calc" | "math" => "calculate",
                    "vis" => "visual",
                    "sfx" | "play" => "sound",
                    other => other,
                };

                if self.chaos_skills.has(skill_cmd) {
                    let snap = self.chaos_snapshot_rx.borrow().clone();
                    let ctx = SkillContext {
                        chaos: &snap,
                        feedback_tx: &self.chaos_feedback_tx,
                        args,
                    };
                    match self.chaos_skills.get(skill_cmd).unwrap().execute(ctx).await {
                        Ok(output) => {
                            // 1. Clean for TUI display (strip ANSI but keep ASCII formatting)
                            let mut ui_clean = String::new();
                            let mut in_ansi = false;
                            for c in output.display.chars() {
                                if c == '\x1b' {
                                    in_ansi = true;
                                } else if in_ansi && c == 'm' {
                                    in_ansi = false;
                                } else if !in_ansi {
                                    ui_clean.push(c);
                                }
                            }
                            // Start on a new line to align the ASCII box
                            let ui_text = format!("\n{}", ui_clean);
                            let _ = self.action_tx.send(Action::AgentResponse(ui_text));

                            // 2. Clean for LLM memory (strip everything, smash to one line)
                            if output.inject_to_conversation {
                                let llm_clean: String = ui_clean
                                    .chars()
                                    .filter(|c| {
                                        !matches!(
                                            c,
                                            '\u{250c}'
                                                | '\u{2510}'
                                                | '\u{2514}'
                                                | '\u{2518}'
                                                | '\u{251c}'
                                                | '\u{2524}'
                                                | '\u{2500}'
                                                | '\u{2502}'
                                                | '\u{2554}'
                                                | '\u{2557}'
                                                | '\u{255a}'
                                                | '\u{255d}'
                                                | '\u{2551}'
                                                | '\u{2550}'
                                                | '\u{2560}'
                                                | '\u{2563}'
                                                | '/'
                                                | '\\'
                                                | '_'
                                        )
                                    })
                                    .collect::<String>()
                                    .split_whitespace()
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                let llm_text = llm_clean.chars().take(300).collect::<String>();

                                self.messages.push(gzmo_core::types::Message {
                                    role: gzmo_core::types::Role::System,
                                    content: format!("[Skill /{}] {}", skill_cmd, llm_text),
                                    is_meta: true,
                                    tool_calls: None,
                                    tool_call_id: None,
                                });
                            }
                        }
                        Err(e) => {
                            let _ = self
                                .action_tx
                                .send(Action::AgentResponse(format!("⚙ Skill error: {}", e)));
                        }
                    }
                } else {
                    let _ = self
                        .action_tx
                        .send(Action::AgentResponse(format!("⚙ Unknown command: {}", cmd)));
                }
            }
        }
    }
}
