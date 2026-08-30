//! Idle-triggered living research: cooldown only. No vault writes.

use std::time::{Duration, SystemTime};

/// At most one idle evolve per this window.
pub const IDLE_EVOLVE_COOLDOWN: Duration = Duration::from_secs(6 * 60 * 60);

/// True when there is no stamp, or the stamp is at least `cooldown` old.
pub fn idle_evolve_due(stamp: Option<SystemTime>, now: SystemTime, cooldown: Duration) -> bool {
    match stamp {
        None => true,
        Some(t) => now.duration_since(t).is_ok_and(|d| d >= cooldown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: Duration = Duration::from_secs(60 * 60);

    fn t0() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
    }

    #[test]
    fn due_when_no_stamp() {
        assert!(idle_evolve_due(None, t0(), IDLE_EVOLVE_COOLDOWN));
    }

    #[test]
    fn due_when_stamp_exactly_cooldown_old() {
        let now = t0();
        assert!(idle_evolve_due(
            Some(now - IDLE_EVOLVE_COOLDOWN),
            now,
            IDLE_EVOLVE_COOLDOWN
        ));
    }

    #[test]
    fn not_due_inside_cooldown() {
        let now = t0();
        assert!(!idle_evolve_due(
            Some(now - IDLE_EVOLVE_COOLDOWN + HOUR),
            now,
            IDLE_EVOLVE_COOLDOWN
        ));
    }

    #[test]
    fn not_due_when_stamp_in_future() {
        let now = t0();
        assert!(!idle_evolve_due(
            Some(now + HOUR),
            now,
            IDLE_EVOLVE_COOLDOWN
        ));
    }
}
