//! `gzmo immune plan` — plan-only contradiction patrol (never applies).

use anyhow::Result;
use chrono::Utc;
use gzmo_core::config::GzmoConfig;
use gzmo_core::immune::{self, ImmunePlan};
use gzmo_core::memory::vault::SqliteVault;

pub async fn run_plan(config: &GzmoConfig) -> Result<()> {
    let vault = SqliteVault::open(&config.memory.vault_db)?;
    let night = Utc::now().date_naive();
    let path = immune::run_patrol(&vault, night, &[])?;
    let plan: ImmunePlan = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    println!("Immune plan (dry_run) → {}", path.display());
    println!(
        "candidates={} truths_scanned={} night={}",
        plan.candidates.len(),
        plan.truths_scanned,
        plan.night_id
    );
    for c in plan.candidates.iter().take(12) {
        println!(
            "- [{}] {} — {}",
            c.reason,
            c.fact_id,
            c.content.chars().take(100).collect::<String>()
        );
    }
    if plan.candidates.len() > 12 {
        println!("… {} more (see JSON)", plan.candidates.len() - 12);
    }
    Ok(())
}
