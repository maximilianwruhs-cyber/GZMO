use serde::Deserialize;

use super::defaults::*;

// ─── Spark ──────────────────────────────────────────────────────────────

/// When to run spark: fixed UTC cron slots or dice-scheduler jitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SparkScheduleMode {
    #[default]
    Cron,
    Dice,
}

/// Settings for the SparkEngine (serendipitous recall).
#[derive(Debug, Deserialize, Clone)]
pub struct SparkConfig {
    /// Master switch — when false, daemon and CLI skip spark runs.
    #[serde(default = "default_spark_enabled")]
    pub enabled: bool,

    #[serde(default = "default_dream_verify")]
    pub verify: bool,

    #[serde(default = "default_dream_min_confidence")]
    pub min_confidence: f64,

    #[serde(default = "default_spark_hypothesis_temperature")]
    pub hypothesis_temperature: f32,

    #[serde(default = "default_dream_verify_temperature")]
    pub verify_temperature: f32,

    /// How many stale vault facts to consider before picking the anchor.
    #[serde(default = "default_spark_candidate_limit")]
    pub candidate_limit: usize,

    /// How many recent facts to offer as contrast context.
    #[serde(default = "default_spark_recent_limit")]
    pub recent_limit: usize,

    /// Confidence for optional quarantine audit entries (always below vault gate).
    #[serde(default = "default_spark_quarantine_confidence")]
    pub quarantine_confidence: f64,

    /// Hours (UTC) when the daemon runs spark, at minute `cron_minute`.
    #[serde(default = "default_spark_cron_hours")]
    pub cron_hours: Vec<u32>,

    #[serde(default = "default_spark_cron_minute")]
    pub cron_minute: u32,

    /// `cron` = fixed hours above; `dice` = d6 jitter between min/max minutes.
    #[serde(default)]
    pub schedule_mode: SparkScheduleMode,

    #[serde(default = "default_spark_dice_min")]
    pub dice_min_minutes: u32,

    #[serde(default = "default_spark_dice_max")]
    pub dice_max_minutes: u32,

    /// LCG seed for dice rolls (defaults from `[chaos].seed` in daemon when unset).
    #[serde(default)]
    pub dice_seed: Option<u64>,

    /// Shallow job caps — keep spark off the deep-research path (AI-Q pattern).
    #[serde(default = "default_spark_max_tokens_hypothesis")]
    pub max_tokens_hypothesis: u32,

    #[serde(default = "default_spark_max_tokens_verify")]
    pub max_tokens_verify: u32,

    #[serde(default = "default_spark_max_connection_chars")]
    pub max_connection_chars: usize,

    /// Minimum quotable span length per anchor (LDR / dream firewall).
    #[serde(default = "default_spark_min_citation_chars")]
    pub min_citation_chars: usize,

    /// Substrings that disqualify a vault fact from being a spark anchor.
    #[serde(default = "default_spark_exclude_anchor_substrings")]
    pub exclude_anchor_substrings: Vec<String>,

    /// Vault `decay_class` values eligible for spark anchors (curated wisdom, not ops noise).
    #[serde(default = "default_spark_anchor_decay_classes")]
    pub anchor_decay_classes: Vec<String>,

    /// Minimum days since `last_accessed_at` before an anchor is considered stale enough.
    #[serde(default = "default_spark_anchor_min_stale_days")]
    pub anchor_min_stale_days: u32,

    /// Maximum days since `created_at` for anchor candidacy (avoid ancient junk).
    #[serde(default = "default_spark_anchor_max_stale_days")]
    pub anchor_max_stale_days: u32,

    /// Anchors must be at least this many hours old (separates fresh ingest slab from recent pool).
    #[serde(default = "default_spark_anchor_min_age_hours")]
    pub anchor_min_age_hours: u32,

    /// Recent pool: only facts created within this many hours (ingest window).
    #[serde(default = "default_spark_recent_max_age_hours")]
    pub recent_max_age_hours: u32,

    /// Minimum embedding cosine between anchor and at least one recent fact (or shared concept tag).
    #[serde(default = "default_spark_min_anchor_recent_similarity")]
    pub min_anchor_recent_similarity: f64,

    /// Drop near-duplicate recent facts above this cosine similarity.
    #[serde(default = "default_spark_recent_dedupe_similarity")]
    pub recent_dedupe_similarity: f64,

    /// Session anchors older than this many days are skipped (parsed from `[Session YYYY-MM-DD …]`).
    #[serde(default = "default_spark_max_session_anchor_age_days")]
    pub max_session_anchor_age_days: u32,

    /// How many recent spark anchors the refractory field retains.
    #[serde(default = "default_spark_refractory_slots")]
    pub refractory_slots: usize,

    /// Exponential half-life (hours) for refractory suppression.
    #[serde(default = "default_spark_refractory_half_life_hours")]
    pub refractory_half_life_hours: f64,

    /// Strength of refractory penalty in `[0, 1]` (1 = full suppress).
    #[serde(default = "default_spark_refractory_strength")]
    pub refractory_strength: f64,

    /// Soft-pick among this many top-scored anchors (1 = greedy).
    #[serde(default = "default_spark_soft_pick_top_k")]
    pub soft_pick_top_k: usize,

    /// Softmax temperature for soft-pick (0 = always take top score).
    #[serde(default = "default_spark_soft_pick_temperature")]
    pub soft_pick_temperature: f64,
}

impl Default for SparkConfig {
    fn default() -> Self {
        Self {
            enabled: default_spark_enabled(),
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            hypothesis_temperature: default_spark_hypothesis_temperature(),
            verify_temperature: default_dream_verify_temperature(),
            candidate_limit: default_spark_candidate_limit(),
            recent_limit: default_spark_recent_limit(),
            quarantine_confidence: default_spark_quarantine_confidence(),
            cron_hours: default_spark_cron_hours(),
            cron_minute: default_spark_cron_minute(),
            schedule_mode: SparkScheduleMode::default(),
            dice_min_minutes: default_spark_dice_min(),
            dice_max_minutes: default_spark_dice_max(),
            dice_seed: None,
            max_tokens_hypothesis: default_spark_max_tokens_hypothesis(),
            max_tokens_verify: default_spark_max_tokens_verify(),
            max_connection_chars: default_spark_max_connection_chars(),
            min_citation_chars: default_spark_min_citation_chars(),
            exclude_anchor_substrings: default_spark_exclude_anchor_substrings(),
            anchor_decay_classes: default_spark_anchor_decay_classes(),
            anchor_min_stale_days: default_spark_anchor_min_stale_days(),
            anchor_max_stale_days: default_spark_anchor_max_stale_days(),
            anchor_min_age_hours: default_spark_anchor_min_age_hours(),
            recent_max_age_hours: default_spark_recent_max_age_hours(),
            min_anchor_recent_similarity: default_spark_min_anchor_recent_similarity(),
            recent_dedupe_similarity: default_spark_recent_dedupe_similarity(),
            max_session_anchor_age_days: default_spark_max_session_anchor_age_days(),
            refractory_slots: default_spark_refractory_slots(),
            refractory_half_life_hours: default_spark_refractory_half_life_hours(),
            refractory_strength: default_spark_refractory_strength(),
            soft_pick_top_k: default_spark_soft_pick_top_k(),
            soft_pick_temperature: default_spark_soft_pick_temperature(),
        }
    }
}
