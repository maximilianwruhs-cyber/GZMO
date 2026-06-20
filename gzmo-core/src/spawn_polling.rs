//! Polling + eventual-consistency retry (Jules SDK polling.ts pattern).
//!
//! Used when reading remediation tracker / snapshot JSON immediately after spawn flush.

use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PollConfig {
    pub interval: Duration,
    pub timeout: Duration,
    pub max_attempts: u32,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(poll_interval_ms()),
            timeout: Duration::from_millis(poll_timeout_ms()),
            max_attempts: load_retry_attempts(),
        }
    }
}

fn poll_interval_ms() -> u64 {
    std::env::var("SPAWN_POLL_INTERVAL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
}

fn poll_timeout_ms() -> u64 {
    std::env::var("SPAWN_POLL_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2000)
}

fn load_retry_attempts() -> u32 {
    std::env::var("SPAWN_LOAD_RETRY_ATTEMPTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutError {
    pub message: String,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TimeoutError {}

/// Read a file with exponential backoff (eventual consistency after concurrent write).
pub fn read_file_with_retry(path: &Path, config: &PollConfig) -> Option<String> {
    let start = Instant::now();
    let mut attempt = 0u32;
    let mut delay = config.interval;

    loop {
        if path.is_file() {
            if let Ok(raw) = std::fs::read_to_string(path) {
                if !raw.trim().is_empty() {
                    return Some(raw);
                }
            }
        }

        attempt += 1;
        if attempt >= config.max_attempts || start.elapsed() >= config.timeout {
            return None;
        }

        thread::sleep(delay);
        delay = delay.saturating_mul(2).min(Duration::from_millis(500));
    }
}

/// Poll until `predicate` returns true or timeout.
pub fn poll_until<F>(mut predicate: F, config: &PollConfig) -> Result<(), TimeoutError>
where
    F: FnMut() -> bool,
{
    let start = Instant::now();
    loop {
        if predicate() {
            return Ok(());
        }
        if start.elapsed() >= config.timeout {
            return Err(TimeoutError {
                message: format!("poll timed out after {}ms", config.timeout.as_millis()),
            });
        }
        thread::sleep(config.interval);
    }
}

/// Deserialize JSON from path with retry (tracker flush race).
pub fn load_json_with_retry<T: serde::de::DeserializeOwned>(
    path: &Path,
    config: &PollConfig,
) -> Option<T> {
    read_file_with_retry(path, config).and_then(|raw| serde_json::from_str(&raw).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_file_after_retry() {
        let dir = std::env::temp_dir().join(format!(
            "gzmo-poll-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.json");

        let config = PollConfig {
            interval: Duration::from_millis(20),
            timeout: Duration::from_millis(500),
            max_attempts: 10,
        };

        std::thread::spawn({
            let path = path.clone();
            move || {
                std::thread::sleep(Duration::from_millis(80));
                std::fs::write(&path, r#"{"ok":true}"#).unwrap();
            }
        });

        let raw = read_file_with_retry(&path, &config);
        assert!(raw.is_some(), "expected file after delayed write");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn timeout_when_file_never_appears() {
        let path = std::env::temp_dir().join(format!(
            "gzmo-poll-missing-{}",
            std::process::id()
        ));
        let config = PollConfig {
            interval: Duration::from_millis(10),
            timeout: Duration::from_millis(50),
            max_attempts: 3,
        };
        assert!(read_file_with_retry(&path, &config).is_none());
    }
}
