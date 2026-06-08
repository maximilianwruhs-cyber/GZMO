//! CLI for frontend memory tools (`gzmo memory *`).

use std::sync::Arc;

use anyhow::{bail, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::memory::embeddings;
use gzmo_core::platform_memory::PlatformMemory;

const USAGE: &str =
    "Usage:\n  \
     gzmo memory search <query> [--limit N] [--session ID] [--json] [--no-scratch]\n  \
     gzmo memory recall [--session ID]\n  \
     gzmo memory status [--session ID] [--json]\n  \
     gzmo memory turn-start [--session ID]\n  \
     gzmo memory chain <fact-id>\n  \
     \n\
     Session: --session or GZMO_SESSION_ID env.\n\
     See docs/ARCHITECTURE_GZMO_PLATFORM.md";


fn parse_session_flag(args: &[String]) -> Option<String> {
    args.iter()
        .position(|a| a == "--session")
        .and_then(|i| args.get(i + 1).cloned())
}

fn parse_limit(args: &[String], default: usize) -> usize {
    args.iter()
        .position(|a| a == "--limit")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
        .clamp(1, 20)
}

fn wants_json(args: &[String]) -> bool {
    args.iter().any(|a| a == "--json")
}

pub async fn run(config: &GzmoConfig, subargs: Vec<String>) -> Result<()> {
    let Some(sub) = subargs.first().map(|s| s.as_str()) else {
        eprintln!("{USAGE}");
        bail!("missing memory subcommand");
    };

    let session_id = parse_session_flag(&subargs);
    let platform = Arc::new(PlatformMemory::open(config, session_id).await?);

    match sub {
        "search" => {
            let query = subargs
                .get(1)
                .filter(|s| !s.starts_with("--"))
                .ok_or_else(|| anyhow::anyhow!("missing query"))?;
            let limit = parse_limit(&subargs, 5);
            let write_scratch = !subargs.iter().any(|a| a == "--no-scratch");
            let res = platform.memory_search(query, limit, write_scratch).await?;
            if wants_json(&subargs) {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("{}", res.text);
                if res.scratch_written {
                    eprintln!(
                        "\n(scratch updated — session {}; pull with `gzmo memory recall`)",
                        platform.session_id()
                    );
                }
            }
        }
        "recall" => {
            match platform.memory_recall_pull().await? {
                Some(block) => println!("{block}"),
                None => println!("(no scratch recall for this session)"),
            }
        }
        "status" => {
            let st = platform.status().await?;
            if wants_json(&subargs) {
                println!("{}", serde_json::to_string_pretty(&st)?);
            } else {
                println!(
                    "session={} vault_facts={} scratch={} has_recall={}",
                    st.session_id, st.vault_facts, st.scratch_backend, st.scratch_has_recall
                );
            }
        }
        "turn-start" => {
            platform.turn_start().await;
            eprintln!(
                "turn-start: scratch cleared (session {})",
                platform.session_id()
            );
        }
        "chain" => {
            let fact_id = subargs
                .get(1)
                .filter(|s| !s.starts_with("--"))
                .ok_or_else(|| anyhow::anyhow!("missing fact-id"))?;
            let vault = embeddings::open_vault_with_embeddings(
                &config.memory.vault_db,
                &config.embeddings,
                &config.redis,
                &config.rerank,
                &config.qdrant,
            )
            .await?;
            let chain = vault.get_memory_chain(fact_id)?;
            if chain.is_empty() {
                println!("(no honeypot chain for id {fact_id})");
            } else {
                for (i, (content, is_latest, graph_rel)) in chain.iter().enumerate() {
                    let tag = if *is_latest { "latest" } else { "superseded" };
                    let rel = graph_rel.as_deref().unwrap_or("-");
                    println!("[{i}] ({tag}, rel={rel}) {content}");
                }
            }
        }
        _ => {
            eprintln!("{USAGE}");
            bail!("unknown memory subcommand: {sub}");
        }
    }

    Ok(())
}
