//! Structured Observatory health LED snapshots for operator TUIs.
//!
//! Same sources as `gzmo status` — user systemd units + `collect_health_probes`.

use crate::config::GzmoConfig;
use crate::health::{collect_health_probes, ProbeResult};

/// Traffic-light state for a single LED.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedState {
    Up,
    Degraded,
    Down,
    Unknown,
}

impl LedState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Up => "UP",
            Self::Degraded => "DEGRADED",
            Self::Down => "DOWN",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// One ecosystem LED (service unit or dependency probe).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthLed {
    pub id: String,
    pub label: String,
    pub state: LedState,
    pub detail: String,
}

/// Full board snapshot for `gzmo observatory`.
#[derive(Debug, Clone)]
pub struct HealthLedBoard {
    pub instance: String,
    pub engine_mode: String,
    pub engine_model: String,
    pub units: Vec<HealthLed>,
    pub probes: Vec<HealthLed>,
}

impl HealthLedBoard {
    pub fn all_leds(&self) -> impl Iterator<Item = &HealthLed> {
        self.units.iter().chain(self.probes.iter())
    }

    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let mut up = 0;
        let mut deg = 0;
        let mut down = 0;
        let mut unk = 0;
        for led in self.all_leds() {
            match led.state {
                LedState::Up => up += 1,
                LedState::Degraded => deg += 1,
                LedState::Down => down += 1,
                LedState::Unknown => unk += 1,
            }
        }
        (up, deg, down, unk)
    }
}

/// Map systemd `is-active` output to an LED state.
pub fn led_from_systemd(unit: &str, is_active: &str) -> HealthLed {
    let (state, detail) = match is_active {
        "active" => (LedState::Up, "active".to_string()),
        "inactive" | "failed" => (LedState::Down, is_active.to_string()),
        "activating" | "deactivating" | "reloading" => {
            (LedState::Degraded, is_active.to_string())
        }
        other => (LedState::Unknown, other.to_string()),
    };
    HealthLed {
        id: unit.to_string(),
        label: unit.to_string(),
        state,
        detail,
    }
}

/// Map a health probe to an LED state.
pub fn led_from_probe(probe: &ProbeResult) -> HealthLed {
    let detail_l = probe.detail.to_lowercase();
    let state = if probe.ok {
        if detail_l.contains("disabled") {
            LedState::Degraded
        } else {
            LedState::Up
        }
    } else {
        LedState::Down
    };
    HealthLed {
        id: probe.name.to_string(),
        label: probe.name.to_string(),
        state,
        detail: probe.detail.clone(),
    }
}

async fn user_systemd_unit(unit: &str) -> String {
    match tokio::process::Command::new("systemctl")
        .args(["--user", "is-active", unit])
        .output()
        .await
    {
        Ok(o) if o.status.success() => "active".into(),
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                "inactive".into()
            } else {
                s
            }
        }
        Err(_) => "unknown".into(),
    }
}

/// Collect the Observatory health LED board (units + probes).
pub async fn collect_health_led_board(config: &GzmoConfig) -> HealthLedBoard {
    let instance = std::env::var("GZMO_INSTANCE").unwrap_or_else(|_| "legacy".into());
    let active = config.engine.active_engine();
    let mode = format!("{:?}", config.engine.active_mode).to_uppercase();

    let unit_names = [
        "llama-prime.service",
        "gzmo-serve.service",
        "gzmo-scheduler.service",
        "okforge.service",
    ];
    let mut units = Vec::with_capacity(unit_names.len());
    for name in unit_names {
        let state = user_systemd_unit(name).await;
        units.push(led_from_systemd(name, &state));
    }

    let probes_raw = collect_health_probes(config, None).await;
    let mut probes: Vec<HealthLed> = probes_raw.iter().map(led_from_probe).collect();

    // Workflow pack + handoff pointer (operator LED, not a network probe)
    if config.workflow_skills.enabled {
        match crate::workflow_skills::WorkflowSkillIndex::load_from_dir(
            &config.workflow_skills.dir,
            config.workflow_skills.max_active,
            config.workflow_skills.model_can_activate,
            &config.workflow_skills.handoff_dir,
        ) {
            Ok(idx) if !idx.is_empty() => {
                let handoff = idx
                    .latest_handoff()
                    .map(|p| format!("last={}", p.display()))
                    .unwrap_or_else(|| "no handoff yet".into());
                probes.push(HealthLed {
                    id: "workflow_skills".into(),
                    label: "workflow_skills".into(),
                    state: LedState::Up,
                    detail: format!("{} skills; {handoff}", idx.len()),
                });
            }
            Ok(_) => probes.push(HealthLed {
                id: "workflow_skills".into(),
                label: "workflow_skills".into(),
                state: LedState::Degraded,
                detail: format!("enabled but empty dir {}", config.workflow_skills.dir.display()),
            }),
            Err(e) => probes.push(HealthLed {
                id: "workflow_skills".into(),
                label: "workflow_skills".into(),
                state: LedState::Down,
                detail: e.to_string(),
            }),
        }
    }

    HealthLedBoard {
        instance,
        engine_mode: mode,
        engine_model: active.model.clone(),
        units,
        probes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_active_is_up() {
        let led = led_from_systemd("okforge.service", "active");
        assert_eq!(led.state, LedState::Up);
    }

    #[test]
    fn systemd_inactive_is_down() {
        let led = led_from_systemd("gzmo-serve.service", "inactive");
        assert_eq!(led.state, LedState::Down);
    }

    #[test]
    fn probe_ok_is_up() {
        let p = ProbeResult {
            name: "llm",
            ok: true,
            detail: "model → http://x/models".into(),
        };
        assert_eq!(led_from_probe(&p).state, LedState::Up);
    }

    #[test]
    fn probe_disabled_is_degraded() {
        let p = ProbeResult {
            name: "embeddings",
            ok: true,
            detail: "disabled in config".into(),
        };
        assert_eq!(led_from_probe(&p).state, LedState::Degraded);
    }

    #[test]
    fn probe_fail_is_down() {
        let p = ProbeResult {
            name: "qdrant",
            ok: false,
            detail: "unreachable".into(),
        };
        assert_eq!(led_from_probe(&p).state, LedState::Down);
    }
}
