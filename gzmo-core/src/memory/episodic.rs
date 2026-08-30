//! Episodic daily log: append-only memory/YYYY-MM-DD.md ledger.

use std::path::{Path, PathBuf};

use crate::types::{EpisodicEntry, EpisodicSource};
use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
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

    /// List all available episodic log dates (empty if the dir does not exist yet).
    pub async fn list_dates(&self) -> Result<Vec<NaiveDate>> {
        let mut dates = Vec::new();
        if !self.memory_dir.exists() {
            return Ok(dates);
        }
        let mut entries = tokio::fs::read_dir(&self.memory_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_file() {
                continue;
            }
            if let Some(date) = parse_log_date(&entry.file_name().to_string_lossy()) {
                dates.push(date);
            }
        }

        dates.sort();
        Ok(dates)
    }

    /// Concatenate the last `n` dated logs, oldest first. `n == 0` → empty.
    pub async fn read_recent_days(&self, n: usize) -> Result<String> {
        let dates = self.list_dates().await?;
        let mut out = String::new();
        for date in recent_dates(&dates, n) {
            let day = self.read_day(*date).await?;
            if day.is_empty() {
                continue;
            }
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&day);
        }
        Ok(out)
    }
}

/// `YYYY-MM-DD.md` → date. Rejects notes, backups, and invalid calendar days.
pub fn parse_log_date(file_name: &str) -> Option<NaiveDate> {
    let date_str = file_name.strip_suffix(".md")?;
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

/// Last `n` dates from a sorted-ascending slice. `n == 0` or empty → empty.
pub fn recent_dates(dates: &[NaiveDate], n: usize) -> &[NaiveDate] {
    let start = dates.len().saturating_sub(n);
    &dates[start..]
}

/// Format an episodic entry as markdown for the daily log.
fn format_entry(entry: &EpisodicEntry) -> String {
    let source_tag = match &entry.source {
        EpisodicSource::UserChat => "💬 USER",
        EpisodicSource::HeartbeatCheck => "💓 HEARTBEAT",
        EpisodicSource::ToolExecution { tool_name } => {
            return format!(
                "\n### 🔧 TOOL: {} — {}\n{}\n",
                tool_name,
                entry.timestamp.format("%H:%M:%S"),
                entry.content
            )
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).expect("valid date")
    }

    #[test]
    fn parse_log_date_accepts_iso_md_only() {
        assert_eq!(parse_log_date("2026-08-30.md"), Some(d(2026, 8, 30)));
        assert_eq!(parse_log_date("2024-02-29.md"), Some(d(2024, 2, 29)));
        assert_eq!(parse_log_date("2026-02-29.md"), None);
        assert_eq!(parse_log_date(""), None);
        assert_eq!(parse_log_date("notes.md"), None);
        assert_eq!(parse_log_date("2026-08-30.md.bak"), None);
        assert_eq!(parse_log_date("2026-13-01.md"), None);
        assert_eq!(parse_log_date("2026-08-30.txt"), None);
    }

    #[test]
    fn recent_dates_empty_and_bounds() {
        let dates = [d(2026, 8, 28), d(2026, 8, 29), d(2026, 8, 30)];
        assert!(recent_dates(&[], 3).is_empty());
        assert!(recent_dates(&dates, 0).is_empty());
        assert_eq!(recent_dates(&dates, 1), &[d(2026, 8, 30)]);
        assert_eq!(recent_dates(&dates, 2), &[d(2026, 8, 29), d(2026, 8, 30)]);
        assert_eq!(recent_dates(&dates, 99), &dates[..]);
    }

    #[test]
    fn format_entry_tags_user_and_tool() {
        let ts = Utc.with_ymd_and_hms(2026, 8, 30, 18, 1, 2).unwrap();
        let user = EpisodicEntry {
            timestamp: ts,
            source: EpisodicSource::UserChat,
            content: "hello keep".into(),
            is_silent: false,
        };
        let tool = EpisodicEntry {
            timestamp: ts,
            source: EpisodicSource::ToolExecution {
                tool_name: "memory_search".into(),
            },
            content: "q=felt use".into(),
            is_silent: false,
        };
        let u = format_entry(&user);
        assert!(u.contains("💬 USER"));
        assert!(u.contains("18:01:02"));
        assert!(u.contains("hello keep"));
        let t = format_entry(&tool);
        assert!(t.contains("🔧 TOOL: memory_search"));
        assert!(t.contains("q=felt use"));
    }

    #[tokio::test]
    async fn list_dates_missing_dir_is_empty() {
        let dir = std::env::temp_dir().join(format!("gzmo-epi-missing-{}", uuid::Uuid::new_v4()));
        let store = FileEpisodicStore::new(&dir);
        let dates = store.list_dates().await.expect("missing dir");
        assert!(dates.is_empty());
        assert!(store.read_recent_days(3).await.expect("recent").is_empty());
    }

    #[tokio::test]
    async fn read_recent_days_window_oldest_first() {
        let dir = std::env::temp_dir().join(format!("gzmo-epi-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("notes.md"), "ignore").unwrap();
        std::fs::write(dir.join("2026-08-28.md"), "day28\n").unwrap();
        std::fs::write(dir.join("2026-08-29.md"), "day29\n").unwrap();
        std::fs::write(dir.join("2026-08-30.md"), "day30\n").unwrap();
        let store = FileEpisodicStore::new(&dir);
        let dates = store.list_dates().await.expect("dates");
        assert_eq!(dates, vec![d(2026, 8, 28), d(2026, 8, 29), d(2026, 8, 30)]);
        assert!(store.read_recent_days(0).await.expect("n=0").is_empty());
        let window = store.read_recent_days(2).await.expect("n=2");
        assert!(window.contains("day29"));
        assert!(window.contains("day30"));
        assert!(!window.contains("day28"));
        assert!(!window.contains("ignore"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
