//! Daily cron due-checks (catch-up after restart).
//!
//! Copied from gzmo-core::daemon — kept local so this crate never links engines.

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use std::collections::HashSet;

/// UTC minutes since midnight for a wall-clock `(hour, minute)` pair.
pub fn cron_minutes(hour: u32, minute: u32) -> u32 {
    hour * 60 + minute
}

/// True when today's scheduled UTC time has passed and the job has not run today.
///
/// Unlike exact `hour == H && minute == M` matching, this fires on the first tick
/// at or after the scheduled time (including after a scheduler restart).
pub fn cron_due_today(
    now: &DateTime<Utc>,
    hour: u32,
    minute: u32,
    last_run_date: Option<NaiveDate>,
) -> bool {
    let today = now.date_naive();
    if last_run_date == Some(today) {
        return false;
    }
    let now_mins = now.hour() * 60 + now.minute();
    now_mins >= cron_minutes(hour, minute)
}

/// Earliest multi-slot cron hour that is due today and has not run yet (spark).
pub fn cron_slot_due(
    now: &DateTime<Utc>,
    cron_hours: &[u32],
    cron_minute: u32,
    completed: &HashSet<(u32, u32, NaiveDate)>,
) -> Option<(u32, u32)> {
    let today = now.date_naive();
    let now_mins = now.hour() * 60 + now.minute();
    cron_hours
        .iter()
        .copied()
        .filter(|&h| {
            now_mins >= cron_minutes(h, cron_minute)
                && !completed.contains(&(h, cron_minute, today))
        })
        .min_by_key(|h| *h)
        .map(|h| (h, cron_minute))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn due_after_slot_and_once_per_day() {
        let now = Utc.with_ymd_and_hms(2026, 7, 10, 2, 30, 0).unwrap();
        assert!(cron_due_today(&now, 2, 15, None));
        assert!(!cron_due_today(&now, 3, 0, None));
        assert!(!cron_due_today(&now, 2, 15, Some(now.date_naive())));
    }

    #[test]
    fn slot_picks_earliest_unrun_hour() {
        use std::collections::HashSet;
        let now = Utc.with_ymd_and_hms(2026, 7, 10, 23, 0, 0).unwrap();
        let today = now.date_naive();
        let empty: HashSet<(u32, u32, NaiveDate)> = HashSet::new();
        assert_eq!(cron_slot_due(&now, &[3, 22], 30, &empty), Some((3, 30)));
        let mut one = HashSet::new();
        one.insert((3, 30, today));
        assert_eq!(cron_slot_due(&now, &[3, 22], 30, &one), Some((22, 30)));
        one.insert((22, 30, today));
        assert_eq!(cron_slot_due(&now, &[3, 22], 30, &one), None);
    }
}
