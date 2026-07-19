//! Unified skill dispatch: Rust registry first, shell bridge fallback.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::pulse::ChaosSnapshot;
use tokio::sync::mpsc;

use crate::config::GzmoConfig;
use crate::gateway::LlmGateway;

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
        return Ok(DispatchResult {
            output: registry
                .get(cmd)
                .expect("checked registry")
                .execute(ctx)
                .await?,
            used_shell: false,
        });
    }

    let options = ShellSkillOptions {
        skills_dir: ctx.skills_dir,
        cmd,
        args: ctx.args,
        llm_url: Some(shell_bridge::llm_completions_url(&profile.url)),
        llm_model: Some(profile.model.clone()),
        stabilize_delta_rho: stabilize_delta_rho(ctx.config),
    };
    let shell = shell_bridge::run_shell_skill(&options).await?;
    for event in &shell.events {
        let _ = ctx.feedback_tx.send(event.clone()).await;
    }
    Ok(DispatchResult {
        output: SkillOutput {
            display: shell.display,
            feedback: shell.events,
            inject_to_conversation: shell.success,
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

pub fn data_dir_from_skills(skills_dir: &Path) -> PathBuf {
    skills_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("data")
}

pub fn load_live_chaos_snapshot(data_dir: &Path, fallback: &ChaosSnapshot) -> ChaosSnapshot {
    let Ok(raw) = std::fs::read_to_string(data_dir.join("CHAOS_STATE.json")) else {
        return fallback.clone();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return fallback.clone();
    };
    let Some(tick) = value.get("tick").and_then(serde_json::Value::as_u64) else {
        return fallback.clone();
    };
    if tick < fallback.tick {
        return fallback.clone();
    }

    let mut snapshot = fallback.clone();
    snapshot.tick = tick;
    if let Some(value) = value.get("x").and_then(serde_json::Value::as_f64) {
        snapshot.x = value;
    }
    if let Some(value) = value.get("y").and_then(serde_json::Value::as_f64) {
        snapshot.y = value;
    }
    if let Some(value) = value.get("z").and_then(serde_json::Value::as_f64) {
        snapshot.z = value;
    }
    if let Some(value) = value.get("tension").and_then(serde_json::Value::as_f64) {
        snapshot.tension = value;
    }
    if let Some(value) = value.get("energy").and_then(serde_json::Value::as_f64) {
        snapshot.energy = value;
    }
    if let Some(value) = value.get("chaos_val").and_then(serde_json::Value::as_f64) {
        snapshot.chaos_val = value;
    }
    if let Some(value) = value
        .get("llm_temperature")
        .and_then(serde_json::Value::as_f64)
    {
        snapshot.llm_temperature = value as f32;
    }
    if let Some(value) = value
        .get("llm_max_tokens")
        .and_then(serde_json::Value::as_u64)
    {
        snapshot.llm_max_tokens = value as u32;
    }
    snapshot
}

/// Forward feedback only through this process's chaos channel.
pub async fn forward_feedback(event: ChaosEvent, feedback_tx: &mpsc::Sender<ChaosEvent>) {
    let _ = feedback_tx.send(event).await;
}

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
    let ctx = skill_context(
        snap,
        feedback_tx,
        args,
        gateway.as_deref(),
        None,
        config,
        NestedDispatch {
            registry: Some(registry),
            profile: Some(&profile),
            depth: 0,
        },
    );
    Ok(dispatch_skill(registry, cmd, ctx, &profile).await?.output)
}
