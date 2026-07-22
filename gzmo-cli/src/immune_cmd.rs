//! `gzmo immune plan|forget|apply` — plan-only by default; bounded apply gated.

use std::path::PathBuf;

use anyhow::{bail, Result};
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

pub async fn run_forget(config: &GzmoConfig, max: usize) -> Result<()> {
    let vault = SqliteVault::open(&config.memory.vault_db)?;
    let night = Utc::now().date_naive();
    let path = immune::run_value_forgetting_plan(&vault, night, max)?;
    let plan: ImmunePlan = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    println!("Value-forgetting plan (dry_run) → {}", path.display());
    println!(
        "candidates={} night={}",
        plan.candidates.len(),
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
    Ok(())
}

pub async fn run_apply(config: &GzmoConfig, plan_path: Option<PathBuf>, max: usize) -> Result<()> {
    let confirm = std::env::var("IMMUNE_APPLY").ok().as_deref() == Some("1");
    if !confirm {
        bail!("REFUSE: set IMMUNE_APPLY=1 to apply (bounded supersession)");
    }
    // Keep-quality veto: refuse if local latest is RED when present.
    if let Ok(raw) = std::fs::read_to_string("data-next/keep-quality/latest.json") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if v.get("verdict").and_then(|x| x.as_str()) == Some("RED") {
                bail!("REFUSE: keep-quality latest is RED — fix before immune apply");
            }
        }
    }
    let vault = SqliteVault::open(&config.memory.vault_db)?;
    let path = plan_path.unwrap_or_else(|| vault.data_dir().join("immune/latest.json"));
    let plan: ImmunePlan = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    let out = immune::apply_plan(&vault, &plan, max, true)?;
    println!("Immune apply → {}", out.display());
    Ok(())
}
