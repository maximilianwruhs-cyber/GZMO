//! Little Tools Lab recipe runner — for `gzmo assemble` and future GZMO-next runtime.
//! Not wired into CT101 legacy daemon (see docs/CT101_BOUNDARY.md).

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Per-loop backend: inline gzmo-core engine or lab bash recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssemblyBackend {
    #[default]
    Inline,
    Lab,
}

impl AssemblyBackend {
    pub fn is_lab(self) -> bool {
        matches!(self, AssemblyBackend::Lab)
    }

    pub fn label(self) -> &'static str {
        match self {
            AssemblyBackend::Inline => "inline",
            AssemblyBackend::Lab => "lab",
        }
    }
}

/// True only when the process runs as the GZMO-next instance.
pub fn instance_is_next() -> bool {
    std::env::var("GZMO_INSTANCE").is_ok_and(|v| v == "next")
}

/// Where `gzmo-handoff.sh --apply` may write the fused calibration TOML.
///
/// NEVER the live instance config: config-fuse emits a full-file replacement
/// (engine/scheduler sections only), so applying onto gzmo-next.toml would
/// clobber [assembly], [memory] and every other instance section. Instead the
/// fused output lands next to it as `<stem>-fused.toml` for operator review.
pub fn handoff_apply_target() -> Option<PathBuf> {
    let config = PathBuf::from(std::env::var("GZMO_CONFIG").ok()?);
    let stem = config.file_stem()?.to_string_lossy().into_owned();
    Some(config.with_file_name(format!("{stem}-fused.toml")))
}

/// GZMO-next runtime config (`[assembly]` in gzmo-next.toml) — **not** used on CT101 legacy daemon.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct AssemblyConfig {
    #[serde(default)]
    pub distill: AssemblyBackend,
    #[serde(default)]
    pub dream: AssemblyBackend,
    #[serde(default)]
    pub spark: AssemblyBackend,
    #[serde(default)]
    pub ops_health: AssemblyBackend,
    #[serde(default)]
    pub config_handoff: AssemblyBackend,
}

impl AssemblyConfig {
    /// Backend to actually use for a loop. Guardrail: lab backends only
    /// activate when `GZMO_INSTANCE=next`; any other instance (CT101 legacy,
    /// unset env) is forced to Inline regardless of what the TOML says.
    pub fn effective(&self, configured: AssemblyBackend) -> AssemblyBackend {
        if configured.is_lab() && !instance_is_next() {
            return AssemblyBackend::Inline;
        }
        configured
    }
}

/// Resolve little-tools-lab root (same precedence as `gzmo assemble`).
pub fn lab_root() -> PathBuf {
    std::env::var("LITTLE_TOOLS_LAB_ROOT")
        .or_else(|_| {
            std::env::var("GZMO_CLONE_ROOT").map(|r| format!("{r}/little-tools-lab"))
        })
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/home/gzmo/github-clone/little-tools-lab"))
}

fn script_path(name: &str) -> PathBuf {
    lab_root().join("scripts").join(name)
}

/// Run a lab assembly script with args. Used when `[assembly].* = "lab"`.
pub fn run_lab_script(script: &str, args: &[&str]) -> Result<()> {
    let path = script_path(script);
    if !path.is_file() {
        anyhow::bail!("lab script not found: {}", path.display());
    }
    let mut cmd = Command::new("bash");
    cmd.arg(&path);
    for arg in args {
        cmd.arg(arg);
    }
    let status = cmd
        .status()
        .with_context(|| format!("run {}", path.display()))?;
    if !status.success() {
        anyhow::bail!(
            "lab script {} failed (exit {})",
            script,
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backends_are_inline() {
        let cfg = AssemblyConfig::default();
        assert_eq!(cfg.distill, AssemblyBackend::Inline);
        assert_eq!(cfg.config_handoff, AssemblyBackend::Inline);
    }
}
