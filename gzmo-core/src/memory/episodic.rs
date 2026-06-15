//! Episodic daily log: append-only memory/YYYY-MM-DD.md ledger.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use crate::types::{EpisodicEntry, EpisodicSource};
use tokio::io::AsyncWriteExt;
use tracing::debug;

/// File-backed episodic store writing to memory/YYYY-MM-DD.md files.
pub struct FileEpisodicStore {
    memory_dir: PathBuf,
}

impl FileEpisodicStore {
    pub fn new(memory_dir: impl AsRef<Path>) -> Self {
        Self {
            memory_dir: memory_dir.as_ref().to_path_buf(),
        }
    }

    /// Get the path for a specific date's log file.
    fn path_for_date(&self, date: NaiveDate) -> PathBuf {
        self.memory_dir
            .join(format!("{}.md", date.format("%Y-%m-%d")))
    }

    /// Append an entry to today's episodic log.
    pub async fn append(&self, entry: &EpisodicEntry) -> Result<()> {
        let today = Utc::now().date_naive();
        let path = self.path_for_date(today);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let formatted = format_entry(entry);

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .with_context(|| format!("Failed to open episodic log: {:?}", path))?;

        file.write_all(formatted.as_bytes()).await?;
        file.flush().await?;

        debug!(date = %today, bytes = formatted.len(), "Appended to episodic log");
        Ok(())
    }

    /// Read all entries for a given date.
    pub async fn read_day(&self, date: NaiveDate) -> Result<String> {
        let path = self.path_for_date(date);
        if !path.exists() {
            return Ok(String::new());
        }
        tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read episodic log: {:?}", path))
    }

    /// List all available episodic log dates.
    pub async fn list_dates(&self) -> Result<Vec<NaiveDate>> {
        let mut dates = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.memory_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if let Some(date_str) = name_str.strip_suffix(".md") {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    dates.push(date);
                }
            }
        }

        dates.sort();
        Ok(dates)
    }
}

/// Format an episodic entry as markdown for the daily log.
fn format_entry(entry: &EpisodicEntry) -> String {
    let source_tag = match &entry.source {
        EpisodicSource::UserChat => "💬 USER",
        EpisodicSource::HeartbeatCheck => "💓 HEARTBEAT",
        EpisodicSource::ToolExecution { tool_name } => return format!(
            "\n### 🔧 TOOL: {} — {}\n{}\n",
            tool_name,
            entry.timestamp.format("%H:%M:%S"),
            entry.content
        ),
        EpisodicSource::InternalMonologue => "🧠 INTERNAL",
        EpisodicSource::SessionDistill { session_id } => {
            return format!(
                "\n### 📓 SESSION {session_id} — {}\n{}\n",
                entry.timestamp.format("%H:%M:%S"),
                entry.content
            );
        }
    };

    format!(
        "\n### {} — {}\n{}\n",
        source_tag,
        entry.timestamp.format("%H:%M:%S"),
        entry.content
    )
}
