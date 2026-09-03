//! Thin CLI over the `gzmo-evolver` library.
//!
//! Only real commands are exposed. Future lifecycle commands land with their
//! implementations, never as placeholders.

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use gzmo_evolver::{
    prepare_candidate, refresh_baseline_before_mission, CandidateRecord, CoordinatorLock,
    MissionAdapter, RepoEvolverConfig, StateStore, SystemClock, SystemProcessRunner,
};
#[cfg(unix)]
use gzmo_evolver::{run_hidden_worker, RepoEvolver, RunOutcome, RunnerError, StatusV1};
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
    /// Run or resume one candidate through the Evaluating boundary.
    Run {
        /// Emit machine-readable JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
    /// Resume the active/latest candidate without creating a new one.
    Resume {
        /// Emit machine-readable JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
    /// Abort a pre-evaluation candidate without deleting artifacts.
    Abort {
        /// Candidate id to abort.
        candidate_id: String,
        /// Terminal reason (nonempty, bounded).
        #[arg(long)]
        reason: String,
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

/// Structured lifecycle error under `--json` (stderr only; no secrets/logs).
#[derive(Debug, Serialize)]
struct LifecycleErrorV1<'a> {
    schema: &'static str,
    class: &'a str,
    candidate_id: Option<&'a str>,
    reason: &'a str,
    retryable: bool,
}

fn bound_cli_reason(msg: &str) -> String {
    const MAX: usize = 512;
    let trimmed = msg.trim();
    if trimmed.len() <= MAX {
        trimmed.to_owned()
    } else {
        format!("{}...", trimmed.chars().take(MAX).collect::<String>())
    }
}

/// Distinct unattended exit codes for typed lifecycle failures.
/// 3 contention, 4 recovery-required, 5 candidate-failed, 6 later-stage, 7 lock-busy; else 1.
#[cfg(unix)]
fn emit_runner_error(err: &RunnerError, json: bool) -> ExitCode {
    let (code, class, candidate_id, reason, retryable) = match err {
        RunnerError::Contention(msg) => (3u8, "contention", None, bound_cli_reason(msg), true),
        RunnerError::RecoveryRequired(msg) => {
            (4, "recovery_required", None, bound_cli_reason(msg), false)
        }
        RunnerError::Failed {
            reason,
            candidate_id,
        } => (
            5,
            "candidate_failed",
            Some(candidate_id.as_str()),
            bound_cli_reason(reason),
            false,
        ),
        RunnerError::LaterStage(msg) => (6, "later_stage", None, bound_cli_reason(msg), false),
        RunnerError::LockBusy => (
            7,
            "lock_busy",
            None,
            "coordinator lock busy".to_owned(),
            true,
        ),
        other => (
            1,
            "error",
            None,
            bound_cli_reason(&other.to_string()),
            false,
        ),
    };
    if json {
        let body = LifecycleErrorV1 {
            schema: "gzmo.repo_evolver.lifecycle_error/v1",
            class,
            candidate_id,
            reason: &reason,
            retryable,
        };
        match serde_json::to_string(&body) {
            Ok(s) => eprintln!("{s}"),
            Err(_) => eprintln!(
                "{{\"schema\":\"gzmo.repo_evolver.lifecycle_error/v1\",\"class\":\"error\",\"candidate_id\":null,\"reason\":\"serialize\",\"retryable\":false}}"
            ),
        }
    } else {
        eprintln!("error: {err}");
    }
    ExitCode::from(code)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Run { json } | Command::Resume { json } | Command::Abort { json, .. } => {
            match run_lifecycle_entry(&cli.config, &cli.command) {
                Ok(()) => ExitCode::SUCCESS,
                #[cfg(unix)]
                Err(LifecycleFailure::Runner(err)) => emit_runner_error(&err, json),
                Err(LifecycleFailure::Other(err)) => {
                    eprintln!("error: {err:#}");
                    ExitCode::FAILURE
                }
            }
        }
        other => {
            let cli = Cli {
                config: cli.config,
                command: other,
            };
            match run_non_lifecycle(cli) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("error: {err:#}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

enum LifecycleFailure {
    #[cfg(unix)]
    Runner(RunnerError),
    Other(anyhow::Error),
}

impl From<anyhow::Error> for LifecycleFailure {
    fn from(value: anyhow::Error) -> Self {
        Self::Other(value)
    }
}

fn run_lifecycle_entry(
    config: &Option<PathBuf>,
    command: &Command,
) -> Result<(), LifecycleFailure> {
    let config_path = require_cli_config(config.as_deref())?;
    let cfg = RepoEvolverConfig::load(config_path)
        .with_context(|| format!("loading config {}", config_path.display()))?;
    let (op, json) = match command {
        Command::Run { json } => (LifecycleOp::Run, *json),
        Command::Resume { json } => (LifecycleOp::Resume, *json),
        Command::Abort {
            candidate_id,
            reason,
            json,
        } => (
            LifecycleOp::Abort {
                candidate_id: candidate_id.clone(),
                reason: reason.clone(),
            },
            *json,
        ),
        _ => return Err(anyhow::anyhow!("internal: not a lifecycle command").into()),
    };
    run_lifecycle(cfg, json, op)
}

fn run_non_lifecycle(cli: Cli) -> Result<()> {
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
            #[cfg(unix)]
            {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .context("building tokio runtime")?;
                let evolver =
                    RepoEvolver::for_status(cfg).map_err(|e| anyhow::anyhow!("evolver: {e}"))?;
                let report = rt
                    .block_on(evolver.status())
                    .map_err(|e| anyhow::anyhow!("status: {e}"))?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else {
                    print_status_v1_human(&report);
                }
                Ok(())
            }
            #[cfg(not(unix))]
            {
                let _ = cfg;
                bail!("status requires unix runner");
            }
        }
        Command::Refresh { json } => {
            let config_path = require_cli_config(cli.config.as_deref())?;
            let cfg = RepoEvolverConfig::load(config_path)
                .with_context(|| format!("loading config {}", config_path.display()))?;
            let runner = SystemProcessRunner;
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
        Command::Run { .. } | Command::Resume { .. } | Command::Abort { .. } => {
            unreachable!("lifecycle commands are handled in main");
        }
    }
}

enum LifecycleOp {
    Run,
    Resume,
    Abort {
        candidate_id: String,
        reason: String,
    },
}

fn run_lifecycle(
    cfg: RepoEvolverConfig,
    json: bool,
    op: LifecycleOp,
) -> Result<(), LifecycleFailure> {
    #[cfg(unix)]
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("building tokio runtime")?;
        let evolver = match RepoEvolver::production(cfg) {
            Ok(e) => e,
            Err(err) => return Err(LifecycleFailure::Runner(err)),
        };
        let outcome: RunOutcome = match op {
            LifecycleOp::Run => rt
                .block_on(evolver.run_once())
                .map_err(LifecycleFailure::Runner)?,
            LifecycleOp::Resume => rt
                .block_on(evolver.resume())
                .map_err(LifecycleFailure::Runner)?,
            LifecycleOp::Abort {
                candidate_id,
                reason,
            } => rt
                .block_on(evolver.abort(&candidate_id, &reason))
                .map_err(LifecycleFailure::Runner)?,
        };
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&outcome).map_err(anyhow::Error::from)?
            );
        } else {
            print_run_outcome_human(&outcome);
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (cfg, json, op);
        Err(anyhow::anyhow!("lifecycle commands require unix").into())
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

#[cfg(unix)]
fn print_status_v1_human(report: &StatusV1) {
    println!("status: schema={}", report.schema);
    println!("  repository: {}", report.repository);
    println!(
        "  candidate_id: {}",
        report.candidate_id.as_deref().unwrap_or("none")
    );
    println!(
        "  mission_generation_id: {}",
        report.mission_generation_id.as_deref().unwrap_or("none")
    );
    println!("  state: {}", report.state.as_deref().unwrap_or("none"));
    println!(
        "  baseline_digest: {}",
        report.baseline_digest.as_deref().unwrap_or("none")
    );
    println!(
        "  candidate_digest: {}",
        report.candidate_digest.as_deref().unwrap_or("none")
    );
    println!(
        "  policy_digest: {}",
        report.policy_digest.as_deref().unwrap_or("none")
    );
    println!(
        "  manifest_digest: {}",
        report.manifest_digest.as_deref().unwrap_or("none")
    );
    println!(
        "  receipt_digest: {}",
        report.receipt_digest.as_deref().unwrap_or("none")
    );
    println!(
        "  workspace: {}",
        report.workspace.as_deref().unwrap_or("none")
    );
    println!(
        "  worker_state: {}",
        report.worker_state.as_deref().unwrap_or("none")
    );
    println!(
        "  worker_deadline: {}",
        report.worker_deadline.as_deref().unwrap_or("none")
    );
    println!(
        "  last_audit_sequence: {}",
        report
            .last_audit_sequence
            .map(|s| s.to_string())
            .unwrap_or_else(|| "none".to_owned())
    );
    println!(
        "  last_audit_hash: {}",
        report.last_audit_hash.as_deref().unwrap_or("none")
    );
    println!(
        "  terminal_reason: {}",
        report.terminal_reason.as_deref().unwrap_or("none")
    );
    println!("  next_action: {}", report.next_action);
    if let Some(b) = &report.budget_max {
        println!(
            "  budget_max: wall={} files={} lines={} tools={} in={} out={}",
            b.wall_seconds,
            b.max_changed_files,
            b.max_added_lines,
            b.max_tool_calls,
            b.max_input_tokens,
            b.max_output_tokens
        );
    }
    if let Some(u) = &report.budget_used {
        println!(
            "  budget_used: wall={} files={} lines={} tools={} in={} out={}",
            opt_num(u.wall_seconds),
            opt_num(u.changed_files),
            opt_num(u.added_lines),
            opt_num(u.tool_calls),
            opt_num(u.input_tokens),
            opt_num(u.output_tokens)
        );
    }
    if let Some(r) = &report.budget_remaining {
        println!(
            "  budget_remaining: wall={} files={} lines={} tools={} in={} out={}",
            opt_num(r.wall_seconds),
            opt_num(r.changed_files),
            opt_num(r.added_lines),
            opt_num(r.tool_calls),
            opt_num(r.input_tokens),
            opt_num(r.output_tokens)
        );
    }
}

fn opt_num<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map(|x| x.to_string())
        .unwrap_or_else(|| "none".to_owned())
}

#[cfg(unix)]
fn print_run_outcome_human(outcome: &RunOutcome) {
    println!("run: state={}", outcome.state);
    println!("  candidate_id: {}", outcome.candidate_id);
    println!("  mission_id: {}", outcome.mission_id);
    println!("  baseline_digest: {}", outcome.baseline_digest);
    println!(
        "  candidate_digest: {}",
        outcome.candidate_digest.as_deref().unwrap_or("none")
    );
    println!("  policy_digest: {}", outcome.policy_digest);
    println!("  manifest_digest: {}", outcome.manifest_digest);
    println!(
        "  receipt_digest: {}",
        outcome.receipt_digest.as_deref().unwrap_or("none")
    );
    println!(
        "  workspace: {}",
        outcome.workspace.as_deref().unwrap_or("none")
    );
    println!(
        "  terminal_reason: {}",
        outcome.terminal_reason.as_deref().unwrap_or("none")
    );
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

#[cfg(all(test, unix))]
mod lifecycle_error_tests {
    use super::*;
    use gzmo_evolver::RunnerError;

    #[test]
    fn lifecycle_error_codes_distinguish_lock_busy_and_failed() {
        let lock = RunnerError::LockBusy;
        let (code_lock, class_lock, _, _, retry_lock) = match &lock {
            RunnerError::LockBusy => (
                7u8,
                "lock_busy",
                None::<&str>,
                "coordinator lock busy",
                true,
            ),
            _ => unreachable!(),
        };
        let failed = RunnerError::Failed {
            reason: "boom".into(),
            candidate_id: "cand-20260901t120000z-bet-x".into(),
        };
        let (code_fail, class_fail, cand, reason, retry_fail) = match &failed {
            RunnerError::Failed {
                reason,
                candidate_id,
            } => (
                5u8,
                "candidate_failed",
                Some(candidate_id.as_str()),
                reason.as_str(),
                false,
            ),
            _ => unreachable!(),
        };
        assert_ne!(code_lock, code_fail);
        assert_ne!(class_lock, class_fail);
        assert!(retry_lock);
        assert!(!retry_fail);
        assert_eq!(cand, Some("cand-20260901t120000z-bet-x"));
        assert_eq!(reason, "boom");

        // emit_runner_error path: structured JSON shape
        let body_lock = LifecycleErrorV1 {
            schema: "gzmo.repo_evolver.lifecycle_error/v1",
            class: class_lock,
            candidate_id: None,
            reason: "coordinator lock busy",
            retryable: true,
        };
        let body_fail = LifecycleErrorV1 {
            schema: "gzmo.repo_evolver.lifecycle_error/v1",
            class: class_fail,
            candidate_id: cand,
            reason,
            retryable: false,
        };
        let jlock = serde_json::to_value(&body_lock).unwrap();
        let jfail = serde_json::to_value(&body_fail).unwrap();
        assert_eq!(jlock["class"], "lock_busy");
        assert_eq!(jfail["class"], "candidate_failed");
        assert_eq!(jlock["retryable"], true);
        assert_eq!(jfail["retryable"], false);
        assert!(jlock.get("stdout").is_none());
    }
}
