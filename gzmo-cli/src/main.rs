//! # GZMO — Sovereign Agent
//!
//! Thin binary shell. All logic lives in `gzmo-core`.

mod chat;
mod cli_mcp;
mod daemon_cmd;
mod dream_cmd;
mod spark_cmd;
mod ingest_cmd;
mod ingest_dir_cmd;
mod health_cmd;
mod memory_cmd;
mod mcp_serve_cmd;
mod profile_cmd;
mod embed_cmd;
mod distill_cmd;
mod init_cmd;
mod ingest_eval_cmd;
mod wiki_cmd;
#[allow(dead_code)]
mod ui;
pub mod tui;

use anyhow::Result;
use chrono::NaiveDate;
use tracing_subscriber::EnvFilter;
use gzmo_core::memory::vault::SqliteVault;

enum Command {
    Chat,
    ChatRepl,  // Legacy REPL mode via --repl flag
    Daemon,
    /// One-shot dream consolidation for an optional date (default: today).
    Dream(Option<NaiveDate>),
    /// One-shot spark (serendipitous recall) for an optional date (default: today).
    Spark(Option<NaiveDate>),
    Ingest { path: std::path::PathBuf, dry_run: bool },
    IngestDir(std::path::PathBuf),
    IngestEval(std::path::PathBuf),
    Init,
    MemoryDump,
    MemoryEmbed(Option<usize>),
    Memory(Vec<String>),
    Distill(Option<String>),
    Health,
    Profile(Vec<String>),
    McpServe,
    /// Knowledge Gardener ops over the wiki/ layer (sync|lint|search|file-back|status).
    Wiki(Vec<String>),
}

fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        if args[1] == "daemon" { return Command::Daemon; }
        if args[1] == "init" { return Command::Init; }
        if args[1] == "--repl" { return Command::ChatRepl; }
        if args[1] == "dream" {
            let date = args.get(2).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            return Command::Dream(date);
        }
        if args[1] == "spark" {
            let date = args.get(2).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            return Command::Spark(date);
        }
        if args[1] == "ingest" {
            let mut dry_run = false;
            let mut path: Option<std::path::PathBuf> = None;
            for a in args.iter().skip(2) {
                if a == "--dry-run" {
                    dry_run = true;
                } else if path.is_none() {
                    path = Some(std::path::PathBuf::from(a));
                }
            }
            return Command::Ingest {
                path: path.unwrap_or_else(|| std::path::PathBuf::from(".")),
                dry_run,
            };
        }
        if args[1] == "ingest-dir" {
            let Some(path_arg) = args.get(2) else {
                eprintln!("Usage: gzmo ingest-dir <directory>");
                std::process::exit(1);
            };
            return Command::IngestDir(std::path::PathBuf::from(path_arg));
        }
        if args[1] == "ingest-eval" {
            let Some(path_arg) = args.get(2) else {
                eprintln!("Usage: gzmo ingest-eval <file_or_directory>");
                std::process::exit(1);
            };
            return Command::IngestEval(std::path::PathBuf::from(path_arg));
        }
        if args[1] == "memory" {
            if args.get(2).map(|s| s.as_str()) == Some("dump") {
                return Command::MemoryDump;
            }
            if args.get(2).map(|s| s.as_str()) == Some("embed") {
                let limit = args.get(3).and_then(|s| s.parse().ok());
                return Command::MemoryEmbed(limit);
            }
            return Command::Memory(args[2..].to_vec());
        }
        if args[1] == "dump" { return Command::MemoryDump; }
        if args[1] == "distill" {
            let id = args.get(2).cloned();
            return Command::Distill(id);
        }
        if args[1] == "health" { return Command::Health; }
        if args[1] == "wiki" { return Command::Wiki(args[2..].to_vec()); }
        if args[1] == "mcp-serve" { return Command::McpServe; }
        if args[1] == "profile" {
            return Command::Profile(args[2..].to_vec());
        }
    }
    Command::Chat
}

#[tokio::main]
async fn main() -> Result<()> {
    let command = parse_args();

    let default_filter = match command {
        Command::Chat | Command::ChatRepl => "warn",
        Command::Daemon => "info",
        Command::Dream(_) => "info",
        Command::Spark(_) => "info",
        Command::Ingest { .. } => "info",
        Command::IngestDir(_) => "info",
        Command::IngestEval(_) => "info",
        Command::Init => "warn",
        Command::MemoryDump => "info",
        Command::MemoryEmbed(_) => "info",
        Command::Memory(_) => "warn",
        Command::Distill(_) => "info",
        Command::Health => "warn",
        Command::Profile(_) => "warn",
        Command::McpServe => "warn",
        Command::Wiki(_) => "info",
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(default_filter)),
        )
        .with_target(false)
        .init();

    // Init doesn't need existing config
    if matches!(command, Command::Init) {
        return init_cmd::run().await;
    }

    let config = gzmo_core::config::GzmoConfig::load_auto()?;
    let identity = gzmo_core::identity::IdentityEngine::boot(&config.identity.soul_path).await?;

    match command {
        Command::Chat => chat::run(&config, &identity).await,
        Command::ChatRepl => tui::runner::run(&config, &identity).await,
        Command::Daemon => {
            // OS-level singleton lock file
            let pid_file = std::path::PathBuf::from("/tmp/gzmo_rust.pid");

            if pid_file.exists() {
                if let Ok(old_pid_str) = std::fs::read_to_string(&pid_file) {
                    let old_pid = old_pid_str.trim();
                    let proc_path = format!("/proc/{}/cmdline", old_pid);
                    if std::path::Path::new(&proc_path).exists() {
                        anyhow::bail!(
                            "GZMO Daemon is already running (PID {}, lockfile {:?}).",
                            old_pid, pid_file
                        );
                    }
                    tracing::warn!(stale_pid = %old_pid, "Reclaiming stale PID lockfile");
                    let _ = std::fs::remove_file(&pid_file);
                }
            }

            use std::io::Write;
            let mut lock = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&pid_file)
                .map_err(|e| anyhow::anyhow!(
                    "Failed to acquire PID lockfile {:?}: {}. Another instance may have started.",
                    pid_file, e
                ))?;
            write!(lock, "{}", std::process::id())?;
            drop(lock);

            let res = daemon_cmd::run(&config, identity).await;
            let _ = std::fs::remove_file(&pid_file);
            res
        },
        Command::Dream(date) => dream_cmd::run(&config, identity, date).await,
        Command::Spark(date) => spark_cmd::run(&config, identity, date).await,
        Command::Ingest { path, dry_run } => ingest_cmd::run(&config, identity, path, dry_run).await,
        Command::IngestDir(path) => ingest_dir_cmd::run(&config, identity, path).await,
        Command::IngestEval(path) => ingest_eval_cmd::run(&config, identity, path).await,
        Command::Init => unreachable!(),
        Command::MemoryDump => {
            println!("Exporting Native Vault to Markdown...");
            let vault = SqliteVault::open(&config.memory.vault_db)?;
            vault.dump_to_markdown(&config.memory.directory).await?;
            Ok(())
        }
        Command::MemoryEmbed(limit) => embed_cmd::run(&config, &identity, limit).await,
        Command::Memory(sub) => memory_cmd::run(&config, sub).await,
        Command::Distill(session_id) => distill_cmd::run(&config, &identity, session_id).await,
        Command::Health => health_cmd::run(&config, identity).await,
        Command::Profile(args) => profile_cmd::run(&config, &args).await,
        Command::McpServe => mcp_serve_cmd::run(&config).await,
        Command::Wiki(args) => wiki_cmd::run(&config, args).await,
    }
}
