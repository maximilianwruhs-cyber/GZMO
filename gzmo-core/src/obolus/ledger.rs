//! Append-only JSONL token ledger (`data/Obolus/ledger.jsonl`).

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::ObolusAnalyticsConfig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LedgerSource {
    Gateway,
    SynapsePi,
    LlamaLog,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

impl TokenUsage {
    pub fn from_openai(prompt: u64, completion: u64, total: u64) -> Self {
        let total_tokens = if total > 0 {
            total
        } else {
            prompt.saturating_add(completion)
        };
        Self {
            input_tokens: prompt,
            output_tokens: completion,
            total_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntry {
    pub ts: DateTime<Utc>,
    pub source: LedgerSource,
    pub process: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_kind: Option<String>,
    pub caller: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub latency_ms: u64,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedup_key: Option<String>,
}

enum WriterMsg {
    Entry(LedgerEntry),
    Flush,
    Shutdown,
}

/// Background JSONL writer with batched appends.
pub struct ObolusLedger {
    tx: Sender<WriterMsg>,
    path: PathBuf,
    _writer: JoinHandle<()>,
}

impl ObolusLedger {
    pub fn open(cfg: &ObolusAnalyticsConfig) -> Result<Arc<Self>> {
        let path = PathBuf::from(&cfg.ledger_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create ledger dir {}", parent.display()))?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            if path.exists() {
                if let Err(e) = fs::set_permissions(&path, fs::Permissions::from_mode(0o600)) {
                    eprintln!("Warning: failed to set ledger permissions: {}", e);
                }
            }
        }

        let batch_size = cfg.writer_batch_size.max(1);
        let flush_ms = cfg.writer_flush_ms.max(50);
        let (tx, rx) = mpsc::channel::<WriterMsg>();
        let write_path = path.clone();

        let writer = thread::spawn(move || {
            let mut buffer: Vec<LedgerEntry> = Vec::with_capacity(batch_size);
            let mut last_flush = std::time::Instant::now();

            let flush_buffer = |buf: &mut Vec<LedgerEntry>, path: &Path| {
                if buf.is_empty() {
                    return;
                }
                match OpenOptions::new().create(true).append(true).open(path) {
                    Ok(mut file) => {
                        for entry in buf.drain(..) {
                            match serde_json::to_string(&entry) {
                                Ok(line) => {
                                    if let Err(e) = writeln!(file, "{line}") {
                                        eprintln!("Warning: failed to write ledger entry: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Warning: failed to serialize ledger entry: {}", e);
                                }
                            }
                        }
                        if let Err(e) = file.flush() {
                            eprintln!("Warning: failed to flush ledger: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: failed to open ledger for writing: {}", e);
                    }
                }
            };

            loop {
                let timeout = Duration::from_millis(flush_ms);
                match rx.recv_timeout(timeout) {
                    Ok(WriterMsg::Entry(entry)) => {
                        buffer.push(entry);
                        if buffer.len() >= batch_size {
                            flush_buffer(&mut buffer, &write_path);
                            last_flush = std::time::Instant::now();
                        }
                    }
                    Ok(WriterMsg::Flush) => {
                        flush_buffer(&mut buffer, &write_path);
                        last_flush = std::time::Instant::now();
                    }
                    Ok(WriterMsg::Shutdown) => {
                        flush_buffer(&mut buffer, &write_path);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if !buffer.is_empty() && last_flush.elapsed() >= timeout {
                            flush_buffer(&mut buffer, &write_path);
                            last_flush = std::time::Instant::now();
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        flush_buffer(&mut buffer, &write_path);
                        break;
                    }
                }
            }
        });

        Ok(Arc::new(Self {
            tx,
            path,
            _writer: writer,
        }))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn record(&self, entry: LedgerEntry) {
        if let Err(e) = self.tx.send(WriterMsg::Entry(entry)) {
            eprintln!("Warning: failed to send ledger entry to writer thread: {}", e);
        }
    }

    pub fn flush(&self) {
        if let Err(e) = self.tx.send(WriterMsg::Flush) {
            eprintln!("Warning: failed to send flush command to ledger writer: {}", e);
        }
    }

    /// Read all entries with `ts >= since`.
    pub fn read_since(since: DateTime<Utc>, path: &Path) -> Result<Vec<LedgerEntry>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LedgerEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.ts >= since {
                out.push(entry);
            }
        }
        Ok(out)
    }

    /// Returns true if `dedup_key` already exists in the ledger file.
    pub fn dedup_key_exists(path: &Path, key: &str) -> Result<bool> {
        if !path.exists() || key.is_empty() {
            return Ok(false);
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LedgerEntry>(&line) {
                if entry.dedup_key.as_deref() == Some(key) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Drop for ObolusLedger {
    fn drop(&mut self) {
        let _ = self.tx.send(WriterMsg::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_read_roundtrip() {
        let dir = std::env::temp_dir().join(format!("obolus-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ledger.jsonl");
        let cfg = ObolusAnalyticsConfig {
            enabled: true,
            ledger_path: path.to_string_lossy().into_owned(),
            ..ObolusAnalyticsConfig::default()
        };
        let ledger = ObolusLedger::open(&cfg).unwrap();
        let ts = Utc::now();
        ledger.record(LedgerEntry {
            ts,
            source: LedgerSource::Gateway,
            process: "chat".into(),
            task_kind: Some("chat".into()),
            caller: "test".into(),
            input_tokens: 10,
            output_tokens: 5,
            total_tokens: 15,
            latency_ms: 42,
            ok: true,
            model: Some("test-model".into()),
            correlation_id: None,
            action_id: None,
            dedup_key: None,
        });
        ledger.flush();
        thread::sleep(Duration::from_millis(300));
        let entries = ObolusLedger::read_since(ts - chrono::Duration::seconds(1), &path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].total_tokens, 15);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
