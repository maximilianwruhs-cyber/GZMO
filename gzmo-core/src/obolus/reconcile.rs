//! Reconcile external Prime clients (Pi Synapse, llama-server log tail).

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::config::GzmoConfig;
use crate::obolus::efficiency::compute_from_sources;
use crate::obolus::gate::{self, emit_budget_tick, load_balance_since};
use crate::obolus::ledger::{LedgerEntry, LedgerSource, ObolusLedger};
use crate::synapse::{EventSource, EventType, SynapseBus, SynapseEvent};

const SYNAPSE_STATE: &str = "data/Obolus/reconcile-synapse.state.json";
const LOG_STATE: &str = "data/Obolus/reconcile-log.state.json";
const EFFICIENCY_TICK_STATE: &str = "data/Obolus/efficiency-tick.state.json";
const BUDGET_TICK_STATE: &str = "data/Obolus/budget-tick.state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EfficiencyTickState {
    last_hour_bucket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ByteOffsetState {
    byte_offset: u64,
}

fn load_offset(path: &Path) -> ByteOffsetState {
    if !path.exists() {
        return ByteOffsetState::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_offset(path: &Path, state: &ByteOffsetState) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

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

fn project_data_dir(config: &GzmoConfig) -> PathBuf {
    config
        .memory
        .vault_db
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("data"))
}

pub async fn run_tick(config: &GzmoConfig, ledger: &ObolusLedger) -> Result<()> {
    let pi = reconcile_synapse_pi(config, ledger)?;
    let log_n = reconcile_llama_log(config, ledger)?;
    if config.obolus_analytics.efficiency_tick_enabled {
        maybe_emit_efficiency_tick(config, ledger)?;
    }
    if config.obolus_governance.enabled {
        maybe_emit_budget_tick(config)?;
    }
    if pi > 0 || log_n > 0 {
        info!(pi_events = pi, log_lines = log_n, "obolus reconcile tick");
    }
    Ok(())
}

fn maybe_emit_efficiency_tick(config: &GzmoConfig, ledger: &ObolusLedger) -> Result<()> {
    let hour_bucket = chrono::Utc::now().format("%Y%m%d%H").to_string();
    let state_path = project_data_dir(config).join(EFFICIENCY_TICK_STATE);
    let mut state: EfficiencyTickState = if state_path.exists() {
        std::fs::read_to_string(&state_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        EfficiencyTickState::default()
    };
    if state.last_hour_bucket.as_deref() == Some(hour_bucket.as_str()) {
        return Ok(());
    }

    let since = chrono::Utc::now() - chrono::Duration::hours(24);
    let entries = ObolusLedger::read_since(since, ledger.path())?;
    let bus = synapse_bus_path(config);
    let rollups = compute_from_sources(
        &entries,
        &bus,
        since,
        config.obolus_analytics.prime_context_tokens,
        config.obolus_analytics.tokens_per_obl,
    )?;

    let bus_handle = SynapseBus::with_path(synapse_bus_path(config));
    bus_handle.append(&SynapseEvent::with_data(
        EventType::ObolusEfficiencyTick,
        EventSource::GzmoDaemon,
        serde_json::json!({
            "hour_bucket": hour_bucket,
            "rollups": rollups,
        }),
    ));

    state.last_hour_bucket = Some(hour_bucket);
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&state_path, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

fn maybe_emit_budget_tick(config: &GzmoConfig) -> Result<()> {
    let hour_bucket = chrono::Utc::now().format("%Y%m%d%H").to_string();
    let state_path = project_data_dir(config).join(BUDGET_TICK_STATE);
    let mut state: EfficiencyTickState = if state_path.exists() {
        std::fs::read_to_string(&state_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        EfficiencyTickState::default()
    };
    if state.last_hour_bucket.as_deref() == Some(hour_bucket.as_str()) {
        return Ok(());
    }

    let since = chrono::Utc::now() - chrono::Duration::hours(1);
    let balance = load_balance_since(config, since)?;
    let bus_handle = SynapseBus::with_path(synapse_bus_path(config));
    emit_budget_tick(&bus_handle, &balance);

    state.last_hour_bucket = Some(hour_bucket);
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&state_path, serde_json::to_string_pretty(&state)?)?;
    Ok(())
}

fn reconcile_synapse_pi(config: &GzmoConfig, ledger: &ObolusLedger) -> Result<usize> {
    let bus = synapse_bus_path(config);
    if !bus.exists() {
        return Ok(0);
    }
    let state_path = project_data_dir(config).join(SYNAPSE_STATE);
    let mut state = load_offset(&state_path);
    let mut file = File::open(&bus).with_context(|| format!("open {}", bus.display()))?;
    let len = file.metadata()?.len();
    if state.byte_offset > len {
        state.byte_offset = 0;
    }
    file.seek(SeekFrom::Start(state.byte_offset))?;
    let reader = BufReader::new(file);
    let mut count = 0usize;

    for line in reader.lines() {
        let line = line?;
        state.byte_offset += line.len() as u64 + 1;
        if line.trim().is_empty() {
            continue;
        }
        let event: SynapseEvent = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if event.event_type != EventType::QuestComplete {
            continue;
        }
        if event.source != EventSource::PiAgent {
            continue;
        }
        let Some(data) = &event.data else {
            continue;
        };
        let input = data
            .get("inputTokens")
            .or_else(|| data.get("input_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let output = data
            .get("outputTokens")
            .or_else(|| data.get("output_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let dedup_key = format!("synapse:{}", event.id);
        if ObolusLedger::dedup_key_exists(ledger.path(), &dedup_key)? {
            continue;
        }
        ledger.record(LedgerEntry {
            ts: event.timestamp,
            source: LedgerSource::SynapsePi,
            process: "pi_agent".into(),
            task_kind: None,
            caller: "synapse_reconcile".into(),
            input_tokens: input,
            output_tokens: output,
            total_tokens: input.saturating_add(output),
            latency_ms: 0,
            ok: true,
            model: None,
            correlation_id: event.correlation_id.map(|u| u.to_string()),
            action_id: Some(event.id.to_string()),
            dedup_key: Some(dedup_key),
        });
        count += 1;
    }

    save_offset(&state_path, &state)?;
    Ok(count)
}

fn reconcile_llama_log(config: &GzmoConfig, ledger: &ObolusLedger) -> Result<usize> {
    let log_path = config.obolus_analytics.llama_log_path.trim();
    if log_path.is_empty() {
        return Ok(0);
    }
    let path = PathBuf::from(log_path);
    if !path.exists() {
        debug!(path = %path.display(), "obolus llama log path missing");
        return Ok(0);
    }
    let state_path = project_data_dir(config).join(LOG_STATE);
    let mut state = load_offset(&state_path);
    let mut file = File::open(&path)?;
    let len = file.metadata()?.len();
    if state.byte_offset > len {
        state.byte_offset = 0;
    }
    file.seek(SeekFrom::Start(state.byte_offset))?;
    let reader = BufReader::new(file);
    let mut count = 0usize;

    for line in reader.lines() {
        let line = line?;
        state.byte_offset += line.len() as u64 + 1;
        let (input, output) = parse_llama_log_line(&line);
        if input == 0 && output == 0 {
            continue;
        }
        let dedup_key = format!("log:{:x}", {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            line.hash(&mut h);
            h.finish()
        });
        if ObolusLedger::dedup_key_exists(ledger.path(), &dedup_key)? {
            continue;
        }
        ledger.record(LedgerEntry {
            ts: Utc::now(),
            source: LedgerSource::LlamaLog,
            process: "unknown_client".into(),
            task_kind: None,
            caller: "llama_log_reconcile".into(),
            input_tokens: input,
            output_tokens: output,
            total_tokens: input.saturating_add(output),
            latency_ms: 0,
            ok: true,
            model: None,
            correlation_id: None,
            action_id: None,
            dedup_key: Some(dedup_key),
        });
        count += 1;
    }

    save_offset(&state_path, &state)?;
    Ok(count)
}

/// Parse llama.cpp server log lines for token counts.
fn parse_llama_log_line(line: &str) -> (u64, u64) {
    let mut input = 0u64;
    let mut output = 0u64;
    if line.contains("prompt eval") || line.contains("prompt_eval") {
        if let Some(n) = extract_trailing_token_count(line) {
            input = n;
        }
    }
    if line.contains("eval time") && line.contains("decode") {
        if let Some(n) = extract_trailing_token_count(line) {
            output = n;
        }
    } else if line.contains("/ ") && line.contains("tokens") && !line.contains("prompt") {
        if let Some(n) = extract_trailing_token_count(line) {
            output = n;
        }
    }
    (input, output)
}

fn extract_trailing_token_count(line: &str) -> Option<u64> {
    // e.g. "prompt eval time = ... / 1234 tokens"
    let after_slash = line.rsplit('/').next()?;
    let num_part = after_slash.trim().split_whitespace().next()?;
    num_part.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_prompt_eval_line() {
        let line = "slot 0: prompt eval time = 123.45 ms / 4096 tokens";
        let (in_t, out_t) = parse_llama_log_line(line);
        assert_eq!(in_t, 4096);
        assert_eq!(out_t, 0);
    }
}
