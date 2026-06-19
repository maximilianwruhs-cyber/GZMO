//! `gzmo pedagogy oscillate start|stop|status` — inbox trigger for pedagogy chaos_val cycles.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::feedback_ipc;
use gzmo_chaos::pedagogy_oscillator::PedagogyOscillateAction;
use gzmo_core::config::GzmoConfig;
use serde_json::json;

fn data_dir(config: &GzmoConfig) -> PathBuf {
    config
        .memory
        .vault_db
        .parent()
        .unwrap_or(std::path::Path::new("data"))
        .to_path_buf()
}

fn inbox_path(config: &GzmoConfig) -> PathBuf {
    feedback_ipc::default_inbox_path(&data_dir(config))
}

fn state_path(config: &GzmoConfig) -> PathBuf {
    data_dir(config).join("CHAOS_STATE.json")
}

fn synapse_path(config: &GzmoConfig) -> PathBuf {
    data_dir(config).join("Synapse/events.jsonl")
}

fn append_oscillate(config: &GzmoConfig, action: PedagogyOscillateAction) -> Result<()> {
    let event = ChaosEvent::PedagogyOscillate { action };
    feedback_ipc::append_event(&inbox_path(config), &event)
        .with_context(|| format!("append pedagogy oscillate {:?}", action))
}

fn read_state(config: &GzmoConfig) -> Result<serde_json::Value> {
    let path = state_path(config);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).context("parse CHAOS_STATE.json")
}

fn verify_bus_complete(config: &GzmoConfig, since_line: usize) -> Result<(bool, Option<String>)> {
    let path = synapse_path(config);
    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<&str> = raw.lines().collect();
    let tail = if since_line < lines.len() {
        &lines[since_line..]
    } else {
        &[]
    };

    let mut osc_id: Option<String> = None;
    let mut has_start = false;
    let mut has_step = false;
    let mut has_complete = false;

    for line in tail {
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let et = v.get("event_type").and_then(|x| x.as_str()).unwrap_or("");
        if !et.starts_with("pedagogy.oscillation") {
            continue;
        }
        let corr = v
            .get("correlation_id")
            .and_then(|x| x.as_str())
            .map(str::to_string)
            .or_else(|| {
                v.get("data")
                    .and_then(|d| d.get("oscillation_id"))
                    .and_then(|x| x.as_str())
                    .map(str::to_string)
            });
        if osc_id.is_none() {
            osc_id = corr.clone();
        } else if corr.as_ref() != osc_id.as_ref() {
            continue;
        }
        match et {
            "pedagogy.oscillation_start" => has_start = true,
            "pedagogy.oscillation_step" => has_step = true,
            "pedagogy.oscillation_complete" => has_complete = true,
            _ => {}
        }
    }

    let ok = has_start && has_step && has_complete && osc_id.is_some();
    Ok((ok, osc_id))
}

pub async fn run(config: &GzmoConfig, args: &[String]) -> Result<()> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("");
    match sub {
        "start" => {
            if !config.pedagogy.tension_oscillation.enabled {
                bail!(
                    "pedagogy.tension_oscillation.enabled is false — set enabled = true in gzmo.toml"
                );
            }
            let wait = args.iter().any(|a| a == "--wait");
            let strict = args.iter().any(|a| a == "--strict");
            let json_out = args.iter().any(|a| a == "--json");
            let events_before = std::fs::read_to_string(synapse_path(config))
                .map(|s| s.lines().count())
                .unwrap_or(0);

            append_oscillate(config, PedagogyOscillateAction::Start)?;
            if !wait {
                if json_out {
                    println!("{}", json!({"status": "queued", "action": "start"}));
                } else {
                    println!("Pedagogy oscillation start queued (daemon inbox)");
                }
                return Ok(());
            }

            let deadline = std::time::Instant::now() + Duration::from_secs(600);
            let mut last_step = 0u32;
            let mut oscillation_id: Option<String> = None;
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let state = read_state(config)?;
                let active = state
                    .get("pedagogy_oscillation_active")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let step = state
                    .get("pedagogy_step")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let chaos_val = state.get("chaos_val").and_then(|v| v.as_f64());
                let target = state.get("pedagogy_target").and_then(|v| v.as_f64());
                if let Some(id) = state.get("oscillation_id").and_then(|v| v.as_str()) {
                    oscillation_id = Some(id.to_string());
                }
                if step != last_step && step > 0 {
                    last_step = step;
                }

                if strict {
                    let (bus_complete, bus_osc_id) = verify_bus_complete(config, events_before)?;
                    if bus_complete {
                        let summary = json!({
                            "status": "complete",
                            "final_chaos_val": chaos_val,
                            "steps_observed": last_step.max(step),
                            "oscillation_id": bus_osc_id.or(oscillation_id),
                            "bus_complete": true,
                        });
                        if json_out {
                            println!("{}", serde_json::to_string_pretty(&summary)?);
                        } else {
                            println!("Pedagogy oscillation complete (strict bus trail verified)");
                        }
                        return Ok(());
                    }
                } else if !active && last_step > 0 {
                    let summary = json!({
                        "status": "complete",
                        "final_chaos_val": chaos_val,
                        "steps_observed": last_step,
                        "oscillation_id": oscillation_id,
                        "bus_complete": true,
                    });
                    if json_out {
                        println!("{}", serde_json::to_string_pretty(&summary)?);
                    } else {
                        println!("Pedagogy oscillation complete (steps={last_step})");
                        if let Some(id) = summary.get("oscillation_id").and_then(|v| v.as_str()) {
                            println!("oscillation_id={id}");
                        }
                    }
                    return Ok(());
                }

                if std::time::Instant::now() > deadline {
                    if strict {
                        let (bus_complete, bus_osc_id) = verify_bus_complete(config, events_before)?;
                        bail!(
                            "timeout waiting for strict bus complete (active={active}, step={step}, last_step={last_step}, bus_complete={bus_complete}, oscillation_id={:?})",
                            bus_osc_id.or(oscillation_id)
                        );
                    }
                    bail!("timeout waiting for oscillation complete (active={active}, step={step}, chaos_val={chaos_val:?}, target={target:?})");
                }
            }
        }
        "stop" => {
            append_oscillate(config, PedagogyOscillateAction::Stop)?;
            println!("Pedagogy oscillation stop queued");
            Ok(())
        }
        "status" => {
            let state = read_state(config)?;
            let active = state
                .get("pedagogy_oscillation_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let osc = state.get("oscillation_id").and_then(|v| v.as_str());
            println!(
                "oscillation_active={} step={} target={:?} chaos_val={:?} oscillation_id={osc:?}",
                active,
                state.get("pedagogy_step").and_then(|v| v.as_u64()).unwrap_or(0),
                state.get("pedagogy_target"),
                state.get("chaos_val"),
            );
            Ok(())
        }
        _ => bail!(
            "Usage: gzmo pedagogy oscillate start [--wait] [--strict] [--json] | stop | status\n       gzmo pedagogy certify --oscillation-id UUID [--learning-verified true|false]"
        ),
    }
}
