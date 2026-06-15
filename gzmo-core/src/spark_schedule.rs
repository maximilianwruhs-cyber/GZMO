//! Dice-based spark scheduling (port of gzmo-rebuild/dice-scheduler rollInterval).

use chrono::{DateTime, Duration, Utc};

use crate::config::SparkConfig;

/// Advance a simple LCG seed (deterministic, chaos-free).
pub fn advance_seed(seed: u64) -> u64 {
    seed.wrapping_mul(1_103_515_245).wrapping_add(12_345)
}

/// Roll d6 → minutes in [min, max] (inclusive endpoints, 6 steps).
pub fn roll_interval_minutes(seed: u64, min_minutes: u32, max_minutes: u32) -> (u32, u32) {
    let roll = ((seed % 6) as u32) + 1;
    let min = min_minutes.min(max_minutes);
    let max = max_minutes.max(min_minutes);
    let span = max.saturating_sub(min);
    let minutes = min + (span * (roll.saturating_sub(1))) / 5;
    (roll, minutes.max(1))
}

/// Next UTC instant for a spark run after `now`.
pub fn next_spark_after(
    now: DateTime<Utc>,
    config: &SparkConfig,
    seed: u64,
) -> (DateTime<Utc>, u64, u32, u32) {
    let (roll, minutes) =
        roll_interval_minutes(seed, config.dice_min_minutes, config.dice_max_minutes);
    let next_seed = advance_seed(seed);
    let next = now + Duration::minutes(minutes as i64);
    (next, next_seed, roll, minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_interval_within_bounds() {
        let (_, m) = roll_interval_minutes(42, 20, 180);
        assert!((20..=180).contains(&m));
    }
}
