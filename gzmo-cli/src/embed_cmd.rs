//! Backfill vault embedding vectors for facts stored without them.

use anyhow::Result;
use chrono::Utc;
use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::embeddings;
use gzmo_core::metabolism;
use tracing::info;

pub async fn run(config: &GzmoConfig, limit: Option<usize>) -> Result<()> {
    let started = Utc::now();
    let missing = {
        let vault = gzmo_core::memory::vault::SqliteVault::open(&config.memory.vault_db)?;
        vault.count_missing_embeddings()?
    };

    info!(missing, "Vault facts without embeddings");

    if missing == 0 {
        println!("Vault embeddings complete — nothing to backfill.");
        metabolism::write_job_run(config, "embed", "rust", started, true, None);
        return Ok(());
    }

    let vault = embeddings::open_vault_with_embeddings(
        &config.memory.vault_db,
        &config.embeddings,
        &config.redis,
        &config.rerank,
        &config.qdrant,
    )
    .await?;

    match vault.backfill_missing_embeddings(limit).await {
        Ok(report) => {
            println!(
                "Embedding backfill: {} updated, {} failed, {} attempted ({} still missing before run).",
                report.updated, report.failed, report.attempted, missing
            );
            let ok = report.failed == 0;
            let err = if ok {
                None
            } else {
                Some(format!("{} embed failures", report.failed))
            };
            metabolism::write_job_run(config, "embed", "rust", started, ok, err);
            if ok {
                Ok(())
            } else {
                anyhow::bail!("{} embedding backfill failures", report.failed)
            }
        }
        Err(e) => {
            metabolism::write_job_run(config, "embed", "rust", started, false, Some(e.to_string()));
            Err(e)
        }
    }
}
