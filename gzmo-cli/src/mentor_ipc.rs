//! Headless mentor API — Unix socket NDJSON (`teach` / `ping` / `status`).
//!
//! Daemon listens; `gzmo mentor` is the client. One request per connection.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{watch, Mutex, mpsc};
use tracing::{error, info, warn};

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::{GatewayRouter, LlmGateway};
use gzmo_core::types::{Message, Role};

use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::feedback_ipc;
use gzmo_chaos::pulse::ChaosSnapshot;
use gzmo_core::skills::dispatch::{self, data_dir};

use crate::pedagogy_bridge::PedagogyRuntime;
pub use gzmo_core::mentor_client::{MentorRequest, MentorResponse, MentorTurn, client_request};

pub struct MentorServerState {
    pub config: Arc<GzmoConfig>,
    pub router: Arc<GatewayRouter>,
    pub pedagogy: Arc<Mutex<PedagogyRuntime>>,
    pub tutor_gateway: Arc<dyn LlmGateway>,
    pub chaos_feedback_tx: Option<mpsc::Sender<ChaosEvent>>,
    pub chaos_snapshot_rx: Option<watch::Receiver<ChaosSnapshot>>,
}

impl MentorServerState {
    pub async fn boot(config: &GzmoConfig) -> Result<Self> {
        Self::boot_with_chaos(config, None, None).await
    }

    pub async fn boot_with_chaos(
        config: &GzmoConfig,
        chaos_feedback_tx: Option<mpsc::Sender<ChaosEvent>>,
        chaos_snapshot_rx: Option<watch::Receiver<ChaosSnapshot>>,
    ) -> Result<Self> {
        let router = Arc::new(GatewayRouter::new(config));
        let tutor_gateway = Arc::clone(router.gateway(TaskKind::Chat));
        let pedagogy = Arc::new(Mutex::new(PedagogyRuntime::boot(config).await?));
        Ok(Self {
            config: Arc::new(config.clone()),
            router,
            pedagogy,
            tutor_gateway,
            chaos_feedback_tx,
            chaos_snapshot_rx,
        })
    }
}

pub fn socket_path(config: &GzmoConfig) -> PathBuf {
    config.pedagogy.mentor_socket_path()
}

pub async fn run_mentor_server(state: Arc<MentorServerState>, socket_path: PathBuf) -> Result<()> {
    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            warn!(path = %socket_path.display(), "Could not remove stale mentor socket: {e}");
        }
    }
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("bind mentor socket {:?}", socket_path))?;
    info!(path = %socket_path.display(), "Mentor API listening");

    loop {
        let (stream, _) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &state).await {
                error!("Mentor API connection error: {e}");
            }
        });
    }
}

async fn handle_connection(stream: UnixStream, state: &Arc<MentorServerState>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let line = match lines.next_line().await? {
        Some(l) if !l.trim().is_empty() => l,
        _ => {
            write_response(
                &mut writer,
                &MentorResponse {
                    ok: false,
                    response: None,
                    mentor: None,
                    ops_mode: None,
                    learner_id: None,
                    error: Some("empty request".into()),
                },
            )
            .await?;
            return Ok(());
        }
    };

    let req: MentorRequest = serde_json::from_str(&line).context("parse mentor request JSON")?;
    let resp = dispatch_request(req, state).await;
    write_response(&mut writer, &resp).await
}

async fn write_response(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    resp: &MentorResponse,
) -> Result<()> {
    let json = serde_json::to_string(resp)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

async fn dispatch_request(req: MentorRequest, state: &Arc<MentorServerState>) -> MentorResponse {
    match req.method.as_str() {
        "ping" => MentorResponse {
            ok: true,
            response: Some("pong".into()),
            mentor: None,
            ops_mode: None,
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: None,
        },
        "status" => status_response(state).await,
        "reload" => reload_response(state).await,
        "teach" => match teach(&req, state).await {
            Ok(resp) => resp,
            Err(e) => MentorResponse {
                ok: false,
                response: None,
                mentor: None,
                ops_mode: None,
                learner_id: Some(state.config.pedagogy.learner_id().to_string()),
                error: Some(e.to_string()),
            },
        },
        other => MentorResponse {
            ok: false,
            response: None,
            mentor: None,
            ops_mode: None,
            learner_id: None,
            error: Some(format!("unknown method: {other}")),
        },
    }
}

async fn status_response(state: &Arc<MentorServerState>) -> MentorResponse {
    let mut pedagogy = state.pedagogy.lock().await;
    if let Err(e) = pedagogy.reload_from_disk().await {
        return MentorResponse {
            ok: false,
            response: None,
            mentor: None,
            ops_mode: None,
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: Some(e.to_string()),
        };
    }
    MentorResponse {
        ok: true,
        response: None,
        mentor: Some(!pedagogy.session.ops_mode),
        ops_mode: Some(pedagogy.session.ops_mode),
        learner_id: Some(state.config.pedagogy.learner_id().to_string()),
        error: None,
    }
}

async fn reload_response(state: &Arc<MentorServerState>) -> MentorResponse {
    let mut pedagogy = state.pedagogy.lock().await;
    match pedagogy.reload_from_disk().await {
        Ok(()) => MentorResponse {
            ok: true,
            response: Some("reloaded".into()),
            mentor: Some(!pedagogy.session.ops_mode),
            ops_mode: Some(pedagogy.session.ops_mode),
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: None,
        },
        Err(e) => MentorResponse {
            ok: false,
            response: None,
            mentor: None,
            ops_mode: None,
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: Some(e.to_string()),
        },
    }
}

async fn teach(req: &MentorRequest, state: &Arc<MentorServerState>) -> Result<MentorResponse> {
    if !state.config.pedagogy.enabled {
        bail!("pedagogy disabled in config");
    }
    let message = req.message.trim();
    if message.is_empty() {
        bail!("message required");
    }

    let mut pedagogy = state.pedagogy.lock().await;
    pedagogy.reload_from_disk().await?;
    if pedagogy.session.ops_mode {
        return Ok(MentorResponse {
            ok: false,
            response: None,
            mentor: Some(false),
            ops_mode: Some(true),
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: Some("ops mode active — mentor path disabled; toggle via /ops or session.json".into()),
        });
    }

    let messages = build_messages(&req.conversation, message);
    apply_chaos_snapshot_to_tutor(state);
    let mentor_text = pedagogy
        .maybe_teach(
            state.config.as_ref(),
            state.router.as_ref(),
            state.tutor_gateway.as_ref(),
            message,
            &messages,
        )
        .await?;

    match mentor_text {
        Some(response) => {
            emit_mentor_chaos_feedback_state(
                state,
                message,
                &response,
                req.conversation.len() as u32 + 1,
            );
            Ok(MentorResponse {
                ok: true,
                response: Some(response),
                mentor: Some(true),
                ops_mode: Some(false),
                learner_id: Some(state.config.pedagogy.learner_id().to_string()),
                error: None,
            })
        }
        None => Ok(MentorResponse {
            ok: true,
            response: None,
            mentor: Some(false),
            ops_mode: Some(false),
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: Some("not a mentor turn (ops intent or routing fallback)".into()),
        }),
    }
}

fn build_messages(conversation: &[MentorTurn], user_message: &str) -> Vec<Message> {
    let mut messages = Vec::new();
    for turn in conversation {
        let role = match turn.role.to_lowercase().as_str() {
            "assistant" | "gzmo" | "mentor" => Role::Assistant,
            _ => Role::User,
        };
        messages.push(Message {
            role,
            content: turn.content.clone(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        });
    }
    messages.push(Message {
        role: Role::User,
        content: user_message.to_string(),
        is_meta: false,
        tool_calls: None,
        tool_call_id: None,
    });
    messages
}


pub fn daemon_running() -> bool {
    dispatch::daemon_running()
}

fn apply_chaos_snapshot_to_tutor(state: &MentorServerState) {
    let Some(rx) = state.chaos_snapshot_rx.as_ref() else {
        return;
    };
    let snap = rx.borrow().clone();
    state
        .tutor_gateway
        .set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
}

/// Perturb chaos after a mentor turn — direct tx when in-process, inbox when daemon-only.
pub fn emit_mentor_chaos_feedback(
    config: &GzmoConfig,
    user_message: &str,
    mentor_response: &str,
    turn_count: u32,
    chaos_feedback_tx: Option<mpsc::Sender<ChaosEvent>>,
) {
    let topic_preview: String = user_message.chars().take(120).collect();
    let response_preview: String = mentor_response.chars().take(200).collect();
    if response_preview.trim().is_empty() {
        return;
    }
    let event = ChaosEvent::MentorTeach {
        topic_preview,
        response_preview,
        turn_count,
    };

    if let Some(tx) = chaos_feedback_tx {
        tokio::spawn(async move {
            let _ = tx.send(event).await;
        });
        return;
    }

    if !daemon_running() {
        return;
    }
    let inbox = feedback_ipc::default_inbox_path(data_dir(config));
    let _ = feedback_ipc::append_event(&inbox, &event);
}

fn emit_mentor_chaos_feedback_state(
    state: &MentorServerState,
    user_message: &str,
    mentor_response: &str,
    turn_count: u32,
) {
    emit_mentor_chaos_feedback(
        state.config.as_ref(),
        user_message,
        mentor_response,
        turn_count,
        state.chaos_feedback_tx.clone(),
    );
}
