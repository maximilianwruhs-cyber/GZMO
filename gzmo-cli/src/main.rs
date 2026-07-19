//! # GZMO — Sovereign Agent
//!
//! Thin binary shell. All logic lives in `gzmo-core`.

mod assemble_cmd;
mod chaos_bootstrap;
mod chaos_skill_cmd;
mod chat;
mod cli_mcp;
mod config_cmd;
mod cron_cmd;
mod daemon_cmd;
mod distill_cmd;
mod dream_cmd;
mod embed_cmd;
mod health_cmd;
mod ingest_cmd;
mod ingest_dir_cmd;
mod ingest_eval_cmd;
mod init_cmd;
mod instance_cmd;
mod kg_reconcile_cmd;
mod mcp_serve_cmd;
mod memory_cmd;
mod mentor_cmd;
mod mentor_ipc;
mod metabolism_cmd;
mod observatory_cmd;
mod pedagogy_bridge;
mod profile_cmd;
mod promote_cmd;
mod repl_shared;
mod serve_cmd;
mod session_cmd;
mod spark_cmd;
mod status_cmd;
pub mod tui;
#[allow(dead_code)]
mod ui;
mod wiki_cmd;

use anyhow::Result;
use chrono::NaiveDate;
use gzmo_core::memory::vault::SqliteVault;
use tracing_subscriber::EnvFilter;

enum Command {
    Chat,
    ChatRepl, // Legacy REPL mode via --repl flag
    Daemon,
    /// Thin overnight metabolism runner (ADR-0003).
    Serve,
    /// One-shot dream consolidation for an optional date (default: today).
    Dream(Option<NaiveDate>),
    /// Compact oversized DREAMS.md (+ optional cold session archive).
    DreamCompact {
        max_chars: Option<usize>,
        archive_sessions_days: Option<i64>,
        dry_run: bool,
    },
    /// One-shot spark (serendipitous recall) for an optional date (default: today).
    Spark(Option<NaiveDate>),
    Ingest {
        path: std::path::PathBuf,
        dry_run: bool,
    },
    IngestDir(std::path::PathBuf),
    IngestEval(std::path::PathBuf),
    Init,
    MemoryDump,
    MemoryEmbed(Option<usize>),
    MemoryPromote(Option<usize>),
    Memory(Vec<String>),
    Distill(Option<String>),
    /// Session ops (`close` takeaway ritual → distill queue).
    Session(Vec<String>),
    Health,
    Status,
    /// Ecosystem health LED board (TUI Observatory slice).
    Observatory,
    /// Overnight metabolism job board (TUI) or `watchdog` JSON probe.
    Metabolism(Vec<String>),
    /// Cron wizard — builtins + custom jobs for `gzmo serve`.
    Cron(Vec<String>),
    Profile(Vec<String>),
    Instance(Vec<String>),
    Config(Vec<String>),
    McpServe,
    /// Knowledge Gardener ops over the wiki/ layer (sync|lint|search|file-back|status|push).
    Wiki(Vec<String>),
    /// One-shot Neo4j ontology reconcile via MCP.
    KgReconcile(Vec<String>),
    /// Run a Little Tools Lab assembly recipe (shells out to little-tools-lab/scripts).
    Assemble {
        recipe: String,
        fixture: bool,
        apply: bool,
    },
    /// Headless mentor API client (`gzmo mentor ping|status|teach`).
    Mentor(Vec<String>),
    /// One-shot ritual/lab pantheon skill (`gzmo chaos skill <command>`).
    ChaosSkill(Vec<String>),
}

fn parse_args() -> Command {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        if args[1] == "daemon" {
            return Command::Daemon;
        }
        if args[1] == "mentor" {
            return Command::Mentor(args.get(2..).unwrap_or(&[]).to_vec());
        }
        if args[1] == "chaos" && args.get(2).map(String::as_str) == Some("skill") {
            return Command::ChaosSkill(args.get(3..).unwrap_or(&[]).to_vec());
        }
        if args[1] == "serve" {
            return Command::Serve;
        }
        if args[1] == "init" {
            return Command::Init;
        }
        if args[1] == "--repl" {
            return Command::ChatRepl;
        }
        if args[1] == "dream" {
            if args.get(2).map(|s| s.as_str()) == Some("compact") {
                let mut max_chars = None;
                let mut archive_sessions_days = None;
                let mut dry_run = false;
                let mut i = 3;
                while i < args.len() {
                    match args[i].as_str() {
                        "--dry-run" => {
                            dry_run = true;
                            i += 1;
                        }
                        "--max-chars" => {
                            max_chars = args.get(i + 1).and_then(|s| s.parse().ok());
                            i += 2;
                        }
                        "--archive-sessions-days" => {
                            archive_sessions_days = args.get(i + 1).and_then(|s| s.parse().ok());
                            i += 2;
                        }
                        other => {
                            eprintln!("Unknown dream compact arg: {other}");
                            std::process::exit(2);
                        }
                    }
                }
                return Command::DreamCompact {
                    max_chars,
                    archive_sessions_days,
                    dry_run,
                };
            }
            let date = args
                .get(2)
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            return Command::Dream(date);
        }
        if args[1] == "spark" {
            let date = args
                .get(2)
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
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
            if args.get(2).map(|s| s.as_str()) == Some("promote") {
                let limit = args.get(3).and_then(|s| s.parse().ok());
                return Command::MemoryPromote(limit);
            }
            if args.get(2).map(|s| s.as_str()) == Some("mcp") {
                return Command::McpServe;
            }
            return Command::Memory(args[2..].to_vec());
        }
        if args[1] == "dump" {
            return Command::MemoryDump;
        }
        if args[1] == "distill" {
            let id = args.get(2).cloned();
            return Command::Distill(id);
        }
        if args[1] == "session" {
            return Command::Session(args[2..].to_vec());
        }
        if args[1] == "health" {
            return Command::Health;
        }
        if args[1] == "status" {
            return Command::Status;
        }
        if args[1] == "observatory" {
            return Command::Observatory;
        }
        if args[1] == "metabolism" {
            return Command::Metabolism(args[2..].to_vec());
        }
        if args[1] == "cron" {
            return Command::Cron(args[2..].to_vec());
        }
        if args[1] == "instance" {
            return Command::Instance(args[2..].to_vec());
        }
        if args[1] == "config" {
            return Command::Config(args[2..].to_vec());
        }
        if args[1] == "wiki" {
            return Command::Wiki(args[2..].to_vec());
        }
        if args[1] == "kg-reconcile" {
            return Command::KgReconcile(args[2..].to_vec());
        }
        if args[1] == "mcp-serve" {
            return Command::McpServe;
        }
        if args[1] == "profile" {
            return Command::Profile(args[2..].to_vec());
        }
        if args[1] == "assemble" {
            let recipe = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| "cognition".to_string());
            let fixture = !args.iter().any(|a| a == "--live");
            let apply = args.iter().any(|a| a == "--apply");
            return Command::Assemble {
                recipe,
                fixture,
                apply,
            };
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
        Command::Serve => "info",
        Command::Dream(_) => "info",
        Command::DreamCompact { .. } => "info",
        Command::Spark(_) => "info",
        Command::Ingest { .. } => "info",
        Command::IngestDir(_) => "info",
        Command::IngestEval(_) => "info",
        Command::Init => "warn",
        Command::MemoryDump => "info",
        Command::MemoryEmbed(_) => "info",
        Command::MemoryPromote(_) => "info",
        Command::Memory(_) => "warn",
        Command::Distill(_) => "info",
        Command::Session(_) => "info",
        Command::Health => "warn",
        Command::Status => "warn",
        Command::Observatory => "warn",
        Command::Metabolism(_) => "warn",
        Command::Cron(_) => "warn",
        Command::Instance(_) => "warn",
        Command::Config(_) => "warn",
        Command::Profile(_) => "warn",
        Command::McpServe => "warn",
        Command::Wiki(_) => "info",
        Command::KgReconcile(_) => "info",
        Command::Assemble { .. } => "info",
        Command::Mentor(_) => "warn",
        Command::ChaosSkill(_) => "warn",
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter)),
        )
        .with_target(false)
        .init();

    // Init doesn't need existing config
    if matches!(command, Command::Init) {
        return init_cmd::run().await;
    }

    let config = gzmo_core::config::GzmoConfig::load_auto()?;

    // Memory/health/MCP/status probes must not depend on SOUL.md (wrong CWD / release path).
    let needs_identity = !matches!(
        command,
        Command::Health
            | Command::Status
            | Command::Memory(_)
            | Command::MemoryEmbed(_)
            | Command::MemoryPromote(_)
            | Command::Distill(_)
            | Command::McpServe
            | Command::Observatory
            | Command::Metabolism(_)
            | Command::MemoryDump
            | Command::Assemble { .. }
            | Command::Instance(_)
            | Command::Config(_)
            | Command::Profile(_)
            | Command::Mentor(_)
            | Command::ChaosSkill(_)
            | Command::DreamCompact { .. }
            | Command::Session(_)
    );

    let identity = if needs_identity {
        Some(gzmo_core::identity::IdentityEngine::boot(&config.identity.soul_path).await?)
    } else {
        None
    };

    match command {
        Command::Chat => chat::run(&config, identity.as_ref().unwrap()).await,
        Command::ChatRepl => tui::runner::run(&config, identity.as_ref().unwrap()).await,
        Command::Serve => serve_cmd::run(&config, identity.as_ref().unwrap()).await,
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
                            old_pid,
                            pid_file
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
                .map_err(|e| {
                    anyhow::anyhow!(
                    "Failed to acquire PID lockfile {:?}: {}. Another instance may have started.",
                    pid_file, e
                )
                })?;
            write!(lock, "{}", std::process::id())?;
            drop(lock);

            let res = daemon_cmd::run(&config, identity.expect("daemon needs identity")).await;
            let _ = std::fs::remove_file(&pid_file);
            res
        }
        Command::Dream(date) => dream_cmd::run(&config, identity.as_ref().unwrap(), date).await,
        Command::DreamCompact {
            max_chars,
            archive_sessions_days,
            dry_run,
        } => dream_cmd::run_compact(&config, max_chars, archive_sessions_days, dry_run).await,
        Command::Spark(date) => spark_cmd::run(&config, identity.as_ref().unwrap(), date).await,
        Command::Ingest { path, dry_run } => {
            ingest_cmd::run(
                &config,
                identity.expect("ingest needs identity"),
                path,
                dry_run,
            )
            .await
        }
        Command::IngestDir(path) => {
            ingest_dir_cmd::run(&config, identity.expect("ingest-dir needs identity"), path).await
        }
        Command::IngestEval(path) => {
            ingest_eval_cmd::run(&config, identity.expect("ingest-eval needs identity"), path).await
        }
        Command::Init => unreachable!(),
        Command::MemoryDump => {
            println!("Exporting Native Vault to Markdown...");
            let vault = SqliteVault::open(&config.memory.vault_db)?;
            vault.dump_to_markdown(&config.memory.directory).await?;
            Ok(())
        }
        Command::MemoryEmbed(limit) => embed_cmd::run(&config, limit).await,
        Command::MemoryPromote(limit) => promote_cmd::run(&config, limit).await,
        Command::Memory(sub) => memory_cmd::run(&config, sub).await,
        Command::Distill(session_id) => distill_cmd::run(&config, session_id).await,
        Command::Session(args) => session_cmd::run(&config, &args).await,
        Command::Health => health_cmd::run(&config).await,
        Command::Status => status_cmd::run(&config).await,
        Command::Observatory => observatory_cmd::run(&config).await,
        Command::Metabolism(args) => metabolism_cmd::run(&config, &args).await,
        Command::Cron(args) => cron_cmd::run(&config, identity.as_ref().unwrap(), &args).await,
        Command::Instance(args) => instance_cmd::run(&config, &args).await,
        Command::Config(args) => config_cmd::run(&config, &args).await,
        Command::Profile(args) => profile_cmd::run(&config, &args).await,
        Command::McpServe => mcp_serve_cmd::run(&config).await,
        Command::Wiki(args) => wiki_cmd::run(&config, args).await,
        Command::KgReconcile(args) => kg_reconcile_cmd::run(&config, args).await,
        Command::Assemble {
            recipe,
            fixture,
            apply,
        } => assemble_cmd::run(&config, &recipe, fixture, apply).await,
        Command::Mentor(args) => mentor_cmd::run(&config, &args).await,
        Command::ChaosSkill(args) => chaos_skill_cmd::run(&config, &args).await,
    }
}
