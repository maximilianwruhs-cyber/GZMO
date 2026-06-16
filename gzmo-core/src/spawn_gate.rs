//! Spawn gate — central autospawn policy (rate limits, tiers, circuit breaker).
//!
//! Manual operator spawns (`gzmo kurator approve`, `fix-from-discovery --spawn`) bypass the gate.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::SpawnGateConfig;
use crate::discovery_fixer;
use crate::kurator_monitor::{KuratorMonitorState, PendingRecommendation};
use crate::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};

const STATE_FILE: &str = "data/spawn-gate.state.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnKind {
    DiscoveryFix,
    SessionTriage,
}

impl SpawnKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SpawnKind::DiscoveryFix => "discovery_fix",
            SpawnKind::SessionTriage => "session_triage",
        }
    }
}

pub fn spawn_kind(rec: &PendingRecommendation) -> SpawnKind {
    if discovery_fixer::is_discovery_fix_recommendation(rec) {
        SpawnKind::DiscoveryFix
    } else {
        SpawnKind::SessionTriage
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnGateState {
    #[serde(default)]
    pub executions: Vec<SpawnExecutionRecord>,
    #[serde(default)]
    pub denials: Vec<SpawnDenialRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnExecutionRecord {
    pub at: DateTime<Utc>,
    pub kind: String,
    pub reason_hash: u64,
    pub recommendation_id: String,
    pub session_id: String,
    pub task_id: Option<String>,
    pub approved_via: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnDenialRecord {
    pub at: DateTime<Utc>,
    pub kind: String,
    pub reason: String,
    pub recommendation_id: String,
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnGateDecision {
    pub allowed: bool,
    pub code: String,
    pub message: String,
}

impl SpawnGateDecision {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            code: "allow".to_string(),
            message: "spawn permitted".to_string(),
        }
    }

    pub fn deny(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            allowed: false,
            code: code.into(),
            message: message.into(),
        }
    }
}

pub fn default_state_path(project_root: &Path) -> PathBuf {
    project_root.join(STATE_FILE)
}

pub fn load_state(path: &Path) -> SpawnGateState {
    if !path.exists() {
        return SpawnGateState::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_state(path: &Path, state: &SpawnGateState) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut state = state.clone();
    trim_records(&mut state);
    std::fs::write(path, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

fn trim_records(state: &mut SpawnGateState) {
    let cutoff = Utc::now() - chrono::Duration::hours(48);
    state.executions.retain(|r| r.at >= cutoff);
    state.denials.retain(|r| r.at >= cutoff);
    if state.executions.len() > 500 {
        let drop = state.executions.len() - 500;
        state.executions.drain(0..drop);
    }
    if state.denials.len() > 500 {
        let drop = state.denials.len() - 500;
        state.denials.drain(0..drop);
    }
}

pub fn reason_hash(reason: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    reason.trim().hash(&mut hasher);
    hasher.finish()
}

fn within_hour(records: &[SpawnExecutionRecord]) -> Vec<&SpawnExecutionRecord> {
    let cutoff = Utc::now() - chrono::Duration::hours(1);
    records
        .iter()
        .filter(|r| r.at >= cutoff)
        .collect()
}

fn last_execution(state: &SpawnGateState) -> Option<&SpawnExecutionRecord> {
    state.executions.last()
}

pub fn discovery_fix_pending(kurator: &KuratorMonitorState) -> bool {
    kurator.pending_recommendations.values().any(|rec| {
        discovery_fixer::is_discovery_fix_recommendation(rec) && rec.spawn_task_id.is_none()
    })
}

pub fn discovery_fix_in_flight(kurator: &KuratorMonitorState, cooldown_secs: u64) -> bool {
    if discovery_fix_pending(kurator) {
        return true;
    }
    let cutoff = Utc::now() - chrono::Duration::seconds(cooldown_secs as i64);
    kurator.spawn_history.values().any(|rec| {
        discovery_fixer::is_discovery_fix_recommendation(rec)
            && rec.created_at >= cutoff
            && rec.spawn_task_id.is_some()
    })
}

/// Evaluate whether an autospawn may proceed.
pub fn evaluate_autospawn(
    rec: &PendingRecommendation,
    cfg: &SpawnGateConfig,
    gate_state: &SpawnGateState,
    kurator_state: &KuratorMonitorState,
) -> SpawnGateDecision {
    if !cfg.enabled {
        return SpawnGateDecision::allow();
    }

    let kind = spawn_kind(rec);
    let now = Utc::now();
    let hour_execs = within_hour(&gate_state.executions);

    if hour_execs.len() >= cfg.max_autospawns_per_hour as usize {
        return SpawnGateDecision::deny(
            "max_autospawns_per_hour",
            format!(
                "hourly autospawn budget exhausted ({}/{})",
                hour_execs.len(),
                cfg.max_autospawns_per_hour
            ),
        );
    }

    if let Some(last) = last_execution(gate_state) {
        let elapsed = now.signed_duration_since(last.at);
        if elapsed < chrono::Duration::seconds(cfg.spawn_cooldown_secs as i64) {
            return SpawnGateDecision::deny(
                "spawn_cooldown",
                format!(
                    "cooldown active ({}s remaining)",
                    cfg.spawn_cooldown_secs.saturating_sub(elapsed.num_seconds().max(0) as u64)
                ),
            );
        }
    }

    let rh = reason_hash(&rec.reason);
    let dup_count = hour_execs
        .iter()
        .filter(|e| e.reason_hash == rh)
        .count();
    if dup_count >= cfg.duplicate_reason_max_per_hour as usize {
        return SpawnGateDecision::deny(
            "duplicate_reason",
            format!(
                "same reason hash triggered {} times in the last hour (max {})",
                dup_count, cfg.duplicate_reason_max_per_hour
            ),
        );
    }

    if cfg.prometheus_requires_idle
        && kind == SpawnKind::SessionTriage
        && discovery_fix_in_flight(kurator_state, cfg.spawn_cooldown_secs)
    {
        return SpawnGateDecision::deny(
            "prometheus_requires_idle",
            "session_triage blocked while discovery_fix pending or in cooldown".to_string(),
        );
    }

    SpawnGateDecision::allow()
}

pub fn autospawn_enabled_for(
    rec: &PendingRecommendation,
    auto_spawn_session_triage: bool,
    auto_spawn_discovery_fix: bool,
) -> bool {
    match spawn_kind(rec) {
        SpawnKind::DiscoveryFix => auto_spawn_discovery_fix,
        SpawnKind::SessionTriage => auto_spawn_session_triage,
    }
}

pub fn bypass_gate_for_approved_via(approved_via: &str) -> bool {
    !approved_via.contains("autospawn")
}

pub fn emit_spawn_denied(
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    decision: &SpawnGateDecision,
) -> Uuid {
    let corr = Uuid::parse_str(&rec.session_id).ok();
    let reply = Uuid::parse_str(&rec.event_id).ok();
    let event = SynapseEvent::with_envelope(
        EventType::SpawnDenied,
        EventSource::GzmoDaemon,
        corr,
        reply,
        Some(serde_json::json!({
            "recommendation_id": rec.event_id,
            "session_id": rec.session_id,
            "kind": spawn_kind(rec).as_str(),
            "code": decision.code,
            "reason": decision.message,
            "agent_profile": rec.suggested_agent_profile,
        })),
    );
    let id = event.id;
    let _ = bus.append(&event);
    id
}

pub fn emit_spawn_executed(
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    task_id: &str,
    approved_via: &str,
) -> Uuid {
    let corr = Uuid::parse_str(&rec.session_id).ok();
    let reply = Uuid::parse_str(&rec.event_id).ok();
    let event = SynapseEvent::with_envelope(
        EventType::SpawnExecuted,
        EventSource::GzmoDaemon,
        corr,
        reply,
        Some(serde_json::json!({
            "recommendation_id": rec.event_id,
            "session_id": rec.session_id,
            "kind": spawn_kind(rec).as_str(),
            "task_id": task_id,
            "approved_via": approved_via,
            "agent_profile": rec.suggested_agent_profile,
        })),
    );
    let id = event.id;
    let _ = bus.append(&event);
    id
}

pub fn record_denial(
    gate_state_path: &Path,
    rec: &PendingRecommendation,
    decision: &SpawnGateDecision,
) -> anyhow::Result<()> {
    let mut state = load_state(gate_state_path);
    state.denials.push(SpawnDenialRecord {
        at: Utc::now(),
        kind: spawn_kind(rec).as_str().to_string(),
        reason: format!("{}: {}", decision.code, decision.message),
        recommendation_id: rec.event_id.clone(),
        session_id: rec.session_id.clone(),
    });
    save_state(gate_state_path, &state)
}

pub fn record_execution(
    gate_state_path: &Path,
    rec: &PendingRecommendation,
    task_id: &str,
    approved_via: &str,
) -> anyhow::Result<()> {
    let mut state = load_state(gate_state_path);
    state.executions.push(SpawnExecutionRecord {
        at: Utc::now(),
        kind: spawn_kind(rec).as_str().to_string(),
        reason_hash: reason_hash(&rec.reason),
        recommendation_id: rec.event_id.clone(),
        session_id: rec.session_id.clone(),
        task_id: Some(task_id.to_string()),
        approved_via: approved_via.to_string(),
    });
    save_state(gate_state_path, &state)
}

impl Default for SpawnGateState {
    fn default() -> Self {
        Self {
            executions: Vec::new(),
            denials: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kurator_monitor::KuratorMonitorState;

    fn rec(kind: Option<&str>, reason: &str) -> PendingRecommendation {
        PendingRecommendation {
            event_id: Uuid::new_v4().to_string(),
            session_id: Uuid::new_v4().to_string(),
            reason: reason.to_string(),
            suggested_agent_profile: "epimetheus".to_string(),
            created_at: Utc::now(),
            approved: false,
            spawn_task_id: None,
            kind: kind.map(str::to_string),
            report_path: None,
        }
    }

    fn cfg() -> SpawnGateConfig {
        SpawnGateConfig {
            enabled: true,
            max_autospawns_per_hour: 2,
            spawn_cooldown_secs: 600,
            prometheus_requires_idle: true,
            duplicate_reason_max_per_hour: 3,
            auto_spawn_discovery_fix: true,
            prime_budget_enabled: false,
            prime_spawn_budget_per_hour: 3,
            prime_budget_fail_open: true,
            prime_budget_key_prefix: None,
            prime_budget_ttl_secs: 7200,
        }
    }

    #[test]
    fn allows_when_disabled() {
        let mut c = cfg();
        c.enabled = false;
        let decision = evaluate_autospawn(&rec(Some("discovery_fix"), "x"), &c, &SpawnGateState::default(), &KuratorMonitorState::default());
        assert!(decision.allowed);
    }

    #[test]
    fn blocks_hourly_budget() {
        let c = cfg();
        let mut gate = SpawnGateState::default();
        for _ in 0..2 {
            gate.executions.push(SpawnExecutionRecord {
                at: Utc::now(),
                kind: "discovery_fix".to_string(),
                reason_hash: 1,
                recommendation_id: "a".to_string(),
                session_id: "s".to_string(),
                task_id: Some("t".to_string()),
                approved_via: "kurator autospawn".to_string(),
            });
        }
        let decision = evaluate_autospawn(
            &rec(Some("discovery_fix"), "new"),
            &c,
            &gate,
            &KuratorMonitorState::default(),
        );
        assert!(!decision.allowed);
        assert_eq!(decision.code, "max_autospawns_per_hour");
    }

    #[test]
    fn blocks_prometheus_when_fixer_pending() {
        let c = cfg();
        let mut kurator = KuratorMonitorState::default();
        let pending = rec(Some("discovery_fix"), "fix infra");
        kurator
            .pending_recommendations
            .insert(pending.event_id.clone(), pending);
        let triage = rec(None, "too many turns");
        let decision = evaluate_autospawn(&triage, &c, &SpawnGateState::default(), &kurator);
        assert!(!decision.allowed);
        assert_eq!(decision.code, "prometheus_requires_idle");
    }

    #[test]
    fn autospawn_kind_split() {
        let fix = rec(Some("discovery_fix"), "x");
        let triage = rec(None, "x");
        assert!(autospawn_enabled_for(&fix, false, true));
        assert!(!autospawn_enabled_for(&fix, true, false));
        assert!(autospawn_enabled_for(&triage, true, false));
        assert!(!autospawn_enabled_for(&triage, false, true));
    }
}
