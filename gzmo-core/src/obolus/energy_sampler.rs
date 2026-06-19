//! CPU RAPL energy sampling for Obolus experiment infrastructure.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::System;

const RAPL_OVERFLOW_UJ: u64 = 1u64 << 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CpuEnergySource {
    Rapl,
    Estimate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RaplSamplerState {
    #[serde(default)]
    pub last_energy_uj: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sample_ts: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_gpu_power_w: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_gpu_sample_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpuEnergySample {
    pub ts: DateTime<Utc>,
    pub joules: f64,
    pub joules_wh: f64,
    pub watts_avg: f64,
    pub elapsed_s: f64,
    pub source: CpuEnergySource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rapl_paths: Vec<String>,
}

pub fn default_rapl_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/sys/class/powercap/intel-rapl:0/energy_uj"),
        PathBuf::from("/sys/class/powercap/intel-rapl:1/energy_uj"),
    ]
}

fn read_energy_uj(paths: &[PathBuf]) -> (u64, Vec<String>) {
    let mut total = 0u64;
    let mut active = Vec::new();
    for path in paths {
        if !path.exists() {
            continue;
        }
        match fs::read_to_string(path) {
            Ok(raw) => match raw.trim().parse::<u64>() {
                Ok(v) => {
                    total = total.saturating_add(v);
                    active.push(path.display().to_string());
                }
                Err(_) => continue,
            },
            Err(_) => continue,
        }
    }
    (total, active)
}

fn delta_uj(start: u64, end: u64) -> u64 {
    if end >= start {
        end - start
    } else {
        RAPL_OVERFLOW_UJ - start + end
    }
}

fn estimate_joules(elapsed_s: f64) -> f64 {
    let mut sys = System::new();
    sys.refresh_cpu_usage();
    let load = (sys.global_cpu_usage() as f64 / 100.0).clamp(0.1, 1.0);
    15.0 * load * elapsed_s.max(0.001)
}

/// Sample CPU energy delta since the previous persisted state.
pub fn sample_cpu_rapl(
    paths: &[PathBuf],
    state: &mut RaplSamplerState,
) -> CpuEnergySample {
    let now = Utc::now();
    let (end_uj, active_paths) = read_energy_uj(paths);
    let rapl_available = !active_paths.is_empty();

    let (joules, source) = if rapl_available {
        if let Some(prev_ts) = state.last_sample_ts {
            let _elapsed = (now - prev_ts).num_milliseconds().max(1) as f64 / 1000.0;
            if state.last_energy_uj > 0 || end_uj > 0 {
                let duj = delta_uj(state.last_energy_uj, end_uj);
                let j = duj as f64 / 1_000_000.0;
                (j, CpuEnergySource::Rapl)
            } else {
                (0.0, CpuEnergySource::Rapl)
            }
        } else {
            (0.0, CpuEnergySource::Rapl)
        }
    } else {
        let elapsed = state
            .last_sample_ts
            .map(|prev| (now - prev).num_milliseconds().max(1) as f64 / 1000.0)
            .unwrap_or(1.0);
        (estimate_joules(elapsed), CpuEnergySource::Estimate)
    };

    let elapsed_s = state
        .last_sample_ts
        .map(|prev| (now - prev).num_milliseconds().max(1) as f64 / 1000.0)
        .unwrap_or(0.0);

    let watts_avg = if elapsed_s > 0.0 {
        joules / elapsed_s
    } else {
        0.0
    };

    if rapl_available {
        state.last_energy_uj = end_uj;
    }
    state.last_sample_ts = Some(now);

    CpuEnergySample {
        ts: now,
        joules,
        joules_wh: joules / 3600.0,
        watts_avg,
        elapsed_s,
        source,
        rapl_paths: active_paths,
    }
}

pub fn load_sampler_state(path: &Path) -> RaplSamplerState {
    if !path.exists() {
        return RaplSamplerState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_sampler_state(path: &Path, state: &RaplSamplerState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

/// Integrate GPU power (watts) over elapsed time → estimated joules.
pub fn integrate_gpu_joules(
    gpu_power_w: f64,
    state: &mut RaplSamplerState,
    now: DateTime<Utc>,
) -> (f64, f64) {
    let elapsed_s = state
        .last_gpu_sample_ts
        .map(|prev| (now - prev).num_milliseconds().max(0) as f64 / 1000.0)
        .unwrap_or(0.0);

    let prev_power = state.last_gpu_power_w.unwrap_or(gpu_power_w);
    let avg_power = if elapsed_s > 0.0 {
        (prev_power + gpu_power_w) / 2.0
    } else {
        0.0
    };
    let joules = avg_power * elapsed_s;

    state.last_gpu_power_w = Some(gpu_power_w);
    state.last_gpu_sample_ts = Some(now);

    (joules, elapsed_s)
}

/// Parse `nvidia-smi` CSV output for total GPU power draw (watts).
pub fn parse_nvidia_smi_power(stdout: &str) -> Option<f64> {
    let mut total = 0.0f64;
    let mut any = false;
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        let power_str = parts.last()?;
        if let Ok(p) = power_str.parse::<f64>() {
            if p.is_finite() && p >= 0.0 {
                total += p;
                any = true;
            }
        }
    }
    if any { Some(total) } else { None }
}

pub fn query_nvidia_smi_power() -> Option<f64> {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,power.draw",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_nvidia_smi_power(&stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn delta_uj_handles_overflow() {
        assert_eq!(delta_uj(0xFFFF_FFF0, 0x10), 32);
        assert_eq!(delta_uj(100, 200), 100);
    }

    #[test]
    fn first_rapl_sample_has_zero_joules() {
        let dir = std::env::temp_dir().join(format!("rapl-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let rapl_file = dir.join("energy_uj");
        let mut f = fs::File::create(&rapl_file).unwrap();
        writeln!(f, "1000000").unwrap();
        drop(f);

        let mut state = RaplSamplerState::default();
        let sample = sample_cpu_rapl(&[rapl_file.clone()], &mut state);
        assert_eq!(sample.joules, 0.0);
        assert_eq!(sample.source, CpuEnergySource::Rapl);

        let mut f2 = fs::OpenOptions::new().write(true).truncate(true).open(&rapl_file).unwrap();
        writeln!(f2, "2000000").unwrap();
        let sample2 = sample_cpu_rapl(&[rapl_file], &mut state);
        assert!((sample2.joules - 1.0).abs() < 0.01);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn estimate_when_no_rapl_paths() {
        let mut state = RaplSamplerState::default();
        state.last_sample_ts = Some(Utc::now() - chrono::Duration::seconds(2));
        let sample = sample_cpu_rapl(&[PathBuf::from("/nonexistent/energy_uj")], &mut state);
        assert_eq!(sample.source, CpuEnergySource::Estimate);
        assert!(sample.joules > 0.0);
    }

    #[test]
    fn parse_nvidia_smi_sums_gpus() {
        let out = "0, 45.12\n1, 120.50\n";
        let p = parse_nvidia_smi_power(out).unwrap();
        assert!((p - 165.62).abs() < 0.01);
    }

    #[test]
    fn gpu_integration_trapezoid() {
        let mut state = RaplSamplerState::default();
        let t0 = Utc::now() - chrono::Duration::seconds(10);
        state.last_gpu_sample_ts = Some(t0);
        state.last_gpu_power_w = Some(100.0);
        let (joules, elapsed) = integrate_gpu_joules(200.0, &mut state, Utc::now());
        assert!((elapsed - 10.0).abs() < 0.5);
        assert!((joules - 1500.0).abs() < 100.0);
    }
}
