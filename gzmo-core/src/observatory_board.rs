//! Structured Observatory health LED snapshots for operator TUIs.
//!
//! Same sources as `gzmo status` — user systemd units + `collect_health_probes`,
//! plus expected-offline honesty and the OKForge wiki plane.

use serde::Serialize;

use crate::config::GzmoConfig;
use crate::health::{collect_health_probes, ProbeResult};
use crate::metabolism::read_wiki_plane_summary;

/// Traffic-light state for a single LED.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HealthLed {
    pub id: String,
    pub label: String,
    pub state: LedState,
    pub detail: String,
}

/// Full board snapshot for `gzmo observatory`.
#[derive(Debug, Clone, Serialize)]
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

    /// Knowledge-plane LEDs that must not be DOWN for `gzmo observatory --json` exit 0
    /// when `[wiki] backend = okforge`.
    pub fn knowledge_plane_down(&self) -> Vec<&HealthLed> {
        self.all_leds()
            .filter(|l| {
                matches!(
                    l.id.as_str(),
                    "okforge.service" | "okforge_http" | "wiki_push"
                ) && l.state == LedState::Down
            })
            .collect()
    }

    /// Scriptable snapshot (`gzmo observatory --json`).
    pub fn snapshot_json(&self) -> serde_json::Value {
        let (up, degraded, down, unknown) = self.counts();
        serde_json::json!({
            "schema": "gzmo.observatory.board/v1",
            "instance": self.instance,
            "engine_mode": self.engine_mode,
            "engine_model": self.engine_model,
            "counts": { "up": up, "degraded": degraded, "down": down, "unknown": unknown },
            "knowledge_plane_ok": self.knowledge_plane_down().is_empty(),
            "units": self.units,
            "probes": self.probes,
        })
    }
}

/// Map systemd `is-active` output to an LED state (raw — no expected-offline policy).
pub fn led_from_systemd(unit: &str, is_active: &str) -> HealthLed {
    let (state, detail) = match is_active {
        "active" => (LedState::Up, "active".to_string()),
        "inactive" | "failed" => (LedState::Down, is_active.to_string()),
        "activating" | "deactivating" | "reloading" => (LedState::Degraded, is_active.to_string()),
        other => (LedState::Unknown, other.to_string()),
    };
    HealthLed {
        id: unit.to_string(),
        label: unit.to_string(),
        state,
        detail,
    }
}

/// Map a workstation unit with expected-offline honesty (telescope vs living writer).
pub fn led_from_unit_policy(unit: &str, is_active: &str) -> HealthLed {
    match (unit, is_active) {
        ("gzmo-serve.service", "active") => HealthLed {
            id: unit.into(),
            label: unit.into(),
            state: LedState::Down,
            detail: "dual-writer risk — stop workstation gzmo-serve".into(),
        },
        ("gzmo-serve.service", "inactive") => HealthLed {
            id: unit.into(),
            label: unit.into(),
            state: LedState::Up,
            detail: "expected-offline — living writer is not this host".into(),
        },
        ("gzmo-scheduler.service", "active") => HealthLed {
            id: unit.into(),
            label: unit.into(),
            state: LedState::Degraded,
            detail: "lab scheduler active — confirm not a second overnight writer".into(),
        },
        ("gzmo-scheduler.service", "inactive") => HealthLed {
            id: unit.into(),
            label: unit.into(),
            state: LedState::Up,
            detail: "expected-offline — lab scheduler off by default".into(),
        },
        ("llama-prime.service", "inactive") => HealthLed {
            id: unit.into(),
            label: unit.into(),
            state: LedState::Unknown,
            detail: "unit inactive — judge LLM via health probes, not this unit".into(),
        },
        _ => led_from_systemd(unit, is_active),
    }
}

/// Map an OKForge HTTP probe (any non-5xx means the forge answered).
pub fn led_from_okforge_http(result: Result<(u16, String), String>) -> HealthLed {
    match result {
        Ok((status, path)) if status < 500 => HealthLed {
            id: "okforge_http".into(),
            label: "okforge_http".into(),
            state: LedState::Up,
            detail: format!("HTTP {status} {path}"),
        },
        Ok((status, path)) => HealthLed {
            id: "okforge_http".into(),
            label: "okforge_http".into(),
            state: LedState::Down,
            detail: format!("HTTP {status} {path}"),
        },
        Err(e) => HealthLed {
            id: "okforge_http".into(),
            label: "okforge_http".into(),
            state: LedState::Down,
            detail: e,
        },
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
        units.push(led_from_unit_policy(name, &state));
    }

    let probes_raw = collect_health_probes(config, None).await;
    let mut probes: Vec<HealthLed> = probes_raw.iter().map(led_from_probe).collect();

    let wiki_okforge = config.wiki.enabled && config.wiki.backend == "okforge";
    if wiki_okforge {
        let url = config
            .wiki
            .okforge
            .as_ref()
            .map(|c| c.url.as_str())
            .unwrap_or("http://127.0.0.1:3000");
        probes.push(led_from_okforge_http(
            crate::okforge_client::probe_observatory(url).await,
        ));
        probes.push(led_from_wiki_plane(read_wiki_plane_summary(config)));
    }

    // Spark lineage (Experience B) — operator LED, not a network probe
    match crate::spark_lineage::load_spark_lineage(&config.memory.vault_db) {
        Some(card) if card.experience_b_ok() => probes.push(HealthLed {
            id: "spark_lineage".into(),
            label: "spark_lineage".into(),
            state: LedState::Up,
            detail: card.observatory_detail(),
        }),
        Some(card) => probes.push(HealthLed {
            id: "spark_lineage".into(),
            label: "spark_lineage".into(),
            state: LedState::Degraded,
            detail: card.observatory_detail(),
        }),
        None => probes.push(HealthLed {
            id: "spark_lineage".into(),
            label: "spark_lineage".into(),
            state: LedState::Down,
            detail: "no last-spark-report.json under vault/spark/".into(),
        }),
    }

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
                detail: format!(
                    "enabled but empty dir {}",
                    config.workflow_skills.dir.display()
                ),
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

fn led_from_wiki_plane(wiki: crate::metabolism::WikiPlaneSummary) -> HealthLed {
    let (state, detail) = match wiki.healthy {
        Some(true) => (LedState::Up, wiki.detail),
        Some(false) => (LedState::Down, wiki.detail),
        None if wiki.detail.contains("no wiki-push-latest") => (
            LedState::Degraded,
            "no wiki-push-latest.json yet — run gzmo wiki push or living satellite".into(),
        ),
        None => (LedState::Degraded, wiki.detail),
    };
    HealthLed {
        id: "wiki_push".into(),
        label: "wiki_push".into(),
        state,
        detail,
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
    fn serve_inactive_is_expected_offline_up() {
        let led = led_from_unit_policy("gzmo-serve.service", "inactive");
        assert_eq!(led.state, LedState::Up);
        assert!(led.detail.contains("expected-offline"));
    }

    #[test]
    fn serve_active_is_dual_writer_down() {
        let led = led_from_unit_policy("gzmo-serve.service", "active");
        assert_eq!(led.state, LedState::Down);
        assert!(led.detail.contains("dual-writer"));
    }

    #[test]
    fn scheduler_inactive_is_expected_offline_up() {
        let led = led_from_unit_policy("gzmo-scheduler.service", "inactive");
        assert_eq!(led.state, LedState::Up);
    }

    #[test]
    fn llama_prime_inactive_is_unknown_not_down() {
        let led = led_from_unit_policy("llama-prime.service", "inactive");
        assert_eq!(led.state, LedState::Unknown);
    }

    #[test]
    fn okforge_http_401_is_up() {
        let led = led_from_okforge_http(Ok((401, "/observatory".into())));
        assert_eq!(led.state, LedState::Up);
    }

    #[test]
    fn okforge_http_unreachable_is_down() {
        let led = led_from_okforge_http(Err("connection refused".into()));
        assert_eq!(led.state, LedState::Down);
    }

    #[test]
    fn snapshot_json_has_schema() {
        let board = HealthLedBoard {
            instance: "legacy".into(),
            engine_mode: "LOCAL".into(),
            engine_model: "x".into(),
            units: vec![led_from_unit_policy("gzmo-serve.service", "inactive")],
            probes: vec![led_from_okforge_http(Ok((200, "/observatory".into())))],
        };
        let v = board.snapshot_json();
        assert_eq!(v["schema"], "gzmo.observatory.board/v1");
        assert_eq!(v["knowledge_plane_ok"], true);
        assert_eq!(v["counts"]["up"], 2);
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
