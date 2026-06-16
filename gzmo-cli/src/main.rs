//! # GZMO — Sovereign Agent
//!
//! Thin binary shell. All logic lives in `gzmo-core`.

mod chat;
mod chaos_bootstrap;
mod cli_mcp;
mod pedagogy_bridge;
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
mod chaos_skill_cmd;
mod init_cmd;
mod ingest_eval_cmd;
mod wiki_cmd;
mod honeypot_cmd;
mod pedagogy_graph_cmd;
mod kurator_cmd;
mod obolus_cmd;
mod mentor_ipc;
mod low_tension_dialogue;
mod mentor_cmd;
mod mentor_compute_cmd;
mod mentor_plot_cmd;
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
    /// Distill a Pi agent session JSONL (`gzmo distill pi <path>`).
    DistillPi { path: std::path::PathBuf, start_turn: usize, max_turns: Option<usize> },
    Health,
    Profile(Vec<String>),
    McpServe,
    /// Knowledge Gardener ops over the wiki/ layer (sync|lint|search|file-back|status).
    Wiki(Vec<String>),
    /// Run a chaos slash skill with daemon IPC (`gzmo chaos skill joke`).
    ChaosSkill { cmd: String, args: String, json: bool },
    ChaosFeedbackAudit { tail: usize },
    /// Pedagogy tooling (`gzmo pedagogy graph validate <path>`).
    Pedagogy(Vec<String>),
    /// Honeypot reject log (`gzmo honeypot rejects`).
    Honeypot(Vec<String>),
    /// Headless mentor API client (`gzmo mentor teach <message>`).
    Mentor(Vec<String>),
    /// Kurator monitor status (`gzmo kurator status`).
    Kurator(Vec<String>),
    /// Obolus token analytics (`gzmo obolus status|report|context`).
    Obolus(Vec<String>),
    Help,
}

fn print_cli_help() {
    eprintln!(
        "\
GZMO — Sovereign Agent CLI

COGNITION (Prime)
  Local LLM default: http://localhost:8000/v1  ([engine.local] in gzmo.toml)
  Legacy LM Studio:  http://localhost:1234/v1  (optional; not the GZMO default)

PLATFORM
  Retrieval/embed:   http://192.168.31.110:8081  (VM200)
  Neo4j / Qdrant:    192.168.31.202

USAGE
  gzmo                          Interactive chat (mentor-first)
  gzmo --repl                   Legacy TUI
  gzmo daemon                   Background daemon + mentor socket
  gzmo health                   Probe Prime, embed, vault, graph, MCP
  gzmo mentor ping|status|teach Headless Socratic API (data/gzmo_mentor.sock)
  gzmo dream [YYYY-MM-DD]       DreamEngine consolidation
  gzmo spark [YYYY-MM-DD]       SparkEngine serendipity
  gzmo memory <sub>             Platform memory bridge
  gzmo chaos skill <cmd> [args] Chaos slash skills (Rust registry)
  gzmo chaos skill dice --json     Structured evidence JSON (Pi probes)
  gzmo wiki <action>            Knowledge Gardener (wiki/ layer)
  gzmo pedagogy graph validate  Prerequisite graph lint
  gzmo honeypot rejects         Honeypot promotion reject log
  gzmo honeypot review list     Pending honeypot review queue
  gzmo honeypot review promote  Operator promote from review queue
  gzmo distill [session_id]     Distill GZMO chat sessions → vault
  gzmo distill pi <path.jsonl>  Distill Pi session on session_end
  gzmo obolus status|report|balance  Prime token ledger (E_total, ctx_%)
  gzmo obolus preflight <action>   Gate check (discovery_cycle, spawn_*)
  gzmo obolus efficiency        Wirkungsgrad η = (Q·I)/E_total
  gzmo init                     First-time setup
  gzmo mcp-serve                MCP stdio (memory + wiki search)

FLAGS
  --learner <id>                Learner profile (default: operator or GZMO_LEARNER_ID)

Config: gzmo.toml in repo root (or GZMO_CONFIG). Build: target/release/gzmo
"
    );
}

/// Strip global flags (`--learner <id>`) before subcommand parsing.
fn strip_global_flags(args: &[String]) -> (Option<String>, Vec<String>) {
    let mut learner = None;
    let mut out = vec![args[0].clone()];
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--learner" {
            if let Some(id) = args.get(i + 1) {
                learner = Some(id.clone());
                i += 2;
                continue;
            }
        }
        out.push(args[i].clone());
        i += 1;
    }
    (learner, out)
}

fn parse_args() -> (Option<String>, Command) {
    let raw: Vec<String> = std::env::args().collect();
    let (learner, args) = strip_global_flags(&raw);
    if args.len() >= 2 {
        if args[1] == "--help" || args[1] == "-h" || args[1] == "help" {
            return (learner, Command::Help);
        }
        if args[1] == "daemon" { return (learner, Command::Daemon); }
        if args[1] == "init" { return (learner, Command::Init); }
        if args[1] == "--repl" { return (learner, Command::ChatRepl); }
        if args[1] == "pedagogy" { return (learner, Command::Pedagogy(args[2..].to_vec())); }
        if args[1] == "honeypot" { return (learner, Command::Honeypot(args[2..].to_vec())); }
        if args[1] == "mentor" { return (learner, Command::Mentor(args[2..].to_vec())); }
        if args[1] == "kurator" { return (learner, Command::Kurator(args[2..].to_vec())); }
        if args[1] == "obolus" { return (learner, Command::Obolus(args[2..].to_vec())); }
        if args[1] == "dream" {
            let date = args.get(2).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            return (learner, Command::Dream(date));
        }
        if args[1] == "spark" {
            let date = args.get(2).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            return (learner, Command::Spark(date));
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
            return (
                learner,
                Command::Ingest {
                    path: path.unwrap_or_else(|| std::path::PathBuf::from(".")),
                    dry_run,
                },
            );
        }
        if args[1] == "ingest-dir" {
            let Some(path_arg) = args.get(2) else {
                eprintln!("Usage: gzmo ingest-dir <directory>");
                std::process::exit(1);
            };
            return (learner, Command::IngestDir(std::path::PathBuf::from(path_arg)));
        }
        if args[1] == "ingest-eval" {
            let Some(path_arg) = args.get(2) else {
                eprintln!("Usage: gzmo ingest-eval <file_or_directory>");
                std::process::exit(1);
            };
            return (learner, Command::IngestEval(std::path::PathBuf::from(path_arg)));
        }
        if args[1] == "memory" {
            if args.get(2).map(|s| s.as_str()) == Some("dump") {
                return (learner, Command::MemoryDump);
            }
            if args.get(2).map(|s| s.as_str()) == Some("embed") {
                let limit = args.get(3).and_then(|s| s.parse().ok());
                return (learner, Command::MemoryEmbed(limit));
            }
            return (learner, Command::Memory(args[2..].to_vec()));
        }
        if args[1] == "dump" { return (learner, Command::MemoryDump); }
        if args[1] == "distill" {
            if args.get(2).map(|s| s.as_str()) == Some("pi") {
                let Some(path) = args.get(3) else {
                    eprintln!("Usage: gzmo distill pi <session.jsonl> [--from-turn <N>] [--max-turns <N>]");
                    std::process::exit(1);
                };
                let mut start_turn = 0;
                let mut max_turns = None;
                let mut i = 4;
                while i < args.len() {
                    if args[i] == "--from-turn" {
                        if let Some(val) = args.get(i + 1) {
                            if let Ok(n) = val.parse::<usize>() {
                                start_turn = n;
                            }
                            i += 2;
                            continue;
                        }
                    } else if args[i] == "--max-turns" {
                        if let Some(val) = args.get(i + 1) {
                            if let Ok(n) = val.parse::<usize>() {
                                max_turns = Some(n);
                            }
                            i += 2;
                            continue;
                        }
                    }
                    i += 1;
                }
                return (learner, Command::DistillPi {
                    path: std::path::PathBuf::from(path),
                    start_turn,
                    max_turns,
                });
            }
            let id = args.get(2).cloned();
            return (learner, Command::Distill(id));
        }
        if args[1] == "health" { return (learner, Command::Health); }
        if args[1] == "wiki" { return (learner, Command::Wiki(args[2..].to_vec())); }
        if args[1] == "chaos" {
            if args.get(2).map(|s| s.as_str()) == Some("skill") {
                let cmd = args.get(3).cloned().unwrap_or_else(|| "help".to_string());
                let mut json = false;
                let mut skill_args_parts: Vec<&str> = Vec::new();
                for a in args.iter().skip(4) {
                    if a == "--json" {
                        json = true;
                    } else {
                        skill_args_parts.push(a.as_str());
                    }
                }
                let skill_args = skill_args_parts.join(" ");
                return (learner, Command::ChaosSkill { cmd, args: skill_args, json });
            }
            if args.get(2).map(|s| s.as_str()) == Some("feedback-audit") {
                let mut tail = 20;
                let mut i = 3;
                while i < args.len() {
                    if args[i] == "--tail" {
                        if let Some(val) = args.get(i + 1) {
                            if let Ok(n) = val.parse::<usize>() {
                                tail = n;
                            }
                            i += 2;
                            continue;
                        }
                    }
                    i += 1;
                }
                return (learner, Command::ChaosFeedbackAudit { tail });
            }
        }
        if args[1] == "mcp-serve" { return (learner, Command::McpServe); }
        if args[1] == "profile" {
            return (learner, Command::Profile(args[2..].to_vec()));
        }
    }
    (learner, Command::Chat)
}

#[tokio::main]
async fn main() -> Result<()> {
    let (learner_flag, command) = parse_args();

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
        Command::Distill(_) | Command::DistillPi { .. } => "info",
        Command::Health => "warn",
        Command::Profile(_) => "warn",
        Command::McpServe => "warn",
        Command::Wiki(_) => "info",
        Command::ChaosSkill { .. } => "warn",
        Command::ChaosFeedbackAudit { .. } => "warn",
        Command::Pedagogy(_) => "warn",
        Command::Honeypot(_) => "warn",
        Command::Mentor(_) => "warn",
        Command::Kurator(_) => "warn",
        Command::Obolus(_) => "warn",
        Command::Help => "warn",
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(default_filter)),
        )
        .with_target(false)
        .init();

    // Init / help don't need existing config
    if matches!(command, Command::Init) {
        return init_cmd::run().await;
    }
    if matches!(command, Command::Help) {
        print_cli_help();
        return Ok(());
    }

    let mut config = gzmo_core::config::GzmoConfig::load_auto()?;
    let learner_id =
        gzmo_core::config::PedagogyConfig::resolve_learner_id(learner_flag.as_deref());
    config.pedagogy.active_learner_id = Some(learner_id);
    let identity = gzmo_core::identity::IdentityEngine::boot(&config.identity.soul_path).await?;

    match command {
        Command::Chat => chat::run(&config, &identity).await,
        Command::ChatRepl => tui::runner::run(&config, &identity).await,
        Command::Daemon => {
            // OS-level singleton lock file (shared with start-production.sh / scripts)
            let pid_file = gzmo_core::daemon::daemon_pid_path();

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
        Command::DistillPi { path, start_turn, max_turns } => distill_cmd::run_pi(&config, &identity, &path, start_turn, max_turns).await,
        Command::Health => health_cmd::run(&config, identity).await,
        Command::Profile(args) => profile_cmd::run(&config, &args).await,
        Command::McpServe => mcp_serve_cmd::run(&config).await,
        Command::Wiki(args) => wiki_cmd::run(&config, args).await,
        Command::ChaosSkill { cmd, args, json } => chaos_skill_cmd::run(&config, &cmd, &args, json).await,
        Command::ChaosFeedbackAudit { tail } => {
            use std::io::BufRead;
            let data_dir = gzmo_core::skills::dispatch::data_dir(&config);
            let audit_file = data_dir.join("chaos_feedback_audit.jsonl");
            if !audit_file.exists() {
                println!("No audit trail found at {}", audit_file.display());
                return Ok(());
            }
            let file = std::fs::File::open(&audit_file)?;
            let reader = std::io::BufReader::new(file);
            let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
            let start = lines.len().saturating_sub(tail);
            for line in &lines[start..] {
                println!("{line}");
            }
            Ok(())
        }
        Command::Pedagogy(args) => pedagogy_graph_cmd::run(&args).await,
        Command::Honeypot(args) => honeypot_cmd::run(&config, &args).await,
        Command::Mentor(args) => mentor_cmd::run(&config, &args).await,
        Command::Kurator(args) => kurator_cmd::run(&args, &config).await,
        Command::Obolus(args) => obolus_cmd::run(&args, &config).await,
        Command::Help => unreachable!(),
    }
}
