//! Shell out to Little Tools Lab assembly recipe scripts.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

fn lab_root() -> PathBuf {
    std::env::var("LITTLE_TOOLS_LAB_ROOT")
        .or_else(|_| std::env::var("GZMO_CLONE_ROOT").map(|r| format!("{r}/little-tools-lab")))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/gzmo/github-clone/little-tools-lab"))
}

fn script_path(name: &str) -> PathBuf {
    lab_root().join("scripts").join(name)
}

pub async fn run(recipe: &str, fixture: bool, apply: bool) -> Result<()> {
    let mode_flag = if fixture { "--fixture" } else { "--live" };
    let (script, extra): (&str, Vec<&str>) = match recipe {
        "calibration" | "bench" | "fuse" => ("bench-to-fuse.sh", vec![mode_flag]),
        "handoff" | "gzmo-handoff" => {
            let mut args = vec![mode_flag];
            if apply {
                args.push("--apply");
            }
            ("gzmo-handoff.sh", args)
        }
        "cognition" => ("cognition-smoke.sh", vec![mode_flag]),
        "knowledge" | "dream" => ("session-to-dream.sh", vec![mode_flag]),
        "pedagogy" => ("pedagogy-smoke.sh", vec![mode_flag]),
        "ops" => ("ops-smoke.sh", vec![mode_flag]),
        "synapse-distill" | "synapse" => ("synapse-distill-handoff.sh", vec![mode_flag]),
        other => anyhow::bail!(
            "unknown recipe '{other}'. Try: calibration, cognition, knowledge, pedagogy, ops, handoff, synapse-distill"
        ),
    };

    let path = script_path(script);
    if !path.is_file() {
        anyhow::bail!("lab script not found: {}", path.display());
    }

    let mut cmd = Command::new("bash");
    cmd.arg(&path);
    for arg in extra {
        cmd.arg(arg);
    }

    let status = cmd
        .status()
        .with_context(|| format!("run {}", path.display()))?;
    if !status.success() {
        anyhow::bail!("recipe '{recipe}' failed (exit {})", status);
    }
    Ok(())
}
