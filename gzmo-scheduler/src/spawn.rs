//! Spawn Little Tools Lab recipe scripts as subprocesses.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tokio::process::Command;

/// GZMO repo root (contains `gzmo.toml`, `scripts/`).
pub fn gzmo_root() -> PathBuf {
    std::env::var("GZMO_CLONE_ROOT")
        .map(|r| PathBuf::from(r).join("GZMO"))
        .unwrap_or_else(|_| PathBuf::from("/home/gzmo/github-clone/GZMO"))
}

/// Run `bash <gzmo>/scripts/<script> <args…>`.
pub async fn run_gzmo_script(script: &str, args: &[String]) -> Result<()> {
    let path = gzmo_root().join("scripts").join(script);
    if !path.is_file() {
        bail!("gzmo script not found: {}", path.display());
    }
    let status = Command::new("bash")
        .arg(&path)
        .args(args)
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

/// Run `bash <lab>/scripts/<script> <args…>`, inheriting the scheduler env
/// (GZMO_CONFIG, GZMO_INSTANCE, LLM_URL, CARGO_TARGET_DIR, …).
pub async fn run_lab_script(script: &str, args: &[String]) -> Result<()> {
    let path = lab_root().join("scripts").join(script);
    if !path.is_file() {
        bail!("lab script not found: {}", path.display());
    }
    let status = Command::new("bash")
        .arg(&path)
        .args(args)
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
