//! Obolus analytics — Prime token ledger, rollups, and efficiency (η) groundwork.

pub mod context;
pub mod gateway;
pub mod ledger;
pub mod process;
pub mod efficiency;
pub mod gate;
pub mod outcome;
pub mod reconcile;
pub mod rollup;

pub use context::{current_call_context, with_call_context, CallContextGuard, ObolusCallContext};
pub use efficiency::{compute_efficiency, compute_from_sources, EfficiencyRollup};
pub use gate::{
    evaluate_budget, evaluate_from_config, emit_budget_tick, emit_obolus_denied, emit_obolus_warn,
    ledger_session_tokens, load_balance_since, preflight_allowed, rolling_rollups, ObolusAction,
    ObolusTier, ObolusVerdict, SystemBalance,
};
pub use gateway::{instrument_if_enabled, targets_prime, InstrumentedGateway};
pub use ledger::{LedgerEntry, LedgerSource, ObolusLedger, TokenUsage};
pub use outcome::{collect_from_synapse, process_family, OutcomeSample};
pub use process::{kurator_process_label, process_from_task_kind};
pub use reconcile::synapse_bus_path;
pub use rollup::{aggregate_by_process, ProcessRollup};
