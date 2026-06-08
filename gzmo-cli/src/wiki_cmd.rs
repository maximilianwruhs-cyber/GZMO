//! `gzmo wiki <sub>` — Knowledge Gardener operations over the `wiki/` layer.
//!
//! Subcommands:
//!   gzmo wiki sync                 rebuild index.md from pages on disk
//!   gzmo wiki lint                 structural health report (orphans, etc.)
//!   gzmo wiki search <query> [--limit N]
//!   gzmo wiki file-back <title>    (body read from stdin)
//!   gzmo wiki status               config + page counts

use anyhow::Result;
use tokio::io::AsyncReadExt;

use gzmo_core::config::GzmoConfig;
use gzmo_core::wiki::WikiEngine;

pub async fn run(config: &GzmoConfig, args: Vec<String>) -> Result<()> {
    if !config.wiki.enabled {
        eprintln!("Wiki layer disabled in [wiki] config.");
        return Ok(());
    }
    let engine = WikiEngine::new(config.wiki.clone());
    let sub = args.first().map(|s| s.as_str()).unwrap_or("status");

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
            println!("  emit_on_ingest: {}", config.wiki.emit_on_ingest);
            println!("  schema:         {}", config.wiki.schema_path);
            println!("  entities:       {}", count_md(&format!("{dir}/entities")));
            println!("  concepts:       {}", count_md(&format!("{dir}/concepts")));
            println!("  sources:        {}", count_md(&format!("{dir}/sources")));
        }
        other => {
            eprintln!("Unknown wiki subcommand '{other}'. Use: sync | lint | search | file-back | status");
        }
    }
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
                        && !e
                            .file_name()
                            .to_string_lossy()
                            .starts_with("_lint-")
                })
                .count()
        })
        .unwrap_or(0)
}
