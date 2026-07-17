//! `gzmo kg-reconcile` — one-shot Neo4j ontology reconcile via MCP.

use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use tracing::info;

use gzmo_core::config::GzmoConfig;
use gzmo_core::kg_reconcile;
use gzmo_core::tools::ToolRegistry;

use crate::cli_mcp::McpSession;

pub async fn run(config: &GzmoConfig, args: Vec<String>) -> Result<()> {
    let mut dry_run_override: Option<bool> = None;
    let mut meta: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => {
                dry_run_override = Some(true);
                i += 1;
            }
            "--apply" => {
                dry_run_override = Some(false);
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
                eprintln!("Unknown kg-reconcile flag: {other}");
                i += 1;
            }
        }
    }

    let mut cfg = config.kg_reconcile.clone();
    if let Some(d) = dry_run_override {
        cfg.dry_run = d;
    }

    if !cfg.enabled && dry_run_override.is_none() {
        eprintln!("[kg_reconcile] enabled=false — pass --dry-run or --apply to force");
    }

    let mut tools = ToolRegistry::new();
    let session = McpSession::connect(config, &mut tools).await?;
    let report = kg_reconcile::run_kg_reconcile(&tools, &cfg).await;
    session.close().await;
    let report = report?;

    info!(
        entities = report.entities_scanned,
        relations = report.relations_scanned,
        notes = report.entity_notes_added,
        recanon = report.relations_recanonicalized,
        deleted = report.relations_deleted,
        dry_run = report.dry_run,
        "kg reconcile complete"
    );

    let meta_json = json!({
        "mode": if report.dry_run { "dry_run" } else { "apply" },
        "entities_scanned": report.entities_scanned,
        "relations_scanned": report.relations_scanned,
        "entity_notes_added": report.entity_notes_added,
        "relations_recanonicalized": report.relations_recanonicalized,
        "relations_deleted": report.relations_deleted,
        "dry_run": report.dry_run,
        "healthy": true,
    });

    let meta_path = meta.unwrap_or_else(|| {
        config
            .memory
            .vault_db
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("kg-reconcile-latest.json")
    });
    if let Some(parent) = meta_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        &meta_path,
        serde_json::to_string_pretty(&meta_json)? + "\n",
    )?;

    println!(
        "kg-reconcile: dry_run={} entities={} relations={} notes+={} recanon={} deleted={}",
        report.dry_run,
        report.entities_scanned,
        report.relations_scanned,
        report.entity_notes_added,
        report.relations_recanonicalized,
        report.relations_deleted
    );
    println!("meta: {}", meta_path.display());
    Ok(())
}
