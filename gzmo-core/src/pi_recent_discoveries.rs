//! Curated recent ingest/distill entities for Pi session-start panel (D3).

use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

const DEFAULT_FILENAME: &str = "pi-recent-discoveries.json";
const MAX_ENTRIES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentDiscoveryEntry {
    pub ts: String,
    pub source: String,
    pub label: String,
    pub entities: Vec<String>,
    pub relations_promoted: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentDiscoveriesFile {
    pub updated_at: String,
    pub entries: Vec<RecentDiscoveryEntry>,
}

pub fn default_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(DEFAULT_FILENAME)
}

pub fn record_ingest(
    data_dir: &Path,
    file_name: &str,
    entity_names: &[String],
    relations_promoted: usize,
) -> Result<()> {
    append_entry(
        data_dir,
        RecentDiscoveryEntry {
            ts: Utc::now().to_rfc3339(),
            source: "ingest".into(),
            label: file_name.to_string(),
            entities: entity_names.iter().take(8).cloned().collect(),
            relations_promoted,
        },
    )
}

pub fn record_distill(
    data_dir: &Path,
    session_id: &str,
    entity_names: &[String],
    relations_promoted: usize,
) -> Result<()> {
    append_entry(
        data_dir,
        RecentDiscoveryEntry {
            ts: Utc::now().to_rfc3339(),
            source: "distill".into(),
            label: session_id.to_string(),
            entities: entity_names.iter().take(8).cloned().collect(),
            relations_promoted,
        },
    )
}

fn append_entry(data_dir: &Path, entry: RecentDiscoveryEntry) -> Result<()> {
    if entry.entities.is_empty() && entry.relations_promoted == 0 {
        return Ok(());
    }
    let path = default_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = load_file(&path);
    file.entries.insert(0, entry);
    file.entries.truncate(MAX_ENTRIES);
    file.updated_at = Utc::now().to_rfc3339();
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(&path, json)?;
    Ok(())
}

fn load_file(path: &Path) -> RecentDiscoveriesFile {
    if !path.is_file() {
        return RecentDiscoveriesFile::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
