//! Connected repository evolver library surface for Task 1.
//!
//! Later tasks add modules only when real implementations exist.

pub mod config;
pub mod policy;

pub use config::{
    ConfigError, MissionConfig, PolicyConfig, RepoConfig, RepoEvolverConfig, WorkerConfig,
};
pub use policy::{
    GateCommand, PolicyParseError, TrustedPolicy, MAX_GATE_TIMEOUT_SECONDS, MAX_REPAIR_ATTEMPTS,
    POLICY_SCHEMA, REQUIRED_BRANCH_PREFIX,
};
