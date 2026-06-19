//! Shared helpers for `/implement` and `/fixer` skills.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result};

use crate::config::PedagogyConfig;

pub fn scripts_root(cfg: &PedagogyConfig) -> PathBuf {
    PathBuf::from(&cfg.discovery_scripts_root)
}

pub async fn run_scripts_command(
    root: &Path,
    script_name: &str,
    args: &[&str],
) -> Result<(i32, String)> {
    let script = root.join("scripts").join(script_name);
    if !script.is_file() {
        anyhow::bail!(
            "script not found: {} (check pedagogy.discovery_scripts_root)",
            script.display()
        );
    }

    let mut cmd = tokio::process::Command::new(&script);
    cmd.args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Ok(gzmo_root) = std::env::var("GZMO_ROOT") {
        cmd.env("GZMO_ROOT", gzmo_root);
    }
    if let Ok(bin) = std::env::var("GZMO_BIN") {
        cmd.env("GZMO_BIN", bin);
    }

    let output = cmd
        .output()
        .await
        .with_context(|| format!("run {}", script.display()))?;

    let mut combined = String::new();
    if !output.stdout.is_empty() {
        combined.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    Ok((output.status.code().unwrap_or(1), combined.trim().to_string()))
}
