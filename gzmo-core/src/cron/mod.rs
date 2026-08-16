//! App-level cron registry — metabolism builtins + operator custom jobs.
//!
//! Host crontab is out of scope (ADR-0003: `gzmo serve` owns overnight jobs).

mod persist;
mod schedule;

pub use persist::{
    persist_builtin_enabled, persist_builtin_schedule, persist_custom_job, remove_custom_job,
};
pub use schedule::{cron5_matches, next_runs, parse_cron5, Cron5};

use std::collections::BTreeMap;

use chrono::{DateTime, Timelike, Utc};

use crate::config::{CronConfig, CustomCronJob, CustomCronKind, GzmoConfig};

/// Built-in job ids owned by `gzmo serve`.
pub const BUILTIN_IDS: &[&str] = &["dream", "distill", "promote", "embed", "spark", "wiki_push"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CronJobSource {
    Builtin,
    Custom,
}

#[derive(Debug, Clone)]
pub struct CronJobView {
    pub id: String,
    pub source: CronJobSource,
    pub enabled: bool,
    pub description: String,
    /// Human schedule summary (UTC).
    pub schedule_label: String,
    /// 5-field cron when available (custom jobs + synthesized for builtins).
    pub cron5: Option<String>,
    pub kind_label: String,
    /// Memory-as-metabolism night stage (builtins only).
    pub night_stage: Option<String>,
}

/// Snapshot of all manageable jobs from config.
pub fn list_jobs(config: &GzmoConfig) -> Vec<CronJobView> {
    let mut out = Vec::new();
    out.extend(builtin_views(config));
    let mut custom: Vec<_> = config.cron.jobs.iter().collect();
    custom.sort_by(|a, b| a.0.cmp(b.0));
    for (id, job) in custom {
        out.push(CronJobView {
            id: id.clone(),
            source: CronJobSource::Custom,
            enabled: job.enabled,
            description: if job.description.is_empty() {
                match job.kind {
                    CustomCronKind::Shell => format!("shell: {}", job.command),
                    CustomCronKind::Prompt => {
                        let p = job.prompt.chars().take(60).collect::<String>();
                        format!("prompt: {p}")
                    }
                }
            } else {
                job.description.clone()
            },
            schedule_label: format!("{} (UTC)", job.schedule),
            cron5: Some(job.schedule.clone()),
            kind_label: match job.kind {
                CustomCronKind::Shell => "shell".into(),
                CustomCronKind::Prompt => "prompt".into(),
            },
            night_stage: None,
        });
    }
    out
}

/// Memory as Metabolism: TRIAGE → CONSOLIDATE → AUDIT (labels only).
pub fn night_stage_for(id: &str) -> Option<&'static str> {
    match id {
        "distill" => Some("TRIAGE"),
        "dream" | "promote" | "embed" => Some("CONSOLIDATE"),
        "spark" | "wiki_push" => Some("AUDIT"),
        _ => None,
    }
}

fn builtin_views(config: &GzmoConfig) -> Vec<CronJobView> {
    let mk = |id: &str,
              enabled: bool,
              description: &str,
              schedule_label: String,
              cron5: Option<String>| {
        CronJobView {
            id: id.into(),
            source: CronJobSource::Builtin,
            enabled,
            description: description.into(),
            schedule_label,
            cron5,
            kind_label: "builtin".into(),
            night_stage: night_stage_for(id).map(|s| s.to_string()),
        }
    };
    vec![
        mk(
            "dream",
            config.dreams.enabled,
            "CONSOLIDATE — nightly dream consolidation",
            format!(
                "daily {:02}:{:02} UTC",
                config.dreams.cron_hour, config.dreams.cron_minute
            ),
            Some(format!(
                "{} {} * * *",
                config.dreams.cron_minute, config.dreams.cron_hour
            )),
        ),
        mk(
            "distill",
            config.session_distill.enabled && config.session_distill.daemon_scheduled,
            "TRIAGE — session distill → vault",
            format!(
                "daily {:02}:{:02} UTC",
                config.session_distill.cron_hour, config.session_distill.cron_minute
            ),
            Some(format!(
                "{} {} * * *",
                config.session_distill.cron_minute, config.session_distill.cron_hour
            )),
        ),
        mk(
            "promote",
            config.metabolism.enabled,
            "CONSOLIDATE — promote mature facts → honeypot",
            format!(
                "daily {:02}:{:02} UTC",
                config.metabolism.promote_cron_hour, config.metabolism.promote_cron_minute
            ),
            Some(format!(
                "{} {} * * *",
                config.metabolism.promote_cron_minute, config.metabolism.promote_cron_hour
            )),
        ),
        mk(
            "embed",
            config.metabolism.enabled,
            "CONSOLIDATE — embed vault (+ Qdrant sync when enabled)",
            format!(
                "daily {:02}:{:02} UTC",
                config.metabolism.embed_cron_hour, config.metabolism.embed_cron_minute
            ),
            Some(format!(
                "{} {} * * *",
                config.metabolism.embed_cron_minute, config.metabolism.embed_cron_hour
            )),
        ),
        mk(
            "spark",
            config.spark.enabled,
            "AUDIT — serendipitous spark recall",
            {
                let hours: Vec<String> = config
                    .spark
                    .cron_hours
                    .iter()
                    .map(|h| format!("{h:02}:{:02}", config.spark.cron_minute))
                    .collect();
                format!("slots {} UTC", hours.join(", "))
            },
            None,
        ),
        mk(
            "wiki_push",
            config.wiki.enabled && config.wiki.backend == "okforge",
            "AUDIT — OKForge wiki push (soft-fail satellite)",
            format!(
                "daily {:02}:{:02} UTC",
                config.wiki.push_cron_hour, config.wiki.push_cron_minute
            ),
            Some(format!(
                "{} {} * * *",
                config.wiki.push_cron_minute, config.wiki.push_cron_hour
            )),
        ),
    ]
}

/// Preview next UTC fire times for a job id (or raw cron5 with `id` empty and cron5 set).
pub fn preview_job(
    config: &GzmoConfig,
    id: &str,
    n: usize,
    from: DateTime<Utc>,
) -> anyhow::Result<Vec<DateTime<Utc>>> {
    if let Some(job) = config.cron.jobs.get(id) {
        let parsed = parse_cron5(&job.schedule)?;
        return Ok(next_runs(&parsed, from, n));
    }
    if id == "spark" {
        return Ok(preview_spark(config, n, from));
    }
    let views = list_jobs(config);
    let view = views
        .iter()
        .find(|v| v.id == id)
        .ok_or_else(|| anyhow::anyhow!("Unknown cron job: {id}"))?;
    let cron5 = view
        .cron5
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Job '{id}' has no cron5 schedule"))?;
    let parsed = parse_cron5(cron5)?;
    Ok(next_runs(&parsed, from, n))
}

fn preview_spark(config: &GzmoConfig, n: usize, from: DateTime<Utc>) -> Vec<DateTime<Utc>> {
    let mut out = Vec::new();
    let mut t = from + chrono::Duration::minutes(1);
    t = t
        .with_second(0)
        .and_then(|x| x.with_nanosecond(0))
        .unwrap_or(t);
    let minute = config.spark.cron_minute;
    let hours: std::collections::HashSet<u32> = config.spark.cron_hours.iter().copied().collect();
    for _ in 0..14 * 24 * 60 {
        if out.len() >= n {
            break;
        }
        if hours.contains(&t.hour()) && t.minute() == minute {
            out.push(t);
        }
        t += chrono::Duration::minutes(1);
    }
    out
}

/// Whether a custom job should fire at `now` (minute resolution) given last fire slot.
pub fn custom_due(
    job: &CustomCronJob,
    now: DateTime<Utc>,
    last: Option<(chrono::NaiveDate, u32, u32)>,
) -> bool {
    if !job.enabled {
        return false;
    }
    let Ok(parsed) = parse_cron5(&job.schedule) else {
        return false;
    };
    if !cron5_matches(&parsed, now) {
        return false;
    }
    let slot = (now.date_naive(), now.hour(), now.minute());
    match last {
        Some(l) if l == slot => false,
        _ => true,
    }
}

/// Validate a custom job definition.
pub fn validate_custom(job: &CustomCronJob) -> anyhow::Result<()> {
    parse_cron5(&job.schedule)?;
    match job.kind {
        CustomCronKind::Shell => {
            if job.command.trim().is_empty() {
                anyhow::bail!("shell job requires non-empty command");
            }
        }
        CustomCronKind::Prompt => {
            if job.prompt.trim().is_empty() {
                anyhow::bail!("prompt job requires non-empty prompt");
            }
        }
    }
    Ok(())
}

/// Format a table for CLI listing.
pub fn format_job_table(jobs: &[CronJobView]) -> String {
    let mut lines = vec![format!(
        "{:<14} {:<8} {:<12} {:<8} {:<24} {}",
        "ID", "SOURCE", "STAGE", "ON", "SCHEDULE", "DESCRIPTION"
    )];
    lines.push("-".repeat(96));
    for j in jobs {
        let src = match j.source {
            CronJobSource::Builtin => "builtin",
            CronJobSource::Custom => "custom",
        };
        let on = if j.enabled { "yes" } else { "no" };
        let stage = j.night_stage.as_deref().unwrap_or("-");
        lines.push(format!(
            "{:<14} {:<8} {:<12} {:<8} {:<24} {}",
            j.id,
            src,
            truncate(stage, 12),
            on,
            truncate(&j.schedule_label, 24),
            truncate(&j.description, 36)
        ));
    }
    lines.join("\n")
}

fn truncate(s: &str, max: usize) -> String {
    let t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        format!("{t}…")
    } else {
        t
    }
}

/// Schedule presets offered by the wizard (label → cron5 or builtin hint).
pub fn schedule_presets() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("hourly", "0 * * * *"),
        ("every_6h", "0 */6 * * *"),
        ("daily_0100", "0 1 * * *"),
        ("daily_0215", "15 2 * * *"),
        ("daily_0300", "0 3 * * *"),
        ("daily_0600", "0 6 * * *"),
        ("weekdays_0900", "0 9 * * 1-5"),
        ("sunday_0600", "0 6 * * 0"),
    ])
}

/// Empty cron config helper for tests.
pub fn empty_cron_config() -> CronConfig {
    CronConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_includes_builtins() {
        let cfg = GzmoConfig::load_auto().unwrap_or_else(|_| {
            // Minimal parse
            toml::from_str::<GzmoConfig>("").unwrap()
        });
        let jobs = list_jobs(&cfg);
        assert!(jobs.iter().any(|j| j.id == "dream"));
        assert!(jobs.iter().any(|j| j.id == "distill"));
        let distill = jobs.iter().find(|j| j.id == "distill").unwrap();
        assert_eq!(distill.night_stage.as_deref(), Some("TRIAGE"));
        let dream = jobs.iter().find(|j| j.id == "dream").unwrap();
        assert_eq!(dream.night_stage.as_deref(), Some("CONSOLIDATE"));
        let spark = jobs.iter().find(|j| j.id == "spark").unwrap();
        assert_eq!(spark.night_stage.as_deref(), Some("AUDIT"));
    }

    #[test]
    fn custom_due_respects_enabled_and_slot() {
        let job = CustomCronJob {
            enabled: true,
            schedule: "30 3 * * *".into(),
            kind: CustomCronKind::Shell,
            command: "true".into(),
            prompt: String::new(),
            description: String::new(),
        };
        let now = DateTime::parse_from_rfc3339("2026-07-17T03:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(custom_due(&job, now, None));
        assert!(!custom_due(&job, now, Some((now.date_naive(), 3, 30))));
        let mut off = job.clone();
        off.enabled = false;
        assert!(!custom_due(&off, now, None));
    }
}
