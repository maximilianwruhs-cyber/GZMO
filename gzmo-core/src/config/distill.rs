use serde::Deserialize;

use super::defaults::*;

// ─── Session distill ──────────────────────────────────────────────────────

/// Settings for `gzmo distill` — sessions JSON → SessionDistill vault + episodic.
#[derive(Debug, Deserialize, Clone)]
pub struct SessionDistillConfig {
    #[serde(default = "default_session_distill_enabled")]
    pub enabled: bool,

    #[serde(default = "default_sessions_dir")]
    pub sessions_dir: std::path::PathBuf,

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

    #[serde(default = "default_pipeline_chunk_chars")]
    pub chunk_chars: usize,

    #[serde(default = "default_session_distill_max_transcript")]
    pub max_transcript_chars: usize,

    /// Use `[librarian]` for KG extract; Prime (local engine) stays on verify.
    #[serde(default = "default_session_distill_use_librarian")]
    pub use_librarian: bool,

    /// Short narrative for episodic via librarian (falls back to entity list).
    #[serde(default = "default_session_distill_librarian_summary")]
    pub librarian_summary: bool,

    /// Run `gzmo distill` on a daily cron inside the daemon (default 02:15 UTC).
    #[serde(default = "default_session_distill_daemon_scheduled")]
    pub daemon_scheduled: bool,

    #[serde(default = "default_session_distill_cron_hour")]
    pub cron_hour: u32,

    #[serde(default = "default_session_distill_cron_minute")]
    pub cron_minute: u32,
}

impl SessionDistillConfig {
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

impl Default for SessionDistillConfig {
    fn default() -> Self {
        Self {
            enabled: default_session_distill_enabled(),
            sessions_dir: default_sessions_dir(),
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            verify_temperature: default_dream_verify_temperature(),
            extract_temperature: default_dream_verify_temperature(),
            require_evidence: default_kg_require_evidence(),
            strict_kg: default_kg_strict(),
            chunk_chars: default_pipeline_chunk_chars(),
            max_transcript_chars: default_session_distill_max_transcript(),
            use_librarian: default_session_distill_use_librarian(),
            librarian_summary: default_session_distill_librarian_summary(),
            daemon_scheduled: default_session_distill_daemon_scheduled(),
            cron_hour: default_session_distill_cron_hour(),
            cron_minute: default_session_distill_cron_minute(),
        }
    }
}
