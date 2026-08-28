use serde::Deserialize;

use super::defaults::*;

// ─── Metabolism (`gzmo serve`) ──────────────────────────────────────────

/// Overnight promote/embed cron slots for the thin typed runner (ADR-0003).
#[derive(Debug, Deserialize, Clone)]
pub struct MetabolismConfig {
    #[serde(default = "default_metabolism_enabled")]
    pub enabled: bool,
    #[serde(default = "default_metabolism_promote_hour")]
    pub promote_cron_hour: u32,
    #[serde(default = "default_metabolism_promote_minute")]
    pub promote_cron_minute: u32,
    #[serde(default = "default_metabolism_embed_hour")]
    pub embed_cron_hour: u32,
    #[serde(default = "default_metabolism_embed_minute")]
    pub embed_cron_minute: u32,
}

impl Default for MetabolismConfig {
    fn default() -> Self {
        Self {
            enabled: default_metabolism_enabled(),
            promote_cron_hour: default_metabolism_promote_hour(),
            promote_cron_minute: default_metabolism_promote_minute(),
            embed_cron_hour: default_metabolism_embed_hour(),
            embed_cron_minute: default_metabolism_embed_minute(),
        }
    }
}
