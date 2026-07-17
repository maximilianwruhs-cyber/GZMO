//! Minimal 5-field cron (min hour dom month dow) matching + next-run scan.

use anyhow::{bail, Result};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

#[derive(Debug, Clone)]
pub struct Cron5 {
    pub minute: Field,
    pub hour: Field,
    pub day_of_month: Field,
    pub month: Field,
    pub day_of_week: Field,
}

#[derive(Debug, Clone)]
pub enum Field {
    Any,
    /// Exact values
    List(Vec<u32>),
    /// */step from lo..=hi
    Step {
        lo: u32,
        hi: u32,
        step: u32,
    },
}

/// Parse classic 5-field cron: `min hour dom month dow`.
pub fn parse_cron5(expr: &str) -> Result<Cron5> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        bail!("Expected 5-field cron (min hour dom month dow), got: {expr}");
    }
    Ok(Cron5 {
        minute: parse_field(parts[0], 0, 59)?,
        hour: parse_field(parts[1], 0, 23)?,
        day_of_month: parse_field(parts[2], 1, 31)?,
        month: parse_field(parts[3], 1, 12)?,
        day_of_week: parse_field(parts[4], 0, 6)?,
    })
}

fn parse_field(raw: &str, lo: u32, hi: u32) -> Result<Field> {
    if raw == "*" {
        return Ok(Field::Any);
    }
    if let Some(rest) = raw.strip_prefix("*/") {
        let step: u32 = rest
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid step in '{raw}'"))?;
        if step == 0 {
            bail!("Step cannot be 0 in '{raw}'");
        }
        return Ok(Field::Step { lo, hi, step });
    }
    if raw.contains(',') {
        let mut vals = Vec::new();
        for p in raw.split(',') {
            vals.extend(expand_token(p, lo, hi)?);
        }
        vals.sort_unstable();
        vals.dedup();
        return Ok(Field::List(vals));
    }
    Ok(Field::List(expand_token(raw, lo, hi)?))
}

fn expand_token(tok: &str, lo: u32, hi: u32) -> Result<Vec<u32>> {
    if let Some((a, b)) = tok.split_once('-') {
        let start: u32 = a
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid range start in '{tok}'"))?;
        let end: u32 = b
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid range end in '{tok}'"))?;
        if start < lo || end > hi || start > end {
            bail!("Range '{tok}' out of bounds {lo}-{hi}");
        }
        return Ok((start..=end).collect());
    }
    let v: u32 = tok
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid cron field token '{tok}'"))?;
    if v < lo || v > hi {
        bail!("Value {v} out of bounds {lo}-{hi}");
    }
    Ok(vec![v])
}

fn field_matches(field: &Field, value: u32) -> bool {
    match field {
        Field::Any => true,
        Field::List(vals) => vals.contains(&value),
        Field::Step { lo, hi, step } => value >= *lo && value <= *hi && (value - lo) % step == 0,
    }
}

pub fn cron5_matches(cron: &Cron5, now: DateTime<Utc>) -> bool {
    let dow = now.weekday().num_days_from_sunday(); // 0=Sun
    field_matches(&cron.minute, now.minute())
        && field_matches(&cron.hour, now.hour())
        && field_matches(&cron.day_of_month, now.day())
        && field_matches(&cron.month, now.month())
        && field_matches(&cron.day_of_week, dow)
}

/// Next `n` matching minutes after `from` (exclusive of `from` minute if already matching).
pub fn next_runs(cron: &Cron5, from: DateTime<Utc>, n: usize) -> Vec<DateTime<Utc>> {
    let mut out = Vec::new();
    let mut t = from + Duration::minutes(1);
    t = t
        .with_second(0)
        .and_then(|x| x.with_nanosecond(0))
        .unwrap_or(t);
    // Cap scan: ~400 days
    for _ in 0..400 * 24 * 60 {
        if out.len() >= n {
            break;
        }
        if cron5_matches(cron, t) {
            out.push(t);
        }
        t += Duration::minutes(1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_match_daily() {
        let c = parse_cron5("30 3 * * *").unwrap();
        let hit = DateTime::parse_from_rfc3339("2026-07-17T03:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let miss = DateTime::parse_from_rfc3339("2026-07-17T03:31:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(cron5_matches(&c, hit));
        assert!(!cron5_matches(&c, miss));
    }

    #[test]
    fn next_runs_finds_future() {
        let c = parse_cron5("0 6 * * 1-5").unwrap();
        let from = DateTime::parse_from_rfc3339("2026-07-17T12:00:00Z") // Friday
            .unwrap()
            .with_timezone(&Utc);
        let runs = next_runs(&c, from, 3);
        assert_eq!(runs.len(), 3);
        assert!(runs[0] > from);
    }

    #[test]
    fn step_every_six_hours() {
        let c = parse_cron5("0 */6 * * *").unwrap();
        let hit = DateTime::parse_from_rfc3339("2026-07-17T18:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(cron5_matches(&c, hit));
    }
}
