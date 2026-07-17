//! Shell out to Little Tools Lab assembly recipe scripts.
//!
//! Instance-aware: recipes receive the loaded config's vault path, and the
//! handoff recipe targets the active `GZMO_CONFIG` file — so under
//! `GZMO_INSTANCE=next` everything reads/writes gzmo-next.toml + data-next/.

use anyhow::{Context, Result};
use gzmo_core::assembly::{instance_is_next, lab_root};
use gzmo_core::config::GzmoConfig;
use std::path::PathBuf;
use std::process::Command;

fn script_path(name: &str) -> PathBuf {
    lab_root().join("scripts").join(name)
}

pub async fn run(config: &GzmoConfig, recipe: &str, fixture: bool, apply: bool) -> Result<()> {
    let mode_flag = if fixture { "--fixture" } else { "--live" };
    let vault = config.memory.vault_db.to_string_lossy().into_owned();
    let (script, extra): (&str, Vec<String>) = match recipe {
        "calibration" | "bench" | "fuse" => ("bench-to-fuse.sh", vec![mode_flag.into()]),
        "handoff" | "gzmo-handoff" => {
            let mut args = vec![mode_flag.to_string()];
            if apply {
                args.push("--apply".into());
            }
            // Apply target is the sibling *-fused.toml next to GZMO_CONFIG —
            // never the live instance config, which config-fuse output would
            // clobber wholesale (it emits engine/bench sections only).
            if let Some(target) = gzmo_core::assembly::handoff_apply_target() {
                args.push("--gzmo-config".into());
                args.push(target.to_string_lossy().into_owned());
            }
            ("gzmo-handoff.sh", args)
        }
        "cognition" => (
            "cognition-smoke.sh",
            vec![mode_flag.into(), "--vault".into(), vault],
        ),
        "knowledge" | "dream" => ("session-to-dream.sh", vec![mode_flag.into()]),
        "pedagogy" => ("pedagogy-smoke.sh", vec![mode_flag.into()]),
        "ops" => ("ops-smoke.sh", vec![mode_flag.into()]),
        "synapse-distill" | "synapse" => ("synapse-distill-handoff.sh", vec![mode_flag.into()]),
        other => anyhow::bail!(
            "unknown recipe '{other}'. Try: calibration, cognition, knowledge, pedagogy, ops, handoff, synapse-distill"
        ),
    };

    let path = script_path(script);
    if !path.is_file() {
        anyhow::bail!("lab script not found: {}", path.display());
    }

    tracing::info!(
        recipe,
        script,
        instance = if instance_is_next() { "next" } else { "legacy" },
        "Running assembly recipe"
    );

    let mut cmd = Command::new("bash");
    cmd.arg(&path);
    for arg in &extra {
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
