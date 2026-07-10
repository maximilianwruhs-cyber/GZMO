//! Read-only Synapse bus reader — never writes to the append-only bus.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::memory::episodic::FileEpisodicStore;
use crate::memory::scratch::{DistillJob, DistillSource, ScratchService};
use crate::synapse::{EventSource, EventType, SynapseEvent};
use crate::types::{EpisodicEntry, EpisodicSource};

const STATE_FILE: &str = "data/synapse-reader.state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SynapseReaderState {
    pub byte_offset: u64,
    pub last_pull_at: Option<DateTime<Utc>>,
    pub events_processed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEndHandoff {
    pub session_id: String,
    pub reason: Option<String>,
    pub session_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PiEventSummary {
    pub events_read: usize,
    pub quest_complete: usize,
    pub session_start: usize,
    pub session_end: usize,
    pub session_end_ids: Vec<String>,
    pub distill_enqueued: usize,
    pub summary_text: String,
}

/// Tail Pi events from the append-only bus without mutating it.
pub fn read_new_pi_events(
    bus_path: &Path,
    state_path: &Path,
    max_events: usize,
) -> Result<(Vec<SynapseEvent>, SynapseReaderState)> {
    let mut state = load_state(state_path)?;
    if !bus_path.exists() {
        return Ok((Vec::new(), state));
    }

    let mut file = File::open(bus_path).with_context(|| format!("open {}", bus_path.display()))?;
    let len = file.metadata()?.len();
    if state.byte_offset > len {
        state.byte_offset = 0;
    }
    file.seek(SeekFrom::Start(state.byte_offset))?;

    let reader = BufReader::new(file);
    let mut events = Vec::new();
    let mut lines_read = 0u64;

    for line in reader.lines() {
        let line = line?;
        lines_read += line.len() as u64 + 1;
        if line.trim().is_empty() {
            continue;
        }
        let event: SynapseEvent = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if event.source != EventSource::PiAgent {
            continue;
        }
        events.push(event);
        if events.len() >= max_events {
            break;
        }
    }

    state.byte_offset += lines_read;
    state.last_pull_at = Some(Utc::now());
    state.events_processed += events.len() as u64;
    save_state(state_path, &state)?;

    Ok((events, state))
}

pub fn extract_session_end_handoffs(events: &[SynapseEvent]) -> Vec<SessionEndHandoff> {
    let mut out = Vec::new();
    for e in events {
        if !matches!(e.event_type, EventType::SessionEnd) {
            continue;
        }
        let session_id = e
            .data
            .as_ref()
            .and_then(|d| d.get("session_id").and_then(|v| v.as_str()))
            .map(str::to_string)
            .unwrap_or_else(|| format!("pi-{}", e.id));
        let reason = e
            .data
            .as_ref()
            .and_then(|d| d.get("reason").and_then(|v| v.as_str()))
            .map(str::to_string);
        let session_file = e
            .data
            .as_ref()
            .and_then(|d| {
                d.get("sessionFile")
                    .or_else(|| d.get("session_file"))
                    .and_then(|v| v.as_str())
            })
            .map(str::to_string);
        out.push(SessionEndHandoff {
            session_id,
            reason,
            session_file,
        });
    }
    out
}

/// Enqueue distill jobs for Pi `session_end` events (feeds BRPOP worker).
pub async fn enqueue_session_end_distills(
    scratch: &ScratchService,
    events: &[SynapseEvent],
    fallback_transcript: &str,
) -> Result<usize> {
    let handoffs = extract_session_end_handoffs(events);
    let mut n = 0usize;
    for h in handoffs {
        let transcript = if let Some(path) = &h.session_file {
            std::fs::read_to_string(path).unwrap_or_else(|_| {
                format!(
                    "{fallback_transcript}\nsession_id={}\nreason={:?}",
                    h.session_id, h.reason
                )
            })
        } else {
            format!(
                "{fallback_transcript}\nsession_id={}\nreason={:?}",
                h.session_id, h.reason
            )
        };
        scratch
            .enqueue_distill(DistillJob {
                session_id: h.session_id.clone(),
                transcript,
                source: DistillSource::MainArchive,
            })
            .await?;
        n += 1;
        info!(session_id = %h.session_id, "Enqueued Pi session_end distill job");
    }
    Ok(n)
}

pub fn summarize_pi_events(events: &[SynapseEvent]) -> PiEventSummary {
    let mut quest_complete = 0usize;
    let mut session_start = 0usize;
    let mut session_end = 0usize;
    let mut snippets = Vec::new();

    for e in events {
        match e.event_type {
            EventType::QuestComplete => {
                quest_complete += 1;
                if let Some(data) = &e.data {
                    if let Some(text) = data.get("messageText").and_then(|v| v.as_str()) {
                        let clip: String = text.chars().take(400).collect();
                        if !clip.trim().is_empty() {
                            snippets.push(clip);
                        }
                    }
                }
            }
            EventType::SessionStart => session_start += 1,
            EventType::SessionEnd => session_end += 1,
            _ => {}
        }
    }

    let mut summary = format!(
        "Pi Synapse pull: {} events (quest_complete={}, session_start={}, session_end={})",
        events.len(),
        quest_complete,
        session_start,
        session_end
    );
    if !snippets.is_empty() {
        summary.push_str("\n\nRecent Pi turn excerpts:\n");
        for (i, s) in snippets.iter().take(5).enumerate() {
            summary.push_str(&format!("\n### Excerpt {}\n{s}", i + 1));
        }
    }

    let handoffs = extract_session_end_handoffs(events);
    let session_end_ids: Vec<String> = handoffs.iter().map(|h| h.session_id.clone()).collect();

    PiEventSummary {
        events_read: events.len(),
        quest_complete,
        session_start,
        session_end,
        session_end_ids,
        distill_enqueued: 0,
        summary_text: summary,
    }
}

/// Append Pi activity summary to episodic (feeds DreamEngine on next cycle).
pub async fn pull_and_log_episodic(
    bus_path: &Path,
    state_path: &Path,
    episodic: &FileEpisodicStore,
    max_events: usize,
    scratch: Option<&ScratchService>,
) -> Result<PiEventSummary> {
    let (events, state) = read_new_pi_events(bus_path, state_path, max_events)?;
    let mut summary = summarize_pi_events(&events);
    if summary.events_read == 0 {
        info!("Synapse pull: no new Pi events");
        return Ok(summary);
    }

    if let Some(scratch) = scratch {
        summary.distill_enqueued =
            enqueue_session_end_distills(scratch, &events, "Pi session_end handoff").await?;
    }
    let entry = EpisodicEntry {
        timestamp: Utc::now(),
        source: EpisodicSource::InternalMonologue,
        content: format!(
            "### Pi Synapse Pull ({} events, offset {})\n\n{}",
            summary.events_read, state.byte_offset, summary.summary_text
        ),
        is_silent: false,
    };
    episodic.append(&entry).await?;
    info!(
        events = summary.events_read,
        quest = summary.quest_complete,
        session_end = summary.session_end,
        distill_enqueued = summary.distill_enqueued,
        "Synapse pull logged to episodic"
    );
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn session_end_event(session_id: &str) -> SynapseEvent {
        SynapseEvent {
            id: Uuid::new_v4(),
            event_type: EventType::SessionEnd,
            source: EventSource::PiAgent,
            timestamp: Utc::now(),
            data: Some(serde_json::json!({
                "session_id": session_id,
                "reason": "quit"
            })),
        }
    }

    #[test]
    fn extract_session_end_handoffs_finds_ids() {
        let events = vec![session_end_event("pi-sess-001")];
        let handoffs = extract_session_end_handoffs(&events);
        assert_eq!(handoffs.len(), 1);
        assert_eq!(handoffs[0].session_id, "pi-sess-001");
    }
}

fn load_state(path: &Path) -> Result<SynapseReaderState> {
    if !path.exists() {
        return Ok(SynapseReaderState::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

fn save_state(path: &Path, state: &SynapseReaderState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn default_state_path(project_root: &Path) -> PathBuf {
    project_root.join(STATE_FILE)
}
