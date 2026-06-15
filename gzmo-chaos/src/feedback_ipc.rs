//! File-based chaos feedback inbox for external skill runners (Pi, CLI one-shots).
//!
//! Append-only `data/chaos_feedback_inbox.jsonl`; the daemon drains lines each tick.

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::feedback::{ChaosEvent, SoundCategory, ThoughtSeed};

const INBOX_NAME: &str = "chaos_feedback_inbox.jsonl";
const AUDIT_NAME: &str = "chaos_feedback_audit.jsonl";

/// Default inbox path under the GZMO data directory.
pub fn default_inbox_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(INBOX_NAME)
}

pub fn event_to_json_value(event: &ChaosEvent) -> serde_json::Value {
    serde_json::to_value(ChaosEventDto::from(event)).unwrap_or(serde_json::Value::Null)
}

/// Stable variant label for audit summaries (`DiceRoll`, `JokeGenerated`, ...).
pub fn event_type_label(event: &ChaosEvent) -> &'static str {
    match event {
        ChaosEvent::DiceRoll { .. } => "DiceRoll",
        ChaosEvent::SoundFired { .. } => "SoundFired",
        ChaosEvent::CardForged { .. } => "CardForged",
        ChaosEvent::PkmForged { .. } => "PkmForged",
        ChaosEvent::PoemGenerated { .. } => "PoemGenerated",
        ChaosEvent::StoryGenerated { .. } => "StoryGenerated",
        ChaosEvent::JokeGenerated { .. } => "JokeGenerated",
        ChaosEvent::PersonaShift { .. } => "PersonaShift",
        ChaosEvent::PersonaCleared => "PersonaCleared",
        ChaosEvent::WordGenerated { .. } => "WordGenerated",
        ChaosEvent::QuoteSurfaced { .. } => "QuoteSurfaced",
        ChaosEvent::Stabilize { .. } => "Stabilize",
        ChaosEvent::MentorTeach { .. } => "MentorTeach",
        ChaosEvent::Custom { .. } => "Custom",
    }
}

pub fn append_audit(data_dir: &Path, event: &ChaosEvent, source: &str) -> std::io::Result<()> {
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)?;
    }
    let path = data_dir.join(AUDIT_NAME);
    let dto = ChaosEventDto::from(event);
    let ts = chrono::Utc::now().to_rfc3339();
    let audit_entry = serde_json::json!({
        "ts": ts,
        "source": source,
        "event": dto,
    });
    let line = serde_json::to_string(&audit_entry).map_err(std::io::Error::other)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Append one event to the inbox (atomic line write).
pub fn append_event(path: &Path, event: &ChaosEvent) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let dto = ChaosEventDto::from(event);
    let line = serde_json::to_string(&dto).map_err(std::io::Error::other)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;

    if let Some(parent) = path.parent() {
        let _ = append_audit(parent, event, "inbox");
    }
    Ok(())
}

/// Drain and remove all pending events from the inbox.
pub fn drain_inbox(path: &Path) -> Vec<ChaosEvent> {
    if !path.exists() {
        return vec![];
    }
    let processing_path = path.with_extension("jsonl.draining");
    let _ = std::fs::remove_file(&processing_path);

    if std::fs::rename(path, &processing_path).is_err() {
        return vec![];
    }

    let events = read_events(&processing_path);
    let _ = std::fs::remove_file(&processing_path);
    events
}

fn read_events(path: &Path) -> Vec<ChaosEvent> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| {
            serde_json::from_str::<ChaosEventDto>(&line)
                .ok()
                .and_then(|dto| dto.into_event())
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ChaosEventDto {
    DiceRoll { value: u8, max: u8 },
    SoundFired { category: String },
    CardForged { name: String, card_type: String },
    PkmForged { name: String, element: String },
    PoemGenerated { text: String },
    StoryGenerated { text: String },
    JokeGenerated { text: String },
    PersonaShift { persona: String },
    PersonaCleared,
    WordGenerated { word: String, definition: String },
    QuoteSurfaced { text: String },
    Stabilize { delta_rho: f64 },
    MentorTeach {
        topic_preview: String,
        response_preview: String,
        turn_count: u32,
    },
    Custom {
        tension_delta: f64,
        energy_delta: f64,
        thought_seed: Option<ThoughtSeedDto>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ThoughtSeedDto {
    category: String,
    text: String,
}

impl From<&ThoughtSeed> for ThoughtSeedDto {
    fn from(s: &ThoughtSeed) -> Self {
        Self {
            category: s.category.clone(),
            text: s.text.clone(),
        }
    }
}

impl From<&ChaosEvent> for ChaosEventDto {
    fn from(e: &ChaosEvent) -> Self {
        match e {
            ChaosEvent::DiceRoll { value, max } => ChaosEventDto::DiceRoll {
                value: *value,
                max: *max,
            },
            ChaosEvent::SoundFired { category } => ChaosEventDto::SoundFired {
                category: sound_category_str(category).to_string(),
            },
            ChaosEvent::CardForged { name, card_type } => ChaosEventDto::CardForged {
                name: name.clone(),
                card_type: card_type.clone(),
            },
            ChaosEvent::PkmForged { name, element } => ChaosEventDto::PkmForged {
                name: name.clone(),
                element: element.clone(),
            },
            ChaosEvent::PoemGenerated { text } => ChaosEventDto::PoemGenerated { text: text.clone() },
            ChaosEvent::StoryGenerated { text } => ChaosEventDto::StoryGenerated { text: text.clone() },
            ChaosEvent::JokeGenerated { text } => ChaosEventDto::JokeGenerated { text: text.clone() },
            ChaosEvent::PersonaShift { persona } => ChaosEventDto::PersonaShift {
                persona: persona.clone(),
            },
            ChaosEvent::PersonaCleared => ChaosEventDto::PersonaCleared,
            ChaosEvent::WordGenerated { word, definition } => ChaosEventDto::WordGenerated {
                word: word.clone(),
                definition: definition.clone(),
            },
            ChaosEvent::QuoteSurfaced { text } => ChaosEventDto::QuoteSurfaced { text: text.clone() },
            ChaosEvent::Stabilize { delta_rho } => ChaosEventDto::Stabilize {
                delta_rho: *delta_rho,
            },
            ChaosEvent::MentorTeach {
                topic_preview,
                response_preview,
                turn_count,
            } => ChaosEventDto::MentorTeach {
                topic_preview: topic_preview.clone(),
                response_preview: response_preview.clone(),
                turn_count: *turn_count,
            },
            ChaosEvent::Custom {
                tension_delta,
                energy_delta,
                thought_seed,
            } => ChaosEventDto::Custom {
                tension_delta: *tension_delta,
                energy_delta: *energy_delta,
                thought_seed: thought_seed.as_ref().map(ThoughtSeedDto::from),
            },
        }
    }
}

impl ChaosEventDto {
    fn into_event(self) -> Option<ChaosEvent> {
        Some(match self {
            ChaosEventDto::DiceRoll { value, max } => ChaosEvent::DiceRoll { value, max },
            ChaosEventDto::SoundFired { category } => ChaosEvent::SoundFired {
                category: parse_sound_category(&category)?,
            },
            ChaosEventDto::CardForged { name, card_type } => {
                ChaosEvent::CardForged { name, card_type }
            }
            ChaosEventDto::PkmForged { name, element } => {
                ChaosEvent::PkmForged { name, element }
            }
            ChaosEventDto::PoemGenerated { text } => ChaosEvent::PoemGenerated { text },
            ChaosEventDto::StoryGenerated { text } => ChaosEvent::StoryGenerated { text },
            ChaosEventDto::JokeGenerated { text } => ChaosEvent::JokeGenerated { text },
            ChaosEventDto::PersonaShift { persona } => ChaosEvent::PersonaShift { persona },
            ChaosEventDto::PersonaCleared => ChaosEvent::PersonaCleared,
            ChaosEventDto::WordGenerated { word, definition } => {
                ChaosEvent::WordGenerated { word, definition }
            }
            ChaosEventDto::QuoteSurfaced { text } => ChaosEvent::QuoteSurfaced { text },
            ChaosEventDto::Stabilize { delta_rho } => ChaosEvent::Stabilize { delta_rho },
            ChaosEventDto::MentorTeach {
                topic_preview,
                response_preview,
                turn_count,
            } => ChaosEvent::MentorTeach {
                topic_preview,
                response_preview,
                turn_count,
            },
            ChaosEventDto::Custom {
                tension_delta,
                energy_delta,
                thought_seed,
            } => ChaosEvent::Custom {
                tension_delta,
                energy_delta,
                thought_seed: thought_seed.map(|s| ThoughtSeed {
                    category: s.category,
                    text: s.text,
                }),
            },
        })
    }
}

fn sound_category_str(c: &SoundCategory) -> &'static str {
    match c {
        SoundCategory::Explosion => "Explosion",
        SoundCategory::Thunder => "Thunder",
        SoundCategory::Alarm => "Alarm",
        SoundCategory::Roar => "Roar",
        SoundCategory::Bell => "Bell",
        SoundCategory::Guitar => "Guitar",
        SoundCategory::Drum => "Drum",
        SoundCategory::Wave => "Wave",
        SoundCategory::Chime => "Chime",
        SoundCategory::Piano => "Piano",
        SoundCategory::Wind => "Wind",
        SoundCategory::Hum => "Hum",
    }
}

fn parse_sound_category(s: &str) -> Option<SoundCategory> {
    match s {
        "Explosion" => Some(SoundCategory::Explosion),
        "Thunder" => Some(SoundCategory::Thunder),
        "Alarm" => Some(SoundCategory::Alarm),
        "Roar" => Some(SoundCategory::Roar),
        "Bell" => Some(SoundCategory::Bell),
        "Guitar" => Some(SoundCategory::Guitar),
        "Drum" => Some(SoundCategory::Drum),
        "Wave" => Some(SoundCategory::Wave),
        "Chime" => Some(SoundCategory::Chime),
        "Piano" => Some(SoundCategory::Piano),
        "Wind" => Some(SoundCategory::Wind),
        "Hum" => Some(SoundCategory::Hum),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_inbox_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("gzmo-chaos-{name}-{}-{nanos}.jsonl", std::process::id()))
    }

    #[test]
    fn roundtrip_joke_event() {
        let event = ChaosEvent::JokeGenerated {
            text: "test joke".to_string(),
        };
        let dto = ChaosEventDto::from(&event);
        let json = serde_json::to_string(&dto).unwrap();
        let back: ChaosEventDto = serde_json::from_str(&json).unwrap();
        match back.into_event().unwrap() {
            ChaosEvent::JokeGenerated { text } => assert_eq!(text, "test joke"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn unknown_sound_category_is_dropped() {
        let dto = ChaosEventDto::SoundFired {
            category: "Unknown".to_string(),
        };

        assert!(dto.into_event().is_none());
    }

    #[test]
    fn drain_inbox_roundtrips_events_and_removes_file() {
        let path = temp_inbox_path("roundtrip");
        append_event(&path, &ChaosEvent::JokeGenerated { text: "j".to_string() }).unwrap();
        append_event(&path, &ChaosEvent::DiceRoll { value: 20, max: 20 }).unwrap();

        let events = drain_inbox(&path);

        assert_eq!(events.len(), 2);
        assert!(!path.exists());
    }
}
