//! Coordinator state machine: prepare → build → evaluate boundary.
//!
//! Holds [`CoordinatorLock`] across every mutation and await. Audit/record and
//! immutable artifacts are revalidated before every resume action. Stops exactly
//! at [`CandidateState::Evaluating`]; never evaluates quality, pushes, or opens a PR.

use crate::config::RepoEvolverConfig;
use crate::git::{prepare_candidate, GitError, GitRepository, PrepareError};
use crate::mission::{Clock, Mission, MissionAdapter, MissionError, SystemClock};
use crate::process::{ProcessRunner, ProcessSpec, SystemProcessRunner};
use crate::state::{
    CandidateRecord, CoordinatorLock, StateError, StateStore, TransitionMetadata,
    MAX_TERMINAL_REASON_BYTES,
};
use crate::worker::{
    canonical_profile_tree_digest, load_worker_receipt, probe_omp_version, render_mission_prompt,
    render_omp_overlay, render_system_prompt, resolve_fixed_worker_identity, seal_worker_bundle,
    try_load_existing_sealed_request, validate_code_candidate_profile, worker_runtime_dirs,
    EffectiveIdentity, SealWorkerInput, SystemdWorkerLauncher, SystemdWorkerRuntimeProvisioner,
    WorkerCompanions, WorkerError, WorkerLauncher, WorkerReceipt, WorkerRequest, WorkerRoots,
    WorkerRuntimeProvisioner, WorkerUnitState, REQUEST_FILE_NAME, WORKER_SAFE_PATH,
};
use chrono::{DateTime, Utc};
use evolution_contracts::{
    canonical_json_bytes, sha256_hex, CandidateId, CandidateManifest, CandidateState,
    ResourceBudget, ResourceUsage,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Status schema id.
pub const STATUS_SCHEMA: &str = "gzmo.repo_evolver.status/v1";
/// Run/resume outcome schema id.
pub const RUN_OUTCOME_SCHEMA: &str = "gzmo.repo_evolver.run_outcome/v1";
/// Fixed OMP provider model pin for sealed overlay.
pub const CODE_CANDIDATE_PROVIDER_MODEL: &str = "local/code";
/// Bounded terminal reason length for runner-authored failures.
const MAX_REASON: usize = 512;

/// Errors from the repository evolver coordinator.
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("runner invalid: {0}")]
    Invalid(String),
    #[error("runner trust failure: {0}")]
    Trust(String),
    #[error("coordinator lock busy")]
    LockBusy,
    #[error("transient contention: {0}")]
    Contention(String),
    #[error("recovery required (artifacts preserved): {0}")]
    RecoveryRequired(String),
    #[error("candidate failed: {reason}")]
    Failed {
        reason: String,
        candidate_id: String,
    },
    #[error("later-stage candidate refused: {0}")]
    LaterStage(String),
    #[error(transparent)]
    State(#[from] StateError),
    #[error("prepare: {0}")]
    Prepare(String),
    #[error("git: {0}")]
    Git(String),
    #[error("mission: {0}")]
    Mission(String),
    #[error("worker: {0}")]
    Worker(String),
}

impl RunnerError {
    fn bound(msg: impl std::fmt::Display) -> String {
        let s = msg.to_string();
        let trimmed = s.trim();
        if trimmed.len() <= MAX_REASON {
            trimmed.to_owned()
        } else {
            let mut out: String = trimmed.chars().take(MAX_REASON).collect();
            out.push('\u{2026}');
            out
        }
    }

    fn from_prepare(err: PrepareError) -> Self {
        match err {
            PrepareError::Failed {
                reason,
                candidate_id,
            } => Self::Failed {
                reason: Self::bound(reason),
                candidate_id,
            },
            PrepareError::Git(msg) if msg.contains("mirror lease busy") => {
                Self::Contention("mirror lease busy".to_owned())
            }
            other => Self::Prepare(Self::bound(other)),
        }
    }

    fn from_git(err: GitError) -> Self {
        match err {
            GitError::MirrorLockBusy => Self::Contention("mirror lease busy".to_owned()),
            other => Self::Git(Self::bound(other)),
        }
    }

    fn from_worker(err: WorkerError) -> Self {
        match err {
            WorkerError::LeaseBusy => Self::Contention("worker/runtime lease busy".to_owned()),
            other => Self::Worker(Self::bound(other)),
        }
    }

    fn from_mission(err: MissionError) -> Self {
        Self::Mission(Self::bound(err))
    }
}

/// Outcome of `run_once` / `resume` stopping at or before Evaluating.
#[derive(Debug, Clone, Serialize)]
pub struct RunOutcome {
    pub schema: &'static str,
    pub candidate_id: String,
    pub state: CandidateState,
    pub mission_id: String,
    pub baseline_digest: String,
    pub candidate_digest: Option<String>,
    pub policy_digest: String,
    pub manifest_digest: String,
    pub receipt_digest: Option<String>,
    pub workspace: Option<String>,
    pub terminal_reason: Option<String>,
}

impl RunOutcome {
    fn from_record(record: &CandidateRecord) -> Self {
        Self {
            schema: RUN_OUTCOME_SCHEMA,
            candidate_id: record.id().as_str().to_owned(),
            state: record.state(),
            mission_id: record.manifest().mission_id.clone(),
            baseline_digest: record.manifest().baseline_digest.clone(),
            candidate_digest: record.candidate_digest().map(str::to_owned),
            policy_digest: record.policy_digest().to_owned(),
            manifest_digest: record.manifest_digest().to_owned(),
            receipt_digest: record.receipt_digest().map(str::to_owned),
            workspace: record.workspace().map(|p| p.display().to_string()),
            terminal_reason: record.terminal_reason().map(str::to_owned),
        }
    }
}

/// One structured read-only status model.
#[derive(Debug, Clone, Serialize)]
pub struct StatusV1 {
    pub schema: &'static str,
    pub repository: String,
    pub mission_generation_id: Option<String>,
    pub candidate_id: Option<String>,
    pub state: Option<String>,
    pub baseline_digest: Option<String>,
    pub candidate_digest: Option<String>,
    pub policy_digest: Option<String>,
    pub manifest_digest: Option<String>,
    pub receipt_digest: Option<String>,
    pub budget_max: Option<BudgetSnapshot>,
    pub budget_used: Option<UsageSnapshot>,
    pub budget_remaining: Option<RemainingSnapshot>,
    pub workspace: Option<String>,
    pub worker_state: Option<String>,
    pub worker_deadline: Option<String>,
    pub last_audit_sequence: Option<u64>,
    pub last_audit_hash: Option<String>,
    pub terminal_reason: Option<String>,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BudgetSnapshot {
    pub wall_seconds: u64,
    pub max_changed_files: u32,
    pub max_added_lines: u32,
    pub max_tool_calls: u32,
    pub max_input_tokens: u64,
    pub max_output_tokens: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSnapshot {
    pub wall_seconds: Option<u64>,
    pub changed_files: Option<u32>,
    pub added_lines: Option<u32>,
    pub tool_calls: Option<u32>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemainingSnapshot {
    pub wall_seconds: Option<u64>,
    pub changed_files: Option<u32>,
    pub added_lines: Option<u32>,
    pub tool_calls: Option<u32>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

/// Fixed worker identity provider (production resolves OS account; tests inject).
pub trait WorkerIdentity: Send + Sync {
    fn identity(&self) -> Result<EffectiveIdentity, RunnerError>;
}

/// Production identity: fixed operations-installed worker account.
#[derive(Debug, Default, Clone, Copy)]
pub struct FixedWorkerIdentity;

impl WorkerIdentity for FixedWorkerIdentity {
    fn identity(&self) -> Result<EffectiveIdentity, RunnerError> {
        resolve_fixed_worker_identity().map_err(RunnerError::from_worker)
    }
}

/// Repository evolver coordinator.
pub struct RepoEvolver<R, L, P, I, C = SystemClock>
where
    R: ProcessRunner + Send + Sync + 'static,
    L: WorkerLauncher + 'static,
    P: WorkerRuntimeProvisioner + 'static,
    I: WorkerIdentity + 'static,
    C: Clock + Send + Sync + 'static,
{
    config: RepoEvolverConfig,
    runner: Arc<R>,
    launcher: Arc<L>,
    provisioner: Arc<P>,
    identity: Arc<I>,
    clock: Arc<C>,
    roots: WorkerRoots,
    coordinator_uid: u32,
}

impl
    RepoEvolver<
        SystemProcessRunner,
        SystemdWorkerLauncher<SystemProcessRunner>,
        SystemdWorkerRuntimeProvisioner<SystemProcessRunner>,
        FixedWorkerIdentity,
        SystemClock,
    >
{
    /// Production construction: fixed systemd provisioner/launcher and worker account.
    pub fn production(config: RepoEvolverConfig) -> Result<Self, RunnerError> {
        let roots = WorkerRoots::production();
        roots
            .validate_intrinsic()
            .map_err(RunnerError::from_worker)?;
        let bin = std::env::current_exe()
            .map_err(|e| RunnerError::Invalid(format!("current_exe: {e}")))?;
        let launcher = SystemdWorkerLauncher::new(SystemProcessRunner, bin, roots.clone())
            .map_err(RunnerError::from_worker)?;
        let provisioner = SystemdWorkerRuntimeProvisioner::new(
            SystemProcessRunner,
            roots.clone(),
            config.worker().profile().to_owned(),
        )
        .map_err(RunnerError::from_worker)?;
        let coordinator_uid = nix::unistd::Uid::effective().as_raw();
        if coordinator_uid == 0 {
            return Err(RunnerError::Trust(
                "coordinator must not run as root".to_owned(),
            ));
        }
        Ok(Self {
            config,
            runner: Arc::new(SystemProcessRunner),
            launcher: Arc::new(launcher),
            provisioner: Arc::new(provisioner),
            identity: Arc::new(FixedWorkerIdentity),
            clock: Arc::new(SystemClock),
            roots,
            coordinator_uid,
        })
    }
}

impl<R, L, P, I, C> RepoEvolver<R, L, P, I, C>
where
    R: ProcessRunner + Send + Sync + 'static,
    L: WorkerLauncher + 'static,
    P: WorkerRuntimeProvisioner + 'static,
    I: WorkerIdentity + 'static,
    C: Clock + Send + Sync + 'static,
{
    /// Injectable constructor for hermetic tests (fakes live outside product authority).
    #[allow(clippy::too_many_arguments)]
    pub fn with_deps(
        config: RepoEvolverConfig,
        runner: R,
        launcher: L,
        provisioner: P,
        identity: I,
        clock: C,
        roots: WorkerRoots,
        coordinator_uid: u32,
    ) -> Result<Self, RunnerError> {
        roots
            .validate_intrinsic()
            .map_err(RunnerError::from_worker)?;
        if coordinator_uid == 0 {
            return Err(RunnerError::Trust(
                "coordinator_uid must be nonzero".to_owned(),
            ));
        }
        Ok(Self {
            config,
            runner: Arc::new(runner),
            launcher: Arc::new(launcher),
            provisioner: Arc::new(provisioner),
            identity: Arc::new(identity),
            clock: Arc::new(clock),
            roots,
            coordinator_uid,
        })
    }

    fn repository_key(&self) -> String {
        format!(
            "{}/{}",
            self.config.repo().owner(),
            self.config.repo().repository()
        )
    }

    fn now(&self) -> DateTime<Utc> {
        self.clock.now()
    }

    fn acquire_lock_and_store(&self) -> Result<(CoordinatorLock, StateStore), RunnerError> {
        let lock = CoordinatorLock::try_acquire(self.config.state_dir()).map_err(|err| {
            if matches!(err, StateError::LockBusy) {
                RunnerError::LockBusy
            } else {
                RunnerError::from(err)
            }
        })?;
        let store = StateStore::open(self.config.state_dir())?;
        store.verify_audit_chain()?;
        Ok((lock, store))
    }

    /// Run or resume until Evaluating (or terminal failure / later-stage refuse).
    pub async fn run_once(&self) -> Result<RunOutcome, RunnerError> {
        let (_lock, store) = self.acquire_lock_and_store()?;
        let repo = self.repository_key();
        if let Some(active) = store.active_candidate(&repo)? {
            return self.resume_record(&store, active).await;
        }
        let prepared = prepare_candidate(
            &self.config,
            self.runner.as_ref(),
            self.clock.as_ref(),
            &store,
        )
        .map_err(RunnerError::from_prepare)?;
        store.verify_audit_chain()?;
        self.advance_from_prepared(&store, prepared.record).await
    }

    /// Explicit resume of latest/active candidate without creating a new one.
    pub async fn resume(&self) -> Result<RunOutcome, RunnerError> {
        let (_lock, store) = self.acquire_lock_and_store()?;
        let repo = self.repository_key();
        let record = if let Some(active) = store.active_candidate(&repo)? {
            active
        } else if let Some(latest) = store.latest_candidate(&repo)? {
            latest
        } else {
            return Err(RunnerError::Invalid("no candidate to resume".to_owned()));
        };
        self.resume_record(&store, record).await
    }

    /// Abort a pre-evaluation candidate without deleting artifacts.
    pub async fn abort(&self, candidate_id: &str, reason: &str) -> Result<RunOutcome, RunnerError> {
        if reason.is_empty() {
            return Err(RunnerError::Invalid(
                "abort reason must be nonempty".to_owned(),
            ));
        }
        if reason.len() > MAX_TERMINAL_REASON_BYTES {
            return Err(RunnerError::Invalid(format!(
                "abort reason exceeds {MAX_TERMINAL_REASON_BYTES} bytes"
            )));
        }
        let id =
            CandidateId::parse(candidate_id).map_err(|e| RunnerError::Invalid(e.to_string()))?;
        let (_lock, store) = self.acquire_lock_and_store()?;
        let record = store.load(&id)?;

        match record.state() {
            CandidateState::Evaluating
            | CandidateState::ReviewReady
            | CandidateState::PromotionPending
            | CandidateState::Soaking
            | CandidateState::Accepted
            | CandidateState::RolledBack
            | CandidateState::Rejected
            | CandidateState::Failed => {
                return Err(RunnerError::LaterStage(format!(
                    "abort cannot alter state {}",
                    record.state()
                )));
            }
            CandidateState::Building => {
                if let Err(err) = self.launcher.stop(&id).await {
                    return Err(RunnerError::RecoveryRequired(format!(
                        "abort stop failed (artifacts preserved): {}",
                        RunnerError::bound(err)
                    )));
                }
            }
            CandidateState::Observed | CandidateState::Prepared => {}
        }

        match store.transition(
            &id,
            CandidateState::Failed,
            TransitionMetadata::terminal(reason),
            self.now(),
        ) {
            Ok(updated) => {
                store.verify_audit_chain()?;
                Ok(RunOutcome::from_record(&updated))
            }
            Err(err) => Err(RunnerError::RecoveryRequired(format!(
                "abort transition failed (artifacts preserved): {}",
                RunnerError::bound(err)
            ))),
        }
    }

    /// Read-only structured status (no lock, no mutation).
    pub async fn status(&self) -> Result<StatusV1, RunnerError> {
        let repo = self.repository_key();
        let store = match StateStore::open_existing_readonly(self.config.state_dir())? {
            None => {
                return Ok(empty_status(repo));
            }
            Some(s) => s,
        };
        store.verify_audit_chain()?;
        let head = store.audit_head()?;
        let record = match store.active_candidate(&repo)? {
            Some(r) => Some(r),
            None => store.latest_candidate(&repo)?,
        };

        let mut status = StatusV1 {
            schema: STATUS_SCHEMA,
            repository: repo,
            mission_generation_id: None,
            candidate_id: None,
            state: None,
            baseline_digest: None,
            candidate_digest: None,
            policy_digest: None,
            manifest_digest: None,
            receipt_digest: None,
            budget_max: None,
            budget_used: None,
            budget_remaining: None,
            workspace: None,
            worker_state: None,
            worker_deadline: None,
            last_audit_sequence: head.as_ref().map(|e| e.sequence),
            last_audit_hash: head.as_ref().map(|e| e.event_hash.clone()),
            terminal_reason: None,
            next_action: "run".to_owned(),
        };

        let Some(record) = record else {
            return Ok(status);
        };

        status.mission_generation_id = Some(record.manifest().mission_id.clone());
        status.candidate_id = Some(record.id().as_str().to_owned());
        status.state = Some(record.state().to_string());
        status.baseline_digest = Some(record.manifest().baseline_digest.clone());
        status.candidate_digest = record.candidate_digest().map(str::to_owned);
        status.policy_digest = Some(record.policy_digest().to_owned());
        status.manifest_digest = Some(record.manifest_digest().to_owned());
        status.receipt_digest = record.receipt_digest().map(str::to_owned);
        status.workspace = record.workspace().map(|p| p.display().to_string());
        status.terminal_reason = record.terminal_reason().map(str::to_owned);
        status.budget_max = Some(budget_snapshot(&record.manifest().budget));

        let usage = record
            .worker_receipt_json()
            .and_then(|json| serde_json::from_str::<WorkerReceipt>(json).ok())
            .map(|r| r.usage().clone());
        status.budget_used = Some(usage_snapshot(usage.as_ref()));
        status.budget_remaining = Some(remaining_snapshot(
            &record.manifest().budget,
            usage.as_ref(),
        ));

        if record.state() == CandidateState::Building {
            if let Ok(st) = self.launcher.inspect(record.id()).await {
                status.worker_state = Some(st.to_string());
            }
            if let Ok(Some(req)) = try_load_existing_sealed_request(&self.roots, record.id()) {
                status.worker_deadline = Some(
                    req.deadline()
                        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                );
            }
        }

        status.next_action = next_action_for(record.state());
        Ok(status)
    }

    async fn resume_record(
        &self,
        store: &StateStore,
        record: CandidateRecord,
    ) -> Result<RunOutcome, RunnerError> {
        store.verify_audit_chain()?;
        match record.state() {
            CandidateState::Observed => self.resume_observed(store, record).await,
            CandidateState::Prepared => self.advance_from_prepared(store, record).await,
            CandidateState::Building => self.resume_building(store, record).await,
            CandidateState::Evaluating => Ok(RunOutcome::from_record(&record)),
            CandidateState::ReviewReady
            | CandidateState::PromotionPending
            | CandidateState::Soaking => Err(RunnerError::LaterStage(format!(
                "state {} owned by later plan",
                record.state()
            ))),
            CandidateState::Rejected
            | CandidateState::Accepted
            | CandidateState::RolledBack
            | CandidateState::Failed => Ok(RunOutcome::from_record(&record)),
        }
    }

    async fn resume_observed(
        &self,
        store: &StateStore,
        record: CandidateRecord,
    ) -> Result<RunOutcome, RunnerError> {
        store.verify_audit_chain()?;
        let git = GitRepository::open(&self.config, self.runner.as_ref())
            .map_err(RunnerError::from_git)?;
        let baseline = match git.refresh_and_resolve_baseline() {
            Ok(b) => b,
            Err(GitError::MirrorLockBusy) => {
                return Err(RunnerError::Contention("mirror lease busy".to_owned()));
            }
            Err(err) => {
                return self.fail(store, record.id(), &RunnerError::bound(err));
            }
        };
        let expected = match record.manifest().baseline_digest.strip_prefix("git-sha1:") {
            Some(v) => v,
            None => {
                return self.fail(
                    store,
                    record.id(),
                    "manifest baseline_digest missing git-sha1",
                )
            }
        };
        if baseline != expected {
            return self.fail(store, record.id(), "baseline drift after Observed");
        }
        if let Err(err) = self.revalidate_baseline_policy(&git, &baseline, record.policy_digest()) {
            return self.fail(store, record.id(), &err);
        }
        if let Err(err) = self.load_bound_mission(record.manifest()) {
            return self.fail(store, record.id(), &RunnerError::bound(err));
        }

        let ws = match git.open_or_prepare_workspace(record.manifest()) {
            Ok(ws) => ws,
            Err(GitError::MirrorLockBusy) => {
                return Err(RunnerError::Contention("mirror lease busy".to_owned()));
            }
            Err(err) => return self.fail(store, record.id(), &RunnerError::bound(err)),
        };

        let meta = TransitionMetadata::empty().with_workspace(ws.path());
        match store.transition(record.id(), CandidateState::Prepared, meta, self.now()) {
            Ok(updated) => {
                store.verify_audit_chain()?;
                self.advance_from_prepared(store, updated).await
            }
            Err(err) => Err(RunnerError::RecoveryRequired(format!(
                "Observed→Prepared failed (artifacts preserved): {}",
                RunnerError::bound(err)
            ))),
        }
    }

    async fn advance_from_prepared(
        &self,
        store: &StateStore,
        record: CandidateRecord,
    ) -> Result<RunOutcome, RunnerError> {
        store.verify_audit_chain()?;
        if record.state() != CandidateState::Prepared {
            return Err(RunnerError::Invalid(format!(
                "advance_from_prepared requires Prepared, got {}",
                record.state()
            )));
        }
        let git = GitRepository::open(&self.config, self.runner.as_ref())
            .map_err(RunnerError::from_git)?;
        let ws = match git.open_existing_workspace(record.manifest()) {
            Ok(ws) => ws,
            Err(err) => return self.fail(store, record.id(), &RunnerError::bound(err)),
        };
        if let Some(path) = record.workspace() {
            if path != ws.path() {
                return self.fail(store, record.id(), "workspace path mismatch");
            }
        }

        let baseline = match record.manifest().baseline_digest.strip_prefix("git-sha1:") {
            Some(v) => v.to_owned(),
            None => return self.fail(store, record.id(), "baseline_digest missing prefix"),
        };
        if let Err(err) = self.revalidate_baseline_policy(&git, &baseline, record.policy_digest()) {
            return self.fail(store, record.id(), &err);
        }
        let mission = match self.load_bound_mission(record.manifest()) {
            Ok(m) => m,
            Err(err) => return self.fail(store, record.id(), &RunnerError::bound(err)),
        };

        match self.provisioner.provision(record.id()).await {
            Ok(()) => {}
            Err(WorkerError::LeaseBusy) => {
                return Err(RunnerError::Contention(
                    "runtime provisioner busy".to_owned(),
                ));
            }
            Err(WorkerError::Timeout) => {
                return Err(RunnerError::Contention(
                    "runtime provisioner timeout".to_owned(),
                ));
            }
            Err(err) => return self.fail(store, record.id(), &RunnerError::bound(err)),
        }

        let request = match self.ensure_sealed_request(&record, &mission, ws.path()) {
            Ok(r) => r,
            Err(err) => return self.fail(store, record.id(), &RunnerError::bound(err)),
        };

        let building = match store.transition(
            record.id(),
            CandidateState::Building,
            TransitionMetadata::empty(),
            self.now(),
        ) {
            Ok(r) => r,
            Err(err) => {
                return Err(RunnerError::RecoveryRequired(format!(
                    "Prepared→Building failed (artifacts preserved): {}",
                    RunnerError::bound(err)
                )));
            }
        };
        store.verify_audit_chain()?;
        self.drive_building(store, building, request).await
    }

    async fn resume_building(
        &self,
        store: &StateStore,
        record: CandidateRecord,
    ) -> Result<RunOutcome, RunnerError> {
        store.verify_audit_chain()?;
        let request = match try_load_existing_sealed_request(&self.roots, record.id())
            .map_err(RunnerError::from_worker)?
        {
            Some(r) => {
                if let Err(err) = self.validate_request_against_record(&r, &record) {
                    return self.fail(store, record.id(), &err);
                }
                r
            }
            None => {
                return self.fail(store, record.id(), "building without sealed request");
            }
        };

        if let Some(outcome) = self.try_finish_from_receipt(store, &record, &request)? {
            return Ok(outcome);
        }

        match self.launcher.inspect(record.id()).await {
            Ok(WorkerUnitState::Running) => {
                match self
                    .launcher
                    .wait_existing(record.id(), request.deadline())
                    .await
                {
                    Ok(WorkerUnitState::Succeeded) => {}
                    Ok(WorkerUnitState::Running) => {
                        return Err(RunnerError::Contention(
                            "worker still running after wait".to_owned(),
                        ));
                    }
                    Ok(_) | Err(_) => {
                        return self.fail(store, record.id(), "worker_lost_without_receipt");
                    }
                }
            }
            Ok(WorkerUnitState::Succeeded) => {}
            Ok(WorkerUnitState::Failed) | Ok(WorkerUnitState::NotFound) => {
                // Never start a second unit from Building.
                return self.fail(store, record.id(), "worker_lost_without_receipt");
            }
            Err(err) => return self.fail(store, record.id(), &RunnerError::bound(err)),
        }

        match self.try_finish_from_receipt(store, &record, &request)? {
            Some(outcome) => Ok(outcome),
            None => self.fail(store, record.id(), "worker_lost_without_receipt"),
        }
    }

    async fn drive_building(
        &self,
        store: &StateStore,
        record: CandidateRecord,
        request: WorkerRequest,
    ) -> Result<RunOutcome, RunnerError> {
        store.verify_audit_chain()?;
        let request_path = self
            .roots
            .request_root()
            .join(record.id().as_str())
            .join(REQUEST_FILE_NAME);

        match self.launcher.inspect(record.id()).await {
            Ok(WorkerUnitState::NotFound) => {
                if let Err(err) = self
                    .launcher
                    .launch_and_wait(&request_path, &request, &self.roots)
                    .await
                {
                    if let Some(outcome) = self.try_finish_from_receipt(store, &record, &request)? {
                        return Ok(outcome);
                    }
                    return self.fail(store, record.id(), &RunnerError::bound(err));
                }
            }
            Ok(WorkerUnitState::Running) => {
                let _ = self
                    .launcher
                    .wait_existing(record.id(), request.deadline())
                    .await;
            }
            Ok(WorkerUnitState::Succeeded) => {}
            Ok(WorkerUnitState::Failed) => {
                return self.fail(store, record.id(), "worker unit failed");
            }
            Err(err) => return self.fail(store, record.id(), &RunnerError::bound(err)),
        }

        match self.try_finish_from_receipt(store, &record, &request)? {
            Some(o) => Ok(o),
            None => self.fail(store, record.id(), "worker_lost_without_receipt"),
        }
    }

    fn try_finish_from_receipt(
        &self,
        store: &StateStore,
        record: &CandidateRecord,
        request: &WorkerRequest,
    ) -> Result<Option<RunOutcome>, RunnerError> {
        let git = GitRepository::open(&self.config, self.runner.as_ref())
            .map_err(RunnerError::from_git)?;
        let ws = git
            .open_existing_workspace(record.manifest())
            .map_err(RunnerError::from_git)?;
        let head_before = ws.candidate_commit().map_err(RunnerError::from_git)?;

        let receipt = match load_worker_receipt(request.output_dir(), request, &head_before) {
            Ok(r) => r,
            Err(WorkerError::NotFound(_)) => return Ok(None),
            Err(err) => {
                let msg = err.to_string();
                if msg.contains("receipt must be a regular file")
                    || msg.contains("No such file")
                    || msg.contains("not found")
                {
                    return Ok(None);
                }
                // Deterministic content/trust failure → terminalize.
                return self
                    .fail(store, record.id(), &RunnerError::bound(err))
                    .map(Some);
            }
        };

        let baseline = match record.manifest().baseline_digest.strip_prefix("git-sha1:") {
            Some(v) => v,
            None => return self.fail(store, record.id(), "baseline missing").map(Some),
        };
        let branch = ws.current_branch().map_err(RunnerError::from_git)?;
        let expected_branch = format!("evolve/{}", record.id().as_str());
        if branch != expected_branch {
            return self
                .fail(store, record.id(), "workspace branch mismatch")
                .map(Some);
        }

        let normalized = match ws.ensure_normalized_candidate(
            baseline,
            &record.manifest().mission_id,
            receipt.completed_at(),
        ) {
            Ok(oid) => oid,
            Err(err) => {
                return self
                    .fail(store, record.id(), &RunnerError::bound(err))
                    .map(Some);
            }
        };

        let stats = match ws.diff_stats(baseline, &normalized, record.manifest()) {
            Ok(s) => s,
            Err(err) => {
                return self
                    .fail(store, record.id(), &RunnerError::bound(err))
                    .map(Some);
            }
        };
        if !stats.whitespace_ok {
            return self
                .fail(store, record.id(), "whitespace errors in candidate diff")
                .map(Some);
        }

        let usage = receipt.usage();
        let mut bound_usage = usage.clone();
        bound_usage.changed_files = stats.files.len() as u32;
        bound_usage.added_lines = stats.added_lines;
        if !bound_usage.fits(&record.manifest().budget) || !usage.fits(&record.manifest().budget) {
            return self
                .fail(
                    store,
                    record.id(),
                    "diff/usage exceeds budget after normalize",
                )
                .map(Some);
        }

        let candidate_digest = format!("git-sha1:{normalized}");
        let receipt_bytes = receipt
            .canonical_bytes()
            .map_err(RunnerError::from_worker)?;
        let receipt_json = String::from_utf8(receipt_bytes)
            .map_err(|e| RunnerError::Invalid(format!("receipt utf8: {e}")))?;
        let receipt_digest = format!("sha256:{}", sha256_hex(receipt_json.as_bytes()));

        let meta = TransitionMetadata::empty()
            .with_candidate_digest(candidate_digest)
            .with_receipt(receipt_json, receipt_digest);
        match store.transition(record.id(), CandidateState::Evaluating, meta, self.now()) {
            Ok(updated) => {
                store.verify_audit_chain()?;
                Ok(Some(RunOutcome::from_record(&updated)))
            }
            Err(err) => Err(RunnerError::RecoveryRequired(format!(
                "Building→Evaluating failed (artifacts preserved): {}",
                RunnerError::bound(err)
            ))),
        }
    }

    fn ensure_sealed_request(
        &self,
        record: &CandidateRecord,
        mission: &Mission,
        workspace: &Path,
    ) -> Result<WorkerRequest, RunnerError> {
        if let Some(existing) = try_load_existing_sealed_request(&self.roots, record.id())
            .map_err(RunnerError::from_worker)?
        {
            self.validate_request_against_record(&existing, record)
                .map_err(RunnerError::Trust)?;
            if existing.workspace() != workspace {
                return Err(RunnerError::Trust(
                    "sealed request workspace mismatch".to_owned(),
                ));
            }
            return Ok(existing);
        }

        let worker = self.identity.identity()?;
        if worker.uid == self.coordinator_uid {
            return Err(RunnerError::Trust(
                "worker uid must differ from coordinator".to_owned(),
            ));
        }

        let profile = self.config.worker().profile();
        let profile_dir = self.roots.profile_root().join(profile);
        validate_code_candidate_profile(&profile_dir).map_err(RunnerError::from_worker)?;
        let _ = canonical_profile_tree_digest(&profile_dir).map_err(RunnerError::from_worker)?;

        let dirs = worker_runtime_dirs(self.roots.output_root(), record.id().as_str(), profile)
            .map_err(RunnerError::from_worker)?;
        let output_dir = dirs[0].clone();
        let home = dirs[1].clone();

        let omp_version = probe_omp_version_any(
            self.runner.as_ref(),
            self.config.worker().executable(),
            &home,
        )
        .map_err(RunnerError::from_worker)?;

        let policy_rel = self
            .config
            .policy()
            .repo_path()
            .to_str()
            .ok_or_else(|| RunnerError::Invalid("policy path utf8".into()))?
            .replace('\\', "/");
        let baseline = record
            .manifest()
            .baseline_digest
            .strip_prefix("git-sha1:")
            .ok_or_else(|| RunnerError::Trust("baseline missing".into()))?;
        let git = GitRepository::open(&self.config, self.runner.as_ref())
            .map_err(RunnerError::from_git)?;
        let policy_toml = git
            .read_file_at(baseline, &policy_rel)
            .map_err(RunnerError::from_git)?;

        let manifest_json = canonical_json_bytes(record.manifest())
            .map_err(|e| RunnerError::Invalid(e.to_string()))?;
        let manifest_digest = format!("sha256:{}", sha256_hex(&manifest_json));
        if manifest_digest != record.manifest_digest() {
            return Err(RunnerError::Trust("manifest digest drift".to_owned()));
        }

        let system = render_system_prompt(
            record.id(),
            &record.manifest().baseline_digest,
            workspace,
            &record.manifest().protected_paths,
            &record.manifest().required_gates,
            &record.manifest().budget,
        )
        .map_err(RunnerError::from_worker)?;
        let mission_md =
            render_mission_prompt(&mission.markdown).map_err(RunnerError::from_worker)?;
        let overlay = render_omp_overlay(CODE_CANDIDATE_PROVIDER_MODEL);

        let issued_at = self.now();
        let input = SealWorkerInput {
            candidate_id: record.id().clone(),
            workspace: workspace.to_path_buf(),
            output_dir,
            omp_executable: self.config.worker().executable().to_path_buf(),
            omp_profile: profile.to_owned(),
            omp_version,
            coordinator_uid: self.coordinator_uid,
            expected_uid: worker.uid,
            expected_gid: worker.gid,
            budget: record.manifest().budget.clone(),
            issued_at,
            companions: WorkerCompanions {
                manifest_json,
                policy_toml,
                system_prompt_md: system.into_bytes(),
                mission_md: mission_md.into_bytes(),
                omp_overlay_yml: overlay.into_bytes(),
            },
            manifest_digest: record.manifest_digest().to_owned(),
            policy_digest: record.policy_digest().to_owned(),
        };
        seal_worker_bundle(&self.roots, input).map_err(RunnerError::from_worker)
    }

    fn validate_request_against_record(
        &self,
        request: &WorkerRequest,
        record: &CandidateRecord,
    ) -> Result<(), String> {
        if request.candidate_id() != record.id() {
            return Err("request candidate_id mismatch".to_owned());
        }
        if request.manifest_digest() != record.manifest_digest() {
            return Err("request manifest_digest mismatch".to_owned());
        }
        if request.policy_digest() != record.policy_digest() {
            return Err("request policy_digest mismatch".to_owned());
        }
        if let Some(ws) = record.workspace() {
            if request.workspace() != ws {
                return Err("request workspace mismatch".to_owned());
            }
        }
        if request.budget() != &record.manifest().budget {
            return Err("request budget mismatch".to_owned());
        }
        Ok(())
    }

    fn load_bound_mission(&self, manifest: &CandidateManifest) -> Result<Mission, RunnerError> {
        let adapter = MissionAdapter::new(&self.config, self.runner.as_ref(), self.clock.as_ref());
        let mission = adapter
            .load_generation(&manifest.mission_id)
            .map_err(RunnerError::from_mission)?;
        if mission.generation_id != manifest.mission_id {
            return Err(RunnerError::Trust(
                "loaded generation id mismatch".to_owned(),
            ));
        }
        Ok(mission)
    }

    fn revalidate_baseline_policy(
        &self,
        git: &GitRepository<'_, R>,
        baseline: &str,
        expected_policy_digest: &str,
    ) -> Result<(), String> {
        let policy_rel = self
            .config
            .policy()
            .repo_path()
            .to_str()
            .ok_or_else(|| "policy path utf8".to_owned())?
            .replace('\\', "/");
        let bytes = git
            .read_file_at(baseline, &policy_rel)
            .map_err(|e| RunnerError::bound(e))?;
        let policy =
            crate::policy::TrustedPolicy::parse_toml(&bytes).map_err(|e| RunnerError::bound(e))?;
        let digest = policy.digest().map_err(|e| RunnerError::bound(e))?;
        if digest != expected_policy_digest {
            return Err("baseline policy digest mismatch".to_owned());
        }
        if digest != self.config.working_policy_digest() {
            return Err("working policy digest mismatch vs baseline".to_owned());
        }
        Ok(())
    }

    fn fail(
        &self,
        store: &StateStore,
        id: &CandidateId,
        reason: &str,
    ) -> Result<RunOutcome, RunnerError> {
        let reason = RunnerError::bound(reason);
        match store.transition(
            id,
            CandidateState::Failed,
            TransitionMetadata::terminal(reason.clone()),
            self.now(),
        ) {
            Ok(_updated) => {
                let _ = store.verify_audit_chain();
                Err(RunnerError::Failed {
                    reason,
                    candidate_id: id.as_str().to_owned(),
                })
            }
            Err(err) => Err(RunnerError::RecoveryRequired(format!(
                "terminalize failed (artifacts preserved): {}; original: {reason}",
                RunnerError::bound(err)
            ))),
        }
    }
}

fn empty_status(repository: String) -> StatusV1 {
    StatusV1 {
        schema: STATUS_SCHEMA,
        repository,
        mission_generation_id: None,
        candidate_id: None,
        state: None,
        baseline_digest: None,
        candidate_digest: None,
        policy_digest: None,
        manifest_digest: None,
        receipt_digest: None,
        budget_max: None,
        budget_used: None,
        budget_remaining: None,
        workspace: None,
        worker_state: None,
        worker_deadline: None,
        last_audit_sequence: None,
        last_audit_hash: None,
        terminal_reason: None,
        next_action: "run".to_owned(),
    }
}

fn budget_snapshot(b: &ResourceBudget) -> BudgetSnapshot {
    BudgetSnapshot {
        wall_seconds: b.wall_seconds,
        max_changed_files: b.max_changed_files,
        max_added_lines: b.max_added_lines,
        max_tool_calls: b.max_tool_calls,
        max_input_tokens: b.max_input_tokens,
        max_output_tokens: b.max_output_tokens,
    }
}

fn usage_snapshot(u: Option<&ResourceUsage>) -> UsageSnapshot {
    match u {
        Some(u) => UsageSnapshot {
            wall_seconds: Some(u.wall_seconds),
            changed_files: Some(u.changed_files),
            added_lines: Some(u.added_lines),
            tool_calls: Some(u.tool_calls),
            input_tokens: Some(u.input_tokens),
            output_tokens: Some(u.output_tokens),
        },
        None => UsageSnapshot {
            wall_seconds: None,
            changed_files: None,
            added_lines: None,
            tool_calls: None,
            input_tokens: None,
            output_tokens: None,
        },
    }
}

fn remaining_snapshot(b: &ResourceBudget, u: Option<&ResourceUsage>) -> RemainingSnapshot {
    match u {
        Some(u) => RemainingSnapshot {
            wall_seconds: Some(b.wall_seconds.saturating_sub(u.wall_seconds)),
            changed_files: Some(b.max_changed_files.saturating_sub(u.changed_files)),
            added_lines: Some(b.max_added_lines.saturating_sub(u.added_lines)),
            tool_calls: Some(b.max_tool_calls.saturating_sub(u.tool_calls)),
            input_tokens: Some(b.max_input_tokens.saturating_sub(u.input_tokens)),
            output_tokens: Some(b.max_output_tokens.saturating_sub(u.output_tokens)),
        },
        None => RemainingSnapshot {
            wall_seconds: None,
            changed_files: None,
            added_lines: None,
            tool_calls: None,
            input_tokens: None,
            output_tokens: None,
        },
    }
}

fn next_action_for(state: CandidateState) -> String {
    match state {
        CandidateState::Observed | CandidateState::Prepared | CandidateState::Building => {
            "resume".to_owned()
        }
        CandidateState::Evaluating => "evaluate".to_owned(),
        CandidateState::ReviewReady => "review".to_owned(),
        CandidateState::PromotionPending => "promote".to_owned(),
        CandidateState::Soaking => "soak".to_owned(),
        CandidateState::Failed
        | CandidateState::Rejected
        | CandidateState::Accepted
        | CandidateState::RolledBack => "run".to_owned(),
    }
}

fn probe_omp_version_any<R: ProcessRunner>(
    runner: &R,
    omp: &Path,
    home: &Path,
) -> Result<String, WorkerError> {
    let mut env = BTreeMap::new();
    env.insert("PATH".to_owned(), WORKER_SAFE_PATH.to_owned());
    env.insert("LC_ALL".to_owned(), "C".to_owned());
    env.insert(
        "HOME".to_owned(),
        home.to_str()
            .ok_or_else(|| WorkerError::Invalid("home path utf8".into()))?
            .to_owned(),
    );
    let spec = ProcessSpec::new(
        omp,
        ["--version".to_owned()],
        home,
        env,
        64 * 1024,
        Duration::from_secs(30),
    )?;
    let out = runner.run(&spec)?;
    let text = String::from_utf8_lossy(&out.stdout);
    // Reuse official probe by extracting version then confirming.
    let ver = {
        let mut found = None;
        for raw in text.split_whitespace() {
            let t = raw.trim().trim_start_matches('v');
            if t.starts_with("18.") && t.chars().all(|c| c.is_ascii_digit() || c == '.') {
                found = Some(t.to_owned());
                break;
            }
        }
        found.ok_or_else(|| WorkerError::Trust("could not parse omp --version".into()))?
    };
    probe_omp_version(runner, omp, &ver, home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_unknown_usage_is_null_not_zero() {
        let snap = usage_snapshot(None);
        assert!(snap.wall_seconds.is_none());
        assert!(snap.changed_files.is_none());
        assert!(snap.tool_calls.is_none());
        let json = serde_json::to_value(&snap).unwrap();
        assert!(json["wall_seconds"].is_null());
        assert!(json["changed_files"].is_null());
    }

    #[test]
    fn next_action_table() {
        assert_eq!(next_action_for(CandidateState::Building), "resume");
        assert_eq!(next_action_for(CandidateState::Evaluating), "evaluate");
        assert_eq!(next_action_for(CandidateState::Failed), "run");
    }
}
