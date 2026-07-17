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
            "--stats".into(),
            cfg.dream_stats_path().to_string_lossy().into_owned(),
            "--vault".into(),
            cfg.memory.vault_db.to_string_lossy().into_owned(),
        ],
    )
}

/// Sunday 05:15 UTC — recall floor → data-next/recall-report.json
pub const RECALL_CRON_HOUR: u32 = 5;
pub const RECALL_CRON_MINUTE: u32 = 15;
pub const RECALL_CRON_WEEKDAY: chrono::Weekday = chrono::Weekday::Sun;

pub fn recall_eval_args(cfg: &SchedulerConfig) -> Vec<String> {
    vec![
        cfg.recall_report_path().to_string_lossy().into_owned(),
        cfg.memory.vault_db.to_string_lossy().into_owned(),
    ]
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

pub fn ingest_batch_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    (
        "ingest-smoke.sh",
        vec![
            "--live".into(),
            "--inbox".into(),
            cfg.ingest.inbox_path.to_string_lossy().into_owned(),
            "--meta".into(),
            cfg.ingest_meta_path().to_string_lossy().into_owned(),
        ],
    )
}

pub fn wiki_okforge_push_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    (
        "wiki-okforge-push.sh",
        vec![
            "--live".into(),
            "--origin".into(),
            "catchup".into(),
            "--meta".into(),
            cfg.wiki_push_meta_path().to_string_lossy().into_owned(),
        ],
    )
}

pub fn kg_reconcile_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    let mut args = vec![
        "--live".into(),
        "--meta".into(),
        cfg.kg_meta_path().to_string_lossy().into_owned(),
    ];
    // Respect dry_run from config — only pass --apply when dry_run=false.
    if !cfg.kg_reconcile.dry_run {
        args.push("--apply".into());
    }
    ("kg-reconcile-smoke.sh", args)
}

pub fn pedagogy_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    (
        "pedagogy-smoke.sh",
        vec![
            "--live".into(),
            "--meta".into(),
            cfg.pedagogy_meta_path().to_string_lossy().into_owned(),
        ],
    )
}

pub fn cabinet_feed_args(cfg: &SchedulerConfig) -> (&'static str, Vec<String>) {
    (
        "cabinet-feed.sh",
        vec![
            "--live".into(),
            "--meta".into(),
            cfg.cabinet_meta_path().to_string_lossy().into_owned(),
        ],
    )
}
