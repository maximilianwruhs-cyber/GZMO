//! Parse Pi agent session JSONL (`~/.pi/agent/sessions/.../*.jsonl`) for SessionDistill.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Build a stable distill session id from a Pi session file path or header.
pub fn pi_session_id_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    if let Some((_prefix, id)) = stem.rsplit_once('_') {
        if id.len() >= 8 {
            return format!("pi-{id}");
        }
    }
    format!("pi-{stem}")
}

/// Parse Pi JSONL into `(session_id, transcript)` for SessionDistill.
pub fn parse_pi_jsonl_transcript(path: &Path, max_chars: usize) -> Result<(String, String)> {
    let file = File::open(path).with_context(|| format!("open Pi session {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut session_id = pi_session_id_from_path(path);
    let mut created_at: Option<DateTime<Utc>> = None;
    let mut out = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: PiJsonlRow = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if row.row_type == "session" {
            if let Some(id) = row.id.as_deref().or(row.header.as_ref().map(|h| h.id.as_str())) {
                session_id = format!("pi-{id}");
            }
            let ts_str = row
                .timestamp
                .as_deref()
                .or(row.header.as_ref().map(|h| h.timestamp.as_str()));
            if let Some(ts) = ts_str {
                if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
                    created_at = Some(parsed.with_timezone(&Utc));
                }
            }
            continue;
        }

        if row.row_type != "message" {
            continue;
        }
        let Some(msg) = row.message else {
            continue;
        };
        let role = msg.role.as_str();
        if role != "user" && role != "assistant" {
            continue;
        }
        let text = extract_message_text(&msg);
        if text.trim().is_empty() {
            continue;
        }
        let label = if role == "user" { "USER" } else { "ASSISTANT" };
        out.push_str(&format!("{label}: {text}\n\n"));
        if out.len() >= max_chars {
            out.truncate(max_chars);
            break;
        }
    }

    let created = created_at
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let mut transcript = format!("Pi session {session_id} — started {created}\n\n");
    transcript.push_str(&out);

    Ok((session_id, transcript))
}

fn extract_message_text(msg: &PiMessage) -> String {
    let mut parts = Vec::new();
    for block in &msg.content {
        if block.block_type == "text" {
            if let Some(ref t) = block.text {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
            }
        }
    }
    parts.join("\n")
}

#[derive(Debug, Deserialize)]
struct PiJsonlRow {
    #[serde(rename = "type")]
    row_type: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    header: Option<PiSessionHeader>,
    #[serde(default)]
    message: Option<PiMessage>,
}

#[derive(Debug, Deserialize)]
struct PiSessionHeader {
    id: String,
    timestamp: String,
}

#[derive(Debug, Deserialize)]
struct PiMessage {
    role: String,
    content: Vec<PiContentBlock>,
}

#[derive(Debug, Deserialize)]
struct PiContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: Option<String>,
}

/// Parse Pi JSONL into `(session_id, transcript)` for SessionDistill with turn range constraints.
pub fn parse_pi_jsonl_transcript_range(
    path: &Path,
    start_turn: usize,
    max_turns: Option<usize>,
    max_chars: usize,
) -> Result<(String, String)> {
    let file = File::open(path).with_context(|| format!("open Pi session {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut session_id = pi_session_id_from_path(path);
    let mut created_at: Option<DateTime<Utc>> = None;
    let mut out = String::new();
    let mut current_turn_index = 0;
    let mut message_count = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: PiJsonlRow = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if row.row_type == "session" {
            if let Some(id) = row.id.as_deref().or(row.header.as_ref().map(|h| h.id.as_str())) {
                session_id = format!("pi-{id}");
            }
            let ts_str = row
                .timestamp
                .as_deref()
                .or(row.header.as_ref().map(|h| h.timestamp.as_str()));
            if let Some(ts) = ts_str {
                if let Ok(parsed) = DateTime::parse_from_rfc3339(ts) {
                    created_at = Some(parsed.with_timezone(&Utc));
                }
            }
            continue;
        }

        if row.row_type != "message" {
            continue;
        }
        let Some(msg) = row.message else {
            continue;
        };
        let role = msg.role.as_str();
        if role != "user" && role != "assistant" {
            continue;
        }

        let turn_idx = current_turn_index;
        current_turn_index += 1;

        if turn_idx < start_turn {
            continue;
        }
        if let Some(limit) = max_turns {
            if message_count >= limit {
                break;
            }
        }

        let text = extract_message_text(&msg);
        if text.trim().is_empty() {
            continue;
        }
        let label = if role == "user" { "USER" } else { "ASSISTANT" };
        out.push_str(&format!("{label}: {text}\n\n"));
        message_count += 1;

        if out.len() >= max_chars {
            out.truncate(max_chars);
            break;
        }
    }

    let created = created_at
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let suffix = if let Some(limit) = max_turns {
        format!("-shift-{}-{}", start_turn, start_turn + limit)
    } else {
        format!("-shift-{}", start_turn)
    };
    let range_session_id = format!("{session_id}{suffix}");

    let mut transcript = format!("Pi session {range_session_id} — started {created}\n\n");
    transcript.push_str(&out);

    Ok((range_session_id, transcript))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_from_filename() {
        let p = Path::new("/tmp/2026-06-11T14-49-34-959Z_019eb728-f3ef-7bd9-bcfe-a28067e94e0d.jsonl");
        assert_eq!(
            pi_session_id_from_path(p),
            "pi-019eb728-f3ef-7bd9-bcfe-a28067e94e0d"
        );
    }

    #[test]
    fn parse_real_pi_jsonl_shape() {
        let dir = std::env::temp_dir().join("gzmo_pi_session_test.jsonl");
        std::fs::write(
            &dir,
            r#"{"type":"session","id":"abc-123","timestamp":"2026-06-11T14:49:34.959Z"}
{"type":"message","message":{"role":"user","content":[{"type":"text","text":"hello distill"}]}}
{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"hi there"}]}}
"#,
        )
        .unwrap();
        let (id, transcript) = parse_pi_jsonl_transcript(&dir, 50_000).unwrap();
        assert_eq!(id, "pi-abc-123");
        assert!(transcript.contains("USER: hello distill"));
        assert!(transcript.contains("ASSISTANT: hi there"));
        let _ = std::fs::remove_file(dir);
    }

    #[test]
    fn parse_pi_jsonl_range_shape() {
        let dir = std::env::temp_dir().join("gzmo_pi_session_range_test.jsonl");
        std::fs::write(
            &dir,
            r#"{"type":"session","id":"abc-123","timestamp":"2026-06-11T14:49:34.959Z"}
{"type":"message","message":{"role":"user","content":[{"type":"text","text":"turn 0"}]}}
{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"turn 1"}]}}
{"type":"message","message":{"role":"user","content":[{"type":"text","text":"turn 2"}]}}
{"type":"message","message":{"role":"assistant","content":[{"type":"text","text":"turn 3"}]}}
"#,
        )
        .unwrap();
        let (id, transcript) = parse_pi_jsonl_transcript_range(&dir, 1, Some(2), 50_000).unwrap();
        assert_eq!(id, "pi-abc-123-shift-1-3");
        assert!(!transcript.contains("USER: turn 0"));
        assert!(transcript.contains("ASSISTANT: turn 1"));
        assert!(transcript.contains("USER: turn 2"));
        assert!(!transcript.contains("ASSISTANT: turn 3"));
        let _ = std::fs::remove_file(dir);
    }
}
