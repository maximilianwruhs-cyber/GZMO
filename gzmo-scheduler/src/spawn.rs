//! Spawn Little Tools Lab recipe scripts as subprocesses.

use crate::config::SchedulerConfig;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::path::PathBuf;
use tokio::process::Command;

/// GZMO repo root (contains `gzmo.toml`, `scripts/`).
pub fn gzmo_root() -> PathBuf {
    std::env::var("GZMO_CLONE_ROOT")
        .map(|r| PathBuf::from(r).join("GZMO"))
        .unwrap_or_else(|_| PathBuf::from("/home/gzmo/github-clone/GZMO"))
}

/// Run `bash <gzmo>/scripts/<script> <args…>`.
pub async fn run_gzmo_script(cfg: &SchedulerConfig, script: &str, args: &[String]) -> Result<()> {
    let path = gzmo_root().join("scripts").join(script);
    if !path.is_file() {
        bail!("gzmo script not found: {}", path.display());
    }
    let night_id = Utc::now().format("%Y-%m-%d").to_string();
    let status = Command::new("bash")
        .arg(&path)
        .args(args)
        .env("LIBRARIAN_URL", recipe_service_url(cfg.librarian_url()))
        .env("LLM_URL", recipe_service_url(cfg.llm_url()))
        .env("EMBED_URL", cfg.embed_url())
        .env("EMBED_MODEL", cfg.embed_model())
        .env("GZMO_NIGHT_ID", &night_id)
        .status()
        .await
        .with_context(|| format!("spawn {}", path.display()))?;
    if !status.success() {
        bail!(
            "gzmo script {} failed (exit {})",
            script,
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

/// Resolve little-tools-lab root (same precedence as `gzmo assemble`).
pub fn lab_root() -> PathBuf {
    std::env::var("LITTLE_TOOLS_LAB_ROOT")
        .or_else(|_| std::env::var("GZMO_CLONE_ROOT").map(|r| format!("{r}/little-tools-lab")))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/gzmo/github-clone/little-tools-lab"))
}

/// Strip trailing `/v1` so recipes that probe `${URL}/v1/models` or append
/// `/v1/chat/completions` (session-distill) match OpenAI-base config URLs.
fn recipe_service_url(url: &str) -> String {
    let u = url.trim().trim_end_matches('/');
    u.strip_suffix("/v1").unwrap_or(u).to_string()
}

/// Run `bash <lab>/scripts/<script> <args…>`, inheriting the scheduler env
/// and injecting librarian/LLM/embed URLs from the instance config.
pub async fn run_lab_script(cfg: &SchedulerConfig, script: &str, args: &[String]) -> Result<()> {
    let path = lab_root().join("scripts").join(script);
    if !path.is_file() {
        bail!("lab script not found: {}", path.display());
    }
    let night_id = Utc::now().format("%Y-%m-%d").to_string();
    let status = Command::new("bash")
        .arg(&path)
        .args(args)
        .env("LIBRARIAN_URL", recipe_service_url(cfg.librarian_url()))
        .env("LLM_URL", recipe_service_url(cfg.llm_url()))
        .env("EMBED_URL", cfg.embed_url())
        .env("EMBED_MODEL", cfg.embed_model())
        .env("GZMO_NIGHT_ID", &night_id)
        .status()
        .await
        .with_context(|| format!("spawn {}", path.display()))?;
    if !status.success() {
        bail!(
            "lab script {} failed (exit {})",
            script,
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}
