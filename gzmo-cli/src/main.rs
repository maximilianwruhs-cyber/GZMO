//! # GZMO — Sovereign Agent
//!
//! Thin binary shell. All logic lives in `gzmo-core`.

mod chat;
mod daemon_cmd;
mod init_cmd;
#[allow(dead_code)]
mod ui;

use anyhow::Result;
use tracing_subscriber::EnvFilter;
use gzmo_core::memory::vault::SqliteVault;

enum Command {
    Chat,
    Daemon,
    Init,
    MemoryDump,
}

fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        if args[1] == "daemon" { return Command::Daemon; }
        if args[1] == "init" { return Command::Init; }
        if args[1] == "memory" && args.get(2).map(|s| s.as_str()) == Some("dump") {
            return Command::MemoryDump;
        }
        if args[1] == "dump" { return Command::MemoryDump; }
    }
    Command::Chat
}

#[tokio::main]
async fn main() -> Result<()> {
    let command = parse_args();

    let default_filter = match command {
        Command::Chat => "warn",
        Command::Daemon => "info",
        Command::Init => "warn",
        Command::MemoryDump => "info",
    };

    tracing_subscriber::fmt()
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
    let soul = identity.snapshot().await;

    match command {
        Command::Chat => chat::run(&config, &soul).await,
        Command::Daemon => {
            // Rust-level singleton lock file (defense in depth alongside boot.sh)
            let pid_file = std::path::PathBuf::from("/tmp/gzmo_rust.pid");
            if pid_file.exists() {
                anyhow::bail!("GZMO Daemon is already running (rust lockfile exists: {:?}). If stale, rm the file.", pid_file);
            }
            std::fs::write(&pid_file, std::process::id().to_string())?;

            // Execute daemon loop
            let res = daemon_cmd::run(&config, identity).await;

            // Cleanup lockfile
            let _ = std::fs::remove_file(&pid_file);
            res
        },
        Command::Init => unreachable!(),
        Command::MemoryDump => {
            println!("Exporting Native Vault to Markdown...");
            let vault = SqliteVault::open(&config.memory.vault_db)?;
            vault.dump_to_markdown(&config.memory.directory).await?;
            Ok(())
        }
    }
}
