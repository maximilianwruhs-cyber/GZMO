//! Connected repository evolver library surface.
//!
//! Later tasks add modules only when real implementations exist.

pub mod config;
pub mod mission;
pub mod policy;
pub mod process;
pub mod state;

pub use config::{
    ConfigError, MissionConfig, PolicyConfig, RepoConfig, RepoEvolverConfig, WorkerConfig,
};
pub use mission::{
    Clock, ManualClock, Mission, MissionAdapter, MissionError, PreparedCandidate, SystemClock,
    CURRENT_POINTER, GENERATIONS_DIR, MAX_AUX_STRING_BYTES, MAX_MISSION_JSON_BYTES,
    MAX_MISSION_MARKDOWN_BYTES, MISSIONS_DIR, MISSION_STAGING_DIR, NEXT_MISSION_SCHEMA,
    REFRESH_OUTPUT_CAP_BYTES, REFRESH_TIMEOUT_SECS, SAFE_PATH,
};
pub use policy::{
    GateCommand, PolicyParseError, TrustedPolicy, MAX_GATE_TIMEOUT_SECONDS, MAX_REPAIR_ATTEMPTS,
    POLICY_SCHEMA, REQUIRED_BRANCH_PREFIX,
};
pub use process::{
    FakeProcessRunner, ProcessError, ProcessOutput, ProcessRunner, ProcessSpec, SystemProcessRunner,
};
pub use state::{
    CandidateRecord, CoordinatorLock, StateError, StateStore, TransitionMetadata,
    MAX_TERMINAL_REASON_BYTES, RUNNER_LOCK_NAME, STATE_APPLICATION_ID, STATE_DB_NAME,
    STATE_SCHEMA_VERSION,
};
