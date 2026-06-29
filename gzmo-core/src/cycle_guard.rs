//! Critical-cycle guard — prevents daemon restarts during Dream / Discovery implement.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const STALE_GUARD_SECS: i64 = 3 * 3600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CycleGuardRecord {
    pub kind: String,
    pub pid: u32,
    pub started_at: String,
}

pub fn guard_path(data_dir: &Path) -> PathBuf {
    data_dir.join("cycle-guard.json")
}

fn pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

fn parse_started_at(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Returns active guard record when a live process holds the lock.
pub fn active_guard(data_dir: &Path) -> Option<CycleGuardRecord> {
    let path = guard_path(data_dir);
    if !path.is_file() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    let rec: CycleGuardRecord = serde_json::from_str(&raw).ok()?;
    if let Some(started) = parse_started_at(&rec.started_at) {
        if Utc::now().signed_duration_since(started).num_seconds() > STALE_GUARD_SECS {
            if let Err(e) = std::fs::remove_file(&path) {
                tracing::debug!(path = %path.display(), error = %e, "Failed to remove expired cycle guard file");
            }
            return None;
        }
    }
    if pid_alive(rec.pid) {
        Some(rec)
    } else {
        if let Err(e) = std::fs::remove_file(&path) {
            tracing::debug!(path = %path.display(), error = %e, "Failed to remove stale cycle guard file");
        }
        None
    }
}

pub struct CycleGuard {
    path: PathBuf,
}

impl CycleGuard {
    pub fn acquire(data_dir: &Path, kind: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let path = guard_path(data_dir);
        let rec = CycleGuardRecord {
            kind: kind.to_string(),
            pid: std::process::id(),
            started_at: Utc::now().to_rfc3339(),
        };
        let json_bytes = serde_json::to_vec(&rec).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON serialization failed: {}", e))
        })?;
        std::fs::write(&path, json_bytes)?;
        Ok(Self { path })
    }

    pub fn release(self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::debug!(path = %self.path.display(), error = %e, "Failed to remove cycle guard file");
        }
    }
}

impl Drop for CycleGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::debug!(path = %self.path.display(), error = %e, "Failed to remove cycle guard file in drop");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_guard_clears_dead_pid() {
        let tmp = std::env::temp_dir().join(format!(
            "gzmo-cycle-guard-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let path = guard_path(&tmp);
        std::fs::write(
            &path,
            r#"{"kind":"dream","pid":999999999,"started_at":"2026-06-20T04:00:00Z"}"#,
        )
        .unwrap();
        assert!(active_guard(&tmp).is_none());
        assert!(!path.exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn acquire_and_release_roundtrip() {
        let tmp = std::env::temp_dir().join(format!(
            "gzmo-cycle-guard-live-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        {
            let _guard = CycleGuard::acquire(&tmp, "dream").unwrap();
            let active = active_guard(&tmp).expect("guard active");
            assert_eq!(active.kind, "dream");
            assert_eq!(active.pid, std::process::id());
        }
        assert!(active_guard(&tmp).is_none());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
