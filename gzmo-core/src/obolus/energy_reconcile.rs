//! Hardware energy sampling orchestration for reconcile ticks.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::Result;
use chrono::Utc;
use tracing::warn;

use crate::config::GzmoConfig;
use crate::obolus::energy_sampler::{
    integrate_gpu_joules, load_sampler_state, query_nvidia_smi_power, sample_cpu_rapl,
    save_sampler_state,
};
use crate::obolus::hsp_client::fetch_hsp_state;
use crate::obolus::power_ledger::{GpuEnergySource, PowerLedger, PowerLedgerEntry};

const RAPL_SAMPLER_STATE: &str = "Obolus/rapl-sampler.state.json";

static POWER_LEDGER: OnceLock<Arc<PowerLedger>> = OnceLock::new();

fn project_data_dir(config: &GzmoConfig) -> PathBuf {
    config
        .memory
        .vault_db
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn power_ledger_instance(config: &GzmoConfig) -> Result<Arc<PowerLedger>> {
    if let Some(existing) = POWER_LEDGER.get() {
        return Ok(existing.clone());
    }
    let ledger = PowerLedger::open(&config.obolus_analytics)?;
    let _ = POWER_LEDGER.set(ledger.clone());
    Ok(ledger)
}

fn rapl_paths(config: &GzmoConfig) -> Vec<PathBuf> {
    let custom = config.obolus_analytics.rapl_energy_path.trim();
    if custom.is_empty() {
        crate::obolus::energy_sampler::default_rapl_paths()
    } else {
        vec![PathBuf::from(custom)]
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname"))
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string()
}

/// Sample CPU RAPL + GPU power and append to `power.jsonl`. Never fails the reconcile tick.
pub async fn sample_and_record_energy(config: &GzmoConfig) {
    if !config.obolus_analytics.energy_sampler_enabled {
        return;
    }

    if let Err(e) = sample_and_record_energy_inner(config).await {
        warn!(error = %e, "obolus energy sampler failed");
    }
}

async fn sample_and_record_energy_inner(config: &GzmoConfig) -> Result<()> {
    let data_dir = project_data_dir(config);
    let state_path = data_dir.join(RAPL_SAMPLER_STATE);
    let mut state = load_sampler_state(&state_path);

    let min_interval = config.obolus_analytics.energy_sample_min_interval_secs;
    if let Some(prev) = state.last_sample_ts {
        let elapsed = (Utc::now() - prev).num_seconds().unsigned_abs();
        if elapsed < min_interval {
            return Ok(());
        }
    }

    let cpu_sample = sample_cpu_rapl(&rapl_paths(config), &mut state);

    let mut gpu_power_w: Option<f64> = None;
    let mut gpu_source = GpuEnergySource::None;
    let mut hsp_metrics_source: Option<String> = None;

    if let Some(hsp) = fetch_hsp_state(&config.obolus_analytics.hsp_state_url).await? {
        if !hsp.warming_up {
            if let Some(p) = hsp.gpu_power_w {
                gpu_power_w = Some(p);
                gpu_source = GpuEnergySource::Hsp;
            }
            hsp_metrics_source = Some(hsp.metrics_source);
        }
    }

    if gpu_power_w.is_none()
        && config.obolus_analytics.nvidia_smi_fallback
    {
        if let Some(p) = query_nvidia_smi_power() {
            gpu_power_w = Some(p);
            gpu_source = GpuEnergySource::NvidiaSmi;
        }
    }

    let now = Utc::now();
    let (gpu_joules_est, _) = if config.obolus_analytics.gpu_joules_integration {
        if let Some(p) = gpu_power_w {
            integrate_gpu_joules(p, &mut state, now)
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    };

    let ledger = power_ledger_instance(config)?;
    ledger.record(PowerLedgerEntry {
        ts: now,
        source: "reconcile_sampler".into(),
        cpu_joules: cpu_sample.joules,
        cpu_joules_wh: cpu_sample.joules_wh,
        cpu_watts_avg: cpu_sample.watts_avg,
        cpu_energy_source: cpu_sample.source,
        gpu_power_w,
        gpu_joules_est,
        gpu_energy_source: gpu_source,
        hsp_metrics_source,
        sample_interval_s: cpu_sample.elapsed_s,
        host: hostname(),
    });
    ledger.flush();

    save_sampler_state(&state_path, &state)?;
    Ok(())
}
