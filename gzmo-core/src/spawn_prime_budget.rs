//! Prime LLM budget for autospawns — Redis hourly slot counter (LXC101).
//!
//! Each governed sub-agent spawn consumes one slot before `SubagentRunner::spawn`.
//! Manual operator spawns bypass this layer via `bypass_gate_for_approved_via`.

use chrono::Utc;
use redis::AsyncCommands;

use crate::config::{RedisConfig, SpawnGateConfig};
use crate::spawn_gate::SpawnGateDecision;

const DEFAULT_KEY_PREFIX: &str = "gzmo:spawn:prime";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimeBudgetOutcome {
    Allowed,
    AllowedFailOpen { reason: String },
    Denied(SpawnGateDecision),
}

impl PrimeBudgetOutcome {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            PrimeBudgetOutcome::Allowed | PrimeBudgetOutcome::AllowedFailOpen { .. }
        )
    }

    pub fn decision_if_denied(&self) -> Option<&SpawnGateDecision> {
        match self {
            PrimeBudgetOutcome::Denied(d) => Some(d),
            _ => None,
        }
    }
}

pub fn hour_bucket_key(prefix: &str) -> String {
    let hour = Utc::now().format("%Y%m%d%H");
    format!("{prefix}:hour:{hour}")
}

/// Check Redis hourly budget without consuming a slot (pre-flight for autospawn queue).
pub async fn check_prime_budget(
    redis_cfg: &RedisConfig,
    gate_cfg: &SpawnGateConfig,
) -> PrimeBudgetOutcome {
    if !gate_cfg.prime_budget_enabled {
        return PrimeBudgetOutcome::Allowed;
    }

    if !redis_cfg.enabled {
        return fail_open_or_deny(
            gate_cfg,
            "redis disabled in config",
        );
    }

    let prefix = gate_cfg
        .prime_budget_key_prefix
        .as_deref()
        .unwrap_or(DEFAULT_KEY_PREFIX);
    let key = hour_bucket_key(prefix);
    let limit = gate_cfg.prime_spawn_budget_per_hour as i64;

    let client = match redis::Client::open(redis_cfg.url.as_str()) {
        Ok(c) => c,
        Err(e) => {
            return fail_open_or_deny(gate_cfg, &format!("redis client: {e}"));
        }
    };

    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            return fail_open_or_deny(gate_cfg, &format!("redis connect: {e}"));
        }
    };

    let current: i64 = conn.get(&key).await.unwrap_or(0);
    if current >= limit {
        return PrimeBudgetOutcome::Denied(SpawnGateDecision::deny(
            "prime_budget_exhausted",
            format!(
                "Prime spawn budget exhausted ({current}/{limit} this hour, key={key})"
            ),
        ));
    }

    PrimeBudgetOutcome::Allowed
}

/// Atomically consume one Prime slot (call immediately before `runner.spawn`).
pub async fn acquire_prime_slot(
    redis_cfg: &RedisConfig,
    gate_cfg: &SpawnGateConfig,
) -> PrimeBudgetOutcome {
    if !gate_cfg.prime_budget_enabled {
        return PrimeBudgetOutcome::Allowed;
    }

    if !redis_cfg.enabled {
        return fail_open_or_deny(gate_cfg, "redis disabled in config");
    }

    let prefix = gate_cfg
        .prime_budget_key_prefix
        .as_deref()
        .unwrap_or(DEFAULT_KEY_PREFIX);
    let key = hour_bucket_key(prefix);
    let limit = gate_cfg.prime_spawn_budget_per_hour as i64;
    let ttl_secs = gate_cfg.prime_budget_ttl_secs;

    let client = match redis::Client::open(redis_cfg.url.as_str()) {
        Ok(c) => c,
        Err(e) => return fail_open_or_deny(gate_cfg, &format!("redis client: {e}")),
    };

    let mut conn = match client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return fail_open_or_deny(gate_cfg, &format!("redis connect: {e}")),
    };

    // INCR then verify — small race window acceptable for homelab; deny if over limit after incr.
    let new_count: i64 = conn.incr(&key, 1).await.unwrap_or(limit + 1);
    if new_count == 1 {
        let _: redis::RedisResult<()> = conn.expire(&key, ttl_secs as i64).await;
    }

    if new_count > limit {
        let _: i64 = conn.decr(&key, 1).await.unwrap_or(0);
        return PrimeBudgetOutcome::Denied(SpawnGateDecision::deny(
            "prime_budget_exhausted",
            format!(
                "Prime spawn budget exhausted ({limit}/{limit} this hour, key={key})"
            ),
        ));
    }

    PrimeBudgetOutcome::Allowed
}

/// Release a slot when spawn failed after `acquire_prime_slot` (best-effort).
pub async fn release_prime_slot(redis_cfg: &RedisConfig, gate_cfg: &SpawnGateConfig) {
    if !gate_cfg.prime_budget_enabled || !redis_cfg.enabled {
        return;
    }
    let prefix = gate_cfg
        .prime_budget_key_prefix
        .as_deref()
        .unwrap_or(DEFAULT_KEY_PREFIX);
    let key = hour_bucket_key(prefix);
    let Ok(client) = redis::Client::open(redis_cfg.url.as_str()) else {
        return;
    };
    let Ok(mut conn) = client.get_multiplexed_async_connection().await else {
        return;
    };
    let _: redis::RedisResult<i64> = conn.decr(&key, 1).await;
}

fn fail_open_or_deny(gate_cfg: &SpawnGateConfig, reason: &str) -> PrimeBudgetOutcome {
    if gate_cfg.prime_budget_fail_open {
        PrimeBudgetOutcome::AllowedFailOpen {
            reason: reason.to_string(),
        }
    } else {
        PrimeBudgetOutcome::Denied(SpawnGateDecision::deny(
            "prime_budget_unavailable",
            format!("Prime budget check failed ({reason}) and fail_open=false"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hour_bucket_key_format() {
        let k = hour_bucket_key("gzmo:spawn:prime");
        assert!(k.starts_with("gzmo:spawn:prime:hour:"));
        assert_eq!(k.len(), "gzmo:spawn:prime:hour:".len() + 10);
    }

    #[test]
    fn disabled_budget_always_allows() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut gate = SpawnGateConfig::default();
        gate.prime_budget_enabled = false;
        let redis = RedisConfig::default();
        let out = rt.block_on(check_prime_budget(&redis, &gate));
        assert!(out.is_allowed());
    }
}
