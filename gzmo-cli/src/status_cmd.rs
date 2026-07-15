//! `gzmo status` — deterministic ecosystem snapshot (no LLM).

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::ecosystem_status::format_ecosystem_status;
use gzmo_core::identity::IdentityEngine;

pub async fn run(config: &GzmoConfig, _identity: &IdentityEngine) -> Result<()> {
    let report = format_ecosystem_status(config).await;
    print!("{report}");
    Ok(())
}
