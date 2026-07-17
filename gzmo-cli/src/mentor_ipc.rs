//! Headless mentor API — Unix socket NDJSON (`teach` / `ping` / `status`).
//!
//! Daemon listens; `gzmo mentor` is the client. One request per connection.
//! Chaos / PulseLoop is intentionally not wired on the living daemon path.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use gzmo_core::config::{GzmoConfig, TaskKind};
use gzmo_core::gateway::GatewayRouter;
use gzmo_core::types::{Message, Role};

use crate::pedagogy_bridge::{delegate_exec_response, should_delegate_exec, PedagogyRuntime};
pub use gzmo_core::mentor_client::{
    client_request, MentorAction, MentorRequest, MentorResponse, MentorTurn,
};

pub struct MentorServerState {
    pub config: Arc<GzmoConfig>,
    pub router: Arc<GatewayRouter>,
    pub pedagogy: Arc<Mutex<PedagogyRuntime>>,
    pub tutor_gateway: Arc<dyn gzmo_core::gateway::LlmGateway>,
}

impl MentorServerState {
    /// Boot mentor stack without chaos / PulseLoop (living CT101 path).
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
                    learner_id: None,
                    error: Some("empty request".into()),
                    ..MentorResponse::base()
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
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            ..MentorResponse::base()
        },
        "status" => status_response(state).await,
        "reload" => reload_response(state).await,
        "teach" => match teach(&req, state).await {
            Ok(resp) => resp,
            Err(e) => MentorResponse {
                ok: false,
                learner_id: Some(state.config.pedagogy.learner_id().to_string()),
                error: Some(e.to_string()),
                ..MentorResponse::base()
            },
        },
        "compute" => match compute(&req, state).await {
            Ok(resp) => resp,
            Err(e) => MentorResponse {
                ok: false,
                learner_id: Some(state.config.pedagogy.learner_id().to_string()),
                error: Some(e.to_string()),
                ..MentorResponse::base()
            },
        },
        other => MentorResponse {
            ok: false,
            learner_id: None,
            error: Some(format!("unknown method: {other}")),
            ..MentorResponse::base()
        },
    }
}

async fn status_response(state: &Arc<MentorServerState>) -> MentorResponse {
    let mut pedagogy = state.pedagogy.lock().await;
    if let Err(e) = pedagogy.reload_from_disk().await {
        return MentorResponse {
            ok: false,
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: Some(e.to_string()),
            ..MentorResponse::base()
        };
    }
    MentorResponse {
        ok: true,
        mentor: Some(!pedagogy.session.ops_mode),
        ops_mode: Some(pedagogy.session.ops_mode),
        auto_triggers: Some(pedagogy.session.auto_triggers_enabled),
        learner_id: Some(state.config.pedagogy.learner_id().to_string()),
        ..MentorResponse::base()
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
            auto_triggers: Some(pedagogy.session.auto_triggers_enabled),
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            ..MentorResponse::base()
        },
        Err(e) => MentorResponse {
            ok: false,
            learner_id: Some(state.config.pedagogy.learner_id().to_string()),
            error: Some(e.to_string()),
            ..MentorResponse::base()
        },
    }
}

pub fn build_discovery_context(req: &MentorRequest) -> Option<String> {
    let pillar = req.discovery_pillar.as_deref()?.trim();
    if pillar.is_empty() {
        return None;
    }
    let topic = req.learn_topic.as_deref().unwrap_or("");
    let probe = req.probe_context.as_deref().unwrap_or("");
    Some(format!(
        "pillar={pillar}; learn_topic={topic}; probe={probe}; \
         soul=S pedagogy+honeypot; personality=A dreams+spark+synapse+wiki+ops; \
         body=B daemon+skills+ingest+distill+routing; skeleton=C health+tools+MCP+vector+systemd"
    ))
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
    let learner_id = state.config.pedagogy.learner_id().to_string();
    if should_delegate_exec(&pedagogy.session, message) {
        return Ok(delegate_exec_response(
            message,
            &pedagogy.session,
            &learner_id,
        ));
    }

    let messages = build_messages(&req.conversation, message);
    let discovery_context = build_discovery_context(req);
    let mentor_turn = pedagogy
        .maybe_teach(
            state.config.as_ref(),
            state.router.as_ref(),
            state.tutor_gateway.as_ref(),
            message,
            &messages,
            None, // chaos_context — living path stays chaos-free
            discovery_context.as_deref(),
            None, // chaos_snapshot_rx
        )
        .await?;

    match mentor_turn {
        Some(turn) => Ok(MentorResponse::teach_with_pedagogy(
            turn.response,
            learner_id,
            &turn.edf_record,
        )),
        None => Ok(delegate_exec_response(
            message,
            &pedagogy.session,
            &learner_id,
        )),
    }
}

/// Headless autonomous teach (available for future low-tension watcher; unused now).
pub async fn teach_autonomous(
    state: &Arc<MentorServerState>,
    message: &str,
) -> Result<MentorResponse> {
    let req = MentorRequest {
        method: "teach".to_string(),
        message: message.to_string(),
        ..Default::default()
    };
    teach(&req, state).await
}

async fn compute(req: &MentorRequest, state: &Arc<MentorServerState>) -> Result<MentorResponse> {
    if !state.config.pedagogy.enabled {
        bail!("pedagogy disabled");
    }
    let message = req.message.trim();
    if message.is_empty() {
        bail!("message required");
    }

    let code = if message.contains('\n') || message.contains(';') || message.contains("print(") {
        message.to_string()
    } else {
        format!("print({})", message)
    };

    use gzmo_core::tools::{python_sandbox::PythonSandboxTool, ToolHandler};
    let tool = PythonSandboxTool::new(&state.config.pedagogy);
    let output = tool.execute(serde_json::json!({ "code": code })).await?;

    Ok(MentorResponse {
        ok: true,
        response: Some(output.trim().to_string()),
        mentor: Some(false),
        action: Some(MentorAction::DelegateCompute),
        learner_id: Some(state.config.pedagogy.learner_id().to_string()),
        ..MentorResponse::base()
    })
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

/// True when the living daemon PID file is present and the process is alive.
pub fn daemon_running() -> bool {
    let path = std::path::Path::new("/tmp/gzmo_rust.pid");
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let pid = raw.trim();
    if pid.is_empty() {
        return false;
    }
    std::path::Path::new("/proc").join(pid).exists()
}
