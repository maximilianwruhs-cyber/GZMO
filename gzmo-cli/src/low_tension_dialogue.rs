//! Autonomous Socratic dialogue when chaos tension drops very low.
//!
//! When `discovery_cycle` is enabled (default), spawns `auto-socratic-discovery-cycle.sh`
//! in gzmo_skills — full pillar probe + cycle report. Otherwise falls back to bare
//! `teach_autonomous` + JSONL log.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gzmo_chaos::chaos::Phase;
use gzmo_chaos::pulse::ChaosSnapshot;
use gzmo_core::config::LowTensionDialogueConfig;
use gzmo_core::pedagogy::{build_opening, persist_socratic_dialogue, LowTensionOpening};
use std::sync::Arc;
use gzmo_core::tools::ToolRegistry;
use tokio::sync::watch;
use tracing::{error, info, warn};

use crate::mentor_ipc::{self, MentorServerState};

pub fn format_opening(template: &str, snap: &ChaosSnapshot) -> String {
    template
        .replace("{tension}", &format!("{:.1}", snap.tension))
        .replace("{tick}", &snap.tick.to_string())
        .replace("{phase}", &format!("{}", snap.phase))
}

pub fn crossed_below_threshold(prev: f64, current: f64, threshold: f64) -> bool {
    prev >= threshold && current < threshold
}

/// True when chaos stays calm: Idle phase and tension below threshold.
pub fn is_calm_plateau(phase: Phase, tension: f64, threshold: f64) -> bool {
    phase == Phase::Idle && tension < threshold
}

/// Secondary trigger: N consecutive watcher polls in calm plateau (5s each).
pub fn calm_polls_trigger(calm_polls: u64, idle_threshold: Option<u64>) -> bool {
    idle_threshold.is_some_and(|n| n > 0 && calm_polls >= n)
}

pub fn should_fire_low_tension(crossed: bool, calm_polls: u64, idle_threshold: Option<u64>) -> bool {
    crossed || calm_polls_trigger(calm_polls, idle_threshold)
}

pub fn discovery_cycle_script(scripts_root: &str) -> PathBuf {
    PathBuf::from(scripts_root).join("scripts/auto-socratic-discovery-cycle.sh")
}

pub async fn spawn_discovery_cycle(
    scripts_root: &str,
    snap: &ChaosSnapshot,
    gzmo_root: &Path,
    opening: Option<&str>,
) -> std::io::Result<()> {
    let script = PathBuf::from(scripts_root).join("scripts/auto-socratic-discovery-cycle.sh");
    if !script.is_file() {
        warn!(path = %script.display(), "Auto discovery script missing");
        return Ok(());
    }

    let log_dir = PathBuf::from(scripts_root).join("data/pi-mentor-discovery/logs");
    let _ = tokio::fs::create_dir_all(&log_dir).await;
    let stderr_log = log_dir.join("auto-socratic-spawn.log");

    let stderr = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&stderr_log)?;

    let mut cmd = tokio::process::Command::new(&script);
    cmd.arg("low_tension")
        .arg(format!("{:.1}", snap.tension))
        .arg(snap.tick.to_string())
        .env("GZMO_ROOT", gzmo_root)
        .env(
            "GZMO_SKILLS_ROOT",
            std::env::var("GZMO_SKILLS_ROOT").unwrap_or_else(|_| scripts_root.to_string()),
        )
        .env(
            "GZMO_LOW_TENSION_OPENING",
            opening.unwrap_or(""),
        )
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(stderr);

    let child = cmd.spawn()?;
    info!(
        pid = child.id(),
        script = %script.display(),
        tension = snap.tension,
        tick = snap.tick,
        "Spawned AUTO Socratic discovery cycle (pillar probe + report)"
    );
    Ok(())
}

pub async fn run_low_tension_watcher(
    state: std::sync::Arc<MentorServerState>,
    snapshot_rx: watch::Receiver<ChaosSnapshot>,
    cfg: LowTensionDialogueConfig,
    scripts_root: String,
    log_path: PathBuf,
    gzmo_root: PathBuf,
    tools: Option<Arc<ToolRegistry>>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let mut prev_tension = snapshot_rx.borrow().tension;
    let mut calm_polls: u64 = 0;
    let mut last_fire: Option<Instant> = None;

    info!(
        threshold = cfg.threshold,
        cooldown_secs = cfg.cooldown_secs,
        discovery_cycle = cfg.discovery_cycle,
        idle_ticks_threshold = ?cfg.idle_ticks_threshold,
        scripts_root = %scripts_root,
        "Low-tension Socratic watcher online"
    );

    loop {
        interval.tick().await;
        if !cfg.enabled {
            continue;
        }

        let snap = snapshot_rx.borrow().clone();
        let crossed = crossed_below_threshold(prev_tension, snap.tension, cfg.threshold);
        prev_tension = snap.tension;

        if is_calm_plateau(snap.phase, snap.tension, cfg.threshold) {
            calm_polls = calm_polls.saturating_add(1);
        } else {
            calm_polls = 0;
        }

        if !should_fire_low_tension(crossed, calm_polls, cfg.idle_ticks_threshold) {
            continue;
        }

        if let Some(last) = last_fire {
            if last.elapsed() < Duration::from_secs(cfg.cooldown_secs) {
                continue;
            }
        }

        {
            let mut pedagogy = state.pedagogy.lock().await;
            if let Err(e) = pedagogy.reload_from_disk().await {
                warn!(error = %e, "Low-tension watcher: session reload failed");
                continue;
            }
            if pedagogy.session.ops_mode {
                continue;
            }
            if !pedagogy.session.auto_triggers_enabled {
                continue;
            }
        }

        let trigger = if crossed {
            "crossed_below"
        } else {
            "calm_plateau"
        };
        info!(
            tension = snap.tension,
            tick = snap.tick,
            trigger,
            calm_polls,
            "Low tension — AUTO Socratic trigger"
        );

        let opening_ctx = resolve_opening(&state, tools.as_deref(), &snap, &cfg).await;
        let opening = opening_ctx.prompt.clone();

        if cfg.discovery_cycle {
            match spawn_discovery_cycle(&scripts_root, &snap, &gzmo_root, Some(&opening)).await {
                Ok(()) => {
                    last_fire = Some(Instant::now());
                    calm_polls = 0;
                    if let Some(ref tools) = tools {
                        let learner_id = state.config.pedagogy.active_learner_id.clone()
                            .unwrap_or_else(|| "operator".into());
                        let _ = persist_socratic_dialogue(
                            tools.as_ref(),
                            &learner_id,
                            &opening_ctx,
                            None,
                            &snap,
                            trigger,
                        )
                        .await;
                    }
                }
                Err(e) => error!(error = %e, "Failed to spawn AUTO discovery cycle"),
            }
            continue;
        }

        match mentor_ipc::teach_autonomous(&state, &opening).await {
            Ok(resp) if resp.ok => {
                last_fire = Some(Instant::now());
                calm_polls = 0;
                if let Some(response) = resp.response.as_deref() {
                    if let Err(e) = append_log(&log_path, &snap, &opening, response).await {
                        warn!(error = %e, "Could not append low-tension dialogue log");
                    }
                    if let Some(ref tools) = tools {
                        let learner_id = state.config.pedagogy.active_learner_id.clone()
                            .unwrap_or_else(|| "operator".into());
                        let _ = persist_socratic_dialogue(
                            tools.as_ref(),
                            &learner_id,
                            &opening_ctx,
                            Some(response),
                            &snap,
                            trigger,
                        )
                        .await;
                    }
                    info!(
                        preview = %response.chars().take(120).collect::<String>(),
                        "Low-tension Socratic dialogue complete (bare teach)"
                    );
                }
            }
            Ok(resp) => {
                warn!(
                    error = ?resp.error,
                    "Low-tension Socratic dialogue failed"
                );
            }
            Err(e) => error!(error = %e, "Low-tension Socratic dialogue error"),
        }
    }
}

async fn resolve_opening(
    state: &MentorServerState,
    tools: Option<&ToolRegistry>,
    snap: &ChaosSnapshot,
    cfg: &LowTensionDialogueConfig,
) -> LowTensionOpening {
    let pedagogy = state.pedagogy.lock().await;
    match build_opening(&state.config, &pedagogy.learner_profile, snap, tools).await {
        Ok(opening) => opening,
        Err(e) => {
            warn!(error = %e, "KG-aware opening failed; using template");
            LowTensionOpening {
                prompt: format_opening(&cfg.opening_template, snap),
                concept_ids: vec![],
                concept_titles: vec![],
            }
        }
    }
}

async fn append_log(
    path: &PathBuf,
    snap: &ChaosSnapshot,
    opening: &str,
    response: &str,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let line = serde_json::json!({
        "ts": chrono::Utc::now().to_rfc3339(),
        "tension": snap.tension,
        "tick": snap.tick,
        "phase": snap.phase.to_string(),
        "opening": opening,
        "response": response,
    });
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    use tokio::io::AsyncWriteExt;
    file.write_all(line.to_string().as_bytes()).await?;
    file.write_all(b"\n").await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_trigger_only_on_crossing() {
        assert!(crossed_below_threshold(20.0, 10.0, 15.0));
        assert!(!crossed_below_threshold(10.0, 8.0, 15.0));
        assert!(!crossed_below_threshold(20.0, 20.0, 15.0));
    }

    #[test]
    fn calm_plateau_requires_idle_phase() {
        assert!(is_calm_plateau(Phase::Idle, 17.0, 18.0));
        assert!(!is_calm_plateau(Phase::Build, 17.0, 18.0));
        assert!(!is_calm_plateau(Phase::Idle, 19.0, 18.0));
    }

    #[test]
    fn idle_poll_trigger_after_n_calm_samples() {
        assert!(!calm_polls_trigger(119, Some(120)));
        assert!(calm_polls_trigger(120, Some(120)));
        assert!(!calm_polls_trigger(200, None));
    }

    #[test]
    fn fire_on_cross_or_plateau() {
        assert!(should_fire_low_tension(true, 0, Some(120)));
        assert!(should_fire_low_tension(false, 120, Some(120)));
        assert!(!should_fire_low_tension(false, 10, Some(120)));
    }

    #[test]
    fn discovery_script_path_under_skills_root() {
        assert!(discovery_cycle_script("/tmp/gzmo_skills")
            .to_string_lossy()
            .ends_with("scripts/auto-socratic-discovery-cycle.sh"));
    }
}
