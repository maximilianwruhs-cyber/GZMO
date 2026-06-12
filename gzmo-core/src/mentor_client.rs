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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MentorAction {
    Teach,
    DelegateExec,
    DelegateCompute,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<MentorAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate_hint: Option<String>,
}

impl MentorResponse {
    /// Empty optional fields — use with struct update syntax for each response variant.
    pub fn base() -> Self {
        Self {
            ok: false,
            response: None,
            mentor: None,
            ops_mode: None,
            learner_id: None,
            error: None,
            action: None,
            delegate_payload: None,
            delegate_hint: None,
        }
    }

    pub fn teach(response: String, learner_id: String) -> Self {
        Self {
            ok: true,
            response: Some(response),
            mentor: Some(true),
            ops_mode: Some(false),
            learner_id: Some(learner_id),
            action: Some(MentorAction::Teach),
            ..Self::base()
        }
    }

    pub fn delegate_exec(message: &str, ops_mode: bool, learner_id: String, hint: &str) -> Self {
        Self {
            ok: true,
            mentor: Some(false),
            ops_mode: Some(ops_mode),
            learner_id: Some(learner_id),
            action: Some(MentorAction::DelegateExec),
            delegate_payload: Some(message.to_string()),
            delegate_hint: Some(hint.to_string()),
            ..Self::base()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mentor_response_round_trip_includes_delegate_fields() {
        let resp = MentorResponse::delegate_exec(
            "list files in /tmp",
            true,
            "operator".to_string(),
            "use bash",
        );
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"action\":\"delegate_exec\""));
        assert!(json.contains("\"delegate_payload\""));
        let back: MentorResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.action, Some(MentorAction::DelegateExec));
        assert_eq!(back.delegate_payload.as_deref(), Some("list files in /tmp"));
    }

    #[test]
    fn mentor_response_omits_unset_optional_fields() {
        let resp = MentorResponse {
            ok: true,
            response: Some("pong".into()),
            learner_id: Some("operator".into()),
            ..MentorResponse::base()
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(!json.contains("delegate_payload"));
        assert!(!json.contains("action"));
    }
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
