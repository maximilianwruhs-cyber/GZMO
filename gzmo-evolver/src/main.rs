//! Thin CLI over the `gzmo-evolver` library.
//!
//! Only real commands are exposed. Future lifecycle commands land with their
//! implementations, never as placeholders.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use gzmo_evolver::RepoEvolverConfig;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "gzmo-evolver",
    about = "Connected-host repository evolver coordinator",
    disable_help_subcommand = true
)]
struct Cli {
    /// Absolute path to the machine-local placement configuration.
    #[arg(long, value_name = "ABSOLUTE_PATH")]
    config: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Load and validate machine config plus the working-tree trusted policy.
    ConfigCheck {
        /// Emit machine-readable JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
struct ConfigCheckReport<'a> {
    schema: &'static str,
    repo_path: String,
    state_dir: String,
    worker_executable: String,
    worker_profile: &'a str,
    owner: &'a str,
    repository: &'a str,
    policy_repo_path: String,
    policy_digest: &'a str,
    budget: BudgetReport,
    protected_path_count: usize,
    required_hard_floors: Vec<&'a str>,
    mission_json_rel: String,
    mission_markdown_rel: String,
    refresh_argv: &'a [String],
}

#[derive(Debug, Serialize)]
struct BudgetReport {
    wall_seconds: u64,
    max_attempts: u8,
    max_changed_files: u32,
    max_added_lines: u32,
    max_tool_calls: u32,
    max_input_tokens: u64,
    max_output_tokens: u64,
    max_energy_joules: Option<u64>,
    allow_missing_energy_meter: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    require_absolute_config(&cli.config)?;

    match cli.command {
        Command::ConfigCheck { json } => {
            let cfg = RepoEvolverConfig::load(&cli.config)
                .with_context(|| format!("loading config {}", cli.config.display()))?;
            let report = build_report(&cfg);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_human(&report);
            }
            Ok(())
        }
    }
}

fn require_absolute_config(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("--config must be an absolute path, got {}", path.display());
    }
    Ok(())
}

fn build_report(cfg: &RepoEvolverConfig) -> ConfigCheckReport<'_> {
    let policy = cfg.working_policy();
    let budget = policy.budget();
    ConfigCheckReport {
        schema: "gzmo.repo_evolver.config_check/v1",
        repo_path: cfg.repo().path().display().to_string(),
        state_dir: cfg.state_dir().display().to_string(),
        worker_executable: cfg.worker().executable().display().to_string(),
        worker_profile: cfg.worker().profile(),
        owner: cfg.repo().owner(),
        repository: cfg.repo().repository(),
        policy_repo_path: cfg.policy().repo_path().display().to_string(),
        policy_digest: cfg.working_policy_digest(),
        budget: BudgetReport {
            wall_seconds: budget.wall_seconds,
            max_attempts: budget.max_attempts,
            max_changed_files: budget.max_changed_files,
            max_added_lines: budget.max_added_lines,
            max_tool_calls: budget.max_tool_calls,
            max_input_tokens: budget.max_input_tokens,
            max_output_tokens: budget.max_output_tokens,
            max_energy_joules: budget.max_energy_joules,
            allow_missing_energy_meter: budget.allow_missing_energy_meter,
        },
        protected_path_count: policy.protected_paths().protected_paths.len(),
        required_hard_floors: policy.required_hard_floor_names(),
        mission_json_rel: cfg.mission().json_rel().display().to_string(),
        mission_markdown_rel: cfg.mission().markdown_rel().display().to_string(),
        refresh_argv: cfg.mission().refresh_argv(),
    }
}

fn print_human(report: &ConfigCheckReport<'_>) {
    println!("config-check: ok");
    println!("  repo_path: {}", report.repo_path);
    println!("  state_dir: {}", report.state_dir);
    println!("  worker_executable: {}", report.worker_executable);
    println!("  worker_profile: {}", report.worker_profile);
    println!("  owner/repository: {}/{}", report.owner, report.repository);
    println!("  policy_repo_path: {}", report.policy_repo_path);
    println!("  policy_digest: {}", report.policy_digest);
    println!(
        "  budget: wall_seconds={} max_attempts={} files={} lines={} tools={} in_tokens={} out_tokens={} energy={:?} allow_missing_energy_meter={}",
        report.budget.wall_seconds,
        report.budget.max_attempts,
        report.budget.max_changed_files,
        report.budget.max_added_lines,
        report.budget.max_tool_calls,
        report.budget.max_input_tokens,
        report.budget.max_output_tokens,
        report.budget.max_energy_joules,
        report.budget.allow_missing_energy_meter
    );
    println!("  protected_path_count: {}", report.protected_path_count);
    println!(
        "  required_hard_floors: {}",
        report.required_hard_floors.join(", ")
    );
    println!("  mission_json_rel: {}", report.mission_json_rel);
    println!("  mission_markdown_rel: {}", report.mission_markdown_rel);
    println!("  refresh_argv: {:?}", report.refresh_argv);
}
