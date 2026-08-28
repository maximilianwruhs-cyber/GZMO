use serde::Deserialize;

use super::defaults::*;

// ─── Ingest ─────────────────────────────────────────────────────────────

/// Settings for gated document ingest (knowledge watcher / `gzmo ingest`).
#[derive(Debug, Deserialize, Clone)]
pub struct IngestConfig {
    /// When true, watchers use IngestEngine instead of headless tool loops.
    #[serde(default = "default_ingest_enabled")]
    pub enabled: bool,

    /// Nightly batch `gzmo ingest-dir` via gzmo-scheduler (watcher stays off).
    #[serde(default)]
    pub batch_enabled: bool,

    /// Inbox directory for batch ingest (relative to config dir or absolute).
    #[serde(default = "default_ingest_inbox")]
    pub inbox_path: String,

    #[serde(default = "default_ingest_batch_hour")]
    pub cron_hour: u32,

    #[serde(default)]
    pub cron_minute: u32,

    #[serde(default = "default_dream_verify")]
    pub verify: bool,

    #[serde(default = "default_dream_min_confidence")]
    pub min_confidence: f64,

    #[serde(default = "default_dream_verify_temperature")]
    pub verify_temperature: f32,

    /// Near-deterministic decoding for the extraction pass (mirrors
    /// `verify_temperature`): keeps KG extract independent of engine/chaos temp.
    #[serde(default = "default_dream_verify_temperature")]
    pub extract_temperature: f32,

    #[serde(default = "default_kg_require_evidence")]
    pub require_evidence: bool,

    #[serde(default = "default_kg_strict")]
    pub strict_kg: bool,

    #[serde(default = "default_ingest_max_source_chars")]
    pub max_source_chars: usize,

    #[serde(default = "default_pipeline_chunk_chars")]
    pub chunk_chars: usize,
}

impl IngestConfig {
    pub fn kg_gate(&self) -> crate::memory::kg_extract::KgGateConfig {
        crate::memory::kg_extract::KgGateConfig {
            verify: self.verify,
            min_confidence: self.min_confidence,
            verify_temperature: self.verify_temperature,
            extract_temperature: self.extract_temperature,
            require_evidence: self.require_evidence,
            strict_kg: self.strict_kg,
        }
    }
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            enabled: default_ingest_enabled(),
            batch_enabled: false,
            inbox_path: default_ingest_inbox(),
            cron_hour: default_ingest_batch_hour(),
            cron_minute: 0,
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            verify_temperature: default_dream_verify_temperature(),
            extract_temperature: default_dream_verify_temperature(),
            require_evidence: default_kg_require_evidence(),
            strict_kg: default_kg_strict(),
            max_source_chars: default_ingest_max_source_chars(),
            chunk_chars: default_pipeline_chunk_chars(),
        }
    }
}
