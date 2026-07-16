//! `gzmo config promote-fused` — review/apply sibling fused calibration TOML.

use anyhow::{bail, Context, Result};
use gzmo_core::assembly::handoff_apply_target;
use gzmo_core::config::GzmoConfig;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "Usage:\n  \
    gzmo config promote-fused --diff\n  \
    gzmo config promote-fused --apply\n  \
    gzmo config promote-fused --diff --apply\n  \
    \n\
    Compares live GZMO_CONFIG to the sibling *-fused.toml produced by\n  \
    `gzmo assemble handoff --live --apply`. --diff prints unified diff;\n  \
    --apply copies fused → live (full file — review carefully).";

pub async fn run(_config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "--help" || a == "-h") || args.is_empty() {
        println!("{USAGE}");
        return Ok(());
    }

    let sub = args.first().map(|s| s.as_str()).unwrap_or("");
    if sub != "promote-fused" {
        bail!("unknown config subcommand '{sub}'. Try: gzmo config promote-fused");
    }

    let want_diff = args.iter().any(|a| a == "--diff");
    let want_apply = args.iter().any(|a| a == "--apply");
    if !want_diff && !want_apply {
        bail!("specify --diff and/or --apply\n\n{USAGE}");
    }

    let live = PathBuf::from(
        std::env::var("GZMO_CONFIG").context("GZMO_CONFIG must be set for promote-fused")?,
    );
    if !live.is_file() {
        bail!("live config not found: {}", live.display());
    }
    let fused = handoff_apply_target().context("could not resolve sibling *-fused.toml path")?;
    if !fused.is_file() {
        bail!(
            "fused config absent: {}\nRun: gzmo assemble handoff --live --apply",
            fused.display()
        );
    }

    if want_diff {
        println!("--- {} (live)", live.display());
        println!("+++ {} (fused)", fused.display());
        let status = Command::new("diff")
            .args(["-u", &live.to_string_lossy(), &fused.to_string_lossy()])
            .status()
            .context("run diff")?;
        // diff exits 1 when files differ — that is success for --diff
        if !status.success() && status.code() != Some(1) {
            bail!("diff failed (exit {})", status.code().unwrap_or(-1));
        }
        if status.code() == Some(0) {
            println!("(no differences)");
        }
    }

    if want_apply {
        promote_copy(&fused, &live)?;
        println!("promoted {} → {}", fused.display(), live.display());
        println!("note: full-file copy — verify [assembly]/[memory] if fuse was engine-only");
    }

    Ok(())
}

fn promote_copy(fused: &Path, live: &Path) -> Result<()> {
    let backup = live.with_extension("toml.bak-promote");
    std::fs::copy(live, &backup)
        .with_context(|| format!("backup live → {}", backup.display()))?;
    std::fs::copy(fused, live)
        .with_context(|| format!("copy fused → {}", live.display()))?;
    println!("backup: {}", backup.display());
    Ok(())
}
