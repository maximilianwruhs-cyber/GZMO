//! `gzmo ripen status` — M5 export gate honesty (no LLM).

use anyhow::Result;
use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::ripen::RipenGateCensus;
use gzmo_core::memory::vault::SqliteVault;

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let sub = args.first().map(String::as_str).unwrap_or("status");
    match sub {
        "status" => run_status(config),
        other => {
            eprintln!("Unknown ripen subcommand: {other}");
            eprintln!("Usage: gzmo ripen status");
            std::process::exit(2);
        }
    }
}

fn run_status(config: &GzmoConfig) -> Result<()> {
    let vault_path = &config.memory.vault_db;
    let data = vault_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let core = data.join("knowledge_core.db");
    let status_path = data.join("ripen/latest.json");

    let vault = SqliteVault::open(vault_path)?;
    let RipenGateCensus {
        latest,
        nonzero_recall,
        dual,
        dual_origin,
    } = vault.ripen_gate_census(0.90, 3)?;
    let core_rows = vault.knowledge_core_row_count(&core).unwrap_or(-1);

    println!("### M5 ripen status\n");
    println!("- **Vault:** `{}`", vault_path.display());
    println!("- **Honeypot latest:** {latest}");
    println!("- **Nonzero recall_count:** {nonzero_recall}");
    println!("- **Dual gate (≥0.90 ∧ recall≥3):** {dual}");
    println!("- **Dual + allowed origin:** {dual_origin}");
    println!(
        "- **knowledge_core.db rows:** {}",
        if core_rows < 0 {
            "missing".into()
        } else {
            core_rows.to_string()
        }
    );
    if status_path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&status_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                let advice = v.get("advice").and_then(|x| x.as_str()).unwrap_or("?");
                let exported = v.get("exported").and_then(|x| x.as_u64()).unwrap_or(0);
                let at = v
                    .get("generated_at")
                    .and_then(|x| x.as_str())
                    .unwrap_or("?");
                println!("- **Last export:** {exported} rows @ {at}");
                println!("- **Advice:** {advice}");
            }
        }
        println!("- **Status path:** `{}`", status_path.display());
    } else {
        println!(
            "- **Status path:** missing (`{}`) — run export-knowledge-core.py",
            status_path.display()
        );
    }

    if nonzero_recall == 0 {
        println!(
            "\n_Starved:_ Felt Use has not marked any latest facts yet; overnight export will emit 0 until living search runs.\n"
        );
    } else if dual_origin == 0 {
        println!(
            "\n_Gate miss:_ recall is moving but dual+origin gate still empty — thicken searches or lower gates deliberately.\n"
        );
    } else {
        println!(
            "\n_Ready:_ dual+origin={dual_origin} — next `honeypot_ripen` / export should emit >0.\n"
        );
    }
    Ok(())
}
