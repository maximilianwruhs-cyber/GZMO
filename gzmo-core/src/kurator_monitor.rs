//! Kurator monitor — aggregates Pi Synapse metrics and emits
//! `spawn.recommended` when thresholds are exceeded. With `auto_spawn_on_recommend`,
//! the daemon spawns governed sub-agents without operator approval.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::config::KuratorConfig;
use crate::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};

const STATE_FILE: &str = "data/kurator-monitor.state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KuratorMonitorState {
    pub sessions: HashMap<String, SessionMetrics>,
    pub recommendations_emitted: u64,
    pub last_eval_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pending_recommendations: HashMap<String, PendingRecommendation>,
    /// Approved spawns (removed from pending on `take_pending_recommendation`).
    #[serde(default)]
    pub spawn_history: HashMap<String, PendingRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRecommendation {
    pub event_id: String,
    pub session_id: String,
    pub reason: String,
    pub suggested_agent_profile: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub approved: bool,
    #[serde(default)]
    pub spawn_task_id: Option<String>,
    /// `discovery_fix` for remediation spawns; absent for Pi metric thresholds.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub report_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub turn_count: u32,
    pub skill_errors: u32,
    pub skill_invokes: u32,
    pub dice_loops_seen: u32,
}

#[derive(Debug, Clone)]
pub struct SpawnRecommendation {
    pub session_id: String,
    pub reason: String,
    pub metrics: SessionMetrics,
    pub suggested_agent_profile: String,
    #[allow(clippy::option_option)]
    pub kind: Option<String>,
    pub report_path: Option<String>,
}

pub fn default_state_path(project_root: &Path) -> std::path::PathBuf {
    project_root.join(STATE_FILE)
}

pub fn load_state(path: &Path) -> KuratorMonitorState {
    if !path.exists() {
        return KuratorMonitorState::default();
    }
    let mut state = fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    if compact_pending_recommendations(&mut state) > 0 {
        let _ = save_state(path, &state);
    }
    state
}

/// Collapse duplicate pending entries (pre-phase-3 backlog) to one per session.
/// Keeps the newest recommendation by `created_at`.
pub fn compact_pending_recommendations(state: &mut KuratorMonitorState) -> usize {
    let before = state.pending_recommendations.len();
    if before <= 1 {
        return 0;
    }

    let mut best_by_session: HashMap<String, PendingRecommendation> = HashMap::new();
    for rec in state.pending_recommendations.values().cloned() {
        match best_by_session.get_mut(&rec.session_id) {
            Some(existing) if rec.created_at > existing.created_at => *existing = rec,
            Some(_) => {}
            None => {
                best_by_session.insert(rec.session_id.clone(), rec);
            }
        }
    }

    state.pending_recommendations = best_by_session
        .into_values()
        .map(|rec| (rec.event_id.clone(), rec))
        .collect();

    before.saturating_sub(state.pending_recommendations.len())
}

pub fn save_state(path: &Path, state: &KuratorMonitorState) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn session_key(event: &SynapseEvent) -> Option<String> {
    if let Some(corr) = event.correlation_id {
        return Some(corr.to_string());
    }
    event
        .data
        .as_ref()
        .and_then(|d| d.get("session_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Fold newly polled Pi events into session metrics.
pub fn ingest_pi_events(state: &mut KuratorMonitorState, events: &[SynapseEvent]) {
    for e in events {
        let Some(key) = session_key(e) else {
            continue;
        };
        let m = state.sessions.entry(key).or_default();
        match e.event_type {
            EventType::QuestComplete => {
                m.turn_count += 1;
                if let Some(data) = &e.data {
                    if let Some(n) = data.get("inputTokens").and_then(|v| v.as_u64()) {
                        m.input_tokens += n;
                    }
                    if let Some(n) = data.get("outputTokens").and_then(|v| v.as_u64()) {
                        m.output_tokens += n;
                    }
                }
            }
            EventType::SkillError => m.skill_errors += 1,
            EventType::SkillInvoke => m.skill_invokes += 1,
            _ => {}
        }
    }
}

/// Count Würfel dice-loop events from any source (daemon telemetry).
pub fn ingest_dice_loop_events(state: &mut KuratorMonitorState, events: &[SynapseEvent]) {
    for e in events {
        if e.event_type != EventType::ChaosDiceLoop {
            continue;
        }
        let key = session_key(e).unwrap_or_else(|| "daemon".to_string());
        state.sessions.entry(key).or_default().dice_loops_seen += 1;
    }
}

/// True when this session already has a pending or completed spawn recommendation.
pub fn session_already_claimed(state: &KuratorMonitorState, session_id: &str) -> bool {
    state
        .pending_recommendations
        .values()
        .any(|p| p.session_id == session_id)
        || state
            .spawn_history
            .values()
            .any(|p| p.session_id == session_id)
}

/// Evaluate thresholds and return recommendations (does not write bus).
pub fn evaluate_thresholds(
    state: &KuratorMonitorState,
    config: &KuratorConfig,
) -> Vec<SpawnRecommendation> {
    if !config.enabled {
        return Vec::new();
    }

    let mut out = Vec::new();
    for (session_id, metrics) in &state.sessions {
        if session_already_claimed(state, session_id) {
            continue;
        }
        let total_tokens = metrics.input_tokens.saturating_add(metrics.output_tokens);
        let error_rate = if metrics.skill_invokes > 0 {
            metrics.skill_errors as f64 / metrics.skill_invokes as f64
        } else {
            0.0
        };

        let mut reasons = Vec::new();
        if metrics.turn_count >= config.max_turns_before_recommend {
            reasons.push(format!(
                "turn_count {} >= {}",
                metrics.turn_count, config.max_turns_before_recommend
            ));
        }
        if total_tokens >= config.max_session_tokens {
            reasons.push(format!(
                "session_tokens {} >= {}",
                total_tokens, config.max_session_tokens
            ));
        }
        if metrics.skill_invokes > 0 && error_rate >= config.skill_error_rate_threshold {
            reasons.push(format!(
                "skill_error_rate {:.2} >= {:.2}",
                error_rate, config.skill_error_rate_threshold
            ));
        }
        if metrics.dice_loops_seen >= config.max_dice_loops_per_hour {
            reasons.push(format!(
                "dice_loops {} >= {}",
                metrics.dice_loops_seen, config.max_dice_loops_per_hour
            ));
        }

        if reasons.is_empty() {
            continue;
        }

        out.push(SpawnRecommendation {
            session_id: session_id.clone(),
            reason: reasons.join("; "),
            metrics: metrics.clone(),
            suggested_agent_profile: config.default_agent_profile.clone(),
            kind: None,
            report_path: None,
        });
    }
    out
}

/// Append `spawn.recommended` events for each recommendation.
pub fn emit_recommendations(
    bus: &SynapseBus,
    state: &mut KuratorMonitorState,
    config: &KuratorConfig,
    recommendations: &[SpawnRecommendation],
) -> Vec<PendingRecommendation> {
    let mut emitted = Vec::new();
    for rec in recommendations {
        if session_already_claimed(state, &rec.session_id) {
            continue;
        }
        let approval_required = !config.auto_spawn_on_recommend;
        let mut data = serde_json::json!({
            "session_id": rec.session_id,
            "reason": rec.reason,
            "suggested_agent_profile": rec.suggested_agent_profile,
            "metrics": {
                "input_tokens": rec.metrics.input_tokens,
                "output_tokens": rec.metrics.output_tokens,
                "turn_count": rec.metrics.turn_count,
                "skill_errors": rec.metrics.skill_errors,
                "skill_invokes": rec.metrics.skill_invokes,
                "dice_loops_seen": rec.metrics.dice_loops_seen,
            },
            "approval_required": approval_required,
            "auto_spawn": config.auto_spawn_on_recommend,
        });
        if let Some(kind) = &rec.kind {
            data["kind"] = serde_json::Value::String(kind.clone());
        }
        if let Some(report_path) = &rec.report_path {
            data["report_path"] = serde_json::Value::String(report_path.clone());
        }
        let corr = Uuid::parse_str(&rec.session_id).ok();
        let event = SynapseEvent::with_envelope(
            EventType::SpawnRecommended,
            EventSource::GzmoDaemon,
            corr,
            None,
            Some(data),
        );
        let event_id = event.id.to_string();
        bus.append(&event);
        state.recommendations_emitted += 1;
        let pending = PendingRecommendation {
            event_id: event_id.clone(),
            session_id: rec.session_id.clone(),
            reason: rec.reason.clone(),
            suggested_agent_profile: rec.suggested_agent_profile.clone(),
            created_at: Utc::now(),
            approved: false,
            spawn_task_id: None,
            kind: rec
                .kind
                .clone()
                .or_else(|| {
                    if rec.reason.starts_with("discovery_fail_gap:")
                        || rec.session_id.starts_with("discovery-fix:")
                    {
                        Some("discovery_fix".to_string())
                    } else {
                        None
                    }
                }),
            report_path: rec.report_path.clone(),
        };
        state
            .pending_recommendations
            .insert(event_id, pending.clone());
        emitted.push(pending);
        info!(
            session = %rec.session_id,
            reason = %rec.reason,
            auto_spawn = config.auto_spawn_on_recommend,
            "Kurator: spawn.recommended emitted"
        );
    }
    state.last_eval_at = Some(Utc::now());
    emitted
}

/// Record one Würfel dice-loop fire (daemon path, not Pi poll).
pub fn record_dice_loop_fire(
    bus: &SynapseBus,
    state_path: &Path,
    config: &KuratorConfig,
) -> anyhow::Result<Vec<PendingRecommendation>> {
    if !config.enabled {
        return Ok(Vec::new());
    }
    let mut state = load_state(state_path);
    state
        .sessions
        .entry("daemon".to_string())
        .or_default()
        .dice_loops_seen += 1;
    let recommendations = evaluate_thresholds(&state, config);
    let emitted = if recommendations.is_empty() {
        Vec::new()
    } else {
        emit_recommendations(bus, &mut state, config, &recommendations)
    };
    save_state(state_path, &state)?;
    Ok(emitted)
}

/// Process Pi poll results: update metrics, evaluate, optionally emit recommendations.
pub fn process_pi_poll(
    bus: &SynapseBus,
    state_path: &Path,
    config: &KuratorConfig,
    pi_events: &[SynapseEvent],
) -> anyhow::Result<Vec<PendingRecommendation>> {
    let mut state = load_state(state_path);
    ingest_pi_events(&mut state, pi_events);
    let recommendations = evaluate_thresholds(&state, config);
    let emitted = if recommendations.is_empty() {
        Vec::new()
    } else {
        emit_recommendations(bus, &mut state, config, &recommendations)
    };
    save_state(state_path, &state)?;
    Ok(emitted)
}

/// After a published discovery report, emit a fixer `spawn.recommended` when FAIL/GAP findings exist.
pub fn process_discovery_report(
    bus: &SynapseBus,
    state_path: &Path,
    config: &KuratorConfig,
    report_path: &Path,
    discovery_session_id: &str,
) -> anyhow::Result<Option<PendingRecommendation>> {
    if !config.enabled || !config.discovery_fixer_enabled {
        return Ok(None);
    }
    if !report_path.is_file() {
        anyhow::bail!("discovery report not found: {}", report_path.display());
    }

    let analysis = crate::discovery_fixer::analyze_discovery_report(report_path)?;
    if analysis.actionable_count() < config.discovery_fixer_min_findings as usize {
        info!(
            report = %report_path.display(),
            actionable = analysis.actionable_count(),
            min = config.discovery_fixer_min_findings,
            "Kurator: discovery report has no actionable FAIL/GAP findings"
        );
        return Ok(None);
    }

    let fix_session_id =
        crate::discovery_fixer::discovery_fix_session_id(discovery_session_id, report_path);
    let mut state = load_state(state_path);
    if session_already_claimed(&state, &fix_session_id) {
        info!(
            session = %fix_session_id,
            "Kurator: discovery fixer already pending or spawned for this report"
        );
        return Ok(None);
    }

    let reason = crate::discovery_fixer::discovery_fix_reason(&analysis);
    let recommendation = SpawnRecommendation {
        session_id: fix_session_id,
        reason,
        metrics: SessionMetrics::default(),
        suggested_agent_profile: config.fixer_agent_profile.clone(),
        kind: Some("discovery_fix".to_string()),
        report_path: Some(report_path.to_string_lossy().into_owned()),
    };

    let emitted = emit_recommendations(bus, &mut state, config, &[recommendation]);
    save_state(state_path, &state)?;
    Ok(emitted.into_iter().next())
}

/// Approve a pending `spawn.recommended` event and return its metadata for spawn.
pub fn take_pending_recommendation(
    state_path: &Path,
    recommendation_id: &str,
) -> anyhow::Result<PendingRecommendation> {
    let mut state = load_state(state_path);
    let rec = if let Some(rec) = state.pending_recommendations.remove(recommendation_id) {
        rec
    } else {
        let key = state
            .pending_recommendations
            .iter()
            .find(|(_, v)| v.session_id == recommendation_id)
            .map(|(k, _)| k.clone());
        match key {
            Some(k) => state.pending_recommendations.remove(&k).unwrap(),
            None => {
                anyhow::bail!("no pending recommendation for id or session {recommendation_id}")
            }
        }
    };
    if rec.approved {
        anyhow::bail!("recommendation already approved");
    }
    save_state(state_path, &state)?;
    Ok(rec)
}

pub fn mark_recommendation_spawned(
    state_path: &Path,
    event_id: &str,
    spawn_task_id: &str,
    rec: PendingRecommendation,
) -> anyhow::Result<()> {
    let mut state = load_state(state_path);
    let mut completed = rec;
    completed.approved = true;
    completed.spawn_task_id = Some(spawn_task_id.to_string());
    state.spawn_history.insert(event_id.to_string(), completed);
    save_state(state_path, &state)?;
    Ok(())
}

/// Put a recommendation back in pending after a failed spawn attempt.
/// Skips restore when the session already has an approved spawn in history.
pub fn restore_pending_recommendation(
    state_path: &Path,
    rec: PendingRecommendation,
) -> anyhow::Result<()> {
    let mut state = load_state(state_path);
    if state
        .spawn_history
        .values()
        .any(|p| p.session_id == rec.session_id && p.approved)
    {
        return Ok(());
    }
    state
        .pending_recommendations
        .insert(rec.event_id.clone(), rec);
    save_state(state_path, &state)?;
    Ok(())
}

pub fn list_pending_recommendations(state_path: &Path) -> Vec<PendingRecommendation> {
    let state = load_state(state_path);
    let mut out: Vec<_> = state.pending_recommendations.values().cloned().collect();
    out.sort_by_key(|r| r.created_at);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synapse::{EventSource, SynapseEvent};

    #[test]
    fn discovery_report_triggers_fixer_recommendation() {
        let dir = std::env::temp_dir().join(format!("kurator_disc_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let bus_path = dir.join("events.jsonl");
        let state_path = dir.join("kurator.state.json");
        let report_path = dir.join("report.md");
        std::fs::write(
            &report_path,
            r#"### F1 — orphans
- Observation: 39 orphaned sessions remain.
- Risk or opportunity: **FAIL**: orphans not cleaned.
"#,
        )
        .unwrap();

        let bus = crate::synapse::SynapseBus::with_path(bus_path.clone());
        let config = KuratorConfig {
            enabled: true,
            discovery_fixer_enabled: true,
            fixer_agent_profile: "epimetheus".to_string(),
            discovery_fixer_min_findings: 1,
            ..Default::default()
        };

        let rec = process_discovery_report(
            &bus,
            &state_path,
            &config,
            &report_path,
            "test-session",
        )
        .expect("process_discovery_report")
        .expect("recommendation");

        assert_eq!(rec.kind.as_deref(), Some("discovery_fix"));
        assert!(rec.reason.contains("discovery_fail_gap"));
        let raw = std::fs::read_to_string(&bus_path).unwrap();
        assert!(raw.contains("spawn.recommended"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn evaluate_turn_threshold() {
        let mut state = KuratorMonitorState::default();
        let sid = Uuid::new_v4().to_string();
        state.sessions.insert(
            sid.clone(),
            SessionMetrics {
                turn_count: 50,
                ..Default::default()
            },
        );
        let config = KuratorConfig {
            enabled: true,
            max_turns_before_recommend: 40,
            ..Default::default()
        };
        let recs = evaluate_thresholds(&state, &config);
        assert_eq!(recs.len(), 1);
        assert!(recs[0].reason.contains("turn_count"));
    }

    #[test]
    fn ingest_quest_complete_tokens() {
        let mut state = KuratorMonitorState::default();
        let sid = Uuid::new_v4();
        let event = SynapseEvent::with_envelope(
            EventType::QuestComplete,
            EventSource::PiAgent,
            Some(sid),
            None,
            Some(serde_json::json!({
                "inputTokens": 100,
                "outputTokens": 50,
            })),
        );
        ingest_pi_events(&mut state, &[event]);
        let m = state.sessions.get(&sid.to_string()).unwrap();
        assert_eq!(m.input_tokens, 100);
        assert_eq!(m.output_tokens, 50);
        assert_eq!(m.turn_count, 1);
    }

    #[test]
    fn spawn_recommended_emitted_when_turn_threshold_exceeded() {
        let dir = std::env::temp_dir().join(format!("kurator_emit_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let bus_path = dir.join("events.jsonl");
        let state_path = dir.join("kurator.state.json");
        let bus = crate::synapse::SynapseBus::with_path(bus_path.clone());

        let sid = Uuid::new_v4();
        let events: Vec<SynapseEvent> = (0..3)
            .map(|_| {
                SynapseEvent::with_envelope(
                    EventType::QuestComplete,
                    EventSource::PiAgent,
                    Some(sid),
                    None,
                    Some(serde_json::json!({"session_id": sid.to_string()})),
                )
            })
            .collect();

        let config = KuratorConfig {
            enabled: true,
            max_turns_before_recommend: 2,
            max_session_tokens: u64::MAX,
            ..Default::default()
        };

        let recs =
            process_pi_poll(&bus, &state_path, &config, &events).expect("process_pi_poll");
        assert_eq!(recs.len(), 1);

        let raw = std::fs::read_to_string(&bus_path).unwrap();
        assert!(raw.contains("spawn.recommended"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn dice_loop_threshold_triggers_recommendation() {
        let mut state = KuratorMonitorState::default();
        state.sessions.insert(
            "daemon".to_string(),
            SessionMetrics {
                dice_loops_seen: 15,
                ..Default::default()
            },
        );
        let config = KuratorConfig {
            enabled: true,
            max_dice_loops_per_hour: 12,
            max_turns_before_recommend: u32::MAX,
            max_session_tokens: u64::MAX,
            ..Default::default()
        };
        let recs = evaluate_thresholds(&state, &config);
        assert_eq!(recs.len(), 1);
        assert!(recs[0].reason.contains("dice_loops"));
    }

    #[test]
    fn compact_pending_keeps_newest_per_session() {
        let sid = Uuid::new_v4().to_string();
        let older = PendingRecommendation {
            event_id: "old".to_string(),
            session_id: sid.clone(),
            reason: "older".to_string(),
            suggested_agent_profile: "prometheus".to_string(),
            created_at: Utc::now() - chrono::Duration::hours(1),
            approved: false,
            spawn_task_id: None,
            kind: None,
            report_path: None,
        };
        let newer = PendingRecommendation {
            event_id: "new".to_string(),
            session_id: sid,
            reason: "newer".to_string(),
            suggested_agent_profile: "prometheus".to_string(),
            created_at: Utc::now(),
            approved: false,
            spawn_task_id: None,
            kind: None,
            report_path: None,
        };
        let mut state = KuratorMonitorState::default();
        state
            .pending_recommendations
            .insert(older.event_id.clone(), older);
        state
            .pending_recommendations
            .insert(newer.event_id.clone(), newer.clone());

        let removed = compact_pending_recommendations(&mut state);
        assert_eq!(removed, 1);
        assert_eq!(state.pending_recommendations.len(), 1);
        assert_eq!(
            state.pending_recommendations.get("new").unwrap().reason,
            "newer"
        );
    }

    #[test]
    fn restore_pending_skips_when_session_already_spawned() {
        let sid = Uuid::new_v4().to_string();
        let mut state = KuratorMonitorState::default();
        state.spawn_history.insert(
            "done".to_string(),
            PendingRecommendation {
                event_id: "done".to_string(),
                session_id: sid.clone(),
                reason: "spawned".to_string(),
                suggested_agent_profile: "prometheus".to_string(),
                created_at: Utc::now(),
                approved: true,
                spawn_task_id: Some("task-1".to_string()),
                kind: None,
                report_path: None,
            },
        );
        let dir = std::env::temp_dir().join(format!("kurator_restore_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");
        save_state(&path, &state).unwrap();

        let failed = PendingRecommendation {
            event_id: "retry".to_string(),
            session_id: sid,
            reason: "turn_count 70 >= 40".to_string(),
            suggested_agent_profile: "prometheus".to_string(),
            created_at: Utc::now(),
            approved: false,
            spawn_task_id: None,
            kind: None,
            report_path: None,
        };
        restore_pending_recommendation(&path, failed).unwrap();
        let loaded = load_state(&path);
        assert!(loaded.pending_recommendations.is_empty());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn restore_pending_puts_failed_recommendation_back() {
        let sid = Uuid::new_v4().to_string();
        let dir = std::env::temp_dir().join(format!("kurator_restore_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");

        let failed = PendingRecommendation {
            event_id: "retry".to_string(),
            session_id: sid.clone(),
            reason: "turn_count 70 >= 40".to_string(),
            suggested_agent_profile: "prometheus".to_string(),
            created_at: Utc::now(),
            approved: false,
            spawn_task_id: None,
            kind: None,
            report_path: None,
        };
        restore_pending_recommendation(&path, failed.clone()).unwrap();
        let loaded = load_state(&path);
        assert_eq!(loaded.pending_recommendations.len(), 1);
        assert_eq!(
            loaded.pending_recommendations.get("retry").unwrap().session_id,
            sid
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn no_duplicate_recommendation_for_claimed_session() {
        let mut state = KuratorMonitorState::default();
        let sid = Uuid::new_v4().to_string();
        state.sessions.insert(
            sid.clone(),
            SessionMetrics {
                turn_count: 50,
                ..Default::default()
            },
        );
        state.spawn_history.insert(
            "prev".to_string(),
            PendingRecommendation {
                event_id: "prev".to_string(),
                session_id: sid.clone(),
                reason: "already spawned".to_string(),
                suggested_agent_profile: "prometheus".to_string(),
                created_at: Utc::now(),
                approved: true,
                spawn_task_id: Some("task-1".to_string()),
                kind: None,
                report_path: None,
            },
        );
        let config = KuratorConfig {
            enabled: true,
            max_turns_before_recommend: 40,
            ..Default::default()
        };
        let recs = evaluate_thresholds(&state, &config);
        assert!(recs.is_empty());
    }
}
