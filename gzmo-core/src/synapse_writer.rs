//! Synapse Writer gate — require a matching `skill.invoke` on the bus before
//! executing chaos skills from the CLI path (Pi emits invoke via synapse-notifier).

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::SynapseWriterConfig;
use crate::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};

const GATE_STATE_FILE: &str = "data/synapse-writer-gate.state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateState {
    pub consumed_invoke_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GateClaim {
    pub invoke_event_id: Uuid,
    pub session_id: Option<String>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error("no matching skill.invoke for skill={skill} within {window_secs}s")]
    NoInvoke { skill: String, window_secs: u64 },
}

pub fn default_gate_state_path(project_root: &Path) -> PathBuf {
    project_root.join(GATE_STATE_FILE)
}

pub fn load_gate_state(path: &Path) -> GateState {
    if !path.exists() {
        return GateState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_gate_state(path: &Path, state: &GateState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut ids = state.consumed_invoke_ids.clone();
    if ids.len() > 10_000 {
        ids = ids.split_off(ids.len().saturating_sub(5_000));
    }
    let trimmed = GateState {
        consumed_invoke_ids: ids,
    };
    fs::write(path, serde_json::to_string_pretty(&trimmed)?)?;
    Ok(())
}

fn data_str(data: &serde_json::Value, key: &str) -> Option<String> {
    data.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Scan the bus tail for an unconsumed `skill.invoke` matching the request.
pub fn claim_skill_invoke(
    bus_path: &Path,
    gate_state_path: &Path,
    cfg: &SynapseWriterConfig,
    skill: &str,
    args: &str,
    tool_call_id: Option<&str>,
) -> Result<GateClaim, GateError> {
    if !cfg.gate_enabled {
        return Ok(GateClaim {
            invoke_event_id: Uuid::nil(),
            session_id: None,
            tool_call_id: tool_call_id.map(|s| s.to_string()),
        });
    }

    if !bus_path.exists() {
        return Err(GateError::NoInvoke {
            skill: skill.to_string(),
            window_secs: cfg.invoke_window_secs,
        });
    }

    let state = load_gate_state(gate_state_path);
    let consumed: HashSet<&str> = state.consumed_invoke_ids.iter().map(|s| s.as_str()).collect();
    let cutoff = Utc::now() - Duration::from_secs(cfg.invoke_window_secs);

    let file = File::open(bus_path).map_err(|_| GateError::NoInvoke {
        skill: skill.to_string(),
        window_secs: cfg.invoke_window_secs,
    })?;
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|_| GateError::NoInvoke {
            skill: skill.to_string(),
            window_secs: cfg.invoke_window_secs,
        })?;

    let start = lines.len().saturating_sub(cfg.tail_scan_lines);
    let skill_norm = skill.trim().to_lowercase();
    let args_norm = args.trim();

    for line in lines.into_iter().skip(start).rev() {
        if line.trim().is_empty() {
            continue;
        }
        let event: SynapseEvent = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if event.event_type != EventType::SkillInvoke {
            continue;
        }
        if event.source != EventSource::PiAgent {
            continue;
        }
        if consumed.contains(event.id.to_string().as_str()) {
            continue;
        }
        if event.timestamp < cutoff {
            continue;
        }
        let Some(data) = event.data.as_ref() else {
            continue;
        };
        let ev_skill = data_str(data, "skill").unwrap_or_default().to_lowercase();
        if ev_skill != skill_norm {
            continue;
        }
        if tool_call_id.is_some() {
            let ev_tid = data_str(data, "toolCallId");
            if ev_tid.as_deref() != tool_call_id {
                continue;
            }
        } else {
            let ev_args = data_str(data, "args").unwrap_or_default();
            if !args_norm.is_empty() && ev_args.trim() != args_norm {
                continue;
            }
        }

        let mut gate_state = load_gate_state(gate_state_path);
        gate_state
            .consumed_invoke_ids
            .push(event.id.to_string());
        let _ = save_gate_state(gate_state_path, &gate_state);

        return Ok(GateClaim {
            invoke_event_id: event.id,
            session_id: data_str(data, "session_id"),
            tool_call_id: data_str(data, "toolCallId"),
        });
    }

    Err(GateError::NoInvoke {
        skill: skill.to_string(),
        window_secs: cfg.invoke_window_secs,
    })
}

pub fn emit_skill_complete(
    bus: &SynapseBus,
    claim: &GateClaim,
    skill: &str,
    duration_ms: u64,
) {
    if claim.invoke_event_id.is_nil() {
        return;
    }
    let corr = claim
        .session_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let data = serde_json::json!({
        "skill": skill,
        "toolCallId": claim.tool_call_id,
        "duration_ms": duration_ms,
        "session_id": claim.session_id,
        "emitted_by": "gzmo_cli",
        "invoke_id": claim.invoke_event_id.to_string(),
    });
    bus.append(&SynapseEvent::with_envelope(
        EventType::SkillComplete,
        EventSource::GzmoCli,
        corr,
        Some(claim.invoke_event_id),
        Some(data),
    ));
}

pub fn emit_skill_error(bus: &SynapseBus, claim: &GateClaim, skill: &str, error: &str) {
    if claim.invoke_event_id.is_nil() {
        return;
    }
    let corr = claim
        .session_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let data = serde_json::json!({
        "skill": skill,
        "toolCallId": claim.tool_call_id,
        "error": error,
        "session_id": claim.session_id,
        "emitted_by": "gzmo_cli",
        "invoke_id": claim.invoke_event_id.to_string(),
    });
    bus.append(&SynapseEvent::with_envelope(
        EventType::SkillError,
        EventSource::GzmoCli,
        corr,
        Some(claim.invoke_event_id),
        Some(data),
    ));
}

pub fn disabled_claim(tool_call_id: Option<String>) -> GateClaim {
    GateClaim {
        invoke_event_id: Uuid::nil(),
        session_id: None,
        tool_call_id,
    }
}

pub fn gate_bypass_from_env() -> bool {
    matches!(
        std::env::var("GZMO_SYNAPSE_GATE_BYPASS").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

// ---------------------------------------------------------------------------
// Forum Romanum — bus writers (GZMO CLI / Kurator approve path)
// ---------------------------------------------------------------------------

/// Threading context for Forum Romanum events (`correlation_id`, `reply_to`, `session_id`).
#[derive(Debug, Clone, Default)]
pub struct ForumThread {
    pub correlation_id: Option<Uuid>,
    pub reply_to: Option<Uuid>,
    pub session_id: Option<String>,
}

impl ForumThread {
    pub fn from_session(session_id: &str) -> Self {
        Self {
            correlation_id: Uuid::parse_str(session_id).ok(),
            reply_to: None,
            session_id: Some(session_id.to_string()),
        }
    }

    pub fn with_reply_to(mut self, reply_to: Uuid) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
}

fn with_session_id(mut data: serde_json::Value, session_id: Option<&str>) -> serde_json::Value {
    if let Some(sid) = session_id {
        if let Some(obj) = data.as_object_mut() {
            obj.entry("session_id".to_string())
                .or_insert_with(|| serde_json::Value::String(sid.to_string()));
        }
    }
    data
}

fn append_forum_event(
    bus: &SynapseBus,
    event_type: EventType,
    source: EventSource,
    thread: &ForumThread,
    data: serde_json::Value,
) -> Uuid {
    let event = SynapseEvent::with_envelope(
        event_type,
        source,
        thread.correlation_id,
        thread.reply_to,
        Some(with_session_id(data, thread.session_id.as_deref())),
    );
    let id = event.id;
    bus.append(&event);
    id
}

/// Emit `agent.spawned` (Kurator approve / governed sub-agent spawn).
pub fn emit_agent_spawned(
    bus: &SynapseBus,
    thread: &ForumThread,
    agent_id: &str,
    extra: serde_json::Value,
) -> Uuid {
    let mut data = serde_json::json!({
        "agent_id": agent_id,
        "emitted_by": "gzmo_cli",
    });
    if let (Some(base), Some(ext)) = (data.as_object_mut(), extra.as_object()) {
        for (k, v) in ext {
            base.insert(k.clone(), v.clone());
        }
    }
    append_forum_event(bus, EventType::AgentSpawned, EventSource::GzmoCli, thread, data)
}

/// Emit `agent.message` (debate / synthesize / explore payload).
pub fn emit_agent_message(
    bus: &SynapseBus,
    thread: &ForumThread,
    agent_id: &str,
    role: &str,
    mode: &str,
    payload: serde_json::Value,
) -> Uuid {
    append_forum_event(
        bus,
        EventType::AgentMessage,
        EventSource::GzmoCli,
        thread,
        serde_json::json!({
            "agent_id": agent_id,
            "role": role,
            "mode": mode,
            "payload": payload,
            "emitted_by": "gzmo_cli",
        }),
    )
}

/// Emit `agent.result` after a governed sub-agent completes.
pub fn emit_agent_result(
    bus: &SynapseBus,
    thread: &ForumThread,
    agent_id: &str,
    status: &str,
    payload: serde_json::Value,
) -> Uuid {
    append_forum_event(
        bus,
        EventType::AgentResult,
        EventSource::GzmoCli,
        thread,
        serde_json::json!({
            "agent_id": agent_id,
            "status": status,
            "payload": payload,
            "emitted_by": "gzmo_cli",
        }),
    )
}

/// Emit `agent.error` when a governed sub-agent fails.
pub fn emit_agent_error(
    bus: &SynapseBus,
    thread: &ForumThread,
    agent_id: &str,
    error: &str,
) -> Uuid {
    append_forum_event(
        bus,
        EventType::AgentError,
        EventSource::GzmoCli,
        thread,
        serde_json::json!({
            "agent_id": agent_id,
            "error": error,
            "emitted_by": "gzmo_cli",
        }),
    )
}

/// Emit `proposal.created` (Prometheus / proposer path).
pub fn emit_proposal_created(
    bus: &SynapseBus,
    thread: &ForumThread,
    agent_id: &str,
    proposal_id: &str,
    title: &str,
    body: &str,
    status: &str,
) -> Uuid {
    append_forum_event(
        bus,
        EventType::ProposalCreated,
        EventSource::GzmoCli,
        thread,
        serde_json::json!({
            "agent_id": agent_id,
            "proposal_id": proposal_id,
            "title": title,
            "body": body,
            "status": status,
            "emitted_by": "gzmo_cli",
        }),
    )
}

/// Emit `proposal.reviewed` (Epimetheus / critic path).
pub fn emit_proposal_reviewed(
    bus: &SynapseBus,
    thread: &ForumThread,
    agent_id: &str,
    proposal_id: &str,
    verdict: &str,
    comments: &str,
) -> Uuid {
    append_forum_event(
        bus,
        EventType::ProposalReviewed,
        EventSource::GzmoCli,
        thread,
        serde_json::json!({
            "agent_id": agent_id,
            "proposal_id": proposal_id,
            "verdict": verdict,
            "comments": comments,
            "emitted_by": "gzmo_cli",
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synapse::SynapseBus;

    fn write_invoke(bus: &SynapseBus, skill: &str, args: &str, tool_call_id: &str) -> Uuid {
        let sid = Uuid::new_v4();
        let event = SynapseEvent::with_envelope(
            EventType::SkillInvoke,
            EventSource::PiAgent,
            Some(sid),
            None,
            Some(serde_json::json!({
                "session_id": sid.to_string(),
                "skill": skill,
                "args": args,
                "toolCallId": tool_call_id,
            })),
        );
        let id = event.id;
        bus.append(&event);
        id
    }

    #[test]
    fn claim_matches_tool_call_id() {
        let dir = std::env::temp_dir().join(format!("sw_gate_{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let bus_path = dir.join("events.jsonl");
        let state_path = dir.join("gate.state.json");
        let bus = SynapseBus::with_path(bus_path.clone());
        write_invoke(&bus, "calculate", "2+3*4", "tc-1");

        let cfg = SynapseWriterConfig {
            gate_enabled: true,
            invoke_window_secs: 300,
            tail_scan_lines: 500,
        };
        let claim = claim_skill_invoke(
            &bus_path,
            &state_path,
            &cfg,
            "calculate",
            "2+3*4",
            Some("tc-1"),
        )
        .expect("claim");
        assert_eq!(claim.tool_call_id.as_deref(), Some("tc-1"));

        let err = claim_skill_invoke(
            &bus_path,
            &state_path,
            &cfg,
            "calculate",
            "2+3*4",
            Some("tc-1"),
        )
        .unwrap_err();
        assert!(matches!(err, GateError::NoInvoke { .. }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn gate_disabled_skips_scan() {
        let dir = std::env::temp_dir().join(format!("sw_gate_off_{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let bus_path = dir.join("events.jsonl");
        let state_path = dir.join("gate.state.json");
        let cfg = SynapseWriterConfig::default();
        let claim = claim_skill_invoke(
            &bus_path,
            &state_path,
            &cfg,
            "calculate",
            "",
            None,
        )
        .expect("bypass");
        assert!(claim.invoke_event_id.is_nil());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn forum_romanum_emitters_write_threaded_chain() {
        let dir = std::env::temp_dir().join(format!("sw_forum_{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let bus_path = dir.join("events.jsonl");
        let bus = SynapseBus::with_path(bus_path.clone());
        let session_id = Uuid::new_v4().to_string();
        let thread = ForumThread::from_session(&session_id);

        let msg_id = emit_agent_message(
            &bus,
            &thread,
            "prometheus",
            "proposer",
            "debate",
            serde_json::json!({"text": "fixture proposal"}),
        );
        let proposal_thread = thread.clone().with_reply_to(msg_id);
        emit_proposal_created(
            &bus,
            &proposal_thread,
            "prometheus",
            &Uuid::new_v4().to_string(),
            "title",
            "body",
            "draft",
        );
        emit_agent_spawned(
            &bus,
            &thread,
            "prometheus",
            serde_json::json!({"recommendation_id": "rec-1"}),
        );

        let raw = fs::read_to_string(&bus_path).unwrap();
        assert!(raw.contains("\"agent.message\""));
        assert!(raw.contains("\"proposal.created\""));
        assert!(raw.contains("\"agent.spawned\""));
        assert!(raw.contains(&session_id));

        let _ = fs::remove_dir_all(dir);
    }
}
