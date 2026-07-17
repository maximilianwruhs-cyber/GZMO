//! `gzmo status` — deterministic ecosystem snapshot (no LLM).

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::ecosystem_status::format_ecosystem_status;

pub async fn run(config: &GzmoConfig) -> Result<()> {
    let report = format_ecosystem_status(config).await;
    print!("{report}");
    Ok(())
}
