//! Read-only Synapse bus reader — never writes to the append-only bus.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::memory::episodic::FileEpisodicStore;
use crate::synapse::{EventSource, EventType, SynapseEvent};
use crate::types::{EpisodicEntry, EpisodicSource};

const STATE_FILE: &str = "data/synapse-reader.state.json";
const DISTILL_STATE_FILE: &str = "data/synapse-pi-distill.state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SynapseReaderState {
    pub byte_offset: u64,
    pub last_pull_at: Option<DateTime<Utc>>,
    pub events_processed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PiDistillState {
    pub distilled_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PiSynapsePollResult {
    pub summary: PiEventSummary,
    pub session_end_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PiEventSummary {
    pub events_read: usize,
    pub quest_complete: usize,
    pub session_start: usize,
    pub session_end: usize,
    pub mentor_teach: usize,
    pub topic_shift_distill: usize,
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

pub fn summarize_pi_events(events: &[SynapseEvent]) -> PiEventSummary {
    let mut quest_complete = 0usize;
    let mut session_start = 0usize;
    let mut session_end = 0usize;
    let mut mentor_teach = 0usize;
    let mut topic_shift_distill = 0usize;
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
            EventType::MentorTeach => {
                mentor_teach += 1;
                if let Some(data) = &e.data {
                    if let Some(text) = data.get("message").and_then(|v| v.as_str()) {
                        let clip: String = text.chars().take(400).collect();
                        if !clip.trim().is_empty() {
                            snippets.push(format!("[mentor] {clip}"));
                        }
                    }
                }
            }
            EventType::MentorLearnStart | EventType::MentorLearnEnd => {}
            EventType::TopicShiftDistill => {
                topic_shift_distill += 1;
                if let Some(data) = &e.data {
                    if let Some(dist) = data.get("distance").and_then(|v| v.as_f64()) {
                        snippets.push(format!("[topic-shift] Triggered mid-session distill (distance: {:.4})", dist));
                    }
                }
            }
            _ => {}
        }
    }

    let mut summary = format!(
        "Pi Synapse pull: {} events (quest_complete={}, session_start={}, session_end={}, mentor_teach={}, topic_shift_distill={})",
        events.len(),
        quest_complete,
        session_start,
        session_end,
        mentor_teach,
        topic_shift_distill
    );
    if !snippets.is_empty() {
        summary.push_str("\n\nRecent Pi turn excerpts:\n");
        for (i, s) in snippets.iter().take(5).enumerate() {
            summary.push_str(&format!("\n### Excerpt {}\n{s}", i + 1));
        }
    }

    PiEventSummary {
        events_read: events.len(),
        quest_complete,
        session_start,
        session_end,
        mentor_teach,
        topic_shift_distill,
        summary_text: summary,
    }
}

/// Paths from Pi `session_end` events (`targetSessionFile` in event data).
pub fn session_end_distill_targets(events: &[SynapseEvent]) -> Vec<String> {
    let mut out = Vec::new();
    for e in events {
        if e.event_type != EventType::SessionEnd {
            continue;
        }
        let Some(data) = &e.data else {
            continue;
        };
        let Some(path) = data.get("targetSessionFile").and_then(|v| v.as_str()) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        if !out.iter().any(|p| p == path) {
            out.push(path.to_string());
        }
    }
    out
}

/// Tail Pi synapse bus: optional episodic batch + session-end distill targets.
pub async fn poll_pi_synapse(
    bus_path: &Path,
    state_path: &Path,
    episodic: &FileEpisodicStore,
    max_events: usize,
    log_episodic_on_events: bool,
) -> Result<PiSynapsePollResult> {
    let (events, state) = read_new_pi_events(bus_path, state_path, max_events)?;
    let summary = summarize_pi_events(&events);
    let session_end_files = session_end_distill_targets(&events);

    if log_episodic_on_events && summary.events_read > 0 {
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
            "Synapse pull logged to episodic"
        );
    } else if summary.events_read == 0 {
        info!("Synapse pull: no new Pi events");
    }

    Ok(PiSynapsePollResult {
        summary,
        session_end_files,
    })
}

/// Append Pi activity summary to episodic (feeds DreamEngine on next cycle).
pub async fn pull_and_log_episodic(
    bus_path: &Path,
    state_path: &Path,
    episodic: &FileEpisodicStore,
    max_events: usize,
) -> Result<PiEventSummary> {
    let result = poll_pi_synapse(bus_path, state_path, episodic, max_events, true).await?;
    Ok(result.summary)
}

pub fn should_distill_pi_session(session_path: &Path, state_path: &Path) -> Result<bool> {
    if !session_path.exists() {
        return Ok(false);
    }
    let key = session_path.to_string_lossy().to_string();
    let state = load_distill_state(state_path)?;
    Ok(!state.distilled_paths.contains(&key))
}

pub fn mark_pi_session_distilled(session_path: &str, state_path: &Path) -> Result<()> {
    let mut state = load_distill_state(state_path)?;
    if state.distilled_paths.iter().any(|p| p == session_path) {
        return Ok(());
    }
    state.distilled_paths.push(session_path.to_string());
    const MAX_TRACKED: usize = 500;
    if state.distilled_paths.len() > MAX_TRACKED {
        let drop = state.distilled_paths.len() - MAX_TRACKED;
        state.distilled_paths.drain(0..drop);
    }
    save_distill_state(state_path, &state)
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

pub fn default_distill_state_path(project_root: &Path) -> PathBuf {
    project_root.join(DISTILL_STATE_FILE)
}

fn load_distill_state(path: &Path) -> Result<PiDistillState> {
    if !path.exists() {
        return Ok(PiDistillState::default());
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}

fn save_distill_state(path: &Path, state: &PiDistillState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synapse::{EventSource, SynapseEvent};
    use uuid::Uuid;

    #[tokio::test]
    async fn poll_pi_synapse_reads_session_end_from_bus() {
        let dir = std::env::temp_dir().join(format!(
            "gzmo_synapse_test_{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let bus = dir.join("events.jsonl");
        let state = dir.join("reader.state.json");
        let mem = dir.join("memory");
        let fixture = dir.join("pi_fixture.jsonl");
        std::fs::write(&fixture, "fixture\n").unwrap();
        std::fs::write(
            &bus,
            format!(
                r#"{{"id":"{}","event_type":"session_end","source":"pi_agent","timestamp":"2026-06-11T15:00:00Z","data":{{"reason":"shutdown","targetSessionFile":"{}"}}}}
"#,
                uuid::Uuid::new_v4(),
                fixture.display()
            ),
        )
        .unwrap();

        let episodic = FileEpisodicStore::new(&mem);
        let result = poll_pi_synapse(&bus, &state, &episodic, 50, false)
            .await
            .unwrap();
        assert_eq!(result.summary.session_end, 1);
        assert_eq!(result.session_end_files, vec![fixture.to_string_lossy().to_string()]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn session_end_targets_from_events() {
        let events = vec![SynapseEvent {
            id: Uuid::new_v4(),
            event_type: EventType::SessionEnd,
            source: EventSource::PiAgent,
            timestamp: Utc::now(),
            data: Some(serde_json::json!({
                "reason": "shutdown",
                "targetSessionFile": "/tmp/foo.jsonl"
            })),
        }];
        assert_eq!(
            session_end_distill_targets(&events),
            vec!["/tmp/foo.jsonl".to_string()]
        );
    }
}
