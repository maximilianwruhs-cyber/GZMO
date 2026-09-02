//! Connected repository evolver library surface.
//!
//! Later tasks add modules only when real implementations exist.

pub mod config;
pub mod policy;
pub mod state;

pub use config::{
    ConfigError, MissionConfig, PolicyConfig, RepoConfig, RepoEvolverConfig, WorkerConfig,
};
pub use policy::{
    GateCommand, PolicyParseError, TrustedPolicy, MAX_GATE_TIMEOUT_SECONDS, MAX_REPAIR_ATTEMPTS,
    POLICY_SCHEMA, REQUIRED_BRANCH_PREFIX,
};
pub use state::{
    CandidateRecord, CoordinatorLock, StateError, StateStore, TransitionMetadata,
    MAX_TERMINAL_REASON_BYTES, RUNNER_LOCK_NAME, STATE_APPLICATION_ID, STATE_DB_NAME,
    STATE_SCHEMA_VERSION,
};
