//! Pre-spark honeypot refresh when the recent curated pool is below threshold.

use anyhow::Result;
use tracing::info;

use crate::config::SparkConfig;
use crate::memory::vault::SqliteVault;

/// When enabled and recent pool count is below `anchor_refresh_min_recent`, touch
/// `promoted_at` on a diverse honeypot sample. Returns number of rows updated.
pub fn maybe_refresh_recent_pool(vault: &SqliteVault, config: &SparkConfig) -> Result<Option<usize>> {
    if !config.anchor_refresh_enabled {
        return Ok(None);
    }
    let recent = vault.spark_recent_pool_count(
        &config.anchor_decay_classes,
        config.recent_max_age_hours,
    )?;
    if recent >= config.anchor_refresh_min_recent {
        return Ok(None);
    }
    let touched = vault.refresh_spark_recent_promoted_at(config.anchor_refresh_min_recent)?;
    info!(
        recent_before = recent,
        min_recent = config.anchor_refresh_min_recent,
        touched,
        "Spark anchor refresh before cycle"
    );
    Ok(Some(touched))
}
