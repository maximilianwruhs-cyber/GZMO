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
use gzmo_core::kurator_spawn;
use gzmo_core::obolus::{ObolusAction, ObolusTier};
use gzmo_core::obolus::gate::preflight_allowed;
use gzmo_core::pedagogy::{build_opening, persist_socratic_dialogue, LowTensionOpening};
use gzmo_core::synapse::{EventType, SynapseBus, SynapseEvent};
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

fn is_session_active(scripts_root: &str) -> bool {
    let state_path = Path::new(scripts_root).join("data/pi-mentor-discovery/state.json");
    if let Ok(content) = std::fs::read_to_string(&state_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(status) = json.get("session_status").and_then(|s| s.as_str()) {
                return status == "active";
            }
        }
    }
    false
}

pub async fn spawn_discovery_cycle(
    scripts_root: &str,
    snap: &ChaosSnapshot,
    gzmo_root: &Path,
    opening: Option<&str>,
    oscillation_id: Option<uuid::Uuid>,
    trigger: &str,
    bus: Option<&SynapseBus>,
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

    // Determine event type based on session active status before spawning
    let session_active = is_session_active(scripts_root);
    let event_type = if session_active {
        EventType::DiscoveryCycleTriggered
    } else {
        EventType::DiscoverySessionStarted
    };

    if let Some(bus) = bus {
        bus.append(&SynapseEvent::with_envelope(
            event_type,
            gzmo_core::synapse::EventSource::GzmoCli,
            oscillation_id,
            None,
            Some(serde_json::json!({
                "trigger": trigger,
                "oscillation_id": oscillation_id.map(|u| u.to_string()),
            })),
        ));
    }

    let mut cmd = tokio::process::Command::new(&script);
    cmd.arg(trigger)
        .arg(format!("{:.1}", snap.tension))
        .arg(snap.tick.to_string())
        .current_dir(scripts_root)
        .env("GZMO_ROOT", gzmo_root)
        .env(
            "GZMO_SKILLS_ROOT",
            std::env::var("GZMO_SKILLS_ROOT").unwrap_or_else(|_| scripts_root.to_string()),
        )
        .env(
            "SCRIPTS_DIR",
            PathBuf::from(scripts_root)
                .join("scripts")
                .to_string_lossy()
                .to_string(),
        )
        .env(
            "GZMO_LOW_TENSION_OPENING",
            opening.unwrap_or(""),
        )
        .env("GZMO_DISCOVERY_TRIGGER", trigger);

    if let Ok(val) = std::env::var("DISCOVERY_AUTO_MULTI_CYCLE") {
        cmd.env("DISCOVERY_AUTO_MULTI_CYCLE", val);
    }

    if let Some(id) = oscillation_id {
        cmd.env("GZMO_OSCILLATION_ID", id.to_string());
        cmd.env("GZMO_CORRELATION_ID", id.to_string());
    }
    cmd.stdin(std::process::Stdio::null())
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

/// Obolus-gated discovery spawn (shared by low-tension watcher and pedagogy oscillation).
pub async fn spawn_discovery_if_allowed(
    config: &gzmo_core::config::GzmoConfig,
    bus: Option<&SynapseBus>,
    scripts_root: &str,
    snap: &ChaosSnapshot,
    gzmo_root: &Path,
    opening: Option<&str>,
    oscillation_id: Option<uuid::Uuid>,
    trigger: &str,
) -> std::io::Result<bool> {
    match preflight_allowed(
        config,
        ObolusAction::DiscoveryCycle,
        ObolusTier::SemiAutonomous,
        bus,
    ) {
        Ok(true) => {
            let window = std::env::var("DISCOVERY_REDUNDANCY_WINDOW")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5usize);
            let threshold = std::env::var("DISCOVERY_REDUNDANCY_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.8f64);
            let defer_redundant = std::env::var("DISCOVERY_REDUNDANCY_DEFER")
                .map(|s| s != "0")
                .unwrap_or(true);
            if defer_redundant
                && discovery_link_redundancy_exceeds(scripts_root, window, threshold)
            {
                info!(
                    window,
                    threshold,
                    "Discovery deferred: LINK fingerprint redundancy above threshold"
                );
                if let Some(bus) = bus {
                    bus.append(&SynapseEvent::with_data(
                        EventType::SpawnDenied,
                        gzmo_core::synapse::EventSource::GzmoCli,
                        serde_json::json!({
                            "reason": "discovery_deferred_redundant",
                            "window": window,
                            "threshold": threshold,
                        }),
                    ));
                }
                return Ok(false);
            }
            spawn_discovery_cycle(
                scripts_root,
                snap,
                gzmo_root,
                opening,
                oscillation_id,
                trigger,
                bus,
            )
            .await?;
            Ok(true)
        }
        Ok(false) => Ok(false),
        Err(e) => {
            warn!(error = %e, "Obolus preflight failed — skipping discovery spawn");
            Ok(false)
        }
    }
}

pub fn discovery_cycle_script(scripts_root: &str) -> PathBuf {
    PathBuf::from(scripts_root).join("scripts/auto-socratic-discovery-cycle.sh")
}

/// True when recent LINK fingerprints in the registry are mostly repeats (AUTO defer).
pub fn discovery_link_redundancy_exceeds(
    scripts_root: &str,
    window: usize,
    duplicate_ratio_threshold: f64,
) -> bool {
    if window < 3 {
        return false;
    }
    let registry = PathBuf::from(scripts_root).join("data/pi-mentor-discovery/link-registry.jsonl");
    let Ok(raw) = std::fs::read_to_string(&registry) else {
        return false;
    };
    let fingerprints: Vec<String> = raw
        .lines()
        .filter_map(|line| {
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            v.get("fingerprint")?.as_str().map(str::to_string)
        })
        .collect();
    if fingerprints.len() < window {
        return false;
    }
    let tail = &fingerprints[fingerprints.len().saturating_sub(window)..];
    let unique: std::collections::HashSet<_> = tail.iter().cloned().collect();
    let duplicate_ratio = 1.0 - (unique.len() as f64 / tail.len() as f64);
    duplicate_ratio >= duplicate_ratio_threshold
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

        let bus = SynapseBus::with_path(kurator_spawn::synapse_bus_path(&state.config));
        match preflight_allowed(
            &state.config,
            ObolusAction::DiscoveryCycle,
            ObolusTier::SemiAutonomous,
            Some(&bus),
        ) {
            Ok(true) => {}
            Ok(false) => {
                info!(
                    tension = snap.tension,
                    tick = snap.tick,
                    "Low-tension discovery deferred by ObolusGate"
                );
                continue;
            }
            Err(e) => {
                warn!(error = %e, "Obolus preflight failed — skipping low-tension discovery");
                continue;
            }
        }

        if cfg.discovery_cycle {
            match spawn_discovery_cycle(
                &scripts_root,
                &snap,
                &gzmo_root,
                Some(&opening),
                None,
                "low_tension",
                Some(&bus),
            )
            .await
            {
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
