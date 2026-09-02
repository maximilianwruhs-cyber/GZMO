//! Local coordinator SQLite state and tamper-evident audit persistence.
//!
//! Owns candidate records and the hash-linked audit chain. Does not decide
//! worker resume policy, hold the coordinator lease across a run, or parse
//! worker receipts beyond opaque JSON/digest pairs.

use chrono::{DateTime, Utc};
use evolution_contracts::{
    canonical_json_bytes, sha256_hex, verify_chain, AuditEvent, CandidateId, CandidateManifest,
    CandidateState, CandidateTarget, ContractError,
};
use fs2::FileExt;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// SQLite database filename under the coordinator state directory.
pub const STATE_DB_NAME: &str = "state.db";
/// Exclusive coordinator lease filename under the state directory.
pub const RUNNER_LOCK_NAME: &str = "runner.lock";
/// Maximum UTF-8 byte length for a terminal reason.
pub const MAX_TERMINAL_REASON_BYTES: usize = 4096;
/// Schema version written via `PRAGMA user_version` for this task's layout.
pub const STATE_SCHEMA_VERSION: i32 = 1;
/// SQLite `application_id` identifying the evolver state database.
pub const STATE_APPLICATION_ID: i32 = 0x475a_4d4f; // 'GZMO'

const CANDIDATES_SQL: &str = "CREATE TABLE candidates (
  id TEXT PRIMARY KEY,
  repository TEXT NOT NULL,
  manifest_json TEXT NOT NULL,
  manifest_digest TEXT NOT NULL,
  policy_digest TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN (
    'observed','prepared','building','evaluating','rejected',
    'review_ready','promotion_pending','soaking','accepted','rolled_back','failed'
  )),
  workspace TEXT,
  candidate_digest TEXT,
  terminal_reason TEXT,
  worker_receipt_json TEXT,
  receipt_digest TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)";

const ONE_ACTIVE_INDEX_SQL: &str = "CREATE UNIQUE INDEX one_active_candidate
ON candidates(repository)
WHERE state NOT IN ('rejected','accepted','rolled_back','failed')";

const AUDIT_EVENTS_SQL: &str = "CREATE TABLE audit_events (
  sequence INTEGER PRIMARY KEY,
  event_json TEXT NOT NULL,
  event_hash TEXT NOT NULL UNIQUE
)";

/// Errors raised by coordinator state persistence.
#[derive(Debug, Error)]
pub enum StateError {
    /// Filesystem failure while creating or opening state paths.
    #[error("state io error: {0}")]
    Io(String),
    /// SQLite failure.
    #[error("state database error: {0}")]
    Db(String),
    /// Input failed validation before mutation.
    #[error("invalid state input: {0}")]
    Invalid(String),
    /// Stored bytes failed integrity or contract checks.
    #[error("state integrity error: {0}")]
    Integrity(String),
    /// Illegal lifecycle edge or metadata set-once violation.
    #[error("illegal transition: {0}")]
    IllegalTransition(String),
    /// Repository already has a nonterminal candidate.
    #[error("repository already has a nonterminal candidate: {0}")]
    AlreadyActive(String),
    /// Coordinator lease is held by another process.
    #[error("coordinator lock busy")]
    LockBusy,
    /// Read-only store rejected a mutating call.
    #[error("state store is read-only")]
    ReadOnly,
    /// Mutating write lost a race against another connection.
    #[error("state write contention: {0}")]
    Contention(String),
    /// Underlying evolution contract validation failed.
    #[error(transparent)]
    Contract(#[from] ContractError),
    /// Audit contract failure.
    #[error(transparent)]
    Audit(#[from] evolution_contracts::AuditError),
}

impl From<rusqlite::Error> for StateError {
    fn from(value: rusqlite::Error) -> Self {
        if is_busy_or_locked(&value) {
            Self::Contention(value.to_string())
        } else {
            Self::Db(value.to_string())
        }
    }
}

impl From<std::io::Error> for StateError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

/// Validated optional fields applied during a candidate transition.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransitionMetadata {
    workspace: Option<PathBuf>,
    candidate_digest: Option<String>,
    worker_receipt_json: Option<String>,
    receipt_digest: Option<String>,
    terminal_reason: Option<String>,
}

impl TransitionMetadata {
    /// Empty metadata (no workspace/digest/receipt/reason).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Terminal failure/reject/rollback reason (validated on apply).
    pub fn terminal(reason: impl Into<String>) -> Self {
        Self {
            terminal_reason: Some(reason.into()),
            ..Self::default()
        }
    }

    /// Absolute candidate workspace path (set-once).
    pub fn with_workspace(mut self, workspace: impl Into<PathBuf>) -> Self {
        self.workspace = Some(workspace.into());
        self
    }

    /// Algorithm-qualified candidate digest (set-once).
    pub fn with_candidate_digest(mut self, digest: impl Into<String>) -> Self {
        self.candidate_digest = Some(digest.into());
        self
    }

    /// Canonical worker-receipt JSON plus matching `sha256:` digest (set-once, paired).
    pub fn with_receipt(mut self, json: impl Into<String>, digest: impl Into<String>) -> Self {
        self.worker_receipt_json = Some(json.into());
        self.receipt_digest = Some(digest.into());
        self
    }

    /// Optional absolute workspace path.
    pub fn workspace(&self) -> Option<&Path> {
        self.workspace.as_deref()
    }

    /// Optional algorithm-qualified candidate digest.
    pub fn candidate_digest(&self) -> Option<&str> {
        self.candidate_digest.as_deref()
    }

    /// Optional opaque canonical worker-receipt JSON.
    pub fn worker_receipt_json(&self) -> Option<&str> {
        self.worker_receipt_json.as_deref()
    }

    /// Optional `sha256:` receipt digest.
    pub fn receipt_digest(&self) -> Option<&str> {
        self.receipt_digest.as_deref()
    }

    /// Optional terminal reason.
    pub fn terminal_reason(&self) -> Option<&str> {
        self.terminal_reason.as_deref()
    }

    fn validate_shape(&self) -> Result<(), StateError> {
        match (&self.worker_receipt_json, &self.receipt_digest) {
            (None, None) => {}
            (Some(_), Some(_)) => {}
            _ => {
                return Err(StateError::Invalid(
                    "worker_receipt_json and receipt_digest must appear together".to_owned(),
                ));
            }
        }

        if let Some(ws) = &self.workspace {
            if ws.to_str().is_none() {
                return Err(StateError::Invalid(format!(
                    "workspace must be valid UTF-8, got {}",
                    ws.display()
                )));
            }
            if !ws.is_absolute() {
                return Err(StateError::Invalid(format!(
                    "workspace must be absolute, got {}",
                    ws.display()
                )));
            }
        }

        if let Some(digest) = &self.candidate_digest {
            validate_algorithm_qualified_digest("candidate_digest", digest)?;
        }

        if let Some(digest) = &self.receipt_digest {
            validate_sha256_digest("receipt_digest", digest)?;
        }

        if let (Some(json), Some(digest)) = (&self.worker_receipt_json, &self.receipt_digest) {
            let canonical = canonicalize_json_text(json)?;
            let expected = format!("sha256:{}", sha256_hex(canonical.as_bytes()));
            if digest != &expected {
                return Err(StateError::Invalid(
                    "receipt_digest does not match canonical worker_receipt_json".to_owned(),
                ));
            }
        }

        if let Some(reason) = &self.terminal_reason {
            if reason.is_empty() {
                return Err(StateError::Invalid(
                    "terminal_reason must be nonempty when set".to_owned(),
                ));
            }
            if reason.len() > MAX_TERMINAL_REASON_BYTES {
                return Err(StateError::Invalid(format!(
                    "terminal_reason exceeds {MAX_TERMINAL_REASON_BYTES} bytes"
                )));
            }
        }

        Ok(())
    }
}

/// Validated candidate row recovered from the store.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateRecord {
    manifest: CandidateManifest,
    manifest_digest: String,
    policy_digest: String,
    state: CandidateState,
    workspace: Option<PathBuf>,
    candidate_digest: Option<String>,
    worker_receipt_json: Option<String>,
    receipt_digest: Option<String>,
    terminal_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    repository: String,
}

impl CandidateRecord {
    /// Validated candidate manifest.
    pub fn manifest(&self) -> &CandidateManifest {
        &self.manifest
    }

    /// Canonical `sha256:` digest of stored manifest JSON.
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    /// Canonical `sha256:` policy digest bound at create time.
    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    /// Current lifecycle state.
    pub fn state(&self) -> CandidateState {
        self.state
    }

    /// Absolute workspace path when set.
    pub fn workspace(&self) -> Option<&Path> {
        self.workspace.as_deref()
    }

    /// Algorithm-qualified candidate digest when set.
    pub fn candidate_digest(&self) -> Option<&str> {
        self.candidate_digest.as_deref()
    }

    /// Opaque canonical worker-receipt JSON when set.
    pub fn worker_receipt_json(&self) -> Option<&str> {
        self.worker_receipt_json.as_deref()
    }

    /// Matching `sha256:` receipt digest when set.
    pub fn receipt_digest(&self) -> Option<&str> {
        self.receipt_digest.as_deref()
    }

    /// Terminal reason when set.
    pub fn terminal_reason(&self) -> Option<&str> {
        self.terminal_reason.as_deref()
    }

    /// Creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Last update timestamp.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Repository key `owner/repository` derived from the manifest target.
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// Candidate id borrowed from the manifest.
    pub fn id(&self) -> &CandidateId {
        &self.manifest.id
    }
}

/// RAII exclusive coordinator lease (`runner.lock`).
#[derive(Debug)]
pub struct CoordinatorLock {
    _file: File,
}

impl CoordinatorLock {
    /// Try to acquire an exclusive lock at `<state_dir>/runner.lock`.
    ///
    /// Creates the lock file when missing. Does not create the state database.
    /// Returns [`StateError::LockBusy`] when another holder owns the lease.
    pub fn try_acquire(state_dir: impl AsRef<Path>) -> Result<Self, StateError> {
        let state_dir = state_dir.as_ref();
        if !state_dir.exists() {
            create_dir_0700(state_dir)?;
        } else {
            set_dir_mode_0700(state_dir)?;
        }
        let lock_path = state_dir.join(RUNNER_LOCK_NAME);
        let file = open_lock_file(&lock_path)?;
        file.try_lock_exclusive().map_err(|err| {
            if err.raw_os_error() == fs2::lock_contended_error().raw_os_error() {
                StateError::LockBusy
            } else {
                StateError::Io(err.to_string())
            }
        })?;
        Ok(Self { _file: file })
    }
}

/// Coordinator candidate + audit persistence.
#[derive(Debug)]
pub struct StateStore {
    conn: Connection,
    readonly: bool,
    /// Optional path used to chmod WAL/SHM sidecars after open.
    db_path: Option<PathBuf>,
}

impl StateStore {
    /// Create/open a mutating store under `state_dir` (dir 0700, db 0600 on Unix).
    ///
    /// Does **not** acquire the coordinator lease.
    pub fn open(state_dir: impl AsRef<Path>) -> Result<Self, StateError> {
        let state_dir = state_dir.as_ref();
        create_dir_0700(state_dir)?;

        let db_path = state_dir.join(STATE_DB_NAME);
        ensure_regular_db_file_0600(&db_path)?;
        let conn = Connection::open(&db_path).map_err(|err| StateError::Db(err.to_string()))?;
        configure_connection(&conn)?;
        initialize_or_verify_schema(&conn)?;
        set_file_mode_0600(&db_path)?;
        set_sidecar_modes_0600(&db_path)?;

        Ok(Self {
            conn,
            readonly: false,
            db_path: Some(db_path),
        })
    }

    /// Open an existing database read-only.
    ///
    /// Returns `Ok(None)` when the state directory or database file is absent,
    /// without creating any path.
    pub fn open_existing_readonly(state_dir: impl AsRef<Path>) -> Result<Option<Self>, StateError> {
        let state_dir = state_dir.as_ref();
        let db_path = state_dir.join(STATE_DB_NAME);
        if !state_dir.exists() || !db_path.exists() {
            return Ok(None);
        }
        reject_nonregular_db(&db_path)?;
        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|err| StateError::Db(err.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        verify_schema(&conn)?;
        Ok(Some(Self {
            conn,
            readonly: true,
            db_path: Some(db_path),
        }))
    }

    /// In-memory mutating store for tests (no lock, no filesystem).
    pub fn open_in_memory() -> Result<Self, StateError> {
        let conn = Connection::open_in_memory().map_err(|err| StateError::Db(err.to_string()))?;
        configure_connection(&conn)?;
        initialize_or_verify_schema(&conn)?;
        Ok(Self {
            conn,
            readonly: false,
            db_path: None,
        })
    }

    /// Insert a repository candidate in `Observed` and append `candidate.observed`.
    pub fn create_candidate(
        &self,
        manifest: &CandidateManifest,
        policy_digest: &str,
        now: DateTime<Utc>,
    ) -> Result<CandidateRecord, StateError> {
        self.require_mutable()?;
        manifest.validate()?;
        validate_sha256_digest("policy_digest", policy_digest)?;
        let repository = repository_key(&manifest.target)?;

        let manifest_json = canonicalize_json_value(manifest)?;
        let manifest_digest = format!("sha256:{}", sha256_hex(manifest_json.as_bytes()));
        let now_text = datetime_to_text(now);
        let state = CandidateState::Observed;
        let state_text = state.to_string();

        begin_immediate(&self.conn)?;
        let result = (|| -> Result<CandidateRecord, StateError> {
            let insert = self.conn.execute(
                r#"
                INSERT INTO candidates (
                  id, repository, manifest_json, manifest_digest, policy_digest, state,
                  workspace, candidate_digest, terminal_reason, worker_receipt_json, receipt_digest,
                  created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, NULL, NULL, NULL, ?7, ?7)
                "#,
                params![
                    manifest.id.as_str(),
                    repository,
                    manifest_json,
                    manifest_digest,
                    policy_digest,
                    state_text,
                    now_text,
                ],
            );
            match insert {
                Ok(1) => {}
                Ok(n) => {
                    return Err(StateError::Db(format!(
                        "expected 1 inserted candidate row, got {n}"
                    )));
                }
                Err(err) if is_active_candidate_violation(&err) => {
                    return Err(StateError::AlreadyActive(repository.clone()));
                }
                Err(err) => return Err(err.into()),
            }

            let payload = LifecyclePayload {
                candidate_id: manifest.id.as_str(),
                from_state: None,
                to_state: state_text.as_str(),
                policy_digest,
                workspace: None,
                candidate_digest: None,
                receipt_digest: None,
                terminal_reason: None,
            };
            append_audit_event(
                &self.conn,
                Some(manifest.id.clone()),
                "candidate.observed",
                &payload,
                now,
            )?;
            // Fully verify the resulting record before COMMIT.
            let events = load_and_verify_audit_events(&self.conn)?;
            load_record_in_tx(&self.conn, &manifest.id, Some(events.as_slice()))
        })();

        let record = match result {
            Ok(record) => record,
            Err(err) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(err);
            }
        };
        if let Err(err) = self.conn.execute_batch("COMMIT") {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(err.into());
        }
        if let Some(path) = &self.db_path {
            let _ = set_sidecar_modes_0600(path);
        }
        Ok(record)
    }

    /// Apply a legal lifecycle transition and append a matching audit event atomically.
    pub fn transition(
        &self,
        id: &CandidateId,
        next: CandidateState,
        metadata: TransitionMetadata,
        now: DateTime<Utc>,
    ) -> Result<CandidateRecord, StateError> {
        self.require_mutable()?;
        metadata.validate_shape()?;

        begin_immediate(&self.conn)?;
        let result = (|| -> Result<CandidateRecord, StateError> {
            // Verify chain and row↔audit inside the write transaction before trusting state.
            let events = load_and_verify_audit_events(&self.conn)?;
            let current = load_record_in_tx(&self.conn, id, Some(events.as_slice()))?;
            if !current.state.can_transition_to(next) {
                return Err(StateError::IllegalTransition(format!(
                    "{} cannot transition from {} to {}",
                    id.as_str(),
                    current.state,
                    next
                )));
            }

            validate_terminal_reason_rules(next, metadata.terminal_reason())?;
            let workspace = merge_set_once_path(
                "workspace",
                current.workspace.clone(),
                metadata.workspace.clone(),
            )?;
            let candidate_digest = merge_set_once_string(
                "candidate_digest",
                current.candidate_digest.clone(),
                metadata.candidate_digest.clone(),
            )?;

            // Canonicalize incoming receipt before set-once comparison.
            let incoming_receipt_json = match &metadata.worker_receipt_json {
                Some(json) => Some(canonicalize_json_text(json)?),
                None => None,
            };
            let (worker_receipt_json, receipt_digest) = merge_receipt_set_once(
                current.worker_receipt_json.clone(),
                current.receipt_digest.clone(),
                incoming_receipt_json,
                metadata.receipt_digest.clone(),
            )?;

            let terminal_reason = match (
                current.terminal_reason.clone(),
                metadata.terminal_reason.clone(),
            ) {
                (Some(existing), Some(new_reason)) if existing != new_reason => {
                    return Err(StateError::IllegalTransition(
                        "terminal_reason is set-once and cannot change".to_owned(),
                    ));
                }
                (Some(existing), _) => Some(existing),
                (None, next_reason) => next_reason,
            };

            let next_text = next.to_string();
            let from_text = current.state.to_string();
            let now_text = datetime_to_text(now);
            let workspace_text = workspace
                .as_ref()
                .map(|p| {
                    p.to_str()
                        .map(str::to_owned)
                        .ok_or_else(|| StateError::Invalid("workspace must be valid UTF-8".into()))
                })
                .transpose()?;

            self.conn.execute(
                r#"
                UPDATE candidates SET
                  state = ?1,
                  workspace = ?2,
                  candidate_digest = ?3,
                  terminal_reason = ?4,
                  worker_receipt_json = ?5,
                  receipt_digest = ?6,
                  updated_at = ?7
                WHERE id = ?8
                "#,
                params![
                    next_text,
                    workspace_text,
                    candidate_digest,
                    terminal_reason,
                    worker_receipt_json,
                    receipt_digest,
                    now_text,
                    id.as_str(),
                ],
            )?;

            let payload = LifecyclePayload {
                candidate_id: id.as_str(),
                from_state: Some(from_text.as_str()),
                to_state: next_text.as_str(),
                policy_digest: current.policy_digest.as_str(),
                workspace: workspace_text.as_deref(),
                candidate_digest: candidate_digest.as_deref(),
                receipt_digest: receipt_digest.as_deref(),
                terminal_reason: terminal_reason.as_deref(),
            };
            let event_type = lifecycle_event_type(next);
            let appended =
                append_audit_event(&self.conn, Some(id.clone()), event_type, &payload, now)?;
            // Post-write chain is the verified prefix plus the event just linked.
            let mut events = events;
            events.push(appended);
            load_record_in_tx(&self.conn, id, Some(events.as_slice()))
        })();

        let record = match result {
            Ok(record) => record,
            Err(err) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                return Err(err);
            }
        };
        if let Err(err) = self.conn.execute_batch("COMMIT") {
            let _ = self.conn.execute_batch("ROLLBACK");
            return Err(err.into());
        }
        if let Some(path) = &self.db_path {
            let _ = set_sidecar_modes_0600(path);
        }
        Ok(record)
    }

    /// Return the single nonterminal candidate for `repository` (`owner/name`), if any.
    pub fn active_candidate(
        &self,
        repository: &str,
    ) -> Result<Option<CandidateRecord>, StateError> {
        let events = load_and_verify_audit_events(&self.conn)?;
        let id: Option<String> = self
            .conn
            .query_row(
                r#"
                SELECT id FROM candidates
                WHERE repository = ?1
                  AND state NOT IN ('rejected','accepted','rolled_back','failed')
                LIMIT 1
                "#,
                params![repository],
                |row| row.get(0),
            )
            .optional()?;
        match id {
            Some(raw) => {
                let id = CandidateId::parse(raw)?;
                Ok(Some(load_record_in_tx(
                    &self.conn,
                    &id,
                    Some(events.as_slice()),
                )?))
            }
            None => Ok(None),
        }
    }

    /// Load one candidate after verifying stored digests and the full audit chain.
    pub fn load(&self, id: &CandidateId) -> Result<CandidateRecord, StateError> {
        load_record_verified(&self.conn, id)
    }

    /// Verify every stored audit event forms a valid hash-linked chain.
    pub fn verify_audit_chain(&self) -> Result<(), StateError> {
        let _ = load_and_verify_audit_events(&self.conn)?;
        Ok(())
    }

    /// Highest-sequence audit event, if the ledger is nonempty.
    pub fn audit_head(&self) -> Result<Option<AuditEvent>, StateError> {
        let events = load_and_verify_audit_events(&self.conn)?;
        Ok(events.into_iter().next_back())
    }

    /// Ordered audit events (validated individually when decoded).
    pub fn load_audit_events(&self) -> Result<Vec<AuditEvent>, StateError> {
        load_and_verify_audit_events(&self.conn)
    }

    fn require_mutable(&self) -> Result<(), StateError> {
        if self.readonly {
            Err(StateError::ReadOnly)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize)]
struct LifecyclePayload<'a> {
    candidate_id: &'a str,
    from_state: Option<&'a str>,
    to_state: &'a str,
    policy_digest: &'a str,
    workspace: Option<&'a str>,
    candidate_digest: Option<&'a str>,
    receipt_digest: Option<&'a str>,
    terminal_reason: Option<&'a str>,
}

fn configure_connection(conn: &Connection) -> Result<(), StateError> {
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(())
}

fn begin_immediate(conn: &Connection) -> Result<(), StateError> {
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|err| {
        if is_busy_or_locked(&err) {
            StateError::Contention(err.to_string())
        } else {
            StateError::Db(err.to_string())
        }
    })
}

fn initialize_or_verify_schema(conn: &Connection) -> Result<(), StateError> {
    let user_version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|err| StateError::Db(err.to_string()))?;
    let application_id: i32 = conn
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(|err| StateError::Db(err.to_string()))?;

    let has_candidates = master_sql(conn, "candidates")?.is_some();
    let has_audit = master_sql(conn, "audit_events")?.is_some();
    let empty = !has_candidates && !has_audit;

    if empty && user_version == 0 && application_id == 0 {
        // Fresh database (or Task-2 empty open): install schema and stamp identity.
        install_schema(conn)?;
        conn.pragma_update(None, "application_id", STATE_APPLICATION_ID)?;
        conn.pragma_update(None, "user_version", STATE_SCHEMA_VERSION)?;
        return verify_schema(conn);
    }

    // Accept this task's valid version-0 DB (created before identity stamp) by
    // upgrading the markers after structural verification.
    if user_version == 0 && application_id == 0 && has_candidates && has_audit {
        verify_schema_structure(conn)?;
        conn.pragma_update(None, "application_id", STATE_APPLICATION_ID)?;
        conn.pragma_update(None, "user_version", STATE_SCHEMA_VERSION)?;
        return Ok(());
    }

    if user_version != 0 && user_version != STATE_SCHEMA_VERSION {
        return Err(StateError::Integrity(format!(
            "unsupported state schema user_version {user_version}; expected 0 or {STATE_SCHEMA_VERSION}"
        )));
    }
    if application_id != 0 && application_id != STATE_APPLICATION_ID {
        return Err(StateError::Integrity(format!(
            "state database application_id {application_id:#x} does not match evolver id {STATE_APPLICATION_ID:#x}"
        )));
    }

    // Install any missing objects from the verified constants, then verify.
    install_schema_if_missing(conn)?;
    if user_version == 0 || application_id == 0 {
        conn.pragma_update(None, "application_id", STATE_APPLICATION_ID)?;
        conn.pragma_update(None, "user_version", STATE_SCHEMA_VERSION)?;
    }
    verify_schema(conn)
}

fn verify_schema(conn: &Connection) -> Result<(), StateError> {
    let user_version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|err| StateError::Db(err.to_string()))?;
    if user_version != STATE_SCHEMA_VERSION && user_version != 0 {
        return Err(StateError::Integrity(format!(
            "unsupported state schema user_version {user_version}"
        )));
    }
    let application_id: i32 = conn
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(|err| StateError::Db(err.to_string()))?;
    if application_id != STATE_APPLICATION_ID && application_id != 0 {
        return Err(StateError::Integrity(format!(
            "state database application_id mismatch: {application_id:#x}"
        )));
    }
    verify_schema_structure(conn)
}

fn install_schema(conn: &Connection) -> Result<(), StateError> {
    // Single source of truth shared with verify_schema_structure.
    for stmt in [CANDIDATES_SQL, ONE_ACTIVE_INDEX_SQL, AUDIT_EVENTS_SQL] {
        conn.execute_batch(stmt)?;
    }
    Ok(())
}

fn install_schema_if_missing(conn: &Connection) -> Result<(), StateError> {
    if master_sql(conn, "candidates")?.is_none() {
        conn.execute_batch(CANDIDATES_SQL)?;
    }
    if master_sql(conn, "one_active_candidate")?.is_none() {
        conn.execute_batch(ONE_ACTIVE_INDEX_SQL)?;
    }
    if master_sql(conn, "audit_events")?.is_none() {
        conn.execute_batch(AUDIT_EVENTS_SQL)?;
    }
    Ok(())
}

fn verify_schema_structure(conn: &Connection) -> Result<(), StateError> {
    assert_master_sql(conn, "table", "candidates", CANDIDATES_SQL)?;
    assert_master_sql(conn, "table", "audit_events", AUDIT_EVENTS_SQL)?;
    assert_master_sql(conn, "index", "one_active_candidate", ONE_ACTIVE_INDEX_SQL)?;
    Ok(())
}

fn master_sql(conn: &Connection, name: &str) -> Result<Option<String>, StateError> {
    conn.query_row(
        "SELECT sql FROM sqlite_master WHERE name = ?1",
        params![name],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map(|opt| opt.flatten())
    .map_err(|err| StateError::Db(err.to_string()))
}

fn assert_master_sql(
    conn: &Connection,
    kind: &str,
    name: &str,
    expected: &str,
) -> Result<(), StateError> {
    let sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = ?1 AND name = ?2",
            params![kind, name],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| StateError::Db(err.to_string()))?;
    let Some(sql) = sql else {
        return Err(StateError::Integrity(format!(
            "missing {kind} {name} in state database"
        )));
    };
    if normalize_sql(&sql) != normalize_sql(expected) {
        return Err(StateError::Integrity(format!(
            "state {kind} {name} definition mismatch"
        )));
    }
    Ok(())
}

fn normalize_sql(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn create_dir_0700(path: &Path) -> Result<(), StateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true).mode(0o700);
        match builder.create(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(StateError::Io(err.to_string())),
        }
        set_dir_mode_0700(path)?;
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        fs::create_dir_all(path).map_err(|err| StateError::Io(err.to_string()))?;
        Ok(())
    }
}

fn set_dir_mode_0700(path: &Path) -> Result<(), StateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = fs::symlink_metadata(path).map_err(|err| StateError::Io(err.to_string()))?;
        if meta.file_type().is_symlink() {
            return Err(StateError::Io(format!(
                "state directory must not be a symlink: {}",
                path.display()
            )));
        }
        let mut perms = meta.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms).map_err(|err| StateError::Io(err.to_string()))?;
    }
    let _ = path;
    Ok(())
}

fn ensure_regular_db_file_0600(path: &Path) -> Result<(), StateError> {
    if path.exists() {
        reject_nonregular_db(path)?;
        set_file_mode_0600(path)?;
        return Ok(());
    }
    create_db_file_0600(path)
}

fn reject_nonregular_db(path: &Path) -> Result<(), StateError> {
    let meta = fs::symlink_metadata(path).map_err(|err| StateError::Io(err.to_string()))?;
    if meta.file_type().is_symlink() {
        return Err(StateError::Io(format!(
            "state database must not be a symlink: {}",
            path.display()
        )));
    }
    if !meta.file_type().is_file() {
        return Err(StateError::Io(format!(
            "state database must be a regular file: {}",
            path.display()
        )));
    }
    Ok(())
}

fn create_db_file_0600(path: &Path) -> Result<(), StateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .mode(0o600)
            .open(path)
            .map_err(|err| StateError::Io(err.to_string()))?;
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)
            .map_err(|err| StateError::Io(err.to_string()))?;
        Ok(())
    }
}

fn set_file_mode_0600(path: &Path) -> Result<(), StateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if !path.exists() {
            return Ok(());
        }
        let mut perms = fs::metadata(path)
            .map_err(|err| StateError::Io(err.to_string()))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms).map_err(|err| StateError::Io(err.to_string()))?;
    }
    let _ = path;
    Ok(())
}

fn sidecar_path(db_path: &Path, suffix: &str) -> PathBuf {
    let mut os = db_path.as_os_str().to_owned();
    os.push(suffix);
    PathBuf::from(os)
}

fn set_sidecar_modes_0600(db_path: &Path) -> Result<(), StateError> {
    let wal = sidecar_path(db_path, "-wal");
    let shm = sidecar_path(db_path, "-shm");
    if wal.exists() {
        set_file_mode_0600(&wal)?;
    }
    if shm.exists() {
        set_file_mode_0600(&shm)?;
    }
    Ok(())
}

fn open_lock_file(path: &Path) -> Result<File, StateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        return OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .map_err(|err| StateError::Io(err.to_string()));
    }
    #[cfg(not(unix))]
    {
        OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|err| StateError::Io(err.to_string()))
    }
}

fn repository_key(target: &CandidateTarget) -> Result<String, StateError> {
    match target {
        CandidateTarget::Repository {
            owner, repository, ..
        } => Ok(format!("{owner}/{repository}")),
        CandidateTarget::Appliance { .. } => Err(StateError::Invalid(
            "appliance targets are rejected by the connected repository evolver".to_owned(),
        )),
    }
}

fn validate_sha256_digest(field: &str, digest: &str) -> Result<(), StateError> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(StateError::Invalid(format!(
            "{field} must start with sha256:, got {digest:?}"
        )));
    };
    if hex.len() != 64 || !hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(StateError::Invalid(format!(
            "{field} must be sha256: + 64 lowercase hex, got {digest:?}"
        )));
    }
    Ok(())
}

fn validate_algorithm_qualified_digest(field: &str, digest: &str) -> Result<(), StateError> {
    if let Some(hex) = digest.strip_prefix("sha256:") {
        if hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return Ok(());
        }
    }
    if let Some(hex) = digest.strip_prefix("git-sha1:") {
        if hex.len() == 40 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return Ok(());
        }
    }
    Err(StateError::Invalid(format!(
        "{field} must be sha256:<64 hex> or git-sha1:<40 hex>, got {digest:?}"
    )))
}

fn canonicalize_json_value<T: Serialize>(value: &T) -> Result<String, StateError> {
    let bytes = canonical_json_bytes(value)?;
    String::from_utf8(bytes).map_err(|err| StateError::Invalid(err.to_string()))
}

fn canonicalize_json_text(text: &str) -> Result<String, StateError> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|err| StateError::Invalid(format!("invalid json: {err}")))?;
    canonicalize_json_value(&value)
}

fn datetime_to_text(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn parse_datetime(text: &str) -> Result<DateTime<Utc>, StateError> {
    DateTime::parse_from_rfc3339(text)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| StateError::Integrity(format!("invalid timestamp {text:?}: {err}")))
}

fn parse_state(text: &str) -> Result<CandidateState, StateError> {
    match text {
        "observed" => Ok(CandidateState::Observed),
        "prepared" => Ok(CandidateState::Prepared),
        "building" => Ok(CandidateState::Building),
        "evaluating" => Ok(CandidateState::Evaluating),
        "rejected" => Ok(CandidateState::Rejected),
        "review_ready" => Ok(CandidateState::ReviewReady),
        "promotion_pending" => Ok(CandidateState::PromotionPending),
        "soaking" => Ok(CandidateState::Soaking),
        "accepted" => Ok(CandidateState::Accepted),
        "rolled_back" => Ok(CandidateState::RolledBack),
        "failed" => Ok(CandidateState::Failed),
        other => Err(StateError::Integrity(format!("unknown state {other:?}"))),
    }
}

fn lifecycle_event_type(state: CandidateState) -> &'static str {
    match state {
        CandidateState::Observed => "candidate.observed",
        CandidateState::Prepared => "candidate.prepared",
        CandidateState::Building => "candidate.building",
        CandidateState::Evaluating => "candidate.evaluating",
        CandidateState::Rejected => "candidate.rejected",
        CandidateState::ReviewReady => "candidate.review_ready",
        CandidateState::PromotionPending => "candidate.promotion_pending",
        CandidateState::Soaking => "candidate.soaking",
        CandidateState::Accepted => "candidate.accepted",
        CandidateState::RolledBack => "candidate.rolled_back",
        CandidateState::Failed => "candidate.failed",
    }
}

fn state_from_lifecycle_event(event_type: &str) -> Option<CandidateState> {
    match event_type {
        "candidate.observed" => Some(CandidateState::Observed),
        "candidate.prepared" => Some(CandidateState::Prepared),
        "candidate.building" => Some(CandidateState::Building),
        "candidate.evaluating" => Some(CandidateState::Evaluating),
        "candidate.rejected" => Some(CandidateState::Rejected),
        "candidate.review_ready" => Some(CandidateState::ReviewReady),
        "candidate.promotion_pending" => Some(CandidateState::PromotionPending),
        "candidate.soaking" => Some(CandidateState::Soaking),
        "candidate.accepted" => Some(CandidateState::Accepted),
        "candidate.rolled_back" => Some(CandidateState::RolledBack),
        "candidate.failed" => Some(CandidateState::Failed),
        _ => None,
    }
}

fn validate_terminal_reason_rules(
    next: CandidateState,
    reason: Option<&str>,
) -> Result<(), StateError> {
    match next {
        CandidateState::Rejected | CandidateState::RolledBack | CandidateState::Failed => {
            if reason.is_none() {
                return Err(StateError::Invalid(format!(
                    "terminal_reason is required for state {next}"
                )));
            }
        }
        CandidateState::Accepted
        | CandidateState::Observed
        | CandidateState::Prepared
        | CandidateState::Building
        | CandidateState::Evaluating
        | CandidateState::ReviewReady
        | CandidateState::PromotionPending
        | CandidateState::Soaking => {
            if reason.is_some() {
                return Err(StateError::Invalid(format!(
                    "terminal_reason is forbidden for state {next}"
                )));
            }
        }
    }
    Ok(())
}

fn merge_set_once_path(
    field: &str,
    current: Option<PathBuf>,
    incoming: Option<PathBuf>,
) -> Result<Option<PathBuf>, StateError> {
    match (current, incoming) {
        (None, next) => Ok(next),
        (Some(existing), None) => Ok(Some(existing)),
        (Some(existing), Some(new_value)) if existing == new_value => Ok(Some(existing)),
        (Some(_), Some(_)) => Err(StateError::IllegalTransition(format!(
            "{field} is set-once and cannot change"
        ))),
    }
}

fn merge_set_once_string(
    field: &str,
    current: Option<String>,
    incoming: Option<String>,
) -> Result<Option<String>, StateError> {
    match (current, incoming) {
        (None, next) => Ok(next),
        (Some(existing), None) => Ok(Some(existing)),
        (Some(existing), Some(new_value)) if existing == new_value => Ok(Some(existing)),
        (Some(_), Some(_)) => Err(StateError::IllegalTransition(format!(
            "{field} is set-once and cannot change"
        ))),
    }
}

fn merge_receipt_set_once(
    cur_json: Option<String>,
    cur_digest: Option<String>,
    in_json: Option<String>,
    in_digest: Option<String>,
) -> Result<(Option<String>, Option<String>), StateError> {
    match (cur_json, cur_digest, in_json, in_digest) {
        (None, None, None, None) => Ok((None, None)),
        (None, None, Some(j), Some(d)) => Ok((Some(j), Some(d))),
        (Some(j), Some(d), None, None) => Ok((Some(j), Some(d))),
        (Some(j), Some(d), Some(nj), Some(nd)) if j == nj && d == nd => Ok((Some(j), Some(d))),
        (Some(_), Some(_), Some(_), Some(_)) => Err(StateError::IllegalTransition(
            "worker receipt is set-once and cannot change".to_owned(),
        )),
        (Some(_), Some(_), None, Some(_))
        | (Some(_), Some(_), Some(_), None)
        | (None, None, Some(_), None)
        | (None, None, None, Some(_)) => Err(StateError::Invalid(
            "worker_receipt_json and receipt_digest must appear together".to_owned(),
        )),
        (Some(_), None, _, _) | (None, Some(_), _, _) => Err(StateError::Integrity(
            "stored receipt pair is incomplete".to_owned(),
        )),
    }
}

fn is_active_candidate_violation(err: &rusqlite::Error) -> bool {
    match err {
        rusqlite::Error::SqliteFailure(info, Some(msg)) => {
            info.code == rusqlite::ErrorCode::ConstraintViolation
                && (msg.contains("one_active_candidate")
                    || msg.contains("candidates.repository")
                    || (msg.to_lowercase().contains("unique")
                        && msg.contains("repository")
                        && !msg.contains("PRIMARY KEY")
                        && !msg.contains("candidates.id")))
        }
        rusqlite::Error::SqliteFailure(info, None) => {
            // Fall back: SQLITE_CONSTRAINT_UNIQUE without message is rare; reject
            // only when extended code is unique AND not primary key if available.
            info.extended_code == 2067 // SQLITE_CONSTRAINT_UNIQUE
        }
        _ => false,
    }
}

fn is_busy_or_locked(err: &rusqlite::Error) -> bool {
    match err {
        rusqlite::Error::SqliteFailure(info, _) => matches!(
            info.code,
            rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
        ),
        _ => {
            let text = err.to_string().to_lowercase();
            text.contains("database is locked")
                || text.contains("database is busy")
                || text.contains("busy_snapshot")
        }
    }
}

fn append_audit_event(
    conn: &Connection,
    candidate_id: Option<CandidateId>,
    event_type: &str,
    payload: &impl Serialize,
    now: DateTime<Utc>,
) -> Result<AuditEvent, StateError> {
    let previous = latest_audit_event(conn)?;
    let event = AuditEvent::next_at(previous.as_ref(), event_type, candidate_id, payload, now)?;
    let event_json = canonicalize_json_value(&event)?;
    conn.execute(
        "INSERT INTO audit_events (sequence, event_json, event_hash) VALUES (?1, ?2, ?3)",
        params![event.sequence as i64, event_json, event.event_hash],
    )?;
    Ok(event)
}

fn latest_audit_event(conn: &Connection) -> Result<Option<AuditEvent>, StateError> {
    let json: Option<String> = conn
        .query_row(
            "SELECT event_json FROM audit_events ORDER BY sequence DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    match json {
        Some(text) => {
            let event: AuditEvent = serde_json::from_str(&text).map_err(|err| {
                StateError::Integrity(format!("audit head json decode failed: {err}"))
            })?;
            Ok(Some(event))
        }
        None => Ok(None),
    }
}

fn load_and_verify_audit_events(conn: &Connection) -> Result<Vec<AuditEvent>, StateError> {
    let mut stmt =
        conn.prepare("SELECT event_json, event_hash FROM audit_events ORDER BY sequence ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut events = Vec::new();
    for row in rows {
        let (json, stored_hash) = row?;
        let event: AuditEvent = serde_json::from_str(&json).map_err(|err| {
            StateError::Integrity(format!("audit event json decode failed: {err}"))
        })?;
        let canonical = canonicalize_json_value(&event)?;
        if canonical != json {
            return Err(StateError::Integrity(format!(
                "audit sequence {} json is not canonical",
                event.sequence
            )));
        }
        if stored_hash != event.event_hash {
            return Err(StateError::Integrity(format!(
                "audit sequence {} event_hash column mismatch",
                event.sequence
            )));
        }
        events.push(event);
    }
    verify_chain(&events)?;
    Ok(events)
}

fn load_record_verified(
    conn: &Connection,
    id: &CandidateId,
) -> Result<CandidateRecord, StateError> {
    let events = load_and_verify_audit_events(conn)?;
    load_record_in_tx(conn, id, Some(events.as_slice()))
}

fn load_record_in_tx(
    conn: &Connection,
    id: &CandidateId,
    events: Option<&[AuditEvent]>,
) -> Result<CandidateRecord, StateError> {
    let row = conn
        .query_row(
            r#"
            SELECT
              id, repository, manifest_json, manifest_digest, policy_digest, state,
              workspace, candidate_digest, terminal_reason, worker_receipt_json, receipt_digest,
              created_at, updated_at
            FROM candidates WHERE id = ?1
            "#,
            params![id.as_str()],
            |row| {
                Ok(RawCandidateRow {
                    id: row.get(0)?,
                    repository: row.get(1)?,
                    manifest_json: row.get(2)?,
                    manifest_digest: row.get(3)?,
                    policy_digest: row.get(4)?,
                    state: row.get(5)?,
                    workspace: row.get(6)?,
                    candidate_digest: row.get(7)?,
                    terminal_reason: row.get(8)?,
                    worker_receipt_json: row.get(9)?,
                    receipt_digest: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        )
        .optional()?;

    let Some(row) = row else {
        return Err(StateError::Invalid(format!(
            "candidate {} not found",
            id.as_str()
        )));
    };
    let record = materialize_record(row)?;
    if let Some(events) = events {
        cross_check_state_with_audit(&record, events)?;
    }
    Ok(record)
}

fn cross_check_state_with_audit(
    record: &CandidateRecord,
    events: &[AuditEvent],
) -> Result<(), StateError> {
    let id = record.id().as_str();
    let latest = events.iter().rev().find(|event| {
        event
            .candidate_id
            .as_ref()
            .map(|cid| cid.as_str() == id)
            .unwrap_or(false)
            && state_from_lifecycle_event(&event.event_type).is_some()
    });
    let Some(event) = latest else {
        return Err(StateError::Integrity(format!(
            "no lifecycle audit event found for candidate {id}"
        )));
    };
    let expected = state_from_lifecycle_event(&event.event_type).expect("filtered");
    if expected != record.state() {
        return Err(StateError::Integrity(format!(
            "candidate {id} state {} does not match latest audit event {}",
            record.state(),
            event.event_type
        )));
    }
    Ok(())
}

struct RawCandidateRow {
    id: String,
    repository: String,
    manifest_json: String,
    manifest_digest: String,
    policy_digest: String,
    state: String,
    workspace: Option<String>,
    candidate_digest: Option<String>,
    terminal_reason: Option<String>,
    worker_receipt_json: Option<String>,
    receipt_digest: Option<String>,
    created_at: String,
    updated_at: String,
}

fn materialize_record(row: RawCandidateRow) -> Result<CandidateRecord, StateError> {
    if row.id.is_empty() {
        return Err(StateError::Integrity("empty candidate id".to_owned()));
    }
    let manifest: CandidateManifest = serde_json::from_str(&row.manifest_json)
        .map_err(|err| StateError::Integrity(format!("manifest json decode failed: {err}")))?;
    manifest.validate()?;
    if manifest.id.as_str() != row.id {
        return Err(StateError::Integrity(format!(
            "manifest id {} does not match row id {}",
            manifest.id.as_str(),
            row.id
        )));
    }

    let expected_repo = repository_key(&manifest.target)?;
    if expected_repo != row.repository {
        return Err(StateError::Integrity(format!(
            "repository key mismatch: row {} vs manifest {expected_repo}",
            row.repository
        )));
    }

    let canonical = canonicalize_json_value(&manifest)?;
    if canonical != row.manifest_json {
        return Err(StateError::Integrity(
            "stored manifest_json is not canonical".to_owned(),
        ));
    }
    let expected_digest = format!("sha256:{}", sha256_hex(canonical.as_bytes()));
    if expected_digest != row.manifest_digest {
        return Err(StateError::Integrity(
            "manifest_digest does not match stored manifest_json".to_owned(),
        ));
    }
    validate_sha256_digest("policy_digest", &row.policy_digest)
        .map_err(|err| StateError::Integrity(err.to_string()))?;

    match (&row.worker_receipt_json, &row.receipt_digest) {
        (None, None) => {}
        (Some(json), Some(digest)) => {
            validate_sha256_digest("receipt_digest", digest)
                .map_err(|err| StateError::Integrity(err.to_string()))?;
            let canonical_receipt = canonicalize_json_text(json)?;
            if canonical_receipt != *json {
                return Err(StateError::Integrity(
                    "stored worker_receipt_json is not canonical".to_owned(),
                ));
            }
            let expected = format!("sha256:{}", sha256_hex(canonical_receipt.as_bytes()));
            if expected != *digest {
                return Err(StateError::Integrity(
                    "receipt_digest does not match worker_receipt_json".to_owned(),
                ));
            }
        }
        _ => {
            return Err(StateError::Integrity(
                "stored receipt pair is incomplete".to_owned(),
            ));
        }
    }

    if let Some(digest) = &row.candidate_digest {
        validate_algorithm_qualified_digest("candidate_digest", digest)
            .map_err(|err| StateError::Integrity(err.to_string()))?;
    }

    let workspace = match row.workspace {
        Some(text) => {
            if !text.is_ascii() && std::str::from_utf8(text.as_bytes()).is_err() {
                return Err(StateError::Integrity(
                    "stored workspace is not valid UTF-8".to_owned(),
                ));
            }
            let path = PathBuf::from(&text);
            if path.to_str().is_none() {
                return Err(StateError::Integrity(
                    "stored workspace is not valid UTF-8".to_owned(),
                ));
            }
            if !path.is_absolute() {
                return Err(StateError::Integrity(format!(
                    "stored workspace is not absolute: {text}"
                )));
            }
            Some(path)
        }
        None => None,
    };

    Ok(CandidateRecord {
        manifest,
        manifest_digest: row.manifest_digest,
        policy_digest: row.policy_digest,
        state: parse_state(&row.state)?,
        workspace,
        candidate_digest: row.candidate_digest,
        worker_receipt_json: row.worker_receipt_json,
        receipt_digest: row.receipt_digest,
        terminal_reason: row.terminal_reason,
        created_at: parse_datetime(&row.created_at)?,
        updated_at: parse_datetime(&row.updated_at)?,
        repository: row.repository,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use evolution_contracts::{AuthorityTier, CandidateKind, ResourceBudget, CANDIDATE_SCHEMA};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use tempfile::TempDir;

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 1, 7, 0, 0).unwrap()
    }

    fn policy_digest() -> String {
        format!("sha256:{}", "ab".repeat(32))
    }

    fn id(raw: &str) -> CandidateId {
        CandidateId::parse(raw).expect("valid id")
    }

    fn valid_budget() -> ResourceBudget {
        ResourceBudget {
            wall_seconds: 2700,
            max_attempts: 1,
            max_changed_files: 20,
            max_added_lines: 1500,
            max_tool_calls: 80,
            max_input_tokens: 250_000,
            max_output_tokens: 50_000,
            max_energy_joules: None,
            allow_missing_energy_meter: true,
        }
    }

    fn manifest(raw_id: &str) -> CandidateManifest {
        let cid = id(raw_id);
        CandidateManifest {
            schema: CANDIDATE_SCHEMA.to_owned(),
            id: cid.clone(),
            mission_id: "mission-felt-use-20260901".to_owned(),
            kind: CandidateKind::Code,
            authority: AuthorityTier::Candidate,
            target: CandidateTarget::Repository {
                owner: "maximilianwruhs-cyber".to_owned(),
                repository: "GZMO".to_owned(),
                base_branch: "main".to_owned(),
                candidate_branch: format!("evolve/{}", cid.as_str()),
            },
            baseline_digest: format!("git-sha1:{}", "a".repeat(40)),
            required_gates: vec!["format".to_owned(), "tests".to_owned()],
            protected_paths: vec![
                "docs/ADR-0014-constitutional-evolution.md".to_owned(),
                "config/repo-evolver.policy.toml".to_owned(),
            ],
            budget: valid_budget(),
            created_at: fixed_now(),
        }
    }

    fn appliance_manifest() -> CandidateManifest {
        CandidateManifest {
            schema: CANDIDATE_SCHEMA.to_owned(),
            id: id("cand-20260901t070000z-app-aaaa1111"),
            mission_id: "mission-appliance-slot".to_owned(),
            kind: CandidateKind::Runtime,
            authority: AuthorityTier::Candidate,
            target: CandidateTarget::Appliance {
                node_id: "ct101".to_owned(),
                target_class: "living-appliance".to_owned(),
                inactive_target: Some("slot-b".to_owned()),
            },
            baseline_digest: format!("sha256:{}", "b".repeat(64)),
            required_gates: vec!["bundle-verify".to_owned()],
            protected_paths: vec!["boot.sh".to_owned()],
            budget: valid_budget(),
            created_at: fixed_now(),
        }
    }

    fn repo_key() -> String {
        "maximilianwruhs-cyber/GZMO".to_owned()
    }

    fn receipt_pair() -> (String, String) {
        let value = serde_json::json!({
            "schema": "gzmo.repo_evolver.worker_receipt/v1",
            "ok": true,
            "files_changed": 1
        });
        let canonical = canonicalize_json_value(&value).unwrap();
        let digest = format!("sha256:{}", sha256_hex(canonical.as_bytes()));
        (canonical, digest)
    }

    #[test]
    fn repository_allows_only_one_nonterminal_candidate() {
        let store = StateStore::open_in_memory().unwrap();
        let now = fixed_now();
        store
            .create_candidate(
                &manifest("cand-20260901t070000z-one-aaaa1111"),
                &policy_digest(),
                now,
            )
            .unwrap();
        let err = store
            .create_candidate(
                &manifest("cand-20260901t080000z-two-bbbb2222"),
                &policy_digest(),
                now,
            )
            .unwrap_err();
        assert!(matches!(err, StateError::AlreadyActive(_)));
        store
            .transition(
                &id("cand-20260901t070000z-one-aaaa1111"),
                CandidateState::Failed,
                TransitionMetadata::terminal("operator abort"),
                now,
            )
            .unwrap();
        assert!(store
            .create_candidate(
                &manifest("cand-20260901t080000z-two-bbbb2222"),
                &policy_digest(),
                now,
            )
            .is_ok());
        assert!(store.verify_audit_chain().is_ok());
    }

    #[test]
    fn rejects_appliance_targets() {
        let store = StateStore::open_in_memory().unwrap();
        let err = store
            .create_candidate(&appliance_manifest(), &policy_digest(), fixed_now())
            .unwrap_err();
        assert!(err.to_string().contains("appliance"));
    }

    #[test]
    fn legal_and_illegal_transitions() {
        let store = StateStore::open_in_memory().unwrap();
        let now = fixed_now();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
            .unwrap();

        assert!(store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty(),
                now
            )
            .is_err());

        store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap();
        store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap();
        store
            .transition(
                &mid,
                CandidateState::Evaluating,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap();
        assert!(store
            .transition(
                &mid,
                CandidateState::Accepted,
                TransitionMetadata::empty(),
                now
            )
            .is_err());
        store
            .transition(
                &mid,
                CandidateState::Failed,
                TransitionMetadata::terminal("gate failed"),
                now,
            )
            .unwrap();
        assert!(store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty(),
                now
            )
            .is_err());
        assert!(store.verify_audit_chain().is_ok());
        let events = store.load_audit_events().unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].event_type, "candidate.observed");
        assert_eq!(events[4].event_type, "candidate.failed");
    }

    #[test]
    fn metadata_set_once_and_receipt_pair_rules() {
        let store = StateStore::open_in_memory().unwrap();
        let now = fixed_now();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
            .unwrap();

        let ws = PathBuf::from("/tmp/gzmo-evolver-candidate-ws");
        let cand_digest = format!("git-sha1:{}", "c".repeat(40));
        let (receipt_json, receipt_digest) = receipt_pair();

        store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty()
                    .with_workspace(ws.clone())
                    .with_candidate_digest(cand_digest.clone())
                    .with_receipt(receipt_json.clone(), receipt_digest.clone()),
                now,
            )
            .unwrap();

        let record = store.load(&mid).unwrap();
        assert_eq!(record.workspace(), Some(ws.as_path()));
        assert_eq!(record.candidate_digest(), Some(cand_digest.as_str()));
        assert_eq!(record.receipt_digest(), Some(receipt_digest.as_str()));
        assert_eq!(record.worker_receipt_json(), Some(receipt_json.as_str()));

        assert!(store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty().with_workspace(PathBuf::from("/tmp/other")),
                now,
            )
            .is_err());
        assert!(store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty()
                    .with_candidate_digest(format!("git-sha1:{}", "d".repeat(40))),
                now,
            )
            .is_err());

        // Receipt pair required together — independent assertions (no short-circuit).
        assert!(TransitionMetadata::empty()
            .with_receipt(receipt_json.clone(), "sha256:dead".to_owned())
            .validate_shape()
            .is_err());
        assert!(store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata {
                    worker_receipt_json: Some(receipt_json.clone()),
                    receipt_digest: None,
                    ..TransitionMetadata::empty()
                },
                now,
            )
            .is_err());

        assert!(store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty()
                    .with_receipt(receipt_json.clone(), format!("sha256:{}", "0".repeat(64)),),
                now,
            )
            .is_err());

        // Non-canonical equivalent receipt text is still set-once-idempotent.
        let noncanonical = r#"{
  "files_changed": 1,
  "ok": true,
  "schema": "gzmo.repo_evolver.worker_receipt/v1"
}"#;
        assert_ne!(noncanonical, receipt_json);
        store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty()
                    .with_workspace(ws)
                    .with_candidate_digest(cand_digest)
                    .with_receipt(noncanonical, receipt_digest),
                now,
            )
            .unwrap();
    }

    #[test]
    fn terminal_reason_rules() {
        let store = StateStore::open_in_memory().unwrap();
        let now = fixed_now();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
            .unwrap();

        assert!(store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::terminal("nope"),
                now,
            )
            .is_err());

        store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap();
        store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap();
        store
            .transition(
                &mid,
                CandidateState::Evaluating,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap();

        assert!(store
            .transition(
                &mid,
                CandidateState::Failed,
                TransitionMetadata::empty(),
                now
            )
            .is_err());
        store
            .transition(
                &mid,
                CandidateState::Failed,
                TransitionMetadata::terminal("boom"),
                now,
            )
            .unwrap();

        let store2 = StateStore::open_in_memory().unwrap();
        let mid2 = id("cand-20260901t080000z-two-bbbb2222");
        store2
            .create_candidate(&manifest(mid2.as_str()), &policy_digest(), now)
            .unwrap();
        let long = "x".repeat(MAX_TERMINAL_REASON_BYTES + 1);
        assert!(store2
            .transition(
                &mid2,
                CandidateState::Failed,
                TransitionMetadata::terminal(long),
                now,
            )
            .is_err());
    }

    #[test]
    fn legal_transition_rolls_back_when_audit_append_fails() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::open(dir.path()).unwrap();
        let now = fixed_now();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
            .unwrap();

        // Corrupt audit head so next_at(prev.validate()) fails after UPDATE.
        let json: String = store
            .conn
            .query_row(
                "SELECT event_json FROM audit_events WHERE sequence = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let hash = value
            .get("event_hash")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_owned();
        let mut chars: Vec<char> = hash.chars().collect();
        chars[0] = if chars[0] == 'a' { 'b' } else { 'a' };
        value["event_hash"] = serde_json::Value::String(chars.into_iter().collect());
        // Keep event_hash column matching the corrupted JSON so load path is not
        // the only failure mode; next_at validates the previous event hash.
        let tampered = serde_json::to_string(&value).unwrap();
        let bad_hash = value["event_hash"].as_str().unwrap().to_owned();
        store
            .conn
            .execute(
                "UPDATE audit_events SET event_json = ?1, event_hash = ?2 WHERE sequence = 1",
                params![tampered, bad_hash],
            )
            .unwrap();

        let err = store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap_err();
        assert!(
            matches!(err, StateError::Audit(_) | StateError::Integrity(_)),
            "expected audit failure, got {err:?}"
        );

        let state: String = store
            .conn
            .query_row(
                "SELECT state FROM candidates WHERE id = ?1",
                params![mid.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "observed");
        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn commit_failure_rolls_back_and_leaves_autocommit() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::open(dir.path()).unwrap();
        let now = fixed_now();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
            .unwrap();

        // Install deferred FK parent/child plus a trigger that poisons COMMIT
        // when audit_events gains a row (no production hook required).
        store
            .conn
            .execute_batch(
                r#"
                CREATE TABLE commit_fail_parent (id INTEGER PRIMARY KEY);
                CREATE TABLE commit_fail_child (
                  id INTEGER PRIMARY KEY,
                  parent_id INTEGER NOT NULL,
                  FOREIGN KEY (parent_id) REFERENCES commit_fail_parent(id)
                    DEFERRABLE INITIALLY DEFERRED
                );
                CREATE TRIGGER audit_commit_fail AFTER INSERT ON audit_events
                BEGIN
                  INSERT INTO commit_fail_child(id, parent_id)
                  VALUES (NEW.sequence, 999);
                END;
                "#,
            )
            .unwrap();

        let err = store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap_err();
        assert!(
            matches!(err, StateError::Db(_) | StateError::Contention(_)),
            "expected COMMIT-time failure, got {err:?}"
        );
        assert!(
            store.conn.is_autocommit(),
            "connection must return to autocommit after COMMIT failure cleanup"
        );

        let state: String = store
            .conn
            .query_row(
                "SELECT state FROM candidates WHERE id = ?1",
                params![mid.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "observed");
        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Remove poison apparatus and prove the store is still usable.
        store
            .conn
            .execute_batch(
                r#"
                DROP TRIGGER IF EXISTS audit_commit_fail;
                DROP TABLE IF EXISTS commit_fail_child;
                DROP TABLE IF EXISTS commit_fail_parent;
                "#,
            )
            .unwrap();

        store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty(),
                now,
            )
            .unwrap();
        let record = store.load(&mid).unwrap();
        assert_eq!(record.state(), CandidateState::Prepared);
        assert!(store.conn.is_autocommit());
    }

    #[test]
    fn concurrent_insert_race_enforced_by_partial_unique_index() {
        let dir = TempDir::new().unwrap();
        let state_dir = dir.path().join("coord");
        drop(StateStore::open(&state_dir).unwrap());

        let barrier = Arc::new(Barrier::new(2));
        let state_dir = Arc::new(state_dir);
        let results = Arc::new(Mutex::new(Vec::new()));

        let mut handles = Vec::new();
        for (idx, cand) in [
            "cand-20260901t070000z-one-aaaa1111",
            "cand-20260901t080000z-two-bbbb2222",
        ]
        .into_iter()
        .enumerate()
        {
            let barrier = Arc::clone(&barrier);
            let state_dir = Arc::clone(&state_dir);
            let results = Arc::clone(&results);
            handles.push(thread::spawn(move || {
                let store = StateStore::open(state_dir.as_path()).unwrap();
                barrier.wait();
                let outcome =
                    store.create_candidate(&manifest(cand), &policy_digest(), fixed_now());
                results.lock().unwrap().push((idx, outcome));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let outcomes = results.lock().unwrap();
        let oks = outcomes.iter().filter(|(_, r)| r.is_ok()).count();
        let domain_errs = outcomes
            .iter()
            .filter(|(_, r)| matches!(r, Err(StateError::AlreadyActive(_))))
            .count();
        assert_eq!(oks, 1, "exactly one insert must succeed: {outcomes:?}");
        assert_eq!(
            domain_errs, 1,
            "loser must be AlreadyActive, not raw lock: {outcomes:?}"
        );

        let store = StateStore::open(state_dir.as_path()).unwrap();
        let active = store.active_candidate(&repo_key()).unwrap().unwrap();
        assert_eq!(active.state(), CandidateState::Observed);
        assert!(store.verify_audit_chain().is_ok());
    }

    #[test]
    fn concurrent_transition_race_reports_domain_contention() {
        let dir = TempDir::new().unwrap();
        let state_dir = dir.path().join("coord");
        let store = StateStore::open(&state_dir).unwrap();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), fixed_now())
            .unwrap();
        drop(store);

        let barrier = Arc::new(Barrier::new(2));
        let state_dir = Arc::new(state_dir);
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        for idx in 0..2 {
            let barrier = Arc::clone(&barrier);
            let state_dir = Arc::clone(&state_dir);
            let results = Arc::clone(&results);
            handles.push(thread::spawn(move || {
                let store = StateStore::open(state_dir.as_path()).unwrap();
                barrier.wait();
                let outcome = store.transition(
                    &id("cand-20260901t070000z-one-aaaa1111"),
                    CandidateState::Prepared,
                    TransitionMetadata::empty(),
                    fixed_now(),
                );
                results.lock().unwrap().push((idx, outcome));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let outcomes = results.lock().unwrap();
        let oks = outcomes.iter().filter(|(_, r)| r.is_ok()).count();
        // BEGIN IMMEDIATE serialises the read-modify-write: the loser blocks on
        // the write lock, then reads the committed state and reports an illegal
        // edge. A deferred transaction would instead fail on a stale snapshot
        // (Contention/BUSY_SNAPSHOT).
        assert_eq!(oks, 1, "exactly one transition succeeds: {outcomes:?}");
        for (_, r) in outcomes.iter() {
            if let Err(err) = r {
                assert!(
                    matches!(err, StateError::IllegalTransition(_)),
                    "loser must observe committed state as IllegalTransition, got {err:?}"
                );
            }
        }

        let store = StateStore::open(state_dir.as_path()).unwrap();
        let record = store.load(&mid).unwrap();
        assert_eq!(record.state(), CandidateState::Prepared);
        let events = store.load_audit_events().unwrap();
        // observed + exactly one prepared
        assert_eq!(
            events
                .iter()
                .filter(|e| e.event_type == "candidate.prepared")
                .count(),
            1
        );
    }

    #[test]
    fn duplicate_id_is_not_reported_as_already_active() {
        let store = StateStore::open_in_memory().unwrap();
        let now = fixed_now();
        let mid = "cand-20260901t070000z-one-aaaa1111";
        store
            .create_candidate(&manifest(mid), &policy_digest(), now)
            .unwrap();
        store
            .transition(
                &id(mid),
                CandidateState::Failed,
                TransitionMetadata::terminal("done"),
                now,
            )
            .unwrap();
        let err = store
            .create_candidate(&manifest(mid), &policy_digest(), now)
            .unwrap_err();
        assert!(
            !matches!(err, StateError::AlreadyActive(_)),
            "duplicate primary key must not look like already-active: {err:?}"
        );
        assert!(
            matches!(err, StateError::Db(_)),
            "expected Db for primary-key collision, got {err:?}"
        );
    }

    #[test]
    fn stored_manifest_digest_tamper_is_detected() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::open(dir.path()).unwrap();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), fixed_now())
            .unwrap();
        store
            .conn
            .execute(
                "UPDATE candidates SET manifest_digest = ?1 WHERE id = ?2",
                params![format!("sha256:{}", "0".repeat(64)), mid.as_str()],
            )
            .unwrap();
        assert!(store.load(&mid).is_err());
        assert!(store.active_candidate(&repo_key()).is_err());
    }

    #[test]
    fn state_column_tamper_detected_against_audit_trail() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::open(dir.path()).unwrap();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), fixed_now())
            .unwrap();
        store
            .conn
            .execute(
                "UPDATE candidates SET state = 'accepted' WHERE id = ?1",
                params![mid.as_str()],
            )
            .unwrap();
        assert!(store.load(&mid).is_err());
        store
            .conn
            .execute(
                "UPDATE candidates SET state = 'building' WHERE id = ?1",
                params![mid.as_str()],
            )
            .unwrap();
        assert!(store.active_candidate(&repo_key()).is_err());

        // Mutate path must refuse to launder forged Building into Evaluating.
        let err = store
            .transition(
                &mid,
                CandidateState::Evaluating,
                TransitionMetadata::empty(),
                fixed_now(),
            )
            .unwrap_err();
        assert!(
            matches!(err, StateError::Integrity(_)),
            "tampered Building must fail in-tx audit cross-check, got {err:?}"
        );
        let state: String = store
            .conn
            .query_row(
                "SELECT state FROM candidates WHERE id = ?1",
                params![mid.as_str()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(state, "building");
        let count: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn audit_tamper_is_detected() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::open(dir.path()).unwrap();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), fixed_now())
            .unwrap();
        let json: String = store
            .conn
            .query_row(
                "SELECT event_json FROM audit_events WHERE sequence = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let hash = value
            .get_mut("event_hash")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();
        let mut chars: Vec<char> = hash.chars().collect();
        chars[0] = if chars[0] == 'a' { 'b' } else { 'a' };
        value["event_hash"] = serde_json::Value::String(chars.into_iter().collect());
        let tampered = value.to_string();
        store
            .conn
            .execute(
                "UPDATE audit_events SET event_json = ?1 WHERE sequence = 1",
                params![tampered],
            )
            .unwrap();
        assert!(store.verify_audit_chain().is_err());
        assert!(store.load(&mid).is_err());
    }

    #[test]
    fn missing_state_open_existing_readonly_is_none_without_filesystem_changes() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("never-created");
        assert!(!missing.exists());
        let before = fs::read_dir(dir.path()).unwrap().count();
        let opened = StateStore::open_existing_readonly(&missing).unwrap();
        assert!(opened.is_none());
        assert!(!missing.exists());
        let after = fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(before, after);
    }

    #[test]
    fn open_creates_unix_modes_and_status_works_while_lock_held() {
        let dir = TempDir::new().unwrap();
        let state_dir = dir.path().join("coord");
        let store = StateStore::open(&state_dir).unwrap();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), fixed_now())
            .unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = fs::metadata(&state_dir).unwrap().permissions().mode() & 0o777;
            assert_eq!(dir_mode, 0o700);
            let db_path = state_dir.join(STATE_DB_NAME);
            let db_mode = fs::metadata(&db_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(db_mode, 0o600);

            // WAL/SHM must exist; force 0644 then prove helper restores 0600.
            let wal = sidecar_path(&db_path, "-wal");
            let shm = sidecar_path(&db_path, "-shm");
            for side in [&wal, &shm] {
                assert!(
                    side.exists(),
                    "{} must exist while store open",
                    side.display()
                );
                let mut perms = fs::metadata(side).unwrap().permissions();
                perms.set_mode(0o644);
                fs::set_permissions(side, perms).unwrap();
            }
            set_sidecar_modes_0600(&db_path).unwrap();
            for side in [&wal, &shm] {
                let mode = fs::metadata(side).unwrap().permissions().mode() & 0o777;
                assert_eq!(mode, 0o600, "{} mode", side.display());
            }
        }
        drop(store);

        let lock = CoordinatorLock::try_acquire(&state_dir).unwrap();
        assert!(matches!(
            CoordinatorLock::try_acquire(&state_dir),
            Err(StateError::LockBusy)
        ));

        let ro = StateStore::open_existing_readonly(&state_dir)
            .unwrap()
            .expect("db exists");
        let active = ro.active_candidate(&repo_key()).unwrap().unwrap();
        assert_eq!(active.id().as_str(), mid.as_str());
        assert!(ro.audit_head().unwrap().is_some());
        assert!(matches!(
            ro.create_candidate(
                &manifest("cand-20260901t090000z-three-cccc3333"),
                &policy_digest(),
                fixed_now()
            ),
            Err(StateError::ReadOnly)
        ));
        drop(lock);
        let _lock2 = CoordinatorLock::try_acquire(&state_dir).unwrap();
    }

    #[test]
    fn rejects_decoy_one_active_index_on_reopen() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::open(dir.path()).unwrap();
        store
            .create_candidate(
                &manifest("cand-20260901t070000z-one-aaaa1111"),
                &policy_digest(),
                fixed_now(),
            )
            .unwrap();
        drop(store);

        let conn = Connection::open(dir.path().join(STATE_DB_NAME)).unwrap();
        conn.execute_batch(
            "DROP INDEX one_active_candidate;
             CREATE UNIQUE INDEX one_active_candidate ON candidates(id);",
        )
        .unwrap();
        drop(conn);

        let err = StateStore::open(dir.path()).unwrap_err();
        assert!(
            matches!(err, StateError::Integrity(_)),
            "decoy index must fail schema verify: {err:?}"
        );
    }

    #[test]
    fn accepts_and_upgrades_valid_version_zero_database() {
        let dir = TempDir::new().unwrap();
        // Build a Task-2-shaped DB without identity stamps from the same
        // constants production installs (no SCHEMA_SQL drift).
        let db_path = dir.path().join(STATE_DB_NAME);
        fs::create_dir_all(dir.path()).unwrap();
        let conn = Connection::open(&db_path).unwrap();
        install_schema(&conn).unwrap();
        // Explicitly leave user_version/application_id at 0.
        drop(conn);

        let store = StateStore::open(dir.path()).unwrap();
        let version: i32 = store
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        let app: i32 = store
            .conn
            .pragma_query_value(None, "application_id", |row| row.get(0))
            .unwrap();
        assert_eq!(version, STATE_SCHEMA_VERSION);
        assert_eq!(app, STATE_APPLICATION_ID);
        store
            .create_candidate(
                &manifest("cand-20260901t070000z-one-aaaa1111"),
                &policy_digest(),
                fixed_now(),
            )
            .unwrap();
    }

    #[test]
    fn rejects_symlink_database_file() {
        #[cfg(unix)]
        {
            let dir = TempDir::new().unwrap();
            let real = dir.path().join("real.db");
            File::create(&real).unwrap();
            let link = dir.path().join(STATE_DB_NAME);
            std::os::unix::fs::symlink(&real, &link).unwrap();
            let err = StateStore::open(dir.path()).unwrap_err();
            assert!(
                matches!(&err, StateError::Io(msg) if msg.contains("must not be a symlink")),
                "expected symlink rejection, got {err:?}"
            );
        }
    }

    #[test]
    fn non_utf8_state_dir_still_chmods_wal_shm_sidecars() {
        #[cfg(unix)]
        {
            use std::ffi::OsString;
            use std::os::unix::ffi::OsStringExt;
            use std::os::unix::fs::PermissionsExt;

            let dir = TempDir::new().unwrap();
            let mut state_os = dir.path().as_os_str().to_owned();
            state_os.push("/");
            state_os.push(OsString::from_vec(b"coord-\xff".to_vec()));
            let state_dir = PathBuf::from(state_os);

            let store = StateStore::open(&state_dir).unwrap();
            store
                .create_candidate(
                    &manifest("cand-20260901t070000z-one-aaaa1111"),
                    &policy_digest(),
                    fixed_now(),
                )
                .unwrap();

            let db_path = state_dir.join(STATE_DB_NAME);
            let wal = sidecar_path(&db_path, "-wal");
            let shm = sidecar_path(&db_path, "-shm");

            for side in [&wal, &shm] {
                assert!(
                    side.exists(),
                    "{} must exist under non-UTF-8 state_dir",
                    side.display()
                );
                let mut perms = fs::metadata(side).unwrap().permissions();
                perms.set_mode(0o644);
                fs::set_permissions(side, perms).unwrap();
            }
            set_sidecar_modes_0600(&db_path).unwrap();

            for side in [&wal, &shm] {
                let mode = fs::metadata(side).unwrap().permissions().mode() & 0o777;
                assert_eq!(
                    mode,
                    0o600,
                    "{} mode under non-UTF-8 state_dir",
                    side.display()
                );
            }
            drop(store);
        }
    }

    #[test]
    fn rejects_non_utf8_workspace_path() {
        #[cfg(unix)]
        {
            use std::ffi::OsString;
            use std::os::unix::ffi::OsStringExt;
            let store = StateStore::open_in_memory().unwrap();
            let now = fixed_now();
            let mid = id("cand-20260901t070000z-one-aaaa1111");
            store
                .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
                .unwrap();
            let bad = PathBuf::from(OsString::from_vec(b"/tmp/ws-\xff".to_vec()));
            let err = store
                .transition(
                    &mid,
                    CandidateState::Prepared,
                    TransitionMetadata::empty().with_workspace(bad),
                    now,
                )
                .unwrap_err();
            assert!(
                matches!(err, StateError::Invalid(_)),
                "expected Invalid for non-UTF-8 workspace, got {err:?}"
            );
        }
    }

    #[test]
    fn active_candidate_and_audit_head_after_create() {
        let store = StateStore::open_in_memory().unwrap();
        assert!(store.active_candidate(&repo_key()).unwrap().is_none());
        assert!(store.audit_head().unwrap().is_none());
        let mid = "cand-20260901t070000z-one-aaaa1111";
        store
            .create_candidate(&manifest(mid), &policy_digest(), fixed_now())
            .unwrap();
        let active = store.active_candidate(&repo_key()).unwrap().unwrap();
        assert_eq!(active.id().as_str(), mid);
        assert_eq!(active.state(), CandidateState::Observed);
        let head = store.audit_head().unwrap().unwrap();
        assert_eq!(head.sequence, 1);
        assert_eq!(head.event_type, "candidate.observed");
    }

    #[test]
    fn rejects_relative_workspace_and_bad_policy_digest() {
        let store = StateStore::open_in_memory().unwrap();
        let now = fixed_now();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        assert!(store
            .create_candidate(&manifest(mid.as_str()), "not-a-digest", now)
            .is_err());
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
            .unwrap();
        assert!(store
            .transition(
                &mid,
                CandidateState::Prepared,
                TransitionMetadata::empty().with_workspace(PathBuf::from("relative/ws")),
                now,
            )
            .is_err());
    }
}
