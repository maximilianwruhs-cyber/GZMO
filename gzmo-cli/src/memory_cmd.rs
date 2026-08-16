//! CLI for frontend memory tools (`gzmo memory *`).

use std::sync::Arc;

use anyhow::{bail, Result};
use gzmo_core::config::GzmoConfig;
use gzmo_core::control_plane::ControlPlaneClient;
use gzmo_core::memory::embeddings;
use gzmo_core::platform_memory::{MemorySearchResult, MemoryStatusReport, PlatformMemory};

const USAGE: &str = "Usage:\n  \
     gzmo memory search <query> [--limit N] [--session ID] [--json] [--no-scratch] [--offline]\n  \
     gzmo memory recall [--session ID] [--offline]\n  \
     gzmo memory status [--session ID] [--json] [--offline]\n  \
     gzmo memory turn-start [--session ID] [--offline]\n  \
     gzmo memory chain <fact-id> [--offline]\n  \
     gzmo memory embed [limit]\n  \
     gzmo memory promote [limit]\n  \
     gzmo memory mcp\n  \
     \n\
     Session: --session or GZMO_SESSION_ID env.\n\
     Owner socket: used when `gzmo serve`/`daemon` is up; `--offline` or GZMO_CONTROL_PLANE=0 \
     forces in-process PlatformMemory.\n\
     MCP: `gzmo memory mcp` == `gzmo mcp-serve` (third surface, ADR-0003).\n\
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

fn wants_offline(args: &[String]) -> bool {
    args.iter().any(|a| a == "--offline")
}

fn print_search(res: &MemorySearchResult, session: &str, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(res)?);
    } else {
        println!("{}", res.text);
        if res.scratch_written {
            eprintln!("\n(scratch updated — session {session}; pull with `gzmo memory recall`)");
        }
    }
    Ok(())
}

fn print_status(st: &MemoryStatusReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(st)?);
    } else {
        let via = st.control_plane.as_deref().unwrap_or("in-process");
        println!(
            "session={} vault={} facts={} honeypot={} scratch={} has_recall={} via={}",
            st.session_id,
            st.vault_path,
            st.vault_facts,
            st.honeypot_latest,
            st.scratch_backend,
            st.scratch_has_recall,
            via
        );
    }
    Ok(())
}

fn print_chain(chain: &[(String, bool, Option<String>)], fact_id: &str) {
    if chain.is_empty() {
        println!("(no honeypot chain for id {fact_id})");
        return;
    }
    for (i, (content, is_latest, graph_rel)) in chain.iter().enumerate() {
        let tag = if *is_latest { "latest" } else { "superseded" };
        let rel = graph_rel.as_deref().unwrap_or("-");
        println!("[{i}] ({tag}, rel={rel}) {content}");
    }
}

pub async fn run(config: &GzmoConfig, subargs: Vec<String>) -> Result<()> {
    let Some(sub) = subargs.first().map(|s| s.as_str()) else {
        eprintln!("{USAGE}");
        bail!("missing memory subcommand");
    };

    let session_id = parse_session_flag(&subargs);
    if !wants_offline(&subargs) {
        if let Some(client) = ControlPlaneClient::connect_if_live(config, session_id.clone()).await {
            return run_via_owner(&client, sub, &subargs).await;
        }
    }

    let platform = Arc::new(PlatformMemory::open(config, session_id).await?);
    run_in_process(config, &platform, sub, &subargs).await
}

async fn run_via_owner(client: &ControlPlaneClient, sub: &str, subargs: &[String]) -> Result<()> {
    let session = client
        .session_id
        .clone()
        .unwrap_or_else(|| "(owner)".into());
    match sub {
        "search" => {
            let query = subargs
                .get(1)
                .filter(|s| !s.starts_with("--"))
                .ok_or_else(|| anyhow::anyhow!("missing query"))?;
            let limit = parse_limit(subargs, 5);
            let write_scratch = !subargs.iter().any(|a| a == "--no-scratch");
            let res = client.search(query, limit, write_scratch).await?;
            print_search(&res, &session, wants_json(subargs))
        }
        "recall" => match client.recall().await? {
            Some(block) => {
                println!("{block}");
                Ok(())
            }
            None => {
                println!("(no scratch recall for this session)");
                Ok(())
            }
        },
        "status" => print_status(&client.status().await?, wants_json(subargs)),
        "turn-start" => {
            eprintln!("{}", client.turn_start().await?);
            Ok(())
        }
        "chain" => {
            let fact_id = subargs
                .get(1)
                .filter(|s| !s.starts_with("--"))
                .ok_or_else(|| anyhow::anyhow!("missing fact-id"))?;
            print_chain(&client.chain(fact_id).await?, fact_id);
            Ok(())
        }
        _ => {
            eprintln!("{USAGE}");
            bail!("unknown memory subcommand: {sub}");
        }
    }
}

async fn run_in_process(
    config: &GzmoConfig,
    platform: &PlatformMemory,
    sub: &str,
    subargs: &[String],
) -> Result<()> {
    match sub {
        "search" => {
            let query = subargs
                .get(1)
                .filter(|s| !s.starts_with("--"))
                .ok_or_else(|| anyhow::anyhow!("missing query"))?;
            let limit = parse_limit(subargs, 5);
            let write_scratch = !subargs.iter().any(|a| a == "--no-scratch");
            let res = platform.memory_search(query, limit, write_scratch).await?;
            print_search(&res, platform.session_id(), wants_json(subargs))
        }
        "recall" => match platform.memory_recall_pull().await? {
            Some(block) => {
                println!("{block}");
                Ok(())
            }
            None => {
                println!("(no scratch recall for this session)");
                Ok(())
            }
        },
        "status" => print_status(&platform.status().await?, wants_json(subargs)),
        "turn-start" => {
            platform.turn_start().await;
            eprintln!(
                "turn-start: scratch cleared (session {})",
                platform.session_id()
            );
            Ok(())
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
            print_chain(&vault.get_memory_chain(fact_id)?, fact_id);
            Ok(())
        }
        _ => {
            eprintln!("{USAGE}");
            bail!("unknown memory subcommand: {sub}");
        }
    }
}
