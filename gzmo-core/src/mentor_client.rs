use std::path::Path;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::pedagogy::{EdfRecord, StealthMetrics, ZpdPhase};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentorTurn {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MentorRequest {
    pub method: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub conversation: Vec<MentorTurn>,
    /// S/A/B/C discovery pillar (Pi mutual discovery sessions)
    #[serde(default)]
    pub discovery_pillar: Option<String>,
    /// Pillar learn topic (matches gzmo_mentor_learn_start)
    #[serde(default)]
    pub learn_topic: Option<String>,
    /// Current probe id e.g. S03, or short action summary
    #[serde(default)]
    pub probe_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MentorAction {
    Teach,
    DelegateExec,
    DelegateCompute,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentorPedagogyMeta {
    pub zpd_phase: ZpdPhase,
    pub hint_level: u8,
    pub stealth: StealthMetrics,
    #[serde(default)]
    pub leakage_detected: bool,
    #[serde(default)]
    pub leakage_retries: u8,
    #[serde(default)]
    pub compute_used: bool,
}

impl From<&EdfRecord> for MentorPedagogyMeta {
    fn from(record: &EdfRecord) -> Self {
        Self {
            zpd_phase: record.zpd_phase,
            hint_level: record.hint_level,
            stealth: record.stealth.clone(),
            leakage_detected: record.leakage_detected,
            leakage_retries: record.leakage_retries,
            compute_used: record.compute_used,
        }
    }
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
    pub auto_triggers: Option<bool>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pedagogy: Option<MentorPedagogyMeta>,
}

impl MentorResponse {
    /// Empty optional fields — use with struct update syntax for each response variant.
    pub fn base() -> Self {
        Self {
            ok: false,
            response: None,
            mentor: None,
            ops_mode: None,
            auto_triggers: None,
            learner_id: None,
            error: None,
            action: None,
            delegate_payload: None,
            delegate_hint: None,
            pedagogy: None,
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

    pub fn teach_with_pedagogy(response: String, learner_id: String, record: &EdfRecord) -> Self {
        Self {
            pedagogy: Some(MentorPedagogyMeta::from(record)),
            ..Self::teach(response, learner_id)
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
    fn teach_response_can_include_pedagogy_metadata() {
        let record = EdfRecord {
            timestamp: chrono::Utc::now(),
            user_input: "what is a symlink?".into(),
            evidence: "question about filesystem concepts".into(),
            decision: "zpd=we_do hint=3".into(),
            zpd_phase: ZpdPhase::WeDo,
            hint_level: 3,
            stealth: StealthMetrics {
                psu: 0.7,
                sdr: 0.6,
                lvd: 0.8,
            },
            tutor_response_preview: "What relationship do you expect?".into(),
            leakage_detected: false,
            leakage_retries: 0,
            compute_used: true,
        };
        let resp = MentorResponse::teach_with_pedagogy(
            "What relationship do you expect?".to_string(),
            "operator".to_string(),
            &record,
        );
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("\"pedagogy\""));
        assert!(json.contains("\"zpd_phase\":\"we_do\""));
        let back: MentorResponse = serde_json::from_str(&json).expect("deserialize");
        let meta = back.pedagogy.expect("pedagogy metadata");
        assert_eq!(meta.zpd_phase, ZpdPhase::WeDo);
        assert_eq!(meta.hint_level, 3);
        assert!(meta.compute_used);
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
