//! Overnight promote: mature vault facts → honeypot.

use anyhow::Result;
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::vault::SqliteVault;

pub async fn run(config: &GzmoConfig, limit: Option<usize>) -> Result<()> {
    info!("GZMO — promote mature vault → honeypot");
    let vault = SqliteVault::open(&config.memory.vault_db)?;
    let report = vault.promote_mature_to_honeypot(limit)?;
    println!(
        "Promote: {} candidates, {} promoted, {} skipped (gate).",
        report.candidates, report.promoted, report.skipped
    );
    if report.promoted == 0 && report.candidates == 0 {
        eprintln!("(honeypot already covers mature vault rows, or vault is empty)");
    }
    Ok(())
}
