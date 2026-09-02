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

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS candidates (
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
);
CREATE UNIQUE INDEX IF NOT EXISTS one_active_candidate
ON candidates(repository)
WHERE state NOT IN ('rejected','accepted','rolled_back','failed');

CREATE TABLE IF NOT EXISTS audit_events (
  sequence INTEGER PRIMARY KEY,
  event_json TEXT NOT NULL,
  event_hash TEXT NOT NULL UNIQUE
);
"#;

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
    /// Coordinator lease is held by another process.
    #[error("coordinator lock busy")]
    LockBusy,
    /// Read-only store rejected a mutating call.
    #[error("state store is read-only")]
    ReadOnly,
    /// Underlying evolution contract validation failed.
    #[error(transparent)]
    Contract(#[from] ContractError),
    /// Audit contract failure.
    #[error(transparent)]
    Audit(#[from] evolution_contracts::AuditError),
}

impl From<rusqlite::Error> for StateError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Db(value.to_string())
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
            fs::create_dir_all(state_dir).map_err(|err| StateError::Io(err.to_string()))?;
            set_dir_mode_0700(state_dir)?;
        }
        let lock_path = state_dir.join(RUNNER_LOCK_NAME);
        let file = open_lock_file(&lock_path)?;
        file.try_lock_exclusive().map_err(|err| {
            if err.kind() == std::io::ErrorKind::WouldBlock {
                StateError::LockBusy
            } else {
                StateError::Io(err.to_string())
            }
        })?;
        Ok(Self { _file: file })
    }
}

/// Coordinator candidate + audit persistence.
pub struct StateStore {
    conn: Connection,
    readonly: bool,
}

impl StateStore {
    /// Create/open a mutating store under `state_dir` (dir 0700, db 0600 on Unix).
    ///
    /// Does **not** acquire the coordinator lease.
    pub fn open(state_dir: impl AsRef<Path>) -> Result<Self, StateError> {
        let state_dir = state_dir.as_ref();
        fs::create_dir_all(state_dir).map_err(|err| StateError::Io(err.to_string()))?;
        set_dir_mode_0700(state_dir)?;

        let db_path = state_dir.join(STATE_DB_NAME);
        let conn = Connection::open(&db_path).map_err(|err| StateError::Db(err.to_string()))?;
        configure_connection(&conn)?;
        conn.execute_batch(SCHEMA_SQL)?;
        set_file_mode_0600(&db_path)?;

        Ok(Self {
            conn,
            readonly: false,
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
        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|err| StateError::Db(err.to_string()))?;
        // Read-only connections still honor busy_timeout / FK; WAL query is fine.
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Some(Self {
            conn,
            readonly: true,
        }))
    }

    /// In-memory mutating store for tests (no lock, no filesystem).
    pub fn open_in_memory() -> Result<Self, StateError> {
        let conn = Connection::open_in_memory().map_err(|err| StateError::Db(err.to_string()))?;
        configure_connection(&conn)?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self {
            conn,
            readonly: false,
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

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|err| StateError::Db(err.to_string()))?;

        let insert = tx.execute(
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
            Err(err) if is_unique_violation(&err) => {
                return Err(StateError::IllegalTransition(format!(
                    "repository {repository} already has a nonterminal candidate"
                )));
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
            &tx,
            Some(manifest.id.clone()),
            "candidate.observed",
            &payload,
            now,
        )?;

        tx.commit()?;
        self.load(&manifest.id)
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

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|err| StateError::Db(err.to_string()))?;

        let current = load_record_in_tx(&tx, id)?;
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
        let (worker_receipt_json, receipt_digest) = merge_receipt_set_once(
            current.worker_receipt_json.clone(),
            current.receipt_digest.clone(),
            metadata.worker_receipt_json.clone(),
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

        // Receipt JSON stored canonical when newly set.
        let worker_receipt_json = match &worker_receipt_json {
            Some(json) => Some(canonicalize_json_text(json)?),
            None => None,
        };

        let next_text = next.to_string();
        let from_text = current.state.to_string();
        let now_text = datetime_to_text(now);
        let workspace_text = workspace.as_ref().map(|p| p.display().to_string());

        tx.execute(
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
        append_audit_event(&tx, Some(id.clone()), event_type, &payload, now)?;
        tx.commit()?;
        self.load(id)
    }

    /// Return the single nonterminal candidate for `repository` (`owner/name`), if any.
    pub fn active_candidate(
        &self,
        repository: &str,
    ) -> Result<Option<CandidateRecord>, StateError> {
        self.verify_audit_chain()?;
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
                Ok(Some(self.load(&id)?))
            }
            None => Ok(None),
        }
    }

    /// Load one candidate after verifying stored digests and the full audit chain.
    pub fn load(&self, id: &CandidateId) -> Result<CandidateRecord, StateError> {
        self.verify_audit_chain()?;
        let record = load_record_in_tx(&self.conn, id)?;
        Ok(record)
    }

    /// Verify every stored audit event forms a valid hash-linked chain.
    pub fn verify_audit_chain(&self) -> Result<(), StateError> {
        let events = self.load_audit_events()?;
        verify_chain(&events)?;
        // Also ensure stored event_hash column matches JSON.
        for event in &events {
            let stored: String = self.conn.query_row(
                "SELECT event_hash FROM audit_events WHERE sequence = ?1",
                params![event.sequence as i64],
                |row| row.get(0),
            )?;
            if stored != event.event_hash {
                return Err(StateError::Integrity(format!(
                    "audit sequence {} event_hash column mismatch",
                    event.sequence
                )));
            }
        }
        Ok(())
    }

    /// Highest-sequence audit event, if the ledger is nonempty.
    pub fn audit_head(&self) -> Result<Option<AuditEvent>, StateError> {
        self.verify_audit_chain()?;
        let events = self.load_audit_events()?;
        Ok(events.into_iter().next_back())
    }

    /// Ordered audit events (validated individually when decoded).
    pub fn load_audit_events(&self) -> Result<Vec<AuditEvent>, StateError> {
        let mut stmt = self
            .conn
            .prepare("SELECT event_json FROM audit_events ORDER BY sequence ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut events = Vec::new();
        for row in rows {
            let json = row?;
            let event: AuditEvent = serde_json::from_str(&json).map_err(|err| {
                StateError::Integrity(format!("audit event json decode failed: {err}"))
            })?;
            // Confirm stored JSON is canonical for the event value.
            let canonical = canonicalize_json_value(&event)?;
            if canonical != json {
                return Err(StateError::Integrity(format!(
                    "audit sequence {} json is not canonical",
                    event.sequence
                )));
            }
            events.push(event);
        }
        Ok(events)
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

fn set_dir_mode_0700(path: &Path) -> Result<(), StateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|err| StateError::Io(err.to_string()))?
            .permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms).map_err(|err| StateError::Io(err.to_string()))?;
    }
    let _ = path;
    Ok(())
}

fn set_file_mode_0600(path: &Path) -> Result<(), StateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|err| StateError::Io(err.to_string()))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms).map_err(|err| StateError::Io(err.to_string()))?;
    }
    let _ = path;
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
        // Corrupt current row: pair missing one side.
        (Some(_), None, _, _) | (None, Some(_), _, _) => Err(StateError::Integrity(
            "stored receipt pair is incomplete".to_owned(),
        )),
    }
}

fn is_unique_violation(err: &rusqlite::Error) -> bool {
    match err {
        rusqlite::Error::SqliteFailure(info, _) => {
            info.code == rusqlite::ErrorCode::ConstraintViolation
        }
        _ => false,
    }
}

fn append_audit_event(
    tx: &rusqlite::Transaction<'_>,
    candidate_id: Option<CandidateId>,
    event_type: &str,
    payload: &impl Serialize,
    now: DateTime<Utc>,
) -> Result<AuditEvent, StateError> {
    let previous = latest_audit_event(tx)?;
    let event = AuditEvent::next_at(previous.as_ref(), event_type, candidate_id, payload, now)?;
    let event_json = canonicalize_json_value(&event)?;
    tx.execute(
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

fn load_record_in_tx(conn: &Connection, id: &CandidateId) -> Result<CandidateRecord, StateError> {
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
    materialize_record(row)
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
            let path = PathBuf::from(&text);
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
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 1, 7, 0, 0).unwrap()
    }

    fn sha(n: u8) -> String {
        format!("sha256:{}", format!("{n:x}").repeat(64)[..64].to_owned())
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
        assert!(store
            .create_candidate(
                &manifest("cand-20260901t080000z-two-bbbb2222"),
                &policy_digest(),
                now,
            )
            .is_err());
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

        // Illegal skip.
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
        // Illegal jump to accepted.
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
        // Terminal cannot leave.
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

        // Set-once violations.
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

        // Receipt pair required together.
        assert!(
            TransitionMetadata::empty()
                .with_receipt(receipt_json.clone(), "sha256:dead".to_owned())
                .validate_shape()
                .is_err()
                || store
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
                    .is_err()
        );

        // Mismatched digest rejected.
        assert!(store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty()
                    .with_receipt(receipt_json.clone(), format!("sha256:{}", "0".repeat(64)),),
                now,
            )
            .is_err());

        // Same values again are allowed (idempotent set-once).
        store
            .transition(
                &mid,
                CandidateState::Building,
                TransitionMetadata::empty()
                    .with_workspace(ws)
                    .with_candidate_digest(cand_digest)
                    .with_receipt(receipt_json, receipt_digest),
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

        // Reason forbidden on nonterminal.
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

        // Required on failed.
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

        // Overlong reason rejected.
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
    fn illegal_transition_rolls_back_without_audit_append() {
        let store = StateStore::open_in_memory().unwrap();
        let now = fixed_now();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), now)
            .unwrap();
        let before = store.load_audit_events().unwrap().len();
        assert!(store
            .transition(
                &mid,
                CandidateState::Evaluating,
                TransitionMetadata::empty(),
                now
            )
            .is_err());
        let after = store.load_audit_events().unwrap().len();
        assert_eq!(before, after);
        let record = store.load(&mid).unwrap();
        assert_eq!(record.state(), CandidateState::Observed);
        assert!(store.verify_audit_chain().is_ok());
    }

    #[test]
    fn concurrent_insert_race_enforced_by_partial_unique_index() {
        let dir = TempDir::new().unwrap();
        let state_dir = dir.path().join("coord");
        // Prime schema via first open.
        drop(StateStore::open(&state_dir).unwrap());

        let barrier = Arc::new(Barrier::new(2));
        let state_dir = Arc::new(state_dir);
        let results = Arc::new(std::sync::Mutex::new(Vec::new()));

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
                results.lock().unwrap().push((idx, outcome.is_ok()));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let outcomes = results.lock().unwrap();
        let oks = outcomes.iter().filter(|(_, ok)| *ok).count();
        let errs = outcomes.iter().filter(|(_, ok)| !*ok).count();
        assert_eq!(oks, 1, "exactly one insert must succeed: {outcomes:?}");
        assert_eq!(errs, 1, "exactly one insert must fail: {outcomes:?}");

        let store = StateStore::open(state_dir.as_path()).unwrap();
        let active = store.active_candidate(&repo_key()).unwrap().unwrap();
        assert_eq!(active.state(), CandidateState::Observed);
        assert!(store.verify_audit_chain().is_ok());
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
    fn audit_tamper_is_detected() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::open(dir.path()).unwrap();
        let mid = id("cand-20260901t070000z-one-aaaa1111");
        store
            .create_candidate(&manifest(mid.as_str()), &policy_digest(), fixed_now())
            .unwrap();
        // Flip a hex nibble inside stored event_json payload without fixing hash.
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
        drop(store);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = fs::metadata(&state_dir).unwrap().permissions().mode() & 0o777;
            assert_eq!(dir_mode, 0o700);
            let db_mode = fs::metadata(state_dir.join(STATE_DB_NAME))
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(db_mode, 0o600);
        }

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
        // Read-only must refuse mutation.
        assert!(matches!(
            ro.create_candidate(
                &manifest("cand-20260901t090000z-three-cccc3333"),
                &policy_digest(),
                fixed_now()
            ),
            Err(StateError::ReadOnly)
        ));
        drop(lock);
        // Lock released.
        let _lock2 = CoordinatorLock::try_acquire(&state_dir).unwrap();
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
