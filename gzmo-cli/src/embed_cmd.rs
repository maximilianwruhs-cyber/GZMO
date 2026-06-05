//! Backfill vault embedding vectors for facts stored without them.

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::identity::IdentityEngine;
use gzmo_core::memory::embeddings;
use tracing::info;

pub async fn run(config: &GzmoConfig, _identity: &IdentityEngine, limit: Option<usize>) -> Result<()> {
    let missing = {
        let vault = gzmo_core::memory::vault::SqliteVault::open(&config.memory.vault_db)?;
        vault.count_missing_embeddings()?
    };

    info!(missing, "Vault facts without embeddings");

    if missing == 0 {
        println!("Vault embeddings complete — nothing to backfill.");
        return Ok(());
    }

    let vault =
        embeddings::open_vault_with_embeddings(
            &config.memory.vault_db,
            &config.embeddings,
            &config.rerank,
            &config.qdrant,
        )
        .await?;

    let report = vault.backfill_missing_embeddings(limit).await?;

    println!(
        "Embedding backfill: {} updated, {} failed, {} attempted ({} still missing before run).",
        report.updated,
        report.failed,
        report.attempted,
        missing
    );

    Ok(())
}
