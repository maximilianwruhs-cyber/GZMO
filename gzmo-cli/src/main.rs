//! # GZMO — Sovereign Agent
//!
//! Thin binary shell. All logic lives in `gzmo-core`.

mod chat;
mod daemon_cmd;
mod init_cmd;
#[allow(dead_code)]
mod ui;
pub mod tui;

use anyhow::Result;
use tracing_subscriber::EnvFilter;
use gzmo_core::memory::vault::SqliteVault;

enum Command {
    Chat,
    ChatRepl,  // Legacy REPL mode via --repl flag
    Daemon,
    Init,
    MemoryDump,
}

fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        if args[1] == "daemon" { return Command::Daemon; }
        if args[1] == "init" { return Command::Init; }
        if args[1] == "--repl" { return Command::ChatRepl; }
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
        Command::Chat | Command::ChatRepl => "warn",
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
        Command::Init => unreachable!(),
        Command::MemoryDump => {
            println!("Exporting Native Vault to Markdown...");
            let vault = SqliteVault::open(&config.memory.vault_db)?;
            vault.dump_to_markdown(&config.memory.directory).await?;
            Ok(())
        }
    }
}
