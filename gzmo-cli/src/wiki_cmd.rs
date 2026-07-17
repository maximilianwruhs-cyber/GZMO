//! `gzmo wiki <sub>` — Knowledge Gardener operations over the `wiki/` layer.
//!
//! Subcommands:
//!   gzmo wiki sync                 rebuild index.md from pages on disk
//!   gzmo wiki lint                 structural health report (orphans, etc.)
//!   gzmo wiki search <query> [--limit N]
//!   gzmo wiki file-back <title>    (body read from stdin)
//!   gzmo wiki status               config + page counts
//!   gzmo wiki push [--origin NAME] [--limit N] [--dry-run] [--meta PATH]
//!                                  push vault facts to OKForge via OKCP

use anyhow::Result;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

use gzmo_core::config::GzmoConfig;
use gzmo_core::wiki::WikiEngine;
use gzmo_core::wiki_okf;

pub async fn run(config: &GzmoConfig, args: Vec<String>) -> Result<()> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");

    if sub == "push" {
        return push(config, &args[1..]).await;
    }

    if !config.wiki.enabled {
        eprintln!("Wiki layer disabled in [wiki] config.");
        return Ok(());
    }
    let engine = WikiEngine::new(config.wiki.clone());

    match sub {
        "sync" => {
            let r = engine.sync().await?;
            println!(
                "wiki sync: {} index entries across {} pages",
                r.index_entries, r.pages
            );
        }
        "lint" => {
            let r = engine.lint().await?;
            println!(
                "wiki lint: {} pages | {} orphans, {} broken links, {} missing frontmatter, {} stale",
                r.pages,
                r.orphans.len(),
                r.broken_links.len(),
                r.missing_frontmatter.len(),
                r.stale.len()
            );
            println!("report: {}", r.report_path);
        }
        "search" => {
            let (query, limit) = parse_search_args(&args[1..]);
            if query.is_empty() {
                eprintln!("Usage: gzmo wiki search <query> [--limit N]");
                return Ok(());
            }
            let hits = engine.search(&query, limit);
            if hits.is_empty() {
                println!("No wiki pages matched '{query}'.");
            } else {
                for h in hits {
                    println!("[{}] {} ({})", h.score, h.title, h.path);
                    println!("    {}", h.snippet);
                }
            }
        }
        "file-back" => {
            let title = args[1..].join(" ");
            if title.trim().is_empty() {
                eprintln!("Usage: gzmo wiki file-back <title>   (body read from stdin)");
                return Ok(());
            }
            let mut body = String::new();
            tokio::io::stdin().read_to_string(&mut body).await?;
            let path = engine.file_back(title.trim(), &body).await?;
            println!("Filed concept page: {path}");
        }
        "status" => {
            let dir = &config.wiki.directory;
            println!("Wiki layer status");
            println!("  directory:      {dir}");
            println!("  enabled:        {}", config.wiki.enabled);
            println!("  backend:        {}", config.wiki.backend);
            println!("  emit_on_ingest: {}", config.wiki.emit_on_ingest);
            println!("  emit_after_distill: {}", config.wiki.emit_after_distill);
            println!("  emit_after_dream:   {}", config.wiki.emit_after_dream);
            println!("  schema:         {}", config.wiki.schema_path);
            if let Some(okf) = &config.wiki.okforge {
                println!(
                    "  okforge:        {}/{}/{} ({})",
                    okf.owner, okf.repo, "concepts", okf.url
                );
            }
            println!("  entities:       {}", count_md(&format!("{dir}/entities")));
            println!("  concepts:       {}", count_md(&format!("{dir}/concepts")));
            println!("  sources:        {}", count_md(&format!("{dir}/sources")));
        }
        other => {
            eprintln!(
                "Unknown wiki subcommand '{other}'. Use: sync | lint | search | file-back | status | push"
            );
        }
    }
    Ok(())
}

async fn push(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let mut origin = "manual".to_string();
    let mut limit = 40usize;
    let mut dry_run = false;
    let mut meta: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--origin" => {
                if let Some(v) = args.get(i + 1) {
                    origin = v.clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--limit" => {
                if let Some(n) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                    limit = n;
                }
                i += 2;
            }
            "--dry-run" => {
                dry_run = true;
                i += 1;
            }
            "--meta" => {
                if let Some(v) = args.get(i + 1) {
                    meta = Some(PathBuf::from(v));
                    i += 2;
                } else {
                    i += 1;
                }
            }
            other => {
                eprintln!("Unknown wiki push flag: {other}");
                i += 1;
            }
        }
    }

    let report = wiki_okf::push_from_vault(
        &config.wiki,
        &config.memory.vault_db,
        &origin,
        limit,
        dry_run,
    )
    .await?;

    let meta_path = meta.unwrap_or_else(|| {
        config
            .memory
            .vault_db
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("wiki-push-latest.json")
    });
    wiki_okf::write_push_report(&meta_path, &report)?;

    println!(
        "wiki push: mode={} origin={} concepts={} sha={} skipped={}",
        report.mode,
        report.origin,
        report.concepts_written,
        report.commit_sha,
        report.skipped_reason
    );
    println!("meta: {}", meta_path.display());
    Ok(())
}

fn parse_search_args(args: &[String]) -> (String, usize) {
    let mut limit = 5usize;
    let mut terms: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--limit" {
            if let Some(n) = args.get(i + 1).and_then(|s| s.parse().ok()) {
                limit = n;
            }
            i += 2;
        } else {
            terms.push(args[i].clone());
            i += 1;
        }
    }
    (terms.join(" "), limit)
}

fn count_md(dir: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path().extension().and_then(|x| x.to_str()) == Some("md")
                        && !e.file_name().to_string_lossy().starts_with("_lint-")
                })
                .count()
        })
        .unwrap_or(0)
}
