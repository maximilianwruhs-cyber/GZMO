//! Autopoietic `/dice` loop — schedule the next roll from the current outcome.
//!
//! Roll value maps linearly to delay in `[min_minutes, max_minutes]`.
//! State persists in `data/dice_loop_state.json`; the daemon fires when due.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::config::DiceLoopConfig;

const STATE_FILE: &str = "dice_loop_state.json";

pub fn state_path(data_dir: &Path) -> PathBuf {
    data_dir.join(STATE_FILE)
}

/// Roll → delay mapping (same curve as Spark dice scheduler, generalized to any die).
pub fn interval_minutes_from_roll(
    roll: u8,
    die_max: u8,
    min_minutes: u32,
    max_minutes: u32,
) -> u32 {
    let min = min_minutes.min(max_minutes);
    let max = max_minutes.max(min_minutes);
    let span = max.saturating_sub(min);
    let steps = die_max.saturating_sub(1).max(1) as u32;
    let scaled = min + (span * roll.saturating_sub(1) as u32) / steps;
    scaled.max(1)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiceLoopState {
    pub fire_at_utc: String,
    pub scheduled_at_utc: String,
    pub parent_inv: u64,
    pub parent_roll: u8,
    pub parent_max: u8,
    pub delay_minutes: u32,
    pub chain_depth: u32,
    pub die_max: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceLoopScheduleStatus {
    pub scheduled: bool,
    pub cancelled: bool,
    pub delay_minutes: Option<u32>,
    pub fire_at_utc: Option<String>,
    pub chain_depth: u32,
    pub skipped_reason: Option<String>,
}

pub fn load_state(data_dir: &Path) -> Option<DiceLoopState> {
    let path = state_path(data_dir);
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_state(data_dir: &Path, state: &DiceLoopState) -> std::io::Result<()> {
    if let Some(parent) = data_dir.parent() {
        let _ = parent;
    }
    std::fs::create_dir_all(data_dir)?;
    let path = state_path(data_dir);
    let json = serde_json::to_string_pretty(state).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

pub fn clear_state(data_dir: &Path) {
    let _ = std::fs::remove_file(state_path(data_dir));
}

/// Mark state as "in-flight" so the next interval tick skips it.
/// Sets fire_at_utc to now + 1ms, effectively deferring the next check.
pub fn mark_processing(data_dir: &Path, state: &DiceLoopState) -> std::io::Result<()> {
    let mut updated = state.clone();
    updated.fire_at_utc = Utc::now()
        .checked_add_signed(Duration::milliseconds(1))
        .unwrap_or(Utc::now())
        .to_rfc3339();
    save_state(data_dir, &updated)
}

pub fn is_due(now: DateTime<Utc>, state: &DiceLoopState) -> bool {
    DateTime::parse_from_rfc3339(&state.fire_at_utc)
        .map(|t| now >= t.with_timezone(&Utc))
        .unwrap_or(false)
}

/// Schedule (or cancel) the next automatic `/dice` from this roll's outcome.
pub fn schedule_from_roll(
    data_dir: &Path,
    cfg: &DiceLoopConfig,
    roll: u8,
    max: u8,
    inv: u64,
    chain_depth: u32,
) -> DiceLoopScheduleStatus {
    if !cfg.enabled {
        return DiceLoopScheduleStatus {
            scheduled: false,
            cancelled: false,
            delay_minutes: None,
            fire_at_utc: None,
            chain_depth,
            skipped_reason: Some("loop disabled".into()),
        };
    }

    if cfg.cancel_on_nat_1 && roll == 1 {
        clear_state(data_dir);
        return DiceLoopScheduleStatus {
            scheduled: false,
            cancelled: true,
            delay_minutes: None,
            fire_at_utc: None,
            chain_depth,
            skipped_reason: Some("nat 1 — loop cancelled".into()),
        };
    }

    if cfg.max_chain_depth > 0 && chain_depth >= cfg.max_chain_depth {
        return DiceLoopScheduleStatus {
            scheduled: false,
            cancelled: false,
            delay_minutes: None,
            fire_at_utc: None,
            chain_depth,
            skipped_reason: Some(format!(
                "chain depth {chain_depth} >= max {}",
                cfg.max_chain_depth
            )),
        };
    }

    let delay = interval_minutes_from_roll(roll, max, cfg.min_minutes, cfg.max_minutes);
    let now = Utc::now();
    let fire_at = now + Duration::minutes(delay as i64);
    let state = DiceLoopState {
        fire_at_utc: fire_at.to_rfc3339(),
        scheduled_at_utc: now.to_rfc3339(),
        parent_inv: inv,
        parent_roll: roll,
        parent_max: max,
        delay_minutes: delay,
        chain_depth,
        die_max: max,
    };

    match save_state(data_dir, &state) {
        Ok(()) => DiceLoopScheduleStatus {
            scheduled: true,
            cancelled: false,
            delay_minutes: Some(delay),
            fire_at_utc: Some(state.fire_at_utc),
            chain_depth,
            skipped_reason: None,
        },
        Err(e) => DiceLoopScheduleStatus {
            scheduled: false,
            cancelled: false,
            delay_minutes: None,
            fire_at_utc: None,
            chain_depth,
            skipped_reason: Some(format!("state write failed: {e}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_maps_roll_to_range() {
        assert_eq!(interval_minutes_from_roll(1, 20, 5, 120), 5);
        assert_eq!(interval_minutes_from_roll(20, 20, 5, 120), 120);
        assert_eq!(interval_minutes_from_roll(1, 6, 10, 60), 10);
        assert_eq!(interval_minutes_from_roll(6, 6, 10, 60), 60);
    }

    #[test]
    fn nat_1_cancels_when_configured() {
        let dir = std::env::temp_dir().join(format!("dice_loop_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = DiceLoopConfig {
            enabled: true,
            cancel_on_nat_1: true,
            ..Default::default()
        };
        schedule_from_roll(&dir, &cfg, 10, 20, 1, 0);
        assert!(load_state(&dir).is_some());
        let status = schedule_from_roll(&dir, &cfg, 1, 20, 2, 0);
        assert!(status.cancelled);
        assert!(load_state(&dir).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }
}
