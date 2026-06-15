//! Unified skill dispatch: Rust registry first, shell bridge fallback.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::pulse::ChaosSnapshot;
use tokio::sync::mpsc;

use crate::config::GzmoConfig;
use crate::gateway::{LlmGateway, TurboQuantGateway, VllmConfig};

use super::shell_bridge::{self, ShellSkillOptions};
use super::{NestedDispatch, SkillContext, SkillOutput, SkillRegistry};

pub struct DispatchResult {
    pub output: SkillOutput,
    pub used_shell: bool,
}

pub fn skill_context<'a>(
    chaos: &'a ChaosSnapshot,
    feedback_tx: &'a mpsc::Sender<ChaosEvent>,
    args: &'a str,
    gateway: Option<&'a dyn LlmGateway>,
    router: Option<&'a crate::gateway::GatewayRouter>,
    config: &'a GzmoConfig,
    nested: NestedDispatch<'a>,
) -> SkillContext<'a> {
    SkillContext {
        chaos,
        feedback_tx,
        args,
        gateway,
        router,
        config,
        skills_dir: &config.skills.directory,
        data_dir: data_dir(config),
        nested,
    }
}

pub fn stabilize_delta_rho(config: &GzmoConfig) -> f64 {
    config
        .chaos
        .as_ref()
        .and_then(|v| v.clone().try_into().ok())
        .map(|c: gzmo_chaos::pulse::ChaosConfig| c.stabilize_delta_rho)
        .unwrap_or(-1.0)
}

pub async fn dispatch_skill(
    registry: &SkillRegistry,
    cmd: &str,
    ctx: SkillContext<'_>,
    profile: &crate::config::EngineProfileConfig,
) -> Result<DispatchResult> {
    if registry.has(cmd) {
        let output = registry.get(cmd).unwrap().execute(ctx).await?;
        return Ok(DispatchResult {
            output,
            used_shell: false,
        });
    }

    let llm_url = shell_bridge::llm_completions_url(&profile.url);
    let shell_opts = ShellSkillOptions {
        skills_dir: ctx.skills_dir,
        cmd,
        args: ctx.args,
        llm_url: Some(llm_url),
        llm_model: Some(profile.model.clone()),
        stabilize_delta_rho: stabilize_delta_rho(ctx.config),
    };
    let shell = shell_bridge::run_shell_skill(&shell_opts).await?;
    for event in &shell.events {
        let _ = ctx.feedback_tx.send(event.clone()).await;
    }
    let inject = shell.success && !shell.display.is_empty();
    Ok(DispatchResult {
        output: SkillOutput {
            display: shell.display,
            feedback: shell.events,
            inject_to_conversation: inject,
            evidence: None,
        },
        used_shell: true,
    })
}

pub fn data_dir(config: &GzmoConfig) -> &Path {
    config
        .memory
        .vault_db
        .parent()
        .unwrap_or_else(|| Path::new("data"))
}

/// Resolve `data/` from the skills directory (typically repo root via `skills/` parent).
pub fn data_dir_from_skills(skills_dir: &Path) -> PathBuf {
    skills_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("data")
}

/// Read `CHAOS_STATE.json` when present; otherwise return `fallback`.
pub fn load_live_chaos_snapshot(data_dir: &Path, fallback: &ChaosSnapshot) -> ChaosSnapshot {
    let path = data_dir.join("CHAOS_STATE.json");
    let file_snap = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<ChaosSnapshot>(&s).ok());
    match file_snap {
        Some(snap) if snap.tick >= fallback.tick => snap,
        _ => fallback.clone(),
    }
}

pub fn daemon_running() -> bool {
    crate::daemon::daemon_running()
}

pub async fn forward_feedback(
    event: ChaosEvent,
    feedback_tx: &mpsc::Sender<ChaosEvent>,
    inbox_path: &Path,
) {
    if daemon_running() {
        let _ = gzmo_chaos::feedback_ipc::append_event(inbox_path, &event);
    } else {
        let _ = feedback_tx.send(event).await;
    }
}

/// Headless skill execution (daemon dice loop, probes).
pub async fn run_registry_skill(
    registry: &SkillRegistry,
    config: &GzmoConfig,
    cmd: &str,
    args: &str,
    snap: &ChaosSnapshot,
    feedback_tx: &mpsc::Sender<ChaosEvent>,
) -> Result<SkillOutput> {
    run_registry_skill_with_gateway(registry, config, cmd, args, snap, feedback_tx, None).await
}

/// Headless skill execution with optional LLM gateway (required for wild magic generative cascades).
pub async fn run_registry_skill_with_gateway(
    registry: &SkillRegistry,
    config: &GzmoConfig,
    cmd: &str,
    args: &str,
    snap: &ChaosSnapshot,
    feedback_tx: &mpsc::Sender<ChaosEvent>,
    gateway: Option<Arc<dyn LlmGateway>>,
) -> Result<SkillOutput> {
    let profile = config.engine.active_engine();
    let nested = NestedDispatch {
        registry: Some(registry),
        profile: Some(&profile),
        depth: 0,
    };
    let ctx = skill_context(
        snap,
        feedback_tx,
        args,
        gateway.as_deref(),
        None,
        config,
        nested,
    );
    let result = dispatch_skill(registry, cmd, ctx, &profile).await?;
    Ok(result.output)
}

/// Build a gateway for headless generative skills (daemon dice loop wild magic).
pub fn headless_gateway(config: &GzmoConfig, snap: &ChaosSnapshot) -> Arc<dyn LlmGateway> {
    let profile = config.engine.active_engine();
    let gateway = TurboQuantGateway::new(VllmConfig::from(profile.clone()));
    gateway.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens);
    Arc::new(gateway)
}
