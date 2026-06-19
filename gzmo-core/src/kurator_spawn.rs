//! Kurator phase 3 — governed sub-agent spawn (manual approve + daemon autospawn).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Result};

use crate::config::{GzmoConfig, KuratorConfig, RedisConfig};
use crate::context_compress::CcrStore;
use crate::gateway::LlmGateway;
use crate::kurator_monitor::{
    load_state, mark_recommendation_spawned, restore_pending_recommendation,
    take_pending_recommendation, PendingRecommendation,
};
use crate::memory::scratch::ScratchService;
use crate::memory::vault::SqliteVault;
use crate::spawn_gate::{
    self, bypass_gate_for_approved_via, emit_spawn_denied, emit_spawn_executed, evaluate_autospawn,
    record_denial, record_execution,
};
use crate::obolus::gate::{
    self, emit_obolus_denied, emit_obolus_warn, ObolusAction, ObolusTier, ObolusVerdict,
};
use crate::spawn_prime_budget::{acquire_prime_slot, release_prime_slot};
use crate::subagent::{SubagentRunner, SubagentResult, SubagentSpec, SubStatus};
use crate::synapse::SynapseBus;
use crate::synapse_writer::{emit_agent_result, emit_agent_spawned, ForumThread};
use crate::text_util::truncate_chars;
use crate::discovery_fixer::ActionableFinding;

pub fn synapse_bus_path(config: &GzmoConfig) -> PathBuf {
    std::env::var("GZMO_SYNAPSE_BUS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            config
                .memory
                .vault_db
                .parent()
                .unwrap_or_else(|| Path::new("data"))
                .join("Synapse/events.jsonl")
        })
}

fn project_root_from_kurator_state(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn build_subagent_runner(
    config: &GzmoConfig,
    scratch: Arc<ScratchService>,
    vault: Option<Arc<SqliteVault>>,
    gateway: Arc<dyn LlmGateway>,
) -> Arc<SubagentRunner> {
    let ccr = CcrStore::new(&config.redis, &config.context_compress);
    let system_prompt = std::fs::read_to_string(&config.identity.soul_path)
        .unwrap_or_else(|_| "You are a focused GZMO sub-agent.".to_string());
    let serpapi_key = std::env::var("SERPAPI_API_KEY").unwrap_or_default();

    Arc::new(SubagentRunner::new(
        config.subagent.clone(),
        config.context_compress.clone(),
        ccr,
        scratch,
        gateway,
        vault,
        system_prompt,
        serpapi_key,
    ))
}

pub fn spec_from_recommendation(
    rec: &PendingRecommendation,
    config: &KuratorConfig,
) -> SubagentSpec {
    if crate::discovery_execute::is_discovery_execute_recommendation(rec) {
        let plan_dir = rec
            .report_path
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return build_discovery_execute_spec(rec, config, &plan_dir);
    }
    if crate::discovery_plan_agent::is_discovery_plan_recommendation(rec) {
        let report_path = crate::discovery_fixer::resolve_discovery_report_path(rec, None);
        return build_discovery_plan_spec(rec, config, &report_path, None);
    }
    if crate::discovery_code_implementer::is_discovery_code_implement_recommendation(rec) {
        let report_path = crate::discovery_fixer::resolve_discovery_report_path(rec, None);
        return build_code_implement_spec(rec, config, &report_path, None);
    }
    if crate::discovery_fixer::is_discovery_fix_recommendation(rec) {
        let report_path = crate::discovery_fixer::resolve_discovery_report_path(rec, None);
        return build_discovery_fix_spec(rec, config, &report_path, None);
    }

    let brief = truncate_chars(
        &format!(
            "Kurator intervention for Pi session `{session_id}`.\n\
            Trigger: {reason}\n\n\
            Task:\n\
            1. Read session metrics in `data/kurator-monitor.state.json` for this session_id.\n\
            2. Optionally inspect recent Synapse events in `data/Synapse/events.jsonl` (correlation_id = session_id).\n\
            3. Summarize whether the session needs operator attention and what action to take.\n\
            4. Return a concise operator brief.\n\n\
            Do NOT run broad recursive greps for the UUID across /data, /home, or /var.",
            session_id = rec.session_id,
            reason = rec.reason,
        ),
        config.spawn_brief_max_chars,
    );
    SubagentSpec {
        role: rec.suggested_agent_profile.clone(),
        brief,
        max_iterations: 8,
        depth: 1,
        parent_session: rec.session_id.clone(),
        working_dir: None,
        shell_extra_commands: Vec::new(),
    }
}

fn build_discovery_fix_spec(
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    report_path: &Path,
    single_finding: Option<&ActionableFinding>,
) -> SubagentSpec {
    let brief = if let Some(finding) = single_finding {
        crate::discovery_fixer::build_fixer_brief_single(
            report_path,
            &rec.session_id,
            finding,
            config.spawn_brief_max_chars,
        )
    } else {
        let analysis = crate::discovery_fixer::analyze_discovery_report(report_path)
            .unwrap_or_default();
        if analysis.has_actionable() {
            crate::discovery_fixer::build_fixer_brief(
                report_path,
                &rec.session_id,
                &analysis,
                config.spawn_brief_max_chars,
            )
        } else {
            truncate_chars(
                &format!(
                    "Discovery fixer for `{}`.\nTrigger: {}\nRead report at {} and attempt remediation.",
                    rec.session_id,
                    rec.reason,
                    report_path.display(),
                ),
                config.spawn_brief_max_chars,
            )
        }
    };

    SubagentSpec {
        role: rec.suggested_agent_profile.clone(),
        brief,
        max_iterations: config.discovery_fixer_max_iterations,
        depth: 1,
        parent_session: rec.session_id.clone(),
        working_dir: Some(crate::discovery_fixer::discovery_fixer_working_dir()),
        shell_extra_commands: config.discovery_fixer_shell_extra_commands.clone(),
    }
}

fn build_code_implement_spec(
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    report_path: &Path,
    single_finding: Option<&ActionableFinding>,
) -> SubagentSpec {
    let discovery_session_id = rec
        .session_id
        .strip_prefix("discovery-implement:")
        .and_then(|s| s.split(':').next())
        .unwrap_or(&rec.session_id);
    let manifest = crate::discovery_code_implementer::resolve_implement_manifest_path(discovery_session_id);
    let brief = if let Some(finding) = single_finding {
        crate::discovery_code_implementer::build_code_implementer_brief_single(
            report_path,
            discovery_session_id,
            &manifest,
            finding,
            config.spawn_brief_max_chars,
        )
    } else {
        let analysis = crate::discovery_fixer::analyze_discovery_report(report_path)
            .unwrap_or_default();
        crate::discovery_code_implementer::build_code_implementer_brief(
            report_path,
            discovery_session_id,
            &manifest,
            &analysis.findings,
            config.spawn_brief_max_chars,
        )
    };

    SubagentSpec {
        role: rec.suggested_agent_profile.clone(),
        brief,
        max_iterations: config.discovery_code_implementer_max_iterations,
        depth: 1,
        parent_session: rec.session_id.clone(),
        working_dir: Some(crate::discovery_fixer::discovery_fixer_working_dir()),
        shell_extra_commands: config.discovery_fixer_shell_extra_commands.clone(),
    }
}

fn build_discovery_execute_spec(
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    plan_dir: &Path,
) -> SubagentSpec {
    let workstream_id = rec
        .reason
        .strip_prefix("discovery_execute: workstream ")
        .unwrap_or("W1");
    let plan_id = plan_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("plan");
    let git_tag = format!("discovery-baseline/{plan_id}");
    let workstream =
        crate::discovery_execute::load_workstream(plan_dir, workstream_id).unwrap_or(serde_json::json!({}));
    let brief = crate::discovery_execute::build_execute_brief(
        plan_dir,
        workstream_id,
        &workstream,
        &git_tag,
        config.spawn_brief_max_chars,
    );
    SubagentSpec {
        role: rec.suggested_agent_profile.clone(),
        brief,
        max_iterations: config.discovery_code_implementer_max_iterations,
        depth: 1,
        parent_session: rec.session_id.clone(),
        working_dir: Some(crate::discovery_fixer::discovery_fixer_working_dir()),
        shell_extra_commands: config.discovery_fixer_shell_extra_commands.clone(),
    }
}

async fn spawn_discovery_execute(
    runner: &SubagentRunner,
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    project_root: &Path,
    initial_spec: SubagentSpec,
) -> Result<SubagentResult> {
    let plan_dir = rec
        .report_path
        .as_deref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));
    let roots = crate::discovery_fixer::discovery_fixer_search_roots(project_root);
    let max_retries = config.discovery_code_implementer_max_retries;

    let mut spec = initial_spec;
    let mut last_result: Option<SubagentResult> = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            spec = build_discovery_execute_spec(rec, config, plan_dir);
        }

        let mut result = match runner.spawn(spec.clone()).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(attempt, error = %e, "Discovery execute spawn failed");
                if attempt >= max_retries {
                    return Err(e);
                }
                continue;
            }
        };
        let verification = crate::discovery_execute::verify_execute_outcome(
            &result.summary,
            result.hit_max_iterations,
            &roots,
            &result.written_paths,
        );

        if verification.passed {
            last_result = Some(result);
            break;
        }

        tracing::warn!(
            task_id = %result.task_id,
            attempt,
            notes = %verification.notes,
            "Discovery execute verify gate failed"
        );

        result.status = SubStatus::Failed;
        result.summary = format!(
            "{}\n\n[verify_gate FAILED] {}",
            result.summary, verification.notes
        );
        crate::remediation_tracker::emit_discovery_fix_failed(
            bus,
            rec,
            &result.task_id,
            plan_dir,
            &verification,
            attempt,
        );
        last_result = Some(result);

        if attempt >= max_retries {
            break;
        }
    }

    last_result.ok_or_else(|| anyhow::anyhow!("discovery execute produced no result"))
}

fn build_discovery_plan_spec(
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    report_path: &Path,
    eval_feedback: Option<&str>,
) -> SubagentSpec {
    let discovery_session_id = rec
        .session_id
        .strip_prefix("discovery-plan:")
        .unwrap_or(&rec.session_id);
    let plan_id = crate::discovery_plan_agent::plan_id_from_report(report_path, discovery_session_id);
    let output = crate::discovery_plan_agent::resolve_plan_output_paths(&plan_id);
    let analysis = crate::discovery_fixer::analyze_discovery_report(report_path).unwrap_or_default();
    let env_feedback = std::env::var("DISCOVERY_PLAN_EVAL_FEEDBACK")
        .ok()
        .filter(|s| !s.is_empty());
    let feedback = eval_feedback.or(env_feedback.as_deref());
    let brief = crate::discovery_plan_agent::build_plan_agent_brief(
        report_path,
        discovery_session_id,
        &plan_id,
        &analysis.findings,
        &output,
        feedback,
        config.spawn_brief_max_chars,
    );

    SubagentSpec {
        role: rec.suggested_agent_profile.clone(),
        brief,
        max_iterations: config.discovery_plan_max_iterations,
        depth: 1,
        parent_session: rec.session_id.clone(),
        working_dir: Some(crate::discovery_fixer::discovery_fixer_working_dir()),
        shell_extra_commands: config.discovery_fixer_shell_extra_commands.clone(),
    }
}

async fn spawn_discovery_plan_with_retries(
    runner: &SubagentRunner,
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    _project_root: &Path,
    initial_spec: SubagentSpec,
) -> Result<SubagentResult> {
    let report_path = crate::discovery_fixer::resolve_discovery_report_path(rec, None);
    let discovery_session_id = rec
        .session_id
        .strip_prefix("discovery-plan:")
        .unwrap_or(&rec.session_id);
    let plan_id = crate::discovery_plan_agent::plan_id_from_report(&report_path, discovery_session_id);
    let output = crate::discovery_plan_agent::resolve_plan_output_paths(&plan_id);
    let analysis = crate::discovery_fixer::analyze_discovery_report(&report_path)?;
    let max_retries = config.discovery_plan_max_retries;

    let mut spec = initial_spec;
    let mut last_result: Option<SubagentResult> = None;
    let mut last_verify_notes: Option<String> = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            spec = build_discovery_plan_spec(
                rec,
                config,
                &report_path,
                last_verify_notes.as_deref(),
            );
        }

        let mut result = match runner.spawn(spec.clone()).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(attempt, error = %e, "Discovery plan agent spawn failed");
                if attempt >= max_retries {
                    return Err(e);
                }
                continue;
            }
        };
        let verification = crate::discovery_plan_agent::verify_plan_agent_outcome(
            &output,
            &result.written_paths,
            analysis.actionable_count(),
        );

        if verification.passed {
            last_result = Some(result);
            break;
        }

        tracing::warn!(
            task_id = %result.task_id,
            attempt,
            notes = %verification.notes,
            "Discovery plan agent verify gate failed"
        );

        result.status = SubStatus::Failed;
        result.summary = format!(
            "{}\n\n[verify_gate FAILED] {}",
            result.summary, verification.notes
        );
        crate::remediation_tracker::emit_discovery_fix_failed(
            bus,
            rec,
            &result.task_id,
            &report_path,
            &crate::discovery_fixer::DiscoveryFixVerification {
                passed: false,
                missing_paths: vec![],
                hit_max_iterations: result.hit_max_iterations,
                notes: verification.notes.clone(),
            },
            attempt,
        );
        last_verify_notes = Some(verification.notes.clone());
        last_result = Some(result);

        if attempt >= max_retries {
            break;
        }
    }

    last_result.ok_or_else(|| anyhow::anyhow!("discovery plan agent produced no result"))
}

async fn spawn_discovery_code_implement_with_retries(
    runner: &SubagentRunner,
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    project_root: &Path,
    _initial_spec: SubagentSpec,
) -> Result<SubagentResult> {
    let tracker_path = crate::remediation_tracker::default_tracker_path();
    let report_path = crate::discovery_fixer::resolve_discovery_report_path(rec, None);
    let roots = crate::discovery_fixer::discovery_fixer_search_roots(project_root);
    let max_retries = config.discovery_code_implementer_max_retries;

    let mut last_result: Option<SubagentResult> = None;

    while let Some(finding) =
        crate::remediation_tracker::next_probed_finding(&tracker_path, &report_path)
    {
        let finding_id = finding.finding_id.clone();
        let finding_kind = finding.kind;
        let _ = crate::remediation_tracker::mark_finding_in_flight(
            &tracker_path,
            &report_path,
            &finding_id,
            finding_kind,
        );

        let spec = build_code_implement_spec(rec, config, &report_path, Some(&finding));

        let mut result = match runner.spawn(spec).await {
            Ok(r) => r,
            Err(e) => {
                if let Err(reset_err) = crate::remediation_tracker::reset_in_flight_finding(
                    &tracker_path,
                    &report_path,
                    &finding_id,
                    finding_kind,
                ) {
                    tracing::warn!(
                        error = %reset_err,
                        finding_id = %finding_id,
                        "remediation tracker: failed to reset in_flight after spawn error"
                    );
                }
                return Err(e);
            }
        };

        let verification = crate::discovery_code_implementer::verify_code_implement_outcome(
            &result.summary,
            result.hit_max_iterations,
            &roots,
            &result.written_paths,
        );

        let closed_ids = vec![finding_id.clone()];
        if let Err(e) = crate::remediation_tracker::record_spawn_outcome(
            &tracker_path,
            &report_path,
            &result.task_id,
            &closed_ids,
            &verification,
            &result.written_paths,
            max_retries,
        ) {
            tracing::warn!(error = %e, "remediation tracker: failed to record code implement outcome");
        }

        if verification.passed {
            crate::remediation_tracker::emit_discovery_fix_closed(
                bus,
                rec,
                &result.task_id,
                &report_path,
                &verification,
                &closed_ids,
            );
            last_result = Some(result);
            continue;
        }

        tracing::warn!(
            task_id = %result.task_id,
            finding_id = %finding_id,
            notes = %verification.notes,
            "Discovery code implementer verify gate failed"
        );

        result.status = SubStatus::Failed;
        result.summary = format!(
            "{}\n\n[verify_gate FAILED] {}",
            result.summary, verification.notes
        );
        crate::remediation_tracker::emit_discovery_fix_failed(
            bus,
            rec,
            &result.task_id,
            &report_path,
            &verification,
            0,
        );
        last_result = Some(result);

        let state = crate::remediation_tracker::load(&tracker_path);
        let exhausted = state.findings.iter().any(|f| {
            f.report_path == report_path.to_string_lossy()
                && f.finding_id == finding_id
                && f.status == crate::remediation_tracker::RemediationStatus::Failed
        });
        if exhausted {
            tracing::warn!(
                finding_id = %finding_id,
                "Discovery code implementer max retries exceeded for finding"
            );
        }
    }

    last_result.ok_or_else(|| anyhow::anyhow!("code implementer produced no result"))
}

async fn spawn_discovery_fix_with_retries(
    runner: &SubagentRunner,
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    project_root: &Path,
    initial_spec: SubagentSpec,
) -> Result<SubagentResult> {
    let tracker_path = crate::remediation_tracker::default_tracker_path();
    let report_path = crate::discovery_fixer::resolve_discovery_report_path(rec, None);
    let roots = crate::discovery_fixer::discovery_fixer_search_roots(project_root);
    let max_retries = config.discovery_fixer_max_retries;

    let mut spec = initial_spec;
    let mut last_result: Option<SubagentResult> = None;

    for attempt in 0..=max_retries {
        if attempt == 0 {
            let _ = crate::remediation_tracker::mark_all_open_in_flight(&tracker_path, &report_path);
        } else {
            let Some(finding) =
                crate::remediation_tracker::next_open_finding(&tracker_path, &report_path)
            else {
                break;
            };
            let _ = crate::remediation_tracker::mark_finding_in_flight(
                &tracker_path,
                &report_path,
                &finding.finding_id,
                finding.kind,
            );
            spec = build_discovery_fix_spec(rec, config, &report_path, Some(&finding));
        }

        let mut result = runner.spawn(spec.clone()).await?;
        let verification = crate::discovery_fixer::verify_discovery_fix_outcome(
            &result.summary,
            result.hit_max_iterations,
            &roots,
            &result.written_paths,
        );

        let closed_ids =
            crate::remediation_tracker::in_flight_finding_ids(&tracker_path, &report_path);
        if let Err(e) = crate::remediation_tracker::record_spawn_outcome(
            &tracker_path,
            &report_path,
            &result.task_id,
            &closed_ids,
            &verification,
            &result.written_paths,
            max_retries,
        ) {
            tracing::warn!(error = %e, "remediation tracker: failed to record spawn outcome");
        }

        if verification.passed {
            crate::remediation_tracker::emit_discovery_fix_closed(
                bus,
                rec,
                &result.task_id,
                &report_path,
                &verification,
                &closed_ids,
            );
            last_result = Some(result);
            break;
        }

        tracing::warn!(
            task_id = %result.task_id,
            attempt,
            notes = %verification.notes,
            missing = ?verification.missing_paths,
            hit_max = verification.hit_max_iterations,
            "Discovery fixer verify gate failed"
        );

        result.status = SubStatus::Failed;
        result.summary = format!(
            "{}\n\n[verify_gate FAILED] {}",
            result.summary, verification.notes
        );
        crate::remediation_tracker::emit_discovery_fix_failed(
            bus,
            rec,
            &result.task_id,
            &report_path,
            &verification,
            attempt,
        );
        last_result = Some(result);

        if attempt >= max_retries {
            break;
        }
        if crate::remediation_tracker::next_open_finding(&tracker_path, &report_path).is_none() {
            break;
        }
    }

    Ok(last_result.expect("discovery fix spawn produced no result"))
}

async fn spawn_with_remediation_loop(
    runner: &SubagentRunner,
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    config: &KuratorConfig,
    project_root: &Path,
    spec: SubagentSpec,
) -> Result<SubagentResult> {
    if crate::discovery_execute::is_discovery_execute_recommendation(rec) {
        spawn_discovery_execute(runner, bus, rec, config, project_root, spec).await
    } else if crate::discovery_plan_agent::is_discovery_plan_recommendation(rec) {
        spawn_discovery_plan_with_retries(runner, bus, rec, config, project_root, spec).await
    } else if crate::discovery_code_implementer::is_discovery_code_implement_recommendation(rec) {
        spawn_discovery_code_implement_with_retries(runner, bus, rec, config, project_root, spec)
            .await
    } else if crate::discovery_fixer::is_discovery_fix_recommendation(rec) {
        spawn_discovery_fix_with_retries(runner, bus, rec, config, project_root, spec).await
    } else {
        runner.spawn(spec).await
    }
}

fn discovery_agent_closed_loop(rec: &PendingRecommendation, result: &SubagentResult) -> bool {
    !crate::discovery_fixer::is_discovery_fix_recommendation(rec)
        && !crate::discovery_code_implementer::is_discovery_code_implement_recommendation(rec)
        && !crate::discovery_plan_agent::is_discovery_plan_recommendation(rec)
        && !crate::discovery_execute::is_discovery_execute_recommendation(rec)
        || !matches!(result.status, SubStatus::Failed)
}

fn obolus_action_for_rec(rec: &PendingRecommendation) -> ObolusAction {
    if crate::discovery_plan_agent::is_discovery_plan_recommendation(rec) {
        ObolusAction::DiscoveryPlan
    } else if crate::discovery_fixer::is_discovery_fix_recommendation(rec)
        || crate::discovery_code_implementer::is_discovery_code_implement_recommendation(rec)
        || crate::discovery_execute::is_discovery_execute_recommendation(rec)
    {
        ObolusAction::SpawnDiscoveryFix
    } else {
        ObolusAction::SpawnSessionTriage
    }
}

fn obolus_tier_for_spawn(approved_via: &str) -> ObolusTier {
    if approved_via.contains("autospawn") || approved_via.contains("fix-from-discovery") {
        ObolusTier::Autonomous
    } else {
        ObolusTier::Operator
    }
}

fn check_obolus_spawn(
    gzmo: &GzmoConfig,
    bus: &SynapseBus,
    rec: &PendingRecommendation,
    approved_via: &str,
) -> Result<()> {
    let action = obolus_action_for_rec(rec);
    let tier = obolus_tier_for_spawn(approved_via);
    match gate::evaluate_from_config(gzmo, action, tier)? {
        ObolusVerdict::Allow => Ok(()),
        ObolusVerdict::Warn { reason } => {
            emit_obolus_warn(bus, action, &reason);
            tracing::warn!(action = action.as_str(), %reason, "obolus budget warning");
            Ok(())
        }
        ObolusVerdict::Defer { reason } => {
            emit_obolus_denied(bus, action, &reason);
            bail!("obolus gate deferred: {reason}");
        }
        ObolusVerdict::Deny { reason } => {
            emit_obolus_denied(bus, action, &reason);
            bail!("obolus gate denied: {reason}");
        }
    }
}

/// Spawn a governed sub-agent for a recommendation and write Forum Romanum bus events.
pub async fn spawn_recommendation(
    runner: &SubagentRunner,
    bus: &SynapseBus,
    state_path: &Path,
    rec: PendingRecommendation,
    config: &KuratorConfig,
    redis_cfg: &RedisConfig,
    gzmo: &GzmoConfig,
    approved_via: &str,
) -> Result<crate::subagent::SubagentResult> {
    let project_root = project_root_from_kurator_state(state_path);
    let gate_path = spawn_gate::default_state_path(&project_root);

    check_obolus_spawn(gzmo, bus, &rec, approved_via)?;

    if !bypass_gate_for_approved_via(approved_via) {
        let kurator_state = load_state(state_path);
        let gate_state = spawn_gate::load_state(&gate_path);
        let decision = evaluate_autospawn(&rec, &config.spawn_gate, &gate_state, &kurator_state);
        if !decision.allowed {
            emit_spawn_denied(bus, &rec, &decision);
            record_denial(&gate_path, &rec, &decision)?;
            bail!("spawn gate denied: {} — {}", decision.code, decision.message);
        }

        let prime = acquire_prime_slot(redis_cfg, &config.spawn_gate).await;
        if let Some(decision) = prime.decision_if_denied() {
            emit_spawn_denied(bus, &rec, decision);
            record_denial(&gate_path, &rec, decision)?;
            bail!("spawn gate denied: {} — {}", decision.code, decision.message);
        }
        if let crate::spawn_prime_budget::PrimeBudgetOutcome::AllowedFailOpen { reason } = &prime {
            tracing::warn!(reason = %reason, "Prime budget fail-open — spawn proceeding");
        }

        let event_id = rec.event_id.clone();
        let session_id = rec.session_id.clone();
        let agent_profile = rec.suggested_agent_profile.clone();
        let spec = spec_from_recommendation(&rec, config);

        let reply_to = uuid::Uuid::parse_str(&event_id).ok();
        let thread = ForumThread::from_session(&session_id);
        let thread = if let Some(id) = reply_to {
            thread.with_reply_to(id)
        } else {
            thread
        };

        let spawn_result = spawn_with_remediation_loop(
            runner,
            bus,
            &rec,
            config,
            &project_root,
            spec,
        )
        .await;
        if spawn_result.is_err() {
            release_prime_slot(redis_cfg, &config.spawn_gate).await;
        }
        let result = spawn_result?;

        emit_agent_spawned(
            bus,
            &thread,
            &agent_profile,
            serde_json::json!({
                "recommendation_id": event_id,
                "approved_via": approved_via,
                "task_id": result.task_id,
                "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
            }),
        );
        emit_agent_result(
            bus,
            &thread,
            &agent_profile,
            &format!("{:?}", result.status).to_lowercase(),
            serde_json::json!({
                "task_id": result.task_id,
                "summary": result.summary,
                "llm_calls": result.llm_calls,
                "tool_calls": result.tool_calls,
                "hit_max_iterations": result.hit_max_iterations,
                "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
            }),
        );
        if discovery_agent_closed_loop(&rec, &result) {
            emit_spawn_executed(bus, &rec, &result.task_id, approved_via);
        }
        record_execution(&gate_path, &rec, &result.task_id, approved_via)?;
        mark_recommendation_spawned(state_path, &event_id, &result.task_id, rec)?;

        return Ok(result);
    }

    let event_id = rec.event_id.clone();
    let session_id = rec.session_id.clone();
    let agent_profile = rec.suggested_agent_profile.clone();
    let spec = spec_from_recommendation(&rec, config);

    let reply_to = uuid::Uuid::parse_str(&event_id).ok();
    let thread = ForumThread::from_session(&session_id);
    let thread = if let Some(id) = reply_to {
        thread.with_reply_to(id)
    } else {
        thread
    };

    let result = spawn_with_remediation_loop(
        runner,
        bus,
        &rec,
        config,
        &project_root,
        spec,
    )
    .await?;

    emit_agent_spawned(
        bus,
        &thread,
        &agent_profile,
        serde_json::json!({
            "recommendation_id": event_id,
            "approved_via": approved_via,
            "task_id": result.task_id,
            "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
        }),
    );
    emit_agent_result(
        bus,
        &thread,
        &agent_profile,
        &format!("{:?}", result.status).to_lowercase(),
        serde_json::json!({
            "task_id": result.task_id,
            "summary": result.summary,
            "llm_calls": result.llm_calls,
            "tool_calls": result.tool_calls,
            "hit_max_iterations": result.hit_max_iterations,
            "spawn_kind": spawn_gate::spawn_kind(&rec).as_str(),
        }),
    );
    if discovery_agent_closed_loop(&rec, &result) {
        emit_spawn_executed(bus, &rec, &result.task_id, approved_via);
    }
    record_execution(&gate_path, &rec, &result.task_id, approved_via)?;
    mark_recommendation_spawned(state_path, &event_id, &result.task_id, rec)?;

    Ok(result)
}

/// Fire-and-forget autospawn for freshly emitted `spawn.recommended` events.
pub fn autospawn_new_recommendations(
    runner: Arc<SubagentRunner>,
    bus: Arc<SynapseBus>,
    state_path: PathBuf,
    config: KuratorConfig,
    redis_cfg: RedisConfig,
    gzmo: GzmoConfig,
    subagent_enabled: bool,
    new_recs: Vec<PendingRecommendation>,
) {
    if new_recs.is_empty() {
        return;
    }
    if !config.enabled || !config.approve_spawns_subagent || !subagent_enabled {
        return;
    }

    let project_root = project_root_from_kurator_state(&state_path);
    let gate_path = spawn_gate::default_state_path(&project_root);
    let kurator_state = load_state(&state_path);
    let gate_state = spawn_gate::load_state(&gate_path);

    for rec in new_recs {
        if !spawn_gate::autospawn_enabled_for(
            &rec,
            config.auto_spawn_on_recommend,
            config.spawn_gate.auto_spawn_discovery_fix,
        ) {
            tracing::debug!(
                event_id = %rec.event_id,
                kind = %spawn_gate::spawn_kind(&rec).as_str(),
                "Kurator autospawn skipped (disabled for this kind)"
            );
            continue;
        }

        let decision = evaluate_autospawn(&rec, &config.spawn_gate, &gate_state, &kurator_state);
        if !decision.allowed {
            emit_spawn_denied(&bus, &rec, &decision);
            let _ = record_denial(&gate_path, &rec, &decision);
            tracing::info!(
                event_id = %rec.event_id,
                code = %decision.code,
                "Kurator autospawn denied by spawn gate"
            );
            continue;
        }

        let action = obolus_action_for_rec(&rec);
        match gate::evaluate_from_config(&gzmo, action, ObolusTier::Autonomous) {
            Ok(ObolusVerdict::Allow) => {}
            Ok(ObolusVerdict::Warn { reason }) => {
                emit_obolus_warn(&bus, action, &reason);
                tracing::warn!(event_id = %rec.event_id, %reason, "obolus warn on autospawn path");
            }
            Ok(ObolusVerdict::Defer { reason }) | Ok(ObolusVerdict::Deny { reason }) => {
                emit_obolus_denied(&bus, action, &reason);
                let _ = record_denial(
                    &gate_path,
                    &rec,
                    &spawn_gate::SpawnGateDecision::deny("obolus_budget", reason.clone()),
                );
                tracing::info!(
                    event_id = %rec.event_id,
                    %reason,
                    "Kurator autospawn denied by obolus gate"
                );
                continue;
            }
            Err(e) => {
                tracing::warn!(error = %e, "obolus gate evaluation failed");
                if !gzmo.obolus_governance.fail_open_if_ledger_unreadable {
                    continue;
                }
            }
        }

        let event_id = rec.event_id.clone();
        let runner = Arc::clone(&runner);
        let bus = Arc::clone(&bus);
        let state_path = state_path.clone();
        let config = config.clone();
        let redis_cfg = redis_cfg.clone();
        let gzmo = gzmo.clone();
        tokio::spawn(async move {
            let rec = match take_pending_recommendation(&state_path, &event_id) {
                Ok(rec) => rec,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        event_id = %event_id,
                        "Kurator autospawn: recommendation unavailable"
                    );
                    return;
                }
            };
            match spawn_recommendation(
                &runner,
                &bus,
                &state_path,
                rec.clone(),
                &config,
                &redis_cfg,
                &gzmo,
                "kurator autospawn",
            )
            .await
            {
                Ok(result) => tracing::info!(
                    event_id = %event_id,
                    task_id = %result.task_id,
                    status = ?result.status,
                    "Kurator autospawn complete"
                ),
                Err(e) => {
                    if let Err(restore_err) =
                        restore_pending_recommendation(&state_path, rec)
                    {
                        tracing::warn!(
                            error = %restore_err,
                            event_id = %event_id,
                            "Kurator autospawn: failed to restore pending recommendation"
                        );
                    }
                    tracing::error!(
                        error = %e,
                        event_id = %event_id,
                        "Kurator autospawn failed"
                    );
                }
            }
        });
    }
}
