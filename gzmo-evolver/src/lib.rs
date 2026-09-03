//! Connected repository evolver library surface.
//!
//! Later tasks add modules only when real implementations exist.

pub mod config;
pub mod git;
pub mod mission;
pub mod policy;
pub mod process;
#[cfg(unix)]
pub mod runner;
pub mod state;
#[cfg(unix)]
pub mod worker;
pub use config::{
    ConfigError, MissionConfig, PolicyConfig, RepoConfig, RepoEvolverConfig, WorkerConfig,
};
pub use git::{
    cleanup_workspace, prepare_candidate, refresh_baseline_before_mission,
    validate_remote_identity, verify_git_trust, DiffFile, DiffStats, GitError, GitRepository,
    GitWorkspace, PrepareError, PrepareOutcome, CANDIDATE_AUTHOR_EMAIL, CANDIDATE_AUTHOR_NAME,
    GIT_BLOB_CAP_BYTES, GIT_DIFF_CAP_BYTES, GIT_FETCH_TIMEOUT_SECS, GIT_HOME_NAME,
    GIT_OUTPUT_CAP_BYTES, GIT_TIMEOUT_SECS, MAX_DIFF_FILES, MAX_TREE_ENTRIES, MIRROR_LOCK_NAME,
    MIRROR_NAME, NO_FETCH_URL, NO_PUSH_URL, WORKSPACES_DIR,
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
#[cfg(unix)]
pub use runner::{
    BudgetSnapshot, FixedWorkerIdentity, RemainingSnapshot, RepoEvolver, RunOutcome, RunnerError,
    StatusV1, UsageSnapshot, WorkerIdentity, CODE_CANDIDATE_PROVIDER_MODEL, RUN_OUTCOME_SCHEMA,
    STATUS_SCHEMA,
};
pub use state::{
    CandidateRecord, CoordinatorLock, StateError, StateStore, TransitionMetadata,
    MAX_TERMINAL_REASON_BYTES, RUNNER_LOCK_NAME, STATE_APPLICATION_ID, STATE_DB_NAME,
    STATE_SCHEMA_VERSION,
};
#[cfg(unix)]
pub use worker::{
    build_omp_args, canonical_profile_tree_digest, load_sealed_request, load_worker_receipt,
    load_worker_receipt_unbound, omp_child_env, parse_omp_jsonl, probe_omp_version,
    render_mission_prompt, render_omp_overlay, render_system_prompt, resolve_fixed_worker_identity,
    run_hidden_worker, run_worker_request, seal_worker_bundle, try_load_existing_sealed_request,
    validate_code_candidate_profile, worker_runtime_dirs, EffectiveIdentity, OmpJsonlUsage,
    PathAuthority, SealWorkerInput, SystemPathAuthority, SystemdWorkerLauncher,
    SystemdWorkerRuntimeProvisioner, WorkerCompanions, WorkerError, WorkerLauncher, WorkerReceipt,
    WorkerRequest, WorkerRoots, WorkerRuntimeProvisioner, WorkerUnitState, DISABLED_PROVIDERS,
    FORBIDDEN_ENV, OMP_OUTPUT_CAP_BYTES, PROD_MODEL_NETNS, PROD_OUTPUT_ROOT, PROD_PROFILE_ROOT,
    PROD_REQUEST_ROOT, PROD_WORKER_USER, WORKER_HOME_NAME, WORKER_NO_PROXY, WORKER_RECEIPT_SCHEMA,
    WORKER_REQUEST_SCHEMA, WORKER_SAFE_PATH,
};
