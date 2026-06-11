use std::path::Path;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentorTurn {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentorRequest {
    pub method: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub conversation: Vec<MentorTurn>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

/// Client: send one NDJSON request, read one response line over Unix socket.
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
