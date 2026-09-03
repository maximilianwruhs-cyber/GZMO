//! Thin CLI over the `gzmo-evolver` library.
//!
//! Only real commands are exposed. Future lifecycle commands land with their
//! implementations, never as placeholders.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
#[cfg(unix)]
use gzmo_evolver::run_hidden_worker;
use gzmo_evolver::{
    prepare_candidate, refresh_baseline_before_mission, CandidateRecord, CoordinatorLock,
    MissionAdapter, RepoEvolverConfig, StateStore, SystemClock, SystemProcessRunner,
};
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
    ///
    /// Required for public coordinator commands. The hidden `worker` command
    /// never loads coordinator config and ignores this flag.
    #[arg(long, value_name = "ABSOLUTE_PATH")]
    config: Option<PathBuf>,

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
    /// Read-only coordinator status (never creates state or takes the lock).
    Status {
        /// Emit machine-readable JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
    /// Refresh the opportunity mission after verifying mirror baseline + policy.
    ///
    /// Refreshes the coordinator mirror under the mirror lock, requires the clean
    /// trusted checkout HEAD to equal the fetched baseline, and requires the
    /// working-tree policy digest to match the baseline policy — without opening
    /// the candidate database or acquiring the coordinator lease.
    Refresh {
        /// Emit machine-readable JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
    /// Prepare exactly one active candidate workspace (active-first, trust-first).
    Prepare {
        /// Emit machine-readable JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
    /// Hidden uncredentialed OMP worker (no coordinator config).
    #[command(hide = true)]
    Worker {
        /// Absolute sealed request.json path under the request root.
        #[arg(long, value_name = "ABSOLUTE_PATH")]
        request: PathBuf,
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

#[derive(Debug, Serialize)]
struct StatusReport {
    schema: &'static str,
    initialized: bool,
    state_dir: String,
    repository: String,
    active_candidate: Option<ActiveCandidateStatus>,
    audit_head: Option<AuditHeadStatus>,
}

#[derive(Debug, Serialize)]
struct ActiveCandidateStatus {
    id: String,
    state: String,
    mission_id: String,
    kind: String,
    policy_digest: String,
    manifest_digest: String,
    workspace: Option<String>,
    candidate_digest: Option<String>,
    receipt_digest: Option<String>,
    terminal_reason: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct AuditHeadStatus {
    sequence: u64,
    event_type: String,
    event_hash: String,
    candidate_id: Option<String>,
    occurred_at: String,
}

#[derive(Debug, Serialize)]
struct RefreshReport {
    schema: &'static str,
    bet_id: String,
    title: String,
    score: i64,
    ship_bar: bool,
    generated_at: String,
    content_digest: String,
    generation_id: String,
    mission_md: String,
    policy_digest: String,
}

#[derive(Debug, Serialize)]
struct PrepareReport {
    schema: &'static str,
    reused_active: bool,
    baseline: Option<String>,
    candidate: ActiveCandidateStatus,
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

    match cli.command {
        #[cfg(unix)]
        Command::Worker { request } => {
            if cli.config.is_some() {
                bail!("hidden worker must not be invoked with --config");
            }
            run_hidden_worker(&request).map_err(|err| anyhow::anyhow!("worker failed: {err}"))?;
            Ok(())
        }
        #[cfg(not(unix))]
        Command::Worker { .. } => {
            bail!("worker command is only supported on Unix");
        }
        Command::ConfigCheck { json } => {
            let config_path = require_cli_config(cli.config.as_deref())?;
            let cfg = RepoEvolverConfig::load(config_path)
                .with_context(|| format!("loading config {}", config_path.display()))?;
            let report = build_config_report(&cfg);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_config_human(&report);
            }
            Ok(())
        }
        Command::Status { json } => {
            let config_path = require_cli_config(cli.config.as_deref())?;
            let cfg = RepoEvolverConfig::load(config_path)
                .with_context(|| format!("loading config {}", config_path.display()))?;
            let report = build_status_report(&cfg)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_status_human(&report);
            }
            Ok(())
        }
        Command::Refresh { json } => {
            let config_path = require_cli_config(cli.config.as_deref())?;
            let cfg = RepoEvolverConfig::load(config_path)
                .with_context(|| format!("loading config {}", config_path.display()))?;
            let runner = SystemProcessRunner;
            // Trust-first: locked mirror refresh + HEAD/policy before producer.
            let _baseline = refresh_baseline_before_mission(&cfg, &runner)
                .context("verifying git baseline before refresh")?;
            let clock = SystemClock;
            let adapter = MissionAdapter::new(&cfg, &runner, &clock);
            let mission = adapter
                .refresh_and_load()
                .context("refreshing opportunity mission")?;
            let report = RefreshReport {
                schema: "gzmo.repo_evolver.refresh/v1",
                bet_id: mission.bet_id,
                title: mission.title,
                score: mission.score,
                ship_bar: mission.ship_bar,
                generated_at: mission
                    .generated_at
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                content_digest: mission.content_digest,
                generation_id: mission.generation_id,
                mission_md: mission.mission_md.display().to_string(),
                policy_digest: cfg.working_policy_digest().to_owned(),
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_refresh_human(&report);
            }
            Ok(())
        }
        Command::Prepare { json } => {
            let config_path = require_cli_config(cli.config.as_deref())?;
            let cfg = RepoEvolverConfig::load(config_path)
                .with_context(|| format!("loading config {}", config_path.display()))?;
            let _lock = CoordinatorLock::try_acquire(cfg.state_dir())
                .context("acquiring coordinator lock")?;
            let store =
                StateStore::open(cfg.state_dir()).context("opening coordinator state store")?;
            let runner = SystemProcessRunner;
            let clock = SystemClock;
            let outcome = prepare_candidate(&cfg, &runner, &clock, &store)
                .context("preparing candidate workspace")?;
            let report = PrepareReport {
                schema: "gzmo.repo_evolver.prepare/v1",
                reused_active: outcome.reused_active,
                baseline: outcome.baseline,
                candidate: active_status(&outcome.record),
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_prepare_human(&report);
            }
            Ok(())
        }
    }
}

fn require_cli_config(config: Option<&Path>) -> Result<&Path> {
    let path = config.ok_or_else(|| anyhow::anyhow!("--config is required for this command"))?;
    require_absolute_config(path)?;
    Ok(path)
}

fn require_absolute_config(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("--config must be an absolute path, got {}", path.display());
    }
    Ok(())
}

fn build_config_report(cfg: &RepoEvolverConfig) -> ConfigCheckReport<'_> {
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

fn build_status_report(cfg: &RepoEvolverConfig) -> Result<StatusReport> {
    let repository = format!("{}/{}", cfg.repo().owner(), cfg.repo().repository());
    let state_dir = cfg.state_dir().display().to_string();

    match StateStore::open_existing_readonly(cfg.state_dir())
        .with_context(|| format!("opening state dir {}", cfg.state_dir().display()))?
    {
        None => Ok(StatusReport {
            schema: "gzmo.repo_evolver.status/v1",
            initialized: false,
            state_dir,
            repository,
            active_candidate: None,
            audit_head: None,
        }),
        Some(store) => {
            let active = store
                .active_candidate(&repository)
                .context("loading active candidate")?;
            let head = store.audit_head().context("loading audit head")?;
            Ok(StatusReport {
                schema: "gzmo.repo_evolver.status/v1",
                initialized: true,
                state_dir,
                repository,
                active_candidate: active.as_ref().map(active_status),
                audit_head: head.map(|event| AuditHeadStatus {
                    sequence: event.sequence,
                    event_type: event.event_type,
                    event_hash: event.event_hash,
                    candidate_id: event.candidate_id.map(|id| id.to_string()),
                    occurred_at: event
                        .occurred_at
                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                }),
            })
        }
    }
}

fn active_status(record: &CandidateRecord) -> ActiveCandidateStatus {
    ActiveCandidateStatus {
        id: record.id().as_str().to_owned(),
        state: record.state().to_string(),
        mission_id: record.manifest().mission_id.clone(),
        kind: record.manifest().kind.to_string(),
        policy_digest: record.policy_digest().to_owned(),
        manifest_digest: record.manifest_digest().to_owned(),
        workspace: record.workspace().map(|path| path.display().to_string()),
        candidate_digest: record.candidate_digest().map(str::to_owned),
        receipt_digest: record.receipt_digest().map(str::to_owned),
        terminal_reason: record.terminal_reason().map(str::to_owned),
        created_at: record
            .created_at()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        updated_at: record
            .updated_at()
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    }
}

fn print_config_human(report: &ConfigCheckReport<'_>) {
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

fn print_status_human(report: &StatusReport) {
    println!("status: initialized={}", report.initialized);
    println!("  state_dir: {}", report.state_dir);
    println!("  repository: {}", report.repository);
    match &report.active_candidate {
        None => println!("  active_candidate: none"),
        Some(c) => {
            println!(
                "  active_candidate: id={} state={} mission_id={} kind={}",
                c.id, c.state, c.mission_id, c.kind
            );
            println!("    policy_digest: {}", c.policy_digest);
            println!("    manifest_digest: {}", c.manifest_digest);
            println!(
                "    workspace: {}",
                c.workspace.as_deref().unwrap_or("none")
            );
            println!(
                "    candidate_digest: {}",
                c.candidate_digest.as_deref().unwrap_or("none")
            );
            println!(
                "    receipt_digest: {}",
                c.receipt_digest.as_deref().unwrap_or("none")
            );
            println!(
                "    terminal_reason: {}",
                c.terminal_reason.as_deref().unwrap_or("none")
            );
            println!("    created_at: {}", c.created_at);
            println!("    updated_at: {}", c.updated_at);
        }
    }
    match &report.audit_head {
        None => println!("  audit_head: none"),
        Some(h) => {
            println!(
                "  audit_head: sequence={} event_type={} event_hash={}",
                h.sequence, h.event_type, h.event_hash
            );
            println!(
                "    candidate_id: {}",
                h.candidate_id.as_deref().unwrap_or("none")
            );
            println!("    occurred_at: {}", h.occurred_at);
        }
    }
}

fn print_refresh_human(report: &RefreshReport) {
    println!("refresh: ok");
    println!("  bet_id: {}", report.bet_id);
    println!("  title: {}", report.title);
    println!("  score: {}", report.score);
    println!("  ship_bar: {}", report.ship_bar);
    println!("  generated_at: {}", report.generated_at);
    println!("  content_digest: {}", report.content_digest);
    println!("  generation_id: {}", report.generation_id);
    println!("  mission_md: {}", report.mission_md);
    println!("  policy_digest: {}", report.policy_digest);
}

fn print_prepare_human(report: &PrepareReport) {
    println!("prepare: ok reused_active={}", report.reused_active);
    if let Some(baseline) = &report.baseline {
        println!("  baseline: {baseline}");
    } else {
        println!("  baseline: (unchanged; active candidate reused)");
    }
    let c = &report.candidate;
    println!(
        "  candidate: id={} state={} mission_id={} kind={}",
        c.id, c.state, c.mission_id, c.kind
    );
    println!("    policy_digest: {}", c.policy_digest);
    println!("    manifest_digest: {}", c.manifest_digest);
    println!(
        "    workspace: {}",
        c.workspace.as_deref().unwrap_or("none")
    );
    println!(
        "    candidate_digest: {}",
        c.candidate_digest.as_deref().unwrap_or("none")
    );
    println!("    created_at: {}", c.created_at);
    println!("    updated_at: {}", c.updated_at);
}
