//! # Background Orchestrator
//!
//! Runs scheduled cron jobs as headless agent loops. Supports two modes:
//!
//! - **Simple mode**: A single prompt fires a one-shot agent conversation.
//! - **Pipeline mode**: Multi-step jobs with dependency-aware wave execution,
//!   per-step tool limits, result forwarding, retry logic, and vault persistence.
//!
//! The orchestrator does NOT write to stdout/stderr. All output goes through
//! `tracing` and the memory subsystems.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, Result};
use chrono::Utc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::agent_loop::{run_agent_loop, AgentLoopConfig, AgentMemoryContext};
use crate::config::{JobConfig, JobStep};
use crate::context::ContextConfig;
use crate::gateway::LlmGateway;
use crate::memory::episodic::FileEpisodicStore;
use crate::memory::scratch::{ScratchScope, ScratchService};
use crate::memory::vault::SqliteVault;
use crate::synapse::{resolve_event_source, EventSource, EventType, SynapseBus, SynapseEvent};
use crate::tools::ToolRegistry;
use crate::types::{EpisodicEntry, EpisodicSource, Message, Role};

// ─── Result Types ───────────────────────────────────────────────────────

/// Outcome of a single pipeline step.
#[derive(Debug, Clone)]
pub struct StepOutcome {
    pub name: String,
    pub status: StepStatus,
    pub result_text: String,
    pub duration_ms: u128,
    pub llm_calls: usize,
    pub tool_calls: usize,
}

/// Outcome of an entire job execution.
#[derive(Debug, Clone)]
pub struct JobOutcome {
    pub job_name: String,
    pub steps: Vec<StepOutcome>,
    pub overall_status: StepStatus,
    pub total_duration_ms: u128,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Success,
    Failed,
    Skipped,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Success => write!(f, "✔"),
            StepStatus::Failed => write!(f, "✘"),
            StepStatus::Skipped => write!(f, "⊘"),
        }
    }
}

// ─── Context ────────────────────────────────────────────────────────────

/// Shared execution context for all scheduled jobs.
pub struct OrchestratorContext {
    pub gateway: Arc<dyn LlmGateway>,
    pub tools: Arc<ToolRegistry>,
    pub system_prompt: String,
    /// Optional vault for persisting job results as long-term memory.
    pub vault: Option<Arc<SqliteVault>>,
    /// Optional episodic store for logging job activity.
    pub episodic: Option<Arc<FileEpisodicStore>>,
    /// Optional chaos engine feedback channel for energy injection.
    pub chaos_feedback_tx: Option<tokio::sync::mpsc::Sender<gzmo_chaos::feedback::ChaosEvent>>,
    /// Gated document ingest (watchers use this when [ingest].enabled).
    pub ingest_engine: Option<Arc<crate::ingest::IngestEngine>>,
    /// Optional Synapse event bus for append-only observability.
    pub synapse: Option<Arc<SynapseBus>>,
    /// Hot memory (scratch + archive @ 90%) for pipeline/simple headless steps.
    pub scratch: Arc<ScratchService>,
    /// Shared scope for orchestrator memory_search → scratch inject (updated per job step).
    pub memory_search_scope: Arc<std::sync::Mutex<ScratchScope>>,
    /// Context window budget from `[context_memory]` (not 6k default).
    pub context: ContextConfig,
}

fn orch_scope(job: &str, step: &str) -> ScratchScope {
    ScratchScope::Orch {
        job: job.to_string(),
        step: step.to_string(),
    }
}

// ─── Wave Resolution (Topological Sort) ─────────────────────────────────

/// Resolve pipeline steps into execution waves based on `depends_on`.
///
/// Returns a `Vec<Vec<usize>>` where each inner vec is a wave of step indices
/// that can execute in parallel. Waves execute sequentially.
///
/// Errors on circular dependencies or references to unknown step names.
fn resolve_waves(steps: &[JobStep]) -> Result<Vec<Vec<usize>>> {
    let name_to_idx: HashMap<&str, usize> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();

    // Validate all dependency references
    for step in steps {
        for dep in &step.depends_on {
            if !name_to_idx.contains_key(dep.as_str()) {
                bail!("Step '{}' depends on unknown step '{}'", step.name, dep);
            }
        }
    }

    // Build in-degree map
    let mut in_degree: Vec<usize> = steps.iter().map(|s| s.depends_on.len()).collect();

    // Build adjacency list (dependency → dependents)
    let mut dependents: Vec<Vec<usize>> = vec![vec![]; steps.len()];
    for (i, step) in steps.iter().enumerate() {
        for dep in &step.depends_on {
            let dep_idx = name_to_idx[dep.as_str()];
            dependents[dep_idx].push(i);
        }
    }

    let mut waves: Vec<Vec<usize>> = Vec::new();
    let mut resolved = 0usize;

    loop {
        // Collect all steps with in_degree == 0 (ready to execute)
        let wave: Vec<usize> = in_degree
            .iter()
            .enumerate()
            .filter(|(_, &deg)| deg == 0)
            .map(|(i, _)| i)
            .collect();

        if wave.is_empty() {
            break;
        }

        // Mark these as resolved (set in_degree to sentinel)
        for &idx in &wave {
            in_degree[idx] = usize::MAX; // sentinel: already resolved
            for &dependent in &dependents[idx] {
                in_degree[dependent] -= 1;
            }
        }

        resolved += wave.len();
        waves.push(wave);
    }

    if resolved != steps.len() {
        bail!("Circular dependency detected in pipeline steps");
    }

    Ok(waves)
}

// ─── Scheduler Boot ─────────────────────────────────────────────────────

/// Boot the background scheduler and register all active jobs.
///
/// Returns the `JobScheduler` handle — the caller must keep it alive
/// (e.g. via `tokio::select!`) for jobs to continue firing.
pub async fn start_orchestrator(
    jobs: HashMap<String, JobConfig>,
    ctx: Arc<OrchestratorContext>,
) -> Result<JobScheduler> {
    let sched = JobScheduler::new().await?;

    let active_jobs: Vec<_> = jobs.into_iter().filter(|(_, j)| !j.disabled).collect();

    if active_jobs.is_empty() {
        info!("Orchestrator: no active jobs configured");
        return Ok(sched);
    }

    let job_count = active_jobs.len();

    for (name, job_config) in active_jobs {
        let job_name = name.clone();
        let cron_expr = job_config.cron.clone();
        let mode = if job_config.steps.is_empty() {
            "simple"
        } else {
            "pipeline"
        };
        let step_count = job_config.steps.len();
        let ctx = Arc::clone(&ctx);

        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
            let job_name = job_name.clone();
            let job_cfg = job_config.clone();
            let ctx = Arc::clone(&ctx);

            Box::pin(async move {
                // Fire: DaemonTick
                if let Some(ref bus) = ctx.synapse {
                    bus.append(&SynapseEvent::with_data(
                        EventType::DaemonTick,
                        resolve_event_source(EventSource::GzmoDaemon),
                        serde_json::json!({ "job": job_name }),
                    ));
                }

                info!(job = %job_name, "Orchestrator: job fired");
                let outcome = execute_job(&ctx, &job_name, &job_cfg).await;
                match &outcome {
                    Ok(o) => {
                        info!(
                            job = %job_name,
                            status = %o.overall_status,
                            steps = o.steps.len(),
                            duration_ms = o.total_duration_ms,
                            "Orchestrator: job completed"
                        );
                        // Complete: DaemonJobComplete
                        if let Some(ref bus) = ctx.synapse {
                            let step_metrics: Vec<serde_json::Value> = o
                                .steps
                                .iter()
                                .map(|s| {
                                    serde_json::json!({
                                        "name": s.name,
                                        "status": format!("{}", s.status),
                                        "duration_ms": s.duration_ms,
                                        "llm_calls": s.llm_calls,
                                        "tool_calls": s.tool_calls,
                                    })
                                })
                                .collect();
                            let data = serde_json::json!({
                                "job": job_name,
                                "status": format!("{}", o.overall_status),
                                "steps": step_metrics,
                                "step_count": o.steps.len(),
                                "duration_ms": o.total_duration_ms,
                            });
                            bus.append(&SynapseEvent::with_data(
                                EventType::DaemonJobComplete,
                                resolve_event_source(EventSource::GzmoDaemon),
                                data,
                            ));
                        }
                    }
                    Err(e) => {
                        error!(job = %job_name, error = %e, "Orchestrator: job failed");
                        // Fail: DaemonJobFail
                        if let Some(ref bus) = ctx.synapse {
                            let data = serde_json::json!({
                                "job": job_name,
                                "error": e.to_string(),
                            });
                            bus.append(&SynapseEvent::with_data(
                                EventType::DaemonJobFail,
                                resolve_event_source(EventSource::GzmoDaemon),
                                data,
                            ));
                        }
                    }
                }
            })
        })?;

        sched.add(job).await?;
        info!(
            job = %name,
            cron = %cron_expr,
            mode,
            steps = step_count,
            "Orchestrator: job scheduled"
        );
    }

    sched.start().await?;
    info!(jobs = job_count, "Orchestrator: scheduler running");

    Ok(sched)
}

// ─── Job Execution ──────────────────────────────────────────────────────

/// Execute a job. Routes to simple or pipeline mode based on config.
async fn execute_job(
    ctx: &OrchestratorContext,
    job_name: &str,
    config: &JobConfig,
) -> Result<JobOutcome> {
    let start = Instant::now();

    let outcome = if config.steps.is_empty() {
        // Simple mode: single prompt
        execute_simple(ctx, job_name, &config.prompt, config.max_retries).await
    } else {
        // Pipeline mode: multi-step with waves
        execute_pipeline(ctx, job_name, config).await
    };

    let mut outcome = outcome?;
    outcome.total_duration_ms = start.elapsed().as_millis();

    // Persist results if configured
    if config.persist_results {
        persist_outcome(ctx, &outcome).await;
    }

    // Log to episodic memory
    log_to_episodic(ctx, &outcome).await;

    Ok(outcome)
}

/// Simple mode: backward-compatible single-prompt execution with optional retry.
async fn execute_simple(
    ctx: &OrchestratorContext,
    job_name: &str,
    prompt: &str,
    max_retries: u32,
) -> Result<JobOutcome> {
    let mut last_error: Option<String> = None;

    for attempt in 0..=max_retries {
        let step_start = Instant::now();

        // Build retry-aware prompt
        let effective_prompt = if let Some(ref err) = last_error {
            format!(
                "[BACKGROUND TASK: {}] {}\n\n---\n⚠ Previous attempt failed: {}\nAdjust your approach accordingly.",
                job_name, prompt, err
            )
        } else {
            format!("[BACKGROUND TASK: {}] {}", job_name, prompt)
        };

        let result = execute_headless_inner(ctx, job_name, &effective_prompt, 5).await;

        match result {
            Ok(response) => {
                let step = StepOutcome {
                    name: "main".to_string(),
                    status: StepStatus::Success,
                    result_text: response.text,
                    duration_ms: step_start.elapsed().as_millis(),
                    llm_calls: response.llm_calls,
                    tool_calls: response.tool_results.len(),
                };

                return Ok(JobOutcome {
                    job_name: job_name.to_string(),
                    steps: vec![step],
                    overall_status: StepStatus::Success,
                    total_duration_ms: 0, // filled by caller
                    timestamp: Utc::now(),
                });
            }
            Err(e) => {
                if attempt < max_retries {
                    warn!(
                        job = %job_name,
                        attempt = attempt + 1,
                        max_retries,
                        error = %e,
                        "Orchestrator: retrying job"
                    );
                    last_error = Some(e.to_string());
                } else {
                    let step = StepOutcome {
                        name: "main".to_string(),
                        status: StepStatus::Failed,
                        result_text: e.to_string(),
                        duration_ms: step_start.elapsed().as_millis(),
                        llm_calls: 0,
                        tool_calls: 0,
                    };

                    return Ok(JobOutcome {
                        job_name: job_name.to_string(),
                        steps: vec![step],
                        overall_status: StepStatus::Failed,
                        total_duration_ms: 0,
                        timestamp: Utc::now(),
                    });
                }
            }
        }
    }

    unreachable!()
}

/// Pipeline mode: multi-step job with dependency-aware wave execution.
async fn execute_pipeline(
    ctx: &OrchestratorContext,
    job_name: &str,
    config: &JobConfig,
) -> Result<JobOutcome> {
    let waves = resolve_waves(&config.steps)?;

    info!(
        job = %job_name,
        waves = waves.len(),
        steps = config.steps.len(),
        "Pipeline: resolved execution waves"
    );

    // Accumulate step results for downstream injection
    let mut step_results: HashMap<String, StepOutcome> = HashMap::new();
    let mut all_outcomes: Vec<StepOutcome> = Vec::new();
    let mut pipeline_failed = false;

    for (wave_idx, wave) in waves.iter().enumerate() {
        if pipeline_failed {
            // Skip remaining waves — mark all steps as skipped
            for &step_idx in wave {
                all_outcomes.push(StepOutcome {
                    name: config.steps[step_idx].name.clone(),
                    status: StepStatus::Skipped,
                    result_text: "Skipped: prior step failed".to_string(),
                    duration_ms: 0,
                    llm_calls: 0,
                    tool_calls: 0,
                });
            }
            continue;
        }

        info!(
            job = %job_name,
            wave = wave_idx + 1,
            steps = wave.len(),
            "Pipeline: executing wave"
        );

        if wave.len() == 1 {
            // Single step — no need for join overhead
            let step_idx = wave[0];
            let step = &config.steps[step_idx];
            let outcome =
                execute_step(ctx, job_name, step, &step_results, config.max_retries).await;
            if outcome.status == StepStatus::Failed {
                pipeline_failed = true;
            }
            step_results.insert(step.name.clone(), outcome.clone());
            all_outcomes.push(outcome);
        } else {
            // Parallel execution within wave
            let mut handles = Vec::new();

            for &step_idx in wave {
                let step = config.steps[step_idx].clone();
                let step_results_snapshot: HashMap<String, String> = step_results
                    .iter()
                    .map(|(k, v)| (k.clone(), v.result_text.clone()))
                    .collect();

                // Build the prior context string for this step
                let prior_context = build_prior_context(&step, &step_results_snapshot);
                let system_prompt = ctx.system_prompt.clone();
                let gateway = Arc::clone(&ctx.gateway);
                let tools = Arc::clone(&ctx.tools);
                let max_iters = step.max_iterations;
                let jn = job_name.to_string();
                let step_name = step.name.clone();
                let max_retries = config.max_retries;
                let scratch = Arc::clone(&ctx.scratch);
                let context = ctx.context.clone();

                handles.push(tokio::spawn(async move {
                    let step_start = Instant::now();
                    let effective_prompt = format!(
                        "[BACKGROUND TASK: {} / Step: {}] {}",
                        jn, step.name, step.prompt
                    );

                    let mut last_error: Option<String> = None;
                    for attempt in 0..=max_retries {
                        let retry_prompt = if let Some(ref err) = last_error {
                            format!("{}\n\n⚠ Previous attempt failed: {}", effective_prompt, err)
                        } else {
                            effective_prompt.clone()
                        };

                        let result = run_step_inner(
                            gateway.as_ref(),
                            &tools,
                            &system_prompt,
                            &prior_context,
                            &retry_prompt,
                            max_iters,
                            scratch.clone(),
                            context.clone(),
                            &jn,
                            &step_name,
                            None,
                        )
                        .await;

                        match result {
                            Ok(response) => {
                                return StepOutcome {
                                    name: step.name.clone(),
                                    status: StepStatus::Success,
                                    result_text: response.text,
                                    duration_ms: step_start.elapsed().as_millis(),
                                    llm_calls: response.llm_calls,
                                    tool_calls: response.tool_results.len(),
                                };
                            }
                            Err(e) if attempt < max_retries => {
                                last_error = Some(e.to_string());
                            }
                            Err(e) => {
                                return StepOutcome {
                                    name: step.name.clone(),
                                    status: StepStatus::Failed,
                                    result_text: e.to_string(),
                                    duration_ms: step_start.elapsed().as_millis(),
                                    llm_calls: 0,
                                    tool_calls: 0,
                                };
                            }
                        }
                    }
                    unreachable!()
                }));
            }

            // Collect parallel results
            for handle in handles {
                match handle.await {
                    Ok(outcome) => {
                        if outcome.status == StepStatus::Failed {
                            pipeline_failed = true;
                        }
                        step_results.insert(outcome.name.clone(), outcome.clone());
                        all_outcomes.push(outcome);
                    }
                    Err(e) => {
                        pipeline_failed = true;
                        all_outcomes.push(StepOutcome {
                            name: "unknown".to_string(),
                            status: StepStatus::Failed,
                            result_text: format!("Task join error: {e}"),
                            duration_ms: 0,
                            llm_calls: 0,
                            tool_calls: 0,
                        });
                    }
                }
            }
        }
    }

    let overall = if pipeline_failed {
        StepStatus::Failed
    } else {
        StepStatus::Success
    };

    Ok(JobOutcome {
        job_name: job_name.to_string(),
        steps: all_outcomes,
        overall_status: overall,
        total_duration_ms: 0,
        timestamp: Utc::now(),
    })
}

/// Execute a single pipeline step (sequential path).
async fn execute_step(
    ctx: &OrchestratorContext,
    job_name: &str,
    step: &JobStep,
    prior_results: &HashMap<String, StepOutcome>,
    max_retries: u32,
) -> StepOutcome {
    let step_start = Instant::now();

    // Build prior context from dependency results
    let prior_context_map: HashMap<String, String> = prior_results
        .iter()
        .map(|(k, v)| (k.clone(), v.result_text.clone()))
        .collect();
    let prior_context = build_prior_context(step, &prior_context_map);

    let effective_prompt = format!(
        "[BACKGROUND TASK: {} / Step: {}] {}",
        job_name, step.name, step.prompt
    );

    let mut last_error: Option<String> = None;

    for attempt in 0..=max_retries {
        let retry_prompt = if let Some(ref err) = last_error {
            format!("{}\n\n⚠ Previous attempt failed: {}", effective_prompt, err)
        } else {
            effective_prompt.clone()
        };

        let result = run_step_inner(
            ctx.gateway.as_ref(),
            &ctx.tools,
            &ctx.system_prompt,
            &prior_context,
            &retry_prompt,
            step.max_iterations,
            Arc::clone(&ctx.scratch),
            ctx.context.clone(),
            job_name,
            &step.name,
            Some(Arc::clone(&ctx.memory_search_scope)),
        )
        .await;

        match result {
            Ok(response) => {
                info!(
                    job = %job_name,
                    step = %step.name,
                    llm_calls = response.llm_calls,
                    tool_calls = response.tool_results.len(),
                    "Pipeline step completed"
                );
                return StepOutcome {
                    name: step.name.clone(),
                    status: StepStatus::Success,
                    result_text: response.text,
                    duration_ms: step_start.elapsed().as_millis(),
                    llm_calls: response.llm_calls,
                    tool_calls: response.tool_results.len(),
                };
            }
            Err(e) if attempt < max_retries => {
                warn!(
                    job = %job_name,
                    step = %step.name,
                    attempt = attempt + 1,
                    error = %e,
                    "Pipeline step retrying"
                );
                last_error = Some(e.to_string());
            }
            Err(e) => {
                error!(
                    job = %job_name,
                    step = %step.name,
                    error = %e,
                    "Pipeline step failed"
                );
                return StepOutcome {
                    name: step.name.clone(),
                    status: StepStatus::Failed,
                    result_text: e.to_string(),
                    duration_ms: step_start.elapsed().as_millis(),
                    llm_calls: 0,
                    tool_calls: 0,
                };
            }
        }
    }

    unreachable!()
}

// ─── Inner Execution ────────────────────────────────────────────────────

/// Build the prior-results context block injected as a System message.
fn build_prior_context(step: &JobStep, prior_results: &HashMap<String, String>) -> String {
    if step.depends_on.is_empty() {
        return String::new();
    }

    let mut block = String::from("\n\n## Prior Step Results\n\n");
    for dep_name in &step.depends_on {
        if let Some(result) = prior_results.get(dep_name) {
            block.push_str(&format!("### {} (completed)\n{}\n\n", dep_name, result));
        }
    }
    block
}

/// Dynamically read the ./skills directory to empower the Agent with Host-Parasite capabilities.
fn scan_skills_metadata() -> String {
    let mut block = String::from("\n\n## Autonomous Skills (Host-Parasite)\nYou have access to the following shell scripts in the `./skills` directory. Use the `shell_exec` tool to run them (e.g. `{\"command\": \"./skills/web_search.sh rust borrow checker\"}`).\n\n");
    let mut found = false;

    if let Ok(entries) = std::fs::read_dir("./skills") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "sh" || ext == "py" {
                        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                        let mut desc = "No description provided.".to_string();

                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for line in content.lines().take(5) {
                                let trimmed = line.trim();
                                if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
                                    desc = trimmed.trim_start_matches('#').trim().to_string();
                                    break;
                                }
                            }
                        }
                        block.push_str(&format!("- `./skills/{}`: {}\n", file_name, desc));
                        found = true;
                    }
                }
            }
        }
    }

    if found {
        block.push_str("\n> To learn a new capability, use your `shell_exec` tool to write a new script into `./skills/`.\n");
        block
    } else {
        String::new()
    }
}

/// Run a single headless agent loop with prior context injection.
async fn run_step_inner(
    gateway: &dyn LlmGateway,
    tools: &ToolRegistry,
    system_prompt: &str,
    prior_context: &str,
    user_prompt: &str,
    max_iterations: usize,
    scratch: Arc<ScratchService>,
    context: ContextConfig,
    job: &str,
    step: &str,
    memory_search_scope: Option<Arc<std::sync::Mutex<ScratchScope>>>,
) -> Result<crate::agent_loop::AgentResponse> {
    let scope = orch_scope(job, step);
    if let Some(cell) = &memory_search_scope {
        if let Ok(mut g) = cell.lock() {
            *g = scope.clone();
        }
    }
    let _ = scratch.clear(&scope).await;

    let dynamic_skills = scan_skills_metadata();

    let mut messages = vec![
        Message {
            role: Role::System,
            content: format!("{}{}{}", system_prompt, prior_context, dynamic_skills),
            is_meta: true,
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: Role::User,
            content: user_prompt.to_string(),
            is_meta: false,
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    let config = AgentLoopConfig {
        max_iterations,
        verbose_tool_output: false,
        context,
        on_chunk: None,
        memory: Some(AgentMemoryContext {
            scratch,
            session_id: format!("orch:{job}"),
            scope,
        }),
    };

    run_agent_loop(gateway, tools, &mut messages, &config).await
}

/// Execute a single headless agent conversation (used by simple mode and watchers).
async fn execute_headless_inner(
    ctx: &OrchestratorContext,
    job: &str,
    prompt: &str,
    max_iterations: usize,
) -> Result<crate::agent_loop::AgentResponse> {
    run_step_inner(
        ctx.gateway.as_ref(),
        &ctx.tools,
        &ctx.system_prompt,
        "",
        prompt,
        max_iterations,
        Arc::clone(&ctx.scratch),
        ctx.context.clone(),
        job,
        "main",
        Some(Arc::clone(&ctx.memory_search_scope)),
    )
    .await
}

/// Public API for watcher.rs and other callers that need simple headless execution.
pub async fn execute_headless(
    ctx: &OrchestratorContext,
    job_name: &str,
    prompt: &str,
) -> Result<()> {
    let effective_prompt = format!("[BACKGROUND TASK: {}] {}", job_name, prompt);
    let response = execute_headless_inner(ctx, job_name, &effective_prompt, 5).await?;

    info!(
        job = %job_name,
        llm_calls = response.llm_calls,
        tool_calls = response.tool_results.len(),
        response_len = response.text.len(),
        "Orchestrator: job completed"
    );

    let summary = crate::text_util::truncate_chars(&response.text, 200);
    info!(job = %job_name, summary = %summary, "Orchestrator: result");

    Ok(())
}

// ─── Memory Persistence ─────────────────────────────────────────────────

/// Persist a job outcome to the semantic vault.
async fn persist_outcome(ctx: &OrchestratorContext, outcome: &JobOutcome) {
    if let Some(ref vault) = ctx.vault {
        let summary = format_outcome_summary(outcome);
        if let Err(e) = vault.store_text(&summary, "Procedural", 1.0) {
            error!(
                job = %outcome.job_name,
                error = %e,
                "Failed to persist job outcome to vault"
            );
        } else {
            info!(
                job = %outcome.job_name,
                "Job outcome persisted to vault"
            );
        }
    }
}

/// Log a job outcome to episodic memory.
async fn log_to_episodic(ctx: &OrchestratorContext, outcome: &JobOutcome) {
    if let Some(ref episodic) = ctx.episodic {
        let summary = format_outcome_summary(outcome);
        let _ = episodic
            .append(&EpisodicEntry {
                timestamp: outcome.timestamp,
                source: EpisodicSource::InternalMonologue,
                content: summary,
                is_silent: true,
            })
            .await;
    }
}

/// Format a job outcome as a human-readable summary for storage.
fn format_outcome_summary(outcome: &JobOutcome) -> String {
    let mut parts = vec![format!(
        "[Job: {}] {} | {} step(s) | {}ms",
        outcome.job_name,
        outcome.overall_status,
        outcome.steps.len(),
        outcome.total_duration_ms,
    )];

    for step in &outcome.steps {
        let truncated = crate::text_util::truncate_chars(&step.result_text, 150);
        parts.push(format!(
            "  {} {} ({}ms, {} LLM, {} tools): {}",
            step.status, step.name, step.duration_ms, step.llm_calls, step.tool_calls, truncated,
        ));
    }

    parts.join("\n")
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_resolution_linear() {
        let steps = vec![
            JobStep {
                name: "a".into(),
                prompt: "".into(),
                depends_on: vec![],
                max_iterations: 5,
            },
            JobStep {
                name: "b".into(),
                prompt: "".into(),
                depends_on: vec!["a".into()],
                max_iterations: 5,
            },
            JobStep {
                name: "c".into(),
                prompt: "".into(),
                depends_on: vec!["b".into()],
                max_iterations: 5,
            },
        ];

        let waves = resolve_waves(&steps).unwrap();
        assert_eq!(waves.len(), 3);
        assert_eq!(waves[0], vec![0]);
        assert_eq!(waves[1], vec![1]);
        assert_eq!(waves[2], vec![2]);
    }

    #[test]
    fn test_wave_resolution_parallel() {
        let steps = vec![
            JobStep {
                name: "a".into(),
                prompt: "".into(),
                depends_on: vec![],
                max_iterations: 5,
            },
            JobStep {
                name: "b".into(),
                prompt: "".into(),
                depends_on: vec![],
                max_iterations: 5,
            },
            JobStep {
                name: "c".into(),
                prompt: "".into(),
                depends_on: vec!["a".into(), "b".into()],
                max_iterations: 5,
            },
        ];

        let waves = resolve_waves(&steps).unwrap();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0], vec![0, 1]); // a + b in parallel
        assert_eq!(waves[1], vec![2]); // c after both
    }

    #[test]
    fn test_wave_resolution_diamond() {
        // Classic diamond: a → b, a → c, b+c → d
        let steps = vec![
            JobStep {
                name: "a".into(),
                prompt: "".into(),
                depends_on: vec![],
                max_iterations: 5,
            },
            JobStep {
                name: "b".into(),
                prompt: "".into(),
                depends_on: vec!["a".into()],
                max_iterations: 5,
            },
            JobStep {
                name: "c".into(),
                prompt: "".into(),
                depends_on: vec!["a".into()],
                max_iterations: 5,
            },
            JobStep {
                name: "d".into(),
                prompt: "".into(),
                depends_on: vec!["b".into(), "c".into()],
                max_iterations: 5,
            },
        ];

        let waves = resolve_waves(&steps).unwrap();
        assert_eq!(waves.len(), 3);
        assert_eq!(waves[0], vec![0]); // a
        assert_eq!(waves[1], vec![1, 2]); // b + c parallel
        assert_eq!(waves[2], vec![3]); // d
    }

    #[test]
    fn test_wave_resolution_cycle_detected() {
        let steps = vec![
            JobStep {
                name: "a".into(),
                prompt: "".into(),
                depends_on: vec!["b".into()],
                max_iterations: 5,
            },
            JobStep {
                name: "b".into(),
                prompt: "".into(),
                depends_on: vec!["a".into()],
                max_iterations: 5,
            },
        ];

        let result = resolve_waves(&steps);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular"));
    }

    #[test]
    fn test_wave_resolution_unknown_dep() {
        let steps = vec![JobStep {
            name: "a".into(),
            prompt: "".into(),
            depends_on: vec!["nonexistent".into()],
            max_iterations: 5,
        }];

        let result = resolve_waves(&steps);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown step"));
    }

    #[test]
    fn test_wave_resolution_single_step() {
        let steps = vec![JobStep {
            name: "only".into(),
            prompt: "".into(),
            depends_on: vec![],
            max_iterations: 5,
        }];

        let waves = resolve_waves(&steps).unwrap();
        assert_eq!(waves.len(), 1);
        assert_eq!(waves[0], vec![0]);
    }

    #[test]
    fn test_format_outcome_summary() {
        let outcome = JobOutcome {
            job_name: "test_job".to_string(),
            steps: vec![StepOutcome {
                name: "gather".to_string(),
                status: StepStatus::Success,
                result_text: "All systems nominal".to_string(),
                duration_ms: 1234,
                llm_calls: 2,
                tool_calls: 3,
            }],
            overall_status: StepStatus::Success,
            total_duration_ms: 5000,
            timestamp: Utc::now(),
        };

        let summary = format_outcome_summary(&outcome);
        assert!(summary.contains("test_job"));
        assert!(summary.contains("gather"));
        assert!(summary.contains("All systems nominal"));
    }
}
