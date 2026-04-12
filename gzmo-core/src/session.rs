//! # Session Persistence
//!
//! Manages conversation sessions: save, load, list, and auto-resume.
//! Sessions are stored as JSON files under `data/sessions/`.
//!
//! Each session captures the full `Vec<Message>` conversation history
//! along with metadata (creation time, last active time, message count,
//! and an optional human-readable name).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::types::Message;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A persistable conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier.
    pub id: String,

    /// Optional human-readable name (set via `/save my_project`).
    #[serde(default)]
    pub name: Option<String>,

    /// When this session was first created.
    pub created_at: DateTime<Utc>,

    /// When this session was last written to disk.
    pub last_active_at: DateTime<Utc>,

    /// The full conversation history.
    pub messages: Vec<Message>,
}

/// Lightweight metadata for listing sessions without loading full message history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub message_count: usize,
}

// ---------------------------------------------------------------------------
// Session Manager
// ---------------------------------------------------------------------------

/// Manages session persistence on the filesystem.
pub struct SessionManager {
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager rooted at the given directory.
    /// The directory will be created if it doesn't exist.
    pub fn new(sessions_dir: impl AsRef<Path>) -> Self {
        Self {
            sessions_dir: sessions_dir.as_ref().to_path_buf(),
        }
    }

    /// Ensure the sessions directory exists.
    pub async fn ensure_dir(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.sessions_dir)
            .await
            .with_context(|| {
                format!(
                    "Failed to create sessions directory: {}",
                    self.sessions_dir.display()
                )
            })
    }

    /// Generate a new unique session ID.
    pub fn new_session_id() -> String {
        Uuid::new_v4().to_string()[..8].to_string()
    }

    /// Get the file path for a session by ID.
    fn session_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", session_id))
    }

    /// Save a session to disk atomically (write-then-rename).
    pub async fn save(
        &self,
        session_id: &str,
        name: Option<&str>,
        messages: &[Message],
        created_at: DateTime<Utc>,
    ) -> Result<()> {
        self.ensure_dir().await?;

        let session = Session {
            id: session_id.to_string(),
            name: name.map(|s| s.to_string()),
            created_at,
            last_active_at: Utc::now(),
            messages: messages.to_vec(),
        };

        let json = serde_json::to_string_pretty(&session)
            .context("Failed to serialize session")?;

        let path = self.session_path(session_id);
        let tmp_path = self.sessions_dir.join(format!("{}.json.tmp", session_id));

        // Atomic write: write to .tmp then rename (POSIX rename is atomic)
        tokio::fs::write(&tmp_path, json.as_bytes())
            .await
            .with_context(|| format!("Failed to write session tmp file: {}", tmp_path.display()))?;

        tokio::fs::rename(&tmp_path, &path)
            .await
            .with_context(|| format!("Failed to rename session file: {} -> {}", tmp_path.display(), path.display()))?;

        info!(
            session_id = %session_id,
            messages = messages.len(),
            path = %path.display(),
            "Session saved (atomic)"
        );
        Ok(())
    }

    /// Load a session from disk by ID.
    pub async fn load(&self, session_id: &str) -> Result<Session> {
        let path = self.session_path(session_id);

        let content = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read session file: {}", path.display()))?;

        let session: Session = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse session file: {}", path.display()))?;

        info!(
            session_id = %session.id,
            messages = session.messages.len(),
            "Session loaded"
        );
        Ok(session)
    }

    /// Load a session by name (searches all sessions for a matching name).
    pub async fn load_by_name(&self, name: &str) -> Result<Option<Session>> {
        let metas = self.list().await?;
        let name_lower = name.to_lowercase();

        for meta in metas {
            if let Some(ref n) = meta.name {
                if n.to_lowercase() == name_lower {
                    return self.load(&meta.id).await.map(Some);
                }
            }
        }
        Ok(None)
    }

    /// List all saved sessions, sorted by last_active_at descending (most recent first).
    pub async fn list(&self) -> Result<Vec<SessionMeta>> {
        self.ensure_dir().await?;

        let mut entries = tokio::fs::read_dir(&self.sessions_dir).await?;
        let mut metas = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    match serde_json::from_str::<Session>(&content) {
                        Ok(session) => {
                            metas.push(SessionMeta {
                                id: session.id,
                                name: session.name,
                                created_at: session.created_at,
                                last_active_at: session.last_active_at,
                                message_count: session.messages.len(),
                            });
                        }
                        Err(e) => {
                            warn!(
                                path = %path.display(),
                                error = %e,
                                "Skipping corrupt session file"
                            );
                        }
                    }
                }
                Err(e) => {
                    debug!(
                        path = %path.display(),
                        error = %e,
                        "Could not read session file"
                    );
                }
            }
        }

        // Sort by last_active descending
        metas.sort_by(|a, b| b.last_active_at.cmp(&a.last_active_at));

        Ok(metas)
    }

    /// Find the most recently active session (for auto-resume).
    pub async fn most_recent(&self) -> Result<Option<Session>> {
        let metas = self.list().await?;
        match metas.first() {
            Some(meta) => self.load(&meta.id).await.map(Some),
            None => Ok(None),
        }
    }

    /// Delete a session by ID.
    pub async fn delete(&self, session_id: &str) -> Result<()> {
        let path = self.session_path(session_id);
        if path.exists() {
            tokio::fs::remove_file(&path)
                .await
                .with_context(|| format!("Failed to delete session: {}", path.display()))?;
            info!(session_id = %session_id, "Session deleted");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Role;

    #[tokio::test]
    async fn test_session_roundtrip() {
        let dir = std::env::temp_dir().join("sovereign_test_sessions");
        let _ = tokio::fs::remove_dir_all(&dir).await;

        let mgr = SessionManager::new(&dir);
        let id = SessionManager::new_session_id();

        let messages = vec![
            Message {
                role: Role::System,
                content: "You are GZMO.".to_string(),
                is_meta: true, tool_calls: None, tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: "Hello".to_string(),
                is_meta: false, tool_calls: None, tool_call_id: None,
            },
        ];

        // Save
        mgr.save(&id, Some("test_session"), &messages, Utc::now())
            .await
            .expect("save failed");

        // Load
        let loaded = mgr.load(&id).await.expect("load failed");
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.name.as_deref(), Some("test_session"));

        // List
        let list = mgr.list().await.expect("list failed");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].message_count, 2);

        // Load by name
        let by_name = mgr.load_by_name("test_session").await.expect("load_by_name failed");
        assert!(by_name.is_some());

        // Cleanup
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
