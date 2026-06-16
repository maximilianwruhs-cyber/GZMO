//! Roll up ledger entries by process for CLI reports.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ledger::LedgerEntry;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessRollup {
    pub process: String,
    pub call_count: u64,
    pub sum_input: u64,
    pub sum_output: u64,
    pub sum_total: u64,
    pub max_input_single_call: u64,
    pub context_share_pct: f64,
    pub obl_estimate: f64,
    pub gaps: u64,
}

/// Aggregate ledger rows by `process` label.
pub fn aggregate_by_process(
    entries: &[LedgerEntry],
    prime_context_tokens: u64,
    tokens_per_obl: u64,
) -> Vec<ProcessRollup> {
    let mut map: HashMap<String, ProcessRollup> = HashMap::new();
    let ctx_denom = prime_context_tokens.max(1) as f64;
    let obl_denom = tokens_per_obl.max(1) as f64;

    for e in entries {
        let r = map.entry(e.process.clone()).or_insert_with(|| ProcessRollup {
            process: e.process.clone(),
            call_count: 0,
            sum_input: 0,
            sum_output: 0,
            sum_total: 0,
            max_input_single_call: 0,
            context_share_pct: 0.0,
            obl_estimate: 0.0,
            gaps: 0,
        });
        r.call_count += 1;
        r.sum_input += e.input_tokens;
        r.sum_output += e.output_tokens;
        r.sum_total += e.total_tokens;
        r.max_input_single_call = r.max_input_single_call.max(e.input_tokens);
        if e.input_tokens == 0 && e.output_tokens == 0 && e.ok {
            r.gaps += 1;
        }
    }

    let mut out: Vec<ProcessRollup> = map.into_values().collect();
    for r in &mut out {
        r.context_share_pct = (r.sum_input as f64 / ctx_denom) * 100.0;
        r.obl_estimate = r.sum_total as f64 / obl_denom;
    }
    out.sort_by(|a, b| b.sum_total.cmp(&a.sum_total));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obolus::ledger::{LedgerEntry, LedgerSource};
    use chrono::Utc;

    fn entry(process: &str, input: u64, output: u64) -> LedgerEntry {
        LedgerEntry {
            ts: Utc::now(),
            source: LedgerSource::Gateway,
            process: process.to_string(),
            task_kind: None,
            caller: "test".into(),
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
            latency_ms: 1,
            ok: true,
            model: None,
            correlation_id: None,
            action_id: None,
            dedup_key: None,
        }
    }

    #[test]
    fn context_share_at_half_window() {
        let entries = vec![entry("dream_extract", 65_536, 100)];
        let rollups = aggregate_by_process(&entries, 131_072, 1000);
        assert_eq!(rollups.len(), 1);
        assert!((rollups[0].context_share_pct - 50.0).abs() < 0.01);
    }
}
