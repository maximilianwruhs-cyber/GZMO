//! Append-only JSONL hardware energy ledger (`data/Obolus/power.jsonl`).

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
use crate::obolus::energy_sampler::CpuEnergySource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuEnergySource {
    Hsp,
    NvidiaSmi,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PowerLedgerEntry {
    pub ts: DateTime<Utc>,
    pub source: String,
    pub cpu_joules: f64,
    pub cpu_joules_wh: f64,
    pub cpu_watts_avg: f64,
    pub cpu_energy_source: CpuEnergySource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu_power_w: Option<f64>,
    #[serde(default)]
    pub gpu_joules_est: f64,
    pub gpu_energy_source: GpuEnergySource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hsp_metrics_source: Option<String>,
    pub sample_interval_s: f64,
    pub host: String,
}

enum WriterMsg {
    Entry(PowerLedgerEntry),
    Flush,
    Shutdown,
}

/// Background JSONL writer for hardware energy samples.
pub struct PowerLedger {
    tx: Sender<WriterMsg>,
    path: PathBuf,
    _writer: JoinHandle<()>,
}

impl PowerLedger {
    pub fn open(cfg: &ObolusAnalyticsConfig) -> Result<Arc<Self>> {
        let path = PathBuf::from(&cfg.power_ledger_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create power ledger dir {}", parent.display()))?;
        }

        let batch_size = cfg.writer_batch_size.max(1);
        let flush_ms = cfg.writer_flush_ms.max(50);
        let (tx, rx) = mpsc::channel::<WriterMsg>();
        let write_path = path.clone();

        let writer = thread::spawn(move || {
            let mut buffer: Vec<PowerLedgerEntry> = Vec::with_capacity(batch_size);
            let mut last_flush = std::time::Instant::now();

            let flush_buffer = |buf: &mut Vec<PowerLedgerEntry>, path: &Path| {
                if buf.is_empty() {
                    return;
                }
                match OpenOptions::new().create(true).append(true).open(path) {
                    Ok(mut file) => {
                        for entry in buf.drain(..) {
                            match serde_json::to_string(&entry) {
                                Ok(line) => {
                                    if let Err(e) = writeln!(file, "{line}") {
                                        eprintln!("Warning: failed to write power ledger entry: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Warning: failed to serialize power ledger entry: {}", e);
                                }
                            }
                        }
                        if let Err(e) = file.flush() {
                            eprintln!("Warning: failed to flush power ledger: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: failed to open power ledger for writing: {}", e);
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

    pub fn record(&self, entry: PowerLedgerEntry) {
        let _ = self.tx.send(WriterMsg::Entry(entry));
    }

    pub fn flush(&self) {
        let _ = self.tx.send(WriterMsg::Flush);
    }

    pub fn read_since(since: DateTime<Utc>, path: &Path) -> Result<Vec<PowerLedgerEntry>> {
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
            let entry: PowerLedgerEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.ts >= since {
                out.push(entry);
            }
        }
        Ok(out)
    }
}

impl Drop for PowerLedger {
    fn drop(&mut self) {
        let _ = self.tx.send(WriterMsg::Shutdown);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PowerRollup {
    pub joules_cpu_1h: f64,
    pub joules_wh_cpu_1h: f64,
    pub joules_gpu_est_1h: f64,
    pub joules_wh_total_est_1h: f64,
    pub power_sample_count: usize,
}

pub fn rolling_power_rollups(entries: &[PowerLedgerEntry]) -> PowerRollup {
    let mut joules_cpu = 0.0f64;
    let mut joules_gpu = 0.0f64;
    for e in entries {
        joules_cpu += e.cpu_joules;
        joules_gpu += e.gpu_joules_est;
    }
    let joules_wh_cpu = joules_cpu / 3600.0;
    let joules_wh_gpu = joules_gpu / 3600.0;
    PowerRollup {
        joules_cpu_1h: joules_cpu,
        joules_wh_cpu_1h: joules_wh_cpu,
        joules_gpu_est_1h: joules_gpu,
        joules_wh_total_est_1h: joules_wh_cpu + joules_wh_gpu,
        power_sample_count: entries.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_ledger_roundtrip() {
        let dir = std::env::temp_dir().join(format!("power-ledger-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("power.jsonl");
        let cfg = ObolusAnalyticsConfig {
            power_ledger_path: path.to_string_lossy().into_owned(),
            ..ObolusAnalyticsConfig::default()
        };
        let ledger = PowerLedger::open(&cfg).unwrap();
        let ts = Utc::now();
        ledger.record(PowerLedgerEntry {
            ts,
            source: "reconcile_sampler".into(),
            cpu_joules: 10.0,
            cpu_joules_wh: 10.0 / 3600.0,
            cpu_watts_avg: 5.0,
            cpu_energy_source: CpuEnergySource::Rapl,
            gpu_power_w: Some(100.0),
            gpu_joules_est: 20.0,
            gpu_energy_source: GpuEnergySource::Hsp,
            hsp_metrics_source: Some("local".into()),
            sample_interval_s: 60.0,
            host: "test".into(),
        });
        ledger.flush();
        thread::sleep(Duration::from_millis(300));
        let entries =
            PowerLedger::read_since(ts - chrono::Duration::seconds(1), &path).unwrap();
        assert_eq!(entries.len(), 1);
        assert!((entries[0].cpu_joules - 10.0).abs() < 0.01);
        let rollup = rolling_power_rollups(&entries);
        assert!((rollup.joules_cpu_1h - 10.0).abs() < 0.01);
        let _ = fs::remove_dir_all(dir);
    }
}
