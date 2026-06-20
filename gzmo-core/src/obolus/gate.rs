//! ObolusGate — runtime energy budget decisions (E_total + ctx_%).

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::config::GzmoConfig;
use crate::obolus::ledger::{LedgerEntry, ObolusLedger};
use crate::obolus::power_ledger::{rolling_power_rollups, PowerLedger, PowerRollup};
use crate::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObolusTier {
    Operator,
    SemiAutonomous,
    Autonomous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObolusAction {
    SpawnDiscoveryFix,
    SpawnSessionTriage,
    DiscoveryCycle,
    DiceLoop,
    DreamTick,
    SparkTick,
    DiscoveryPlan,
    OperatorChat,
    /// MCP tool invocation (ARD: cost-gated resource discovery).
    McpToolInvoke,
}

impl ObolusAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SpawnDiscoveryFix => "spawn_discovery_fix",
            Self::SpawnSessionTriage => "spawn_session_triage",
            Self::DiscoveryCycle => "discovery_cycle",
            Self::DiscoveryPlan => "discovery_plan",
            Self::DiceLoop => "dice_loop",
            Self::DreamTick => "dream_tick",
            Self::SparkTick => "spark_tick",
            Self::OperatorChat => "operator_chat",
            Self::McpToolInvoke => "mcp_tool_invoke",
        }
    }

    pub fn default_tier(self) -> ObolusTier {
        match self {
            Self::OperatorChat => ObolusTier::Operator,
            Self::McpToolInvoke => ObolusTier::SemiAutonomous,
            Self::DiscoveryCycle | Self::DiscoveryPlan => ObolusTier::SemiAutonomous,
            Self::SpawnDiscoveryFix
            | Self::SpawnSessionTriage
            | Self::DiceLoop
            | Self::DreamTick
            | Self::SparkTick => ObolusTier::Autonomous,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObolusVerdict {
    Allow,
    Warn { reason: String },
    Defer { reason: String },
    Deny { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemBalance {
    /// Sum of `total_tokens` in the rolling window (energy proxy).
    pub e_total: u64,
    /// Max per-process cumulative input / `prime_context_tokens` in the window (gate signal).
    pub ctx_pressure_pct: f64,
    /// Peak single-call input / `prime_context_tokens` in the window (observability).
    pub peak_call_ctx_pct: f64,
    pub window_hours: f64,
    pub entry_count: usize,
    /// CPU joules from RAPL samples in the rolling window (observability only).
    #[serde(default)]
    pub joules_cpu_1h: f64,
    #[serde(default)]
    pub joules_wh_cpu_1h: f64,
    #[serde(default)]
    pub joules_gpu_est_1h: f64,
    #[serde(default)]
    pub joules_wh_total_est_1h: f64,
    #[serde(default)]
    pub power_sample_count: usize,
    /// `e_total / joules_wh_total_est` when Wh > 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_per_wh: Option<f64>,
}

/// Aggregate ledger rows for the rolling window (default: 1h).
///
/// `ctx_pressure_pct` is the **maximum** per-process context share in the window
/// (same semantics as CLI `ctx_%` per process), not the sum across all processes.
pub fn rolling_rollups(entries: &[LedgerEntry], prime_context_tokens: u64) -> SystemBalance {
    let mut e_total = 0u64;
    let mut peak_input = 0u64;
    let mut process_input: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for e in entries {
        e_total = e_total.saturating_add(e.total_tokens);
        peak_input = peak_input.max(e.input_tokens);
        *process_input.entry(e.process.clone()).or_insert(0) =
            process_input.get(&e.process).copied().unwrap_or(0).saturating_add(e.input_tokens);
    }
    let ctx_denom = prime_context_tokens.max(1) as f64;
    let max_process_input = process_input.values().copied().max().unwrap_or(0);
    SystemBalance {
        e_total,
        ctx_pressure_pct: (max_process_input as f64 / ctx_denom) * 100.0,
        peak_call_ctx_pct: (peak_input as f64 / ctx_denom) * 100.0,
        window_hours: 1.0,
        entry_count: entries.len(),
        joules_cpu_1h: 0.0,
        joules_wh_cpu_1h: 0.0,
        joules_gpu_est_1h: 0.0,
        joules_wh_total_est_1h: 0.0,
        power_sample_count: 0,
        tokens_per_wh: None,
    }
}

fn merge_power_into_balance(mut balance: SystemBalance, power: &PowerRollup) -> SystemBalance {
    balance.joules_cpu_1h = power.joules_cpu_1h;
    balance.joules_wh_cpu_1h = power.joules_wh_cpu_1h;
    balance.joules_gpu_est_1h = power.joules_gpu_est_1h;
    balance.joules_wh_total_est_1h = power.joules_wh_total_est_1h;
    balance.power_sample_count = power.power_sample_count;
    if power.joules_wh_total_est_1h > 0.0 {
        balance.tokens_per_wh =
            Some(balance.e_total as f64 / power.joules_wh_total_est_1h);
    }
    balance
}

pub fn load_power_rollups_since(cfg: &GzmoConfig, since: DateTime<Utc>) -> Result<PowerRollup> {
    let path = std::path::PathBuf::from(&cfg.obolus_analytics.power_ledger_path);
    let entries = PowerLedger::read_since(since, &path)?;
    Ok(rolling_power_rollups(&entries))
}

fn reserve_tokens_for_action(action: ObolusAction, cfg: &GzmoConfig) -> u64 {
    let gov = &cfg.obolus_governance;
    let obl = match action {
        ObolusAction::SpawnDiscoveryFix => gov.spawn_discovery_fix_reserve_obl,
        ObolusAction::SpawnSessionTriage => gov.spawn_session_triage_reserve_obl,
        ObolusAction::DiceLoop => gov.dice_loop_reserve_obl,
        _ => 0,
    };
    obl.saturating_mul(cfg.obolus_analytics.tokens_per_obl.max(1))
}

/// Evaluate whether `action` is allowed given current system balance.
pub fn evaluate_budget(
    cfg: &GzmoConfig,
    balance: &SystemBalance,
    action: ObolusAction,
    tier: ObolusTier,
) -> ObolusVerdict {
    if !cfg.obolus_governance.enabled {
        return ObolusVerdict::Allow;
    }

    let reserve = reserve_tokens_for_action(action, cfg);
    let projected = balance.e_total.saturating_add(reserve);
    let mut reasons = Vec::new();

    if projected > cfg.obolus_governance.max_e_total_per_hour {
        reasons.push(format!(
            "E_total {} + reserve {} > max {}",
            balance.e_total, reserve, cfg.obolus_governance.max_e_total_per_hour
        ));
    }
    if balance.ctx_pressure_pct > cfg.obolus_governance.max_ctx_pressure_pct {
        reasons.push(format!(
            "ctx_% {:.1} (max process in window) > max {:.1}",
            balance.ctx_pressure_pct, cfg.obolus_governance.max_ctx_pressure_pct
        ));
    }

    if reasons.is_empty() {
        return ObolusVerdict::Allow;
    }

    let reason = reasons.join("; ");
    let gov = &cfg.obolus_governance;

    match tier {
        ObolusTier::Operator => {
            if gov.operator_warn_only {
                ObolusVerdict::Warn { reason }
            } else if gov.on_budget_exceeded == "deny" {
                ObolusVerdict::Deny { reason }
            } else {
                ObolusVerdict::Warn { reason }
            }
        }
        ObolusTier::SemiAutonomous => ObolusVerdict::Defer { reason },
        ObolusTier::Autonomous => {
            if gov.on_budget_exceeded == "deny" {
                ObolusVerdict::Deny { reason }
            } else {
                ObolusVerdict::Defer { reason }
            }
        }
    }
}

pub fn load_balance_since(cfg: &GzmoConfig, since: DateTime<Utc>) -> Result<SystemBalance> {
    let path = std::path::PathBuf::from(&cfg.obolus_analytics.ledger_path);
    let entries = ObolusLedger::read_since(since, &path)?;
    let balance = rolling_rollups(
        &entries,
        cfg.obolus_analytics.prime_context_tokens,
    );
    if cfg.obolus_analytics.energy_sampler_enabled {
        let power = load_power_rollups_since(cfg, since)?;
        Ok(merge_power_into_balance(balance, &power))
    } else {
        Ok(balance)
    }
}

pub fn evaluate_from_config(
    cfg: &GzmoConfig,
    action: ObolusAction,
    tier: ObolusTier,
) -> Result<ObolusVerdict> {
    let since = Utc::now() - Duration::hours(1);
    match load_balance_since(cfg, since) {
        Ok(balance) => Ok(evaluate_budget(cfg, &balance, action, tier)),
        Err(e) => {
            if cfg.obolus_governance.fail_open_if_ledger_unreadable {
                tracing::warn!(error = %e, "obolus gate fail-open (ledger unreadable)");
                Ok(ObolusVerdict::Allow)
            } else {
                Ok(ObolusVerdict::Deny {
                    reason: format!("ledger unreadable: {e}"),
                })
            }
        }
    }
}

/// Sum ledger tokens for a Pi session (`correlation_id`).
pub fn ledger_session_tokens(
    cfg: &GzmoConfig,
    session_id: &str,
    since: DateTime<Utc>,
) -> Option<u64> {
    let path = std::path::PathBuf::from(&cfg.obolus_analytics.ledger_path);
    let entries = ObolusLedger::read_since(since, &path).ok()?;
    let total: u64 = entries
        .iter()
        .filter(|e| e.correlation_id.as_deref() == Some(session_id))
        .map(|e| e.total_tokens)
        .sum();
    if total == 0 {
        None
    } else {
        Some(total)
    }
}

pub fn emit_obolus_denied(bus: &SynapseBus, action: ObolusAction, reason: &str) {
    let _ = bus.append(&SynapseEvent::with_data(
        EventType::ObolusDenied,
        EventSource::GzmoDaemon,
        serde_json::json!({
            "action": action.as_str(),
            "reason": reason,
        }),
    ));
}

pub fn emit_obolus_warn(bus: &SynapseBus, action: ObolusAction, reason: &str) {
    let _ = bus.append(&SynapseEvent::with_data(
        EventType::ObolusWarn,
        EventSource::GzmoDaemon,
        serde_json::json!({
            "action": action.as_str(),
            "reason": reason,
        }),
    ));
}

pub fn emit_budget_tick(bus: &SynapseBus, balance: &SystemBalance) {
    let _ = bus.append(&SynapseEvent::with_data(
        EventType::ObolusBudgetTick,
        EventSource::GzmoDaemon,
        serde_json::json!({
            "balance": balance,
        }),
    ));
}

pub fn emit_energy_tick(bus: &SynapseBus, balance: &SystemBalance) {
    let _ = bus.append(&SynapseEvent::with_data(
        EventType::ObolusEnergyTick,
        EventSource::GzmoDaemon,
        serde_json::json!({
            "joules_cpu_1h": balance.joules_cpu_1h,
            "joules_wh_cpu_1h": balance.joules_wh_cpu_1h,
            "joules_gpu_est_1h": balance.joules_gpu_est_1h,
            "joules_wh_total_est_1h": balance.joules_wh_total_est_1h,
            "e_total": balance.e_total,
            "tokens_per_wh": balance.tokens_per_wh,
            "power_sample_count": balance.power_sample_count,
        }),
    ));
}

/// Returns `true` when the action may proceed (Allow, or Warn on operator tier).
pub fn preflight_allowed(
    cfg: &GzmoConfig,
    action: ObolusAction,
    tier: ObolusTier,
    bus: Option<&SynapseBus>,
) -> Result<bool> {
    match evaluate_from_config(cfg, action, tier)? {
        ObolusVerdict::Allow => Ok(true),
        ObolusVerdict::Warn { reason } => {
            if let Some(b) = bus {
                emit_obolus_warn(b, action, &reason);
            }
            tracing::warn!(action = action.as_str(), %reason, "obolus budget warning");
            Ok(matches!(tier, ObolusTier::Operator))
        }
        ObolusVerdict::Defer { reason } | ObolusVerdict::Deny { reason } => {
            if let Some(b) = bus {
                emit_obolus_denied(b, action, &reason);
            }
            tracing::info!(action = action.as_str(), %reason, "obolus gate blocked action");
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GzmoConfig;
    use crate::obolus::ledger::{LedgerEntry, LedgerSource};

    fn test_cfg() -> GzmoConfig {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root");
        let mut cfg =
            GzmoConfig::load(&root.join("gzmo.toml.example")).expect("gzmo.toml.example");
        cfg.obolus_governance.enabled = true;
        cfg.obolus_governance.max_e_total_per_hour = 1000;
        cfg.obolus_governance.on_budget_exceeded = "deny".into();
        cfg.obolus_governance.operator_warn_only = true;
        cfg
    }

    fn entry(process: &str, input: u64, output: u64) -> LedgerEntry {
        LedgerEntry {
            ts: Utc::now(),
            source: LedgerSource::Gateway,
            process: process.to_string(),
            task_kind: None,
            caller: "test".into(),
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
            latency_ms: 1,
            ok: true,
            model: None,
            correlation_id: None,
            action_id: None,
            dedup_key: None,
        }
    }

    #[test]
    fn ctx_pressure_is_max_per_process_not_sum() {
        let ctx = 1000u64;
        let balance = rolling_rollups(
            &[
                entry("chat", 350, 10),
                entry("dream_extract", 200, 10),
            ],
            ctx,
        );
        assert!((balance.ctx_pressure_pct - 35.0).abs() < 0.01);
        assert!((balance.peak_call_ctx_pct - 35.0).abs() < 0.01);
    }

    fn entry_tokens(tokens: u64) -> LedgerEntry {
        entry("test", tokens / 2, tokens / 2)
    }

    #[test]
    fn operator_warns_when_over_budget() {
        let cfg = test_cfg();
        let balance = SystemBalance {
            e_total: 2000,
            ctx_pressure_pct: 10.0,
            peak_call_ctx_pct: 5.0,
            window_hours: 1.0,
            entry_count: 1,
            joules_cpu_1h: 0.0,
            joules_wh_cpu_1h: 0.0,
            joules_gpu_est_1h: 0.0,
            joules_wh_total_est_1h: 0.0,
            power_sample_count: 0,
            tokens_per_wh: None,
        };
        let v = evaluate_budget(
            &cfg,
            &balance,
            ObolusAction::OperatorChat,
            ObolusTier::Operator,
        );
        assert!(matches!(v, ObolusVerdict::Warn { .. }));
    }

    #[test]
    fn autonomous_denies_when_over_budget() {
        let cfg = test_cfg();
        let balance = rolling_rollups(&[entry_tokens(2000)], 131_072);
        let v = evaluate_budget(
            &cfg,
            &balance,
            ObolusAction::SpawnDiscoveryFix,
            ObolusTier::Autonomous,
        );
        assert!(matches!(v, ObolusVerdict::Deny { .. }));
    }

    /// Integration smoke: append `obolus.denied` to the project Synapse bus (for `scripts/obolus-gate-smoke.sh`).
    #[test]
    fn synapse_emit_denied_smoke() {
        if std::env::var("OBOLUS_SYNAPSE_SMOKE").ok().as_deref() != Some("1") {
            return;
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root");
        let bus_path = root.join("data/Synapse/events.jsonl");
        if let Some(parent) = bus_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let bus = crate::synapse::SynapseBus::with_path(bus_path);
        let reason = std::env::var("OBOLUS_SMOKE_REASON")
            .unwrap_or_else(|_| "smoke: obolus gate deny test".into());
        emit_obolus_denied(&bus, ObolusAction::SpawnDiscoveryFix, &reason);
    }
}
