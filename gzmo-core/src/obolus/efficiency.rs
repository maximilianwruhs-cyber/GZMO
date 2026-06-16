//! Wirkungsgrad η = (Q · I) / E_total per process family.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ledger::LedgerEntry;
use super::outcome::{collect_from_synapse, process_family, OutcomeSample};
use super::rollup::aggregate_by_process;

/// η rollup for one process family.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EfficiencyRollup {
    pub process: String,
    pub e_total: u64,
    pub q: f64,
    pub i: f64,
    pub eta: f64,
    /// η scaled per 1M tokens for readable CLI output.
    pub eta_per_million_tokens: f64,
    pub outcome_samples: u64,
    pub ledger_calls: u64,
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Build efficiency rollups from ledger entries + synapse outcome samples.
pub fn compute_efficiency(
    ledger_entries: &[LedgerEntry],
    outcomes: &[OutcomeSample],
    prime_context_tokens: u64,
    tokens_per_obl: u64,
) -> Vec<EfficiencyRollup> {
    let token_rollups = aggregate_by_process(ledger_entries, prime_context_tokens, tokens_per_obl);

    let mut q_by_family: HashMap<String, Vec<f64>> = HashMap::new();
    let mut i_by_family: HashMap<String, Vec<f64>> = HashMap::new();
    let mut outcome_counts: HashMap<String, u64> = HashMap::new();

    for o in outcomes {
        let family = o.process.clone();
        q_by_family.entry(family.clone()).or_default().push(o.q);
        i_by_family.entry(family.clone()).or_default().push(o.i);
        *outcome_counts.entry(family).or_insert(0) += 1;
    }

    let mut e_by_family: HashMap<String, u64> = HashMap::new();
    let mut calls_by_family: HashMap<String, u64> = HashMap::new();
    for r in &token_rollups {
        let family = process_family(&r.process).to_string();
        *e_by_family.entry(family.clone()).or_insert(0) += r.sum_total;
        *calls_by_family.entry(family).or_insert(0) += r.call_count;
    }

    let mut families: std::collections::BTreeSet<String> = e_by_family.keys().cloned().collect();
    families.extend(q_by_family.keys().cloned());

    let mut out = Vec::new();
    for family in families {
        let e_total = e_by_family.get(&family).copied().unwrap_or(0);
        let q = mean(q_by_family.get(&family).map(|v| v.as_slice()).unwrap_or(&[]));
        let i = mean(i_by_family.get(&family).map(|v| v.as_slice()).unwrap_or(&[]));
        let e_f = e_total.max(1) as f64;
        let eta = (q * i) / e_f;
        out.push(EfficiencyRollup {
            process: family.clone(),
            e_total,
            q,
            i,
            eta,
            eta_per_million_tokens: eta * 1_000_000.0,
            outcome_samples: outcome_counts.get(&family).copied().unwrap_or(0),
            ledger_calls: calls_by_family.get(&family).copied().unwrap_or(0),
        });
    }

    out.sort_by(|a, b| b.eta_per_million_tokens.partial_cmp(&a.eta_per_million_tokens).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Load synapse outcomes and compute η rollups for a time window.
pub fn compute_from_sources(
    ledger_entries: &[LedgerEntry],
    synapse_bus: &std::path::Path,
    since: chrono::DateTime<chrono::Utc>,
    prime_context_tokens: u64,
    tokens_per_obl: u64,
) -> anyhow::Result<Vec<EfficiencyRollup>> {
    let outcomes = collect_from_synapse(synapse_bus, since)?;
    Ok(compute_efficiency(
        ledger_entries,
        &outcomes,
        prime_context_tokens,
        tokens_per_obl,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obolus::ledger::{LedgerEntry, LedgerSource};
    use chrono::Utc;

    fn ledger_row(process: &str, total: u64) -> LedgerEntry {
        LedgerEntry {
            ts: Utc::now(),
            source: LedgerSource::Gateway,
            process: process.to_string(),
            task_kind: None,
            caller: "t".into(),
            input_tokens: total / 2,
            output_tokens: total / 2,
            total_tokens: total,
            latency_ms: 1,
            ok: true,
            model: None,
            correlation_id: None,
            action_id: None,
            dedup_key: None,
        }
    }

    #[test]
    fn eta_scales_with_tokens() {
        let ledger = vec![ledger_row("dream_extract", 1000), ledger_row("dream_verify", 1000)];
        let outcomes = vec![OutcomeSample {
            ts: Utc::now(),
            process: "dream".into(),
            q: 1.0,
            i: 0.5,
            action_id: None,
        }];
        let rollups = compute_efficiency(&ledger, &outcomes, 131_072, 1000);
        let dream = rollups.iter().find(|r| r.process == "dream").unwrap();
        assert_eq!(dream.e_total, 2000);
        assert!((dream.q - 1.0).abs() < 0.01);
        assert!((dream.i - 0.5).abs() < 0.01);
        assert!((dream.eta - 0.5 / 2000.0).abs() < 1e-9);
    }
}
