//! Unix-socket owner: dispatches memory methods to the process-local `PlatformMemory`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

use crate::platform_memory::PlatformMemory;

use super::client::ControlPlaneClient;
use super::protocol::{ChainEntry, ControlRequest, ControlResponse, PingBody};

pub async fn bind_socket(path: &Path) -> Result<UnixListener> {
    if path.exists() {
        if ControlPlaneClient::ping_path(path).await.is_ok() {
            anyhow::bail!(
                "control plane already listening at {} — second owner refused",
                path.display()
            );
        }
        if let Err(e) = std::fs::remove_file(path) {
            warn!(path = %path.display(), "could not remove stale control-plane socket: {e}");
        }
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let listener = UnixListener::bind(path)
        .with_context(|| format!("bind control plane {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 0600 {}", path.display()))?;
    }
    Ok(listener)
}

pub async fn spawn_server(
    socket_path: PathBuf,
    platform: Arc<PlatformMemory>,
) -> Result<tokio::task::JoinHandle<Result<()>>> {
    let listener = bind_socket(&socket_path).await?;
    info!(path = %socket_path.display(), "control plane listening");
    let handle = tokio::spawn(async move { accept_loop(listener, platform, socket_path).await });
    Ok(handle)
}

async fn accept_loop(
    listener: UnixListener,
    platform: Arc<PlatformMemory>,
    socket_path: PathBuf,
) -> Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;
        let platform = Arc::clone(&platform);
        let socket_path = socket_path.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, &platform, &socket_path).await {
                error!("control plane connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: UnixStream,
    platform: &PlatformMemory,
    socket_path: &Path,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let line = match lines.next_line().await? {
        Some(l) if !l.trim().is_empty() => l,
        _ => {
            let resp = ControlResponse::err("unknown", "empty request");
            write_line(&mut writer, &resp).await?;
            return Ok(());
        }
    };
    let req: ControlRequest = match serde_json::from_str(&line) {
        Ok(r) => r,
        Err(e) => {
            let resp = ControlResponse::err("unknown", format!("bad request: {e}"));
            write_line(&mut writer, &resp).await?;
            return Ok(());
        }
    };
    let resp = dispatch(platform, socket_path, req).await;
    write_line(&mut writer, &resp).await
}

async fn write_line(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    resp: &ControlResponse,
) -> Result<()> {
    let line = serde_json::to_string(resp)?;
    writer.write_all(line.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

async fn dispatch(
    platform: &PlatformMemory,
    socket_path: &Path,
    req: ControlRequest,
) -> ControlResponse {
    let method = req.method.as_str();
    let session = req.session_id.as_deref();
    match method {
        "ping" => {
            let mut resp = ControlResponse::ok_method("ping");
            resp.ping = Some(PingBody {
                pid: std::process::id(),
                vault_path: platform.vault_path().to_string(),
                socket_path: socket_path.display().to_string(),
            });
            resp
        }
        "memory.search" => {
            let Some(query) = req.query.as_deref() else {
                return ControlResponse::err(method, "missing query");
            };
            let limit = req.limit.unwrap_or(5) as usize;
            let write_scratch = req.write_scratch.unwrap_or(true);
            match platform
                .memory_search_scoped(session, query, limit, write_scratch)
                .await
            {
                Ok(search) => {
                    let mut resp = ControlResponse::ok_method(method);
                    resp.search = Some(search);
                    resp
                }
                Err(e) => ControlResponse::err(method, e.to_string()),
            }
        }
        "memory.recall" => match platform.memory_recall_pull_scoped(session).await {
            Ok(recall) => {
                let mut resp = ControlResponse::ok_method(method);
                resp.recall = recall;
                resp
            }
            Err(e) => ControlResponse::err(method, e.to_string()),
        },
        "memory.status" => match platform.status_scoped(session).await {
            Ok(mut status) => {
                status.control_plane = Some(super::protocol::VIA_OWNER.to_string());
                let mut resp = ControlResponse::ok_method(method);
                resp.status = Some(status);
                resp
            }
            Err(e) => ControlResponse::err(method, e.to_string()),
        },
        "memory.turn_start" => {
            let sid = platform.turn_start_scoped(session).await;
            let mut resp = ControlResponse::ok_method(method);
            resp.turn_start = Some(format!("turn-start: scratch cleared (session {sid})"));
            resp
        }
        "memory.chain" => {
            let Some(fact_id) = req.fact_id.as_deref() else {
                return ControlResponse::err(method, "missing fact_id");
            };
            match platform.memory_chain(fact_id) {
                Ok(rows) => {
                    let mut resp = ControlResponse::ok_method(method);
                    resp.chain = Some(
                        rows.into_iter()
                            .map(|(content, latest, graph_rel)| ChainEntry {
                                content,
                                latest,
                                graph_rel,
                            })
                            .collect(),
                    );
                    resp
                }
                Err(e) => ControlResponse::err(method, e.to_string()),
            }
        }
        "memory.profile" => match platform.memory_profile(req.dynamic_only.unwrap_or(false)) {
            Ok(profile) => {
                let mut resp = ControlResponse::ok_method(method);
                resp.profile = Some(profile);
                resp
            }
            Err(e) => ControlResponse::err(method, e.to_string()),
        },
        other => ControlResponse::err(other, "unknown method"),
    }
}
