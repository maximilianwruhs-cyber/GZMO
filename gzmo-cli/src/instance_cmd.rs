//! `gzmo instance status` — effective backends + paths for this process.

use anyhow::Result;
use gzmo_core::assembly::{handoff_apply_target, instance_is_next, lab_root, AssemblyBackend};
use gzmo_core::config::GzmoConfig;

const USAGE: &str = "Usage:\n  \
    gzmo instance status\n  \
    \n\
    Shows GZMO_INSTANCE, config path, data root, lab root, and effective\n  \
    assembly backends (configured vs forced-inline when instance ≠ next).";

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }

    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");
    if sub != "status" {
        anyhow::bail!("unknown instance subcommand '{sub}'. Try: gzmo instance status");
    }

    let instance = std::env::var("GZMO_INSTANCE").unwrap_or_else(|_| "(unset)".into());
    let config_path = std::env::var("GZMO_CONFIG").unwrap_or_else(|_| {
        "(unset — load_auto used cwd/exe gzmo.toml)".into()
    });
    let data_root = config
        .memory
        .vault_db
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unknown)".into());

    let fused = handoff_apply_target();
    let live_config = std::env::var("GZMO_CONFIG")
        .ok()
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists());
    let fused_status = match fused.as_ref() {
        None => "n/a".to_string(),
        Some(fp) if !fp.exists() => "absent".to_string(),
        Some(fp) => {
            let fuse_m = std::fs::metadata(fp).and_then(|m| m.modified()).ok();
            let live_m = live_config
                .as_ref()
                .and_then(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
            match (fuse_m, live_m) {
                (Some(f), Some(l)) if f > l => {
                    "pending — fused newer than live; run: gzmo config promote-fused --diff".into()
                }
                (Some(_), Some(_)) => "present — live at/after fused (promote done or equal)".into(),
                (Some(_), None) => {
                    "present — review + gzmo config promote-fused --diff (no live mtime)".into()
                }
                _ => "present".into(),
            }
        }
    };

    println!("## GZMO instance status\n");
    println!("instance:     {instance}");
    println!("is_next:      {}", instance_is_next());
    println!("GZMO_CONFIG:  {config_path}");
    println!("data_root:    {data_root}");
    println!("vault:        {}", config.memory.vault_db.display());
    println!("lab_root:     {}", lab_root().display());
    println!("skills_root:  {} (authoritative for GZMO_INSTANCE=next)", config.skills.directory.display());
    println!(
        "skills_aux:   gzmo_skills/ — CT101/bridge discovery only (not next chat /skills)"
    );
    println!(
        "fused_toml:   {} ({})",
        fused
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(n/a)".into()),
        fused_status
    );
    println!();
    println!("### Assembly backends (configured → effective)");
    println!();
    print_loop("distill", config.assembly.distill, &config.assembly);
    print_loop("dream", config.assembly.dream, &config.assembly);
    print_loop("spark", config.assembly.spark, &config.assembly);
    print_loop("ops_health", config.assembly.ops_health, &config.assembly);
    print_loop(
        "config_handoff",
        config.assembly.config_handoff,
        &config.assembly,
    );
    println!();
    println!("### Memory plane flags");
    println!(
        "embeddings={} qdrant={} redis={} ingest={}",
        config.embeddings.enabled,
        config.qdrant.enabled,
        config.redis.enabled,
        config.ingest.enabled
    );

    Ok(())
}

fn print_loop(
    name: &str,
    configured: AssemblyBackend,
    asm: &gzmo_core::assembly::AssemblyConfig,
) {
    let effective = asm.effective(configured);
    let note = if configured.is_lab() && !instance_is_next() {
        "  [forced inline — set GZMO_INSTANCE=next for lab]"
    } else {
        ""
    };
    println!(
        "  {name:16} configured={:<6} effective={}{note}",
        configured.label(),
        effective.label()
    );
}
