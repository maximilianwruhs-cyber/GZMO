//! `gzmo honeypot` — reject log and review queue.

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::embeddings::Embedder;
use gzmo_core::memory::honeypot::{self, HONEYPOT_REJECT_LOG};
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::skills::dispatch;
use serde_json::Value;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("rejects") => run_rejects(config, &args[1..]).await,
        Some("review") => run_review(config, &args[1..]).await,
        _ => {
            bail!(
                "Usage:\n  gzmo honeypot rejects [--tail N] [--reason <snake_case>] [--since-hours N]\n  gzmo honeypot review list [--limit N]\n  gzmo honeypot review promote <vault_id>"
            );
        }
    }
}

async fn run_rejects(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let mut tail: Option<usize> = None;
    let mut reason_filter: Option<String> = None;
    let mut since_hours: Option<i64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--tail" => {
                tail = Some(
                    args.get(i + 1)
                        .ok_or_else(|| anyhow::anyhow!("--tail requires a number"))?
                        .parse()?,
                );
                i += 2;
            }
            "--reason" => {
                reason_filter = Some(
                    args.get(i + 1)
                        .ok_or_else(|| anyhow::anyhow!("--reason requires a value"))?
                        .to_lowercase(),
                );
                i += 2;
            }
            "--since-hours" => {
                since_hours = Some(
                    args.get(i + 1)
                        .ok_or_else(|| anyhow::anyhow!("--since-hours requires a number"))?
                        .parse()?,
                );
                i += 2;
            }
            other => bail!("Unknown flag: {other}"),
        }
    }

    let log_path = dispatch::data_dir(config).join(
        Path::new(HONEYPOT_REJECT_LOG)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("honeypot_reject.jsonl")),
    );

    if !log_path.is_file() {
        println!("No reject log at {} (promotions may all qualify)", log_path.display());
        return Ok(());
    }

    let file = std::fs::File::open(&log_path)?;
    let lines: Vec<String> = std::io::BufReader::new(file).lines().map_while(Result::ok).collect();

    let cutoff = since_hours.map(|h| Utc::now() - Duration::hours(h));
    let mut matched: Vec<Value> = Vec::new();
    let mut counts_24h: HashMap<String, usize> = HashMap::new();
    let day_cutoff = Utc::now() - Duration::hours(24);

    for line in &lines {
        let Ok(row) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(ts) = row.get("ts").and_then(|v| v.as_str()) {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
                let utc = parsed.with_timezone(&Utc);
                if utc >= day_cutoff {
                    if let Some(reason) = reason_key(&row) {
                        *counts_24h.entry(reason).or_default() += 1;
                    }
                }
                if let Some(cut) = cutoff {
                    if utc < cut {
                        continue;
                    }
                }
            }
        }
        if let Some(ref want) = reason_filter {
            let got = reason_key(&row).unwrap_or_default();
            if !reason_matches(want, &got) {
                continue;
            }
        }
        matched.push(row);
    }

    let display = if let Some(n) = tail {
        matched
            .into_iter()
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    } else {
        matched
    };

    println!("# honeypot rejects (last 24h by reason)");
    if counts_24h.is_empty() {
        println!("  (none)");
    } else {
        let mut keys: Vec<_> = counts_24h.keys().cloned().collect();
        keys.sort();
        for k in keys {
            println!("  {k}: {}", counts_24h[&k]);
        }
    }
    println!();
    println!("# showing {} line(s) from {}", display.len(), log_path.display());
    for row in display {
        println!("{}", serde_json::to_string(&row)?);
    }
    Ok(())
}

async fn run_review(config: &GzmoConfig, args: &[String]) -> Result<()> {
    match args.first().map(|s| s.as_str()) {
        Some("list") => {
            let mut limit = 20usize;
            let mut i = 1;
            while i < args.len() {
                if args[i] == "--limit" {
                    limit = args
                        .get(i + 1)
                        .context("--limit requires a number")?
                        .parse()?;
                    i += 2;
                } else {
                    bail!("Unknown flag: {}", args[i]);
                }
            }
            let vault = SqliteVault::open(&config.memory.vault_db)?;
            let rows = vault.list_honeypot_review_queue(limit)?;
            if rows.is_empty() {
                println!("Review queue empty");
                return Ok(());
            }
            for (id, reason, preview, conf) in rows {
                println!("{id}\tconf={conf:.2}\t{reason}\t{preview}");
            }
            Ok(())
        }
        Some("promote") => {
            let vault_id = args
                .get(1)
                .context("Usage: gzmo honeypot review promote <vault_id>")?;
            let vault = SqliteVault::open(&config.memory.vault_db)?;
            let content = vault
                .semantic_content(vault_id)
                .with_context(|| format!("vault row not found: {vault_id}"))?;
            let blob: Vec<u8> = if config.embeddings.enabled {
                let embedder = Embedder::from_config(&config.embeddings, &config.redis)?;
                let embedding = embedder.embed(&content).await?;
                embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
            } else {
                bail!("Embeddings required for honeypot review promote");
            };
            let norm = gzmo_core::memory::vault::normalize_truth_content(&content);
            vault.promote_honeypot_from_review(vault_id, &blob, &norm)?;
            println!("Promoted {vault_id} to honeypot (review override)");
            Ok(())
        }
        _ => bail!("Usage: gzmo honeypot review list | gzmo honeypot review promote <vault_id>"),
    }
}

fn reason_key(row: &Value) -> Option<String> {
    row.get("reason")
        .and_then(|r| r.as_object())
        .and_then(|o| o.keys().next())
        .map(|k| k.to_string())
}

fn reason_matches(want: &str, got: &str) -> bool {
    let want = want.replace('_', "").to_lowercase();
    let got = got.replace('_', "").to_lowercase();
    got.contains(&want) || want.contains(&got)
}
