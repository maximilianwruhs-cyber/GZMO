//! `gzmo profile` — static + dynamic operator context from honeypot.

use anyhow::{bail, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::embeddings;
use gzmo_core::memory::profile::{GzmoProfile, ProfileOptions};

const USAGE: &str =
    "Usage:\n  \
     gzmo profile [--scope obolus] [--format yaml|json|md] [--dynamic-only]\n  \
     \n\
     Builds cached static+dynamic profile from honeypot (Spec §5).";

#[derive(Clone, Copy)]
enum OutputFormat {
    Yaml,
    Json,
    Md,
}

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Ok(());
    }

    let scope = args
        .iter()
        .position(|a| a == "--scope")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("obolus");

    let dynamic_only = args.iter().any(|a| a == "--dynamic-only");
    let format = args
        .iter()
        .position(|a| a == "--format")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("yaml");

    let fmt = match format {
        "yaml" | "yml" => OutputFormat::Yaml,
        "json" => OutputFormat::Json,
        "md" | "markdown" => OutputFormat::Md,
        _ => bail!("unknown --format {format} (use yaml, json, or md)"),
    };

    let vault = embeddings::open_vault_with_embeddings(
        &config.memory.vault_db,
        &config.embeddings,
        &config.redis,
        &config.rerank,
        &config.qdrant,
            &config.recall,
    )
    .await?;

    if scope != "obolus" {
        bail!("only --scope obolus is supported today");
    }
    let opts = ProfileOptions {
        container_tag: scope.to_string(),
        dynamic_only,
        ..ProfileOptions::default()
    };

    let started = std::time::Instant::now();
    let profile = vault.build_profile(opts)?;
    let elapsed_ms = started.elapsed().as_millis();

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&profile)?),
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&profile)?);
        }
        OutputFormat::Md => println!("{}", profile.to_markdown()),
    }

    eprintln!(
        "profile: scope={} static={} dynamic={} tokens≈{} build_ms={}",
        profile.container_tag,
        profile.r#static.len(),
        profile.dynamic.len(),
        profile.token_estimate,
        elapsed_ms
    );

    Ok(())
}
