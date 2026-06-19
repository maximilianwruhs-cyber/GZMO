//! Token ↔ joule correlation for experiment analysis.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::obolus::ledger::LedgerEntry;
use crate::obolus::power_ledger::PowerLedgerEntry;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HourlyEnergyBucket {
    pub hour_bucket: String,
    pub e_total: u64,
    pub joules_cpu: f64,
    pub joules_gpu_est: f64,
    pub joules_wh_total_est: f64,
    pub tokens_per_wh: Option<f64>,
    pub power_samples: usize,
    pub token_calls: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnergyCorrelationReport {
    pub since: DateTime<Utc>,
    pub buckets: Vec<HourlyEnergyBucket>,
    pub pearson_tokens_wh: Option<f64>,
}

fn hour_key(ts: DateTime<Utc>) -> String {
    ts.format("%Y%m%d%H").to_string()
}

pub fn hourly_energy_buckets(
    token_entries: &[LedgerEntry],
    power_entries: &[PowerLedgerEntry],
) -> Vec<HourlyEnergyBucket> {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<String, HourlyEnergyBucket> = BTreeMap::new();

    for e in token_entries {
        let key = hour_key(e.ts);
        let bucket = map.entry(key.clone()).or_insert_with(|| HourlyEnergyBucket {
            hour_bucket: key,
            e_total: 0,
            joules_cpu: 0.0,
            joules_gpu_est: 0.0,
            joules_wh_total_est: 0.0,
            tokens_per_wh: None,
            power_samples: 0,
            token_calls: 0,
        });
        bucket.e_total = bucket.e_total.saturating_add(e.total_tokens);
        bucket.token_calls += 1;
    }

    for e in power_entries {
        let key = hour_key(e.ts);
        let bucket = map.entry(key.clone()).or_insert_with(|| HourlyEnergyBucket {
            hour_bucket: key,
            e_total: 0,
            joules_cpu: 0.0,
            joules_gpu_est: 0.0,
            joules_wh_total_est: 0.0,
            tokens_per_wh: None,
            power_samples: 0,
            token_calls: 0,
        });
        bucket.joules_cpu += e.cpu_joules;
        bucket.joules_gpu_est += e.gpu_joules_est;
        bucket.joules_wh_total_est =
            (bucket.joules_cpu + bucket.joules_gpu_est) / 3600.0;
        bucket.power_samples += 1;
    }

    for bucket in map.values_mut() {
        if bucket.joules_wh_total_est > 0.0 {
            bucket.tokens_per_wh = Some(bucket.e_total as f64 / bucket.joules_wh_total_est);
        }
    }

    map.into_values().collect()
}

fn pearson(xs: &[f64], ys: &[f64]) -> Option<f64> {
    if xs.len() != ys.len() || xs.len() < 2 {
        return None;
    }
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;
    for i in 0..xs.len() {
        let dx = xs[i] - mean_x;
        let dy = ys[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }
    let den = (den_x * den_y).sqrt();
    if den == 0.0 {
        None
    } else {
        Some(num / den)
    }
}

pub fn compute_energy_correlation(
    since: DateTime<Utc>,
    token_entries: &[LedgerEntry],
    power_entries: &[PowerLedgerEntry],
) -> EnergyCorrelationReport {
    let buckets = hourly_energy_buckets(token_entries, power_entries);
    let pairs: Vec<(f64, f64)> = buckets
        .iter()
        .filter_map(|b| {
            if b.joules_wh_total_est > 0.0 && b.e_total > 0 {
                Some((b.e_total as f64, b.joules_wh_total_est))
            } else {
                None
            }
        })
        .collect();
    let xs: Vec<f64> = pairs.iter().map(|(t, _)| *t).collect();
    let ys: Vec<f64> = pairs.iter().map(|(_, w)| *w).collect();

    EnergyCorrelationReport {
        since,
        pearson_tokens_wh: pearson(&xs, &ys),
        buckets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obolus::ledger::LedgerSource;

    #[test]
    fn hourly_buckets_merge_token_and_power() {
        let ts = Utc::now();
        let tokens = vec![crate::obolus::ledger::LedgerEntry {
            ts,
            source: LedgerSource::Gateway,
            process: "chat".into(),
            task_kind: None,
            caller: "test".into(),
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            latency_ms: 1,
            ok: true,
            model: None,
            correlation_id: None,
            action_id: None,
            dedup_key: None,
        }];
        let power = vec![crate::obolus::power_ledger::PowerLedgerEntry {
            ts,
            source: "test".into(),
            cpu_joules: 3600.0,
            cpu_joules_wh: 1.0,
            cpu_watts_avg: 1.0,
            cpu_energy_source: crate::obolus::energy_sampler::CpuEnergySource::Rapl,
            gpu_power_w: None,
            gpu_joules_est: 0.0,
            gpu_energy_source: crate::obolus::power_ledger::GpuEnergySource::None,
            hsp_metrics_source: None,
            sample_interval_s: 60.0,
            host: "test".into(),
        }];
        let buckets = hourly_energy_buckets(&tokens, &power);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].e_total, 150);
        assert!((buckets[0].tokens_per_wh.unwrap() - 150.0).abs() < 0.01);
    }
}
