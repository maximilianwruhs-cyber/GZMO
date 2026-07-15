//! Job table — which recipe each loop spawns and with what args.

use crate::config::SchedulerConfig;
use std::path::Path;

/// Daily config-handoff slot (UTC): after dream 01:00, distill 02:15, spark 03:30.
pub const HANDOFF_CRON_HOUR: u32 = 4;
pub const HANDOFF_CRON_MINUTE: u32 = 0;

pub fn ops_args() -> (&'static str, Vec<String>) {
    ("ops-smoke.sh", vec!["--live".into()])
}

pub fn dream_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    (
        "session-to-dream.sh",
        vec![
            "--live".into(),
            "--output".into(),
            cfg.skills.dreams_path.to_string_lossy().into_owned(),
        ],
    )
}

pub fn distill_args() -> (&'static str, Vec<String>) {
    ("synapse-distill-handoff.sh", vec!["--live".into()])
}

pub fn spark_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    (
        "cognition-smoke.sh",
        vec![
            "--live".into(),
            "--vault".into(),
            cfg.memory.vault_db.to_string_lossy().into_owned(),
            "--spark-run".into(),
        ],
    )
}

pub fn handoff_args(config_path: &Path) -> (&'static str, Vec<String>) {
    let mut args = vec!["--live".to_string(), "--apply".to_string()];
    if let Some(target) = SchedulerConfig::handoff_apply_target(config_path) {
        args.push("--gzmo-config".into());
        args.push(target.to_string_lossy().into_owned());
    }
    ("gzmo-handoff.sh", args)
}

pub fn qdrant_sync_script() -> &'static str {
    "qdrant-vault-sync.sh"
}
