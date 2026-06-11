//! Headless mentor API — Unix socket NDJSON (`teach` / `ping` / `status`).
//!
//! Daemon listens; `gzmo mentor` is the client. One request per connection.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::{GatewayRouter, LlmGateway};
use gzmo_core::types::{Message, Role};

use crate::pedagogy_bridge::PedagogyRuntime;

#[derive(Debug, Serialize, Deserialize)]
pub struct MentorRequest {
    pub method: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub conversation: Vec<MentorTurn>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentorTurn {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MentorResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentor: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ops_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct MentorServerState {
    pub config: Arc<GzmoConfig>,
    pub router: Arc<GatewayRouter>,
    pub pedagogy: Arc<Mutex<PedagogyRuntime>>,
    pub tutor_gateway: Arc<dyn LlmGateway>,
}

impl MentorServerState {
    pub async fn boot(config: &GzmoConfig) -> Result<Self> {
        let router = Arc::new(GatewayRouter::new(config));
        let tutor_gateway = Arc::clone(router.gateway(TaskKind::Chat));
        let pedagogy = Arc::new(Mutex::new(PedagogyRuntime::boot(config).await?));
        Ok(Self {
            config: Arc::new(config.clone()),
            router,
            pedagogy,
            tutor_gateway,
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
        Some(response) => Ok(MentorResponse {
            ok: true,
            response: Some(response),
            mentor: Some(true),
            ops_mode: Some(false),
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: None,
        }),
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

/// Client: send one NDJSON request, read one response line.
pub async fn client_request(socket_path: &Path, req: &MentorRequest) -> Result<MentorResponse> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("connect mentor socket {:?}", socket_path))?;
    let json = serde_json::to_string(req)?;
    stream.write_all(json.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    serde_json::from_str(line.trim()).context("parse mentor response JSON")
}

pub fn daemon_running() -> bool {
    gzmo_core::skills::dispatch::daemon_running()
}
