//! Outcome samples from Synapse completion events (Phase B).

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::synapse::{EventType, SynapseEvent};

/// One measurable outcome tied to a process family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeSample {
    pub ts: DateTime<Utc>,
    pub process: String,
    pub q: f64,
    pub i: f64,
    pub action_id: Option<String>,
}

/// Map ledger process labels to outcome families for η rollups.
pub fn process_family(process: &str) -> &str {
    if process.starts_with("dream_") {
        "dream"
    } else if process.starts_with("spark_") {
        "spark"
    } else if process.starts_with("ingest_") {
        "ingest"
    } else if process.starts_with("distill_") {
        "distill"
    } else if process.starts_with("kurator_") {
        process
    } else {
        process
    }
}

/// Scan the Synapse bus for completion events since `since`.
pub fn collect_from_synapse(bus_path: &Path, since: DateTime<Utc>) -> Result<Vec<OutcomeSample>> {
    if !bus_path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(bus_path).with_context(|| format!("open {}", bus_path.display()))?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let event: SynapseEvent = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if event.timestamp < since {
            continue;
        }
        if let Some(sample) = sample_from_event(&event) {
            out.push(sample);
        }
    }
    Ok(out)
}

fn sample_from_event(event: &SynapseEvent) -> Option<OutcomeSample> {
    let data = event.data.as_ref()?;
    match event.event_type {
        EventType::DreamComplete => {
            let extracted = data
                .get("entities_extracted")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let written = data
                .get("kg_entities_written")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let truths = data
                .get("truths_promoted")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let i = if extracted > 0.0 {
                (written / extracted).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let q = if written > 0.0 || truths > 0 {
                1.0
            } else {
                0.0
            };
            Some(OutcomeSample {
                ts: event.timestamp,
                process: "dream".into(),
                q,
                i,
                action_id: data
                    .get("date")
                    .and_then(|v| v.as_str())
                    .map(|d| format!("dream_{d}")),
            })
        }
        EventType::SparkComplete => {
            let promoted = data.get("promoted").and_then(|v| v.as_bool()).unwrap_or(false);
            let links = data
                .get("kg_relations_written")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let q = if promoted { 1.0 } else { 0.0 };
            let i = if promoted {
                (links / 5.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            Some(OutcomeSample {
                ts: event.timestamp,
                process: "spark".into(),
                q,
                i,
                action_id: data
                    .get("date")
                    .and_then(|v| v.as_str())
                    .map(|d| format!("spark_{d}")),
            })
        }
        EventType::IngestComplete => {
            let extracted = data
                .get("entities_extracted")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let promoted = data
                .get("entities_promoted")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let kg = data
                .get("kg_entities_written")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let i = if extracted > 0.0 {
                (promoted / extracted).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let q = if promoted > 0.0 || kg > 0 {
                1.0
            } else {
                0.0
            };
            Some(OutcomeSample {
                ts: event.timestamp,
                process: "ingest".into(),
                q,
                i,
                action_id: data
                    .get("file")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            })
        }
        EventType::AgentResult => {
            let payload = data.get("payload").unwrap_or(data);
            let status = payload
                .get("status")
                .or_else(|| data.get("status"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let llm_calls = payload
                .get("llm_calls")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let tool_calls = payload
                .get("tool_calls")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let spawn_kind = payload
                .get("spawn_kind")
                .and_then(|v| v.as_str())
                .or_else(|| data.get("spawn_kind").and_then(|v| v.as_str()))
                .unwrap_or("session_triage");
            let q = if status == "done" { 1.0 } else { 0.0 };
            let i = if llm_calls > 0.0 {
                (tool_calls / llm_calls).clamp(0.0, 1.0)
            } else if tool_calls > 0.0 {
                1.0
            } else {
                0.0
            };
            Some(OutcomeSample {
                ts: event.timestamp,
                process: format!("kurator_{spawn_kind}"),
                q,
                i,
                action_id: payload
                    .get("task_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            })
        }
        EventType::QuestComplete => {
            let input = data
                .get("inputTokens")
                .or_else(|| data.get("input_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let output = data
                .get("outputTokens")
                .or_else(|| data.get("output_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as f64;
            let i = if input > 0.0 {
                (output / input).clamp(0.0, 1.0)
            } else {
                0.5
            };
            Some(OutcomeSample {
                ts: event.timestamp,
                process: "pi_agent".into(),
                q: 1.0,
                i,
                action_id: Some(event.id.to_string()),
            })
        }
        EventType::QuestFail => Some(OutcomeSample {
            ts: event.timestamp,
            process: "pi_agent".into(),
            q: 0.0,
            i: 0.0,
            action_id: Some(event.id.to_string()),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synapse::{EventSource, SynapseEvent};
    use uuid::Uuid;

    #[test]
    fn dream_complete_sample() {
        let event = SynapseEvent {
            id: Uuid::new_v4(),
            event_type: EventType::DreamComplete,
            source: EventSource::GzmoDaemon,
            timestamp: Utc::now(),
            correlation_id: None,
            reply_to: None,
            data: Some(serde_json::json!({
                "date": "2026-06-15",
                "entities_extracted": 10,
                "kg_entities_written": 8,
                "truths_promoted": 2,
            })),
        };
        let s = sample_from_event(&event).unwrap();
        assert_eq!(s.process, "dream");
        assert!((s.i - 0.8).abs() < 0.01);
        assert_eq!(s.q, 1.0);
    }
}
