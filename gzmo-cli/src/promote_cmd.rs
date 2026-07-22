//! Overnight promote: mature vault facts → honeypot.

use anyhow::Result;
use chrono::Utc;
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::metabolism;

pub async fn run(config: &GzmoConfig, limit: Option<usize>) -> Result<()> {
    info!("GZMO — promote mature vault → honeypot");
    let started = Utc::now();
    let vault = SqliteVault::open(&config.memory.vault_db)?;
    match vault.promote_mature_to_honeypot(limit) {
        Ok(report) => {
            println!(
                "Promote: {} candidates, {} promoted, {} skipped (gate).",
                report.candidates, report.promoted, report.skipped
            );
            if report.promoted == 0 && report.candidates == 0 {
                eprintln!("(honeypot already covers mature vault rows, or vault is empty)");
            }
            metabolism::write_job_run(config, "promote", "rust", started, true, None);
            Ok(())
        }
        Err(e) => {
            metabolism::write_job_run(
                config,
                "promote",
                "rust",
                started,
                false,
                Some(e.to_string()),
            );
            Err(e)
        }
    }
}
