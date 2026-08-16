//! Thin Unix-socket client. One request per connection (same shape as mentor IPC).

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::config::GzmoConfig;
use crate::memory::profile::GzmoProfile;
use crate::platform_memory::{MemorySearchResult, MemoryStatusReport};

use super::protocol::{ControlRequest, ControlResponse, PingBody};
use super::resolved_socket;

/// Client skip switch. Owner (`serve`/`daemon`) never honors this for the flock.
pub fn clients_enabled() -> bool {
    match std::env::var("GZMO_CONTROL_PLANE") {
        Ok(v) if v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off") => {
            false
        }
        _ => true,
    }
}

#[derive(Debug, Clone)]
pub struct ControlPlaneClient {
    pub socket_path: PathBuf,
    pub session_id: Option<String>,
}

impl ControlPlaneClient {
    pub fn new(socket_path: PathBuf, session_id: Option<String>) -> Self {
        Self {
            socket_path,
            session_id,
        }
    }

    pub fn from_config(config: &GzmoConfig, session_id: Option<String>) -> Self {
        Self::new(resolved_socket(config), session_id)
    }

    /// Connect + ping. `None` if the socket is missing or dead (stale file).
    pub async fn connect_if_live(
        config: &GzmoConfig,
        session_id: Option<String>,
    ) -> Option<Self> {
        if !clients_enabled() {
            return None;
        }
        let client = Self::from_config(config, session_id);
        match client.ping().await {
            Ok(_) => Some(client),
            Err(_) => None,
        }
    }

    pub async fn ping(&self) -> Result<PingBody> {
        let resp = self
            .call(ControlRequest {
                method: "ping".into(),
                session_id: self.session_id.clone(),
                query: None,
                limit: None,
                write_scratch: None,
                fact_id: None,
                dynamic_only: None,
            })
            .await?;
        resp.ping.context("ping response missing body")
    }

    pub async fn ping_path(path: &Path) -> Result<PingBody> {
        Self::new(path.to_path_buf(), None).ping().await
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        write_scratch: bool,
    ) -> Result<MemorySearchResult> {
        let resp = self
            .call(ControlRequest {
                method: "memory.search".into(),
                session_id: self.session_id.clone(),
                query: Some(query.to_string()),
                limit: Some(limit as u64),
                write_scratch: Some(write_scratch),
                fact_id: None,
                dynamic_only: None,
            })
            .await?;
        resp.search.context("memory.search missing result")
    }

    pub async fn recall(&self) -> Result<Option<String>> {
        let resp = self
            .call(ControlRequest {
                method: "memory.recall".into(),
                session_id: self.session_id.clone(),
                query: None,
                limit: None,
                write_scratch: None,
                fact_id: None,
                dynamic_only: None,
            })
            .await?;
        Ok(resp.recall)
    }

    pub async fn status(&self) -> Result<MemoryStatusReport> {
        let resp = self
            .call(ControlRequest {
                method: "memory.status".into(),
                session_id: self.session_id.clone(),
                query: None,
                limit: None,
                write_scratch: None,
                fact_id: None,
                dynamic_only: None,
            })
            .await?;
        resp.status.context("memory.status missing result")
    }

    pub async fn turn_start(&self) -> Result<String> {
        let resp = self
            .call(ControlRequest {
                method: "memory.turn_start".into(),
                session_id: self.session_id.clone(),
                query: None,
                limit: None,
                write_scratch: None,
                fact_id: None,
                dynamic_only: None,
            })
            .await?;
        resp.turn_start.context("memory.turn_start missing result")
    }

    pub async fn chain(&self, fact_id: &str) -> Result<Vec<(String, bool, Option<String>)>> {
        let resp = self
            .call(ControlRequest {
                method: "memory.chain".into(),
                session_id: self.session_id.clone(),
                query: None,
                limit: None,
                write_scratch: None,
                fact_id: Some(fact_id.to_string()),
                dynamic_only: None,
            })
            .await?;
        let chain = resp.chain.context("memory.chain missing result")?;
        Ok(chain
            .into_iter()
            .map(|e| (e.content, e.latest, e.graph_rel))
            .collect())
    }

    pub async fn profile(&self, dynamic_only: bool) -> Result<GzmoProfile> {
        let resp = self
            .call(ControlRequest {
                method: "memory.profile".into(),
                session_id: self.session_id.clone(),
                query: None,
                limit: None,
                write_scratch: None,
                fact_id: None,
                dynamic_only: Some(dynamic_only),
            })
            .await?;
        resp.profile.context("memory.profile missing result")
    }

    async fn call(&self, req: ControlRequest) -> Result<ControlResponse> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .with_context(|| format!("connect {}", self.socket_path.display()))?;
        let line = serde_json::to_string(&req)?;
        stream.write_all(line.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let (reader, _) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();
        let raw = lines
            .next_line()
            .await?
            .context("control plane closed without a response")?;
        let resp: ControlResponse = serde_json::from_str(&raw)
            .with_context(|| format!("decode control plane response: {raw}"))?;
        if !resp.ok {
            bail!(
                "control plane {}: {}",
                resp.method,
                resp.error.unwrap_or_else(|| "unknown error".into())
            );
        }
        Ok(resp)
    }
}
