//! Opportunity mission refresh, immutable publication, and pure candidate binding.
//!
//! Stages producer artifacts under coordinator-owned paths, validates them strictly,
//! publishes an immutable generation + atomic `CURRENT` pointer, and converts a
//! validated mission into a policy-bound [`PreparedCandidate`] without Git I/O.

use crate::config::RepoEvolverConfig;
use crate::policy::TrustedPolicy;
use crate::process::{ProcessError, ProcessOutput, ProcessRunner, ProcessSpec};
use chrono::{DateTime, TimeZone, Utc};
use evolution_contracts::{
    canonical_json_bytes, sha256_hex, AuthorityTier, CandidateId, CandidateKind, CandidateManifest,
    CandidateTarget, CANDIDATE_SCHEMA,
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

/// Producer schema for the single active opportunity mission.
pub const NEXT_MISSION_SCHEMA: &str = "gzmo.opportunity.next_mission/v1";
/// Maximum accepted producer JSON size.
pub const MAX_MISSION_JSON_BYTES: usize = 64 * 1024;
/// Maximum accepted mission Markdown size.
pub const MAX_MISSION_MARKDOWN_BYTES: usize = 256 * 1024;
/// Maximum accepted `advice` / `automation_note` UTF-8 byte length.
pub const MAX_AUX_STRING_BYTES: usize = 4 * 1024;
/// Trusted refresh wall timeout.
pub const REFRESH_TIMEOUT_SECS: u64 = 300;
/// Trusted combined stdout+stderr capture ceiling.
pub const REFRESH_OUTPUT_CAP_BYTES: usize = 1024 * 1024;
/// Forward clock skew tolerated for `generated_at` after refresh end.
pub const GENERATED_AT_FORWARD_TOLERANCE_SECS: i64 = 5;
/// Fixed safe PATH for producer launches.
pub const SAFE_PATH: &str = "/usr/bin:/bin";
/// HOME directory name inside a staging root.
pub const STAGING_HOME_NAME: &str = "home";
/// Exclusive liveness lock filename inside a staging root.
pub const STAGING_LOCK_NAME: &str = "liveness.lock";
/// Maximum accepted bet_id UTF-8 bytes (bounded before any error echo).
pub const MAX_BET_ID_BYTES: usize = 128;
/// Staging roots older than 2× refresh timeout may be cleaned if unlocked.
pub const ABANDONED_STAGING_SECS: u64 = REFRESH_TIMEOUT_SECS * 2;
/// Staging parent under the coordinator state directory.
pub const MISSION_STAGING_DIR: &str = "mission-staging";
/// Published missions root under the coordinator state directory.
pub const MISSIONS_DIR: &str = "missions";
/// Immutable generation container under missions/.
pub const GENERATIONS_DIR: &str = "generations";
/// Atomic current-generation pointer filename.
pub const CURRENT_POINTER: &str = "CURRENT";
/// Published markdown filename inside a generation.
pub const GENERATION_MARKDOWN: &str = "mission.md";
/// Published sanitized JSON filename inside a generation.
pub const GENERATION_JSON: &str = "mission.json";
/// Maximum candidate id length (contracts bound).
pub const MAX_CANDIDATE_ID_BYTES: usize = 96;

/// Errors raised while refreshing, loading, or converting missions.
#[derive(Debug, Error)]
pub enum MissionError {
    /// Input failed structural or policy checks.
    #[error("invalid mission: {0}")]
    Invalid(String),
    /// Filesystem failure.
    #[error("mission io error: {0}")]
    Io(String),
    /// Producer process failure.
    #[error("mission process error: {0}")]
    Process(#[from] ProcessError),
    /// Contract validation failure while building a candidate.
    #[error("mission contract error: {0}")]
    Contract(String),
    /// Publication rename succeeded but a post-commit durability step failed.
    /// The new generation remains readable via CURRENT.
    #[error("mission publication durability failure: {0}")]
    Durability(String),
}

impl From<io::Error> for MissionError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<evolution_contracts::ContractError> for MissionError {
    fn from(value: evolution_contracts::ContractError) -> Self {
        Self::Contract(value.to_string())
    }
}

impl From<evolution_contracts::AuditError> for MissionError {
    fn from(value: evolution_contracts::AuditError) -> Self {
        Self::Invalid(format!("canonical json: {value}"))
    }
}

/// Deterministic clock seam around refresh start/end.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// Wall-clock UTC provider.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Test clock that returns a fixed instant (optionally advanced).
#[derive(Debug)]
pub struct ManualClock {
    current: std::sync::Mutex<DateTime<Utc>>,
}

impl ManualClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            current: std::sync::Mutex::new(now),
        }
    }

    pub fn set(&self, now: DateTime<Utc>) {
        *self.current.lock().expect("clock lock") = now;
    }

    pub fn advance_secs(&self, secs: i64) {
        let mut guard = self.current.lock().expect("clock lock");
        *guard = *guard + chrono::Duration::seconds(secs);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.current.lock().expect("clock lock")
    }
}

/// Producer wire payload. Auxiliary strings are length-checked then discarded.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct NextMissionV1 {
    schema: String,
    generated_at: DateTime<Utc>,
    ok: bool,
    bet_id: String,
    title: String,
    score: i64,
    ship_bar: bool,
    mission_md: PathBuf,
    advice: String,
    automation_note: String,
}

/// Canonical sanitized publication payload (no auxiliary authority fields).
#[derive(Debug, Clone, Serialize)]
struct PublishedMissionV1 {
    schema: String,
    generated_at: DateTime<Utc>,
    ok: bool,
    bet_id: String,
    title: String,
    score: i64,
    ship_bar: bool,
    mission_md: String,
    /// Algorithm-qualified `sha256:<hex>` over the published Markdown bytes.
    content_digest: String,
}

/// Validated active opportunity mission snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mission {
    pub schema: String,
    pub generated_at: DateTime<Utc>,
    pub ok: bool,
    pub bet_id: String,
    pub title: String,
    pub score: i64,
    pub ship_bar: bool,
    /// Absolute path of the published generation markdown file.
    pub mission_md: PathBuf,
    /// Markdown body (untrusted prompt content only).
    pub markdown: String,
    /// `sha256:<hex>` over the markdown bytes.
    pub content_digest: String,
    /// Generation directory basename currently bound to this mission.
    pub generation_id: String,
}

/// Policy-bound candidate ready for persistence by later tasks.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedCandidate {
    pub manifest: CandidateManifest,
    pub policy_digest: String,
}

/// Refresh + load coordinator for opportunity missions.
pub struct MissionAdapter<'a, R: ProcessRunner, C: Clock> {
    config: &'a RepoEvolverConfig,
    runner: &'a R,
    clock: &'a C,
    /// Test-only: force a post-rename durability error on this adapter instance.
    /// Production constructors leave this false; it is never a process-global.
    #[cfg(test)]
    force_post_rename_durability_error: bool,
}

impl<'a, R: ProcessRunner, C: Clock> MissionAdapter<'a, R, C> {
    pub fn new(config: &'a RepoEvolverConfig, runner: &'a R, clock: &'a C) -> Self {
        Self {
            config,
            runner,
            clock,
            #[cfg(test)]
            force_post_rename_durability_error: false,
        }
    }

    /// Test-only constructor that injects a post-rename durability failure.
    #[cfg(test)]
    fn with_forced_post_rename_durability_error(
        config: &'a RepoEvolverConfig,
        runner: &'a R,
        clock: &'a C,
    ) -> Self {
        Self {
            config,
            runner,
            clock,
            force_post_rename_durability_error: true,
        }
    }

    /// Run the producer, validate staged artifacts, publish immutably, load.
    pub fn refresh_and_load(&self) -> Result<Mission, MissionError> {
        let state_dir = self.config.state_dir();
        ensure_dir_0700(state_dir)?;
        let staging_root_parent = state_dir.join(MISSION_STAGING_DIR);
        ensure_dir_0700(&staging_root_parent)?;

        // Best-effort cleanup of truly abandoned peers before creating ours.
        let _ = cleanup_abandoned_staging(&staging_root_parent);

        let staging_id = Uuid::new_v4().to_string();
        let staging_root = staging_root_parent.join(&staging_id);
        ensure_dir_0700(&staging_root)?;
        let home_dir = staging_root.join(STAGING_HOME_NAME);
        ensure_dir_0700(&home_dir)?;

        // Hold exclusive liveness lock for the lifetime of this refresh.
        let _liveness = acquire_staging_liveness_lock(&staging_root)?;

        let refresh_start = self.clock.now();
        let run_result = self.invoke_producer(&staging_root, &home_dir);
        let refresh_end = self.clock.now();

        let cleanup_own_staging = |root: &Path| {
            let _ = remove_path_best_effort(root);
        };

        let process_output = match run_result {
            Ok(out) => out,
            Err(err) => {
                cleanup_own_staging(&staging_root);
                return Err(err);
            }
        };
        if process_output.status != 0 {
            cleanup_own_staging(&staging_root);
            return Err(MissionError::Process(ProcessError::NonZeroExit {
                code: process_output.status,
                stdout: process_output.stdout,
                stderr: process_output.stderr,
            }));
        }

        let publish = self.validate_staged_and_publish(&staging_root, refresh_start, refresh_end);

        // Always drop only this invocation's staging root (lock released on drop).
        cleanup_own_staging(&staging_root);

        // Durability errors after CURRENT rename are still failures for the CLI,
        // but the new pair remains loadable.
        publish
    }

    /// Load and revalidate the mission pair referenced by `CURRENT`.
    pub fn load_current(&self) -> Result<Mission, MissionError> {
        let missions = self.config.state_dir().join(MISSIONS_DIR);
        let current_path = missions.join(CURRENT_POINTER);
        if !current_path.exists() {
            return Err(MissionError::Invalid(
                "no published mission CURRENT pointer".to_owned(),
            ));
        }
        reject_symlink_path(&current_path)?;
        let basename = fs::read_to_string(&current_path)
            .map_err(|err| MissionError::Io(format!("read CURRENT: {err}")))?;
        let basename = basename.trim();
        validate_generation_basename(basename)?;
        let generation_dir = missions.join(GENERATIONS_DIR).join(basename);
        load_generation(&generation_dir, basename)
    }

    fn invoke_producer(
        &self,
        staging_root: &Path,
        home_dir: &Path,
    ) -> Result<ProcessOutput, MissionError> {
        let argv = self.config.mission().refresh_argv();
        if argv.len() < 2 {
            return Err(MissionError::Invalid(
                "refresh_argv must contain program and script".to_owned(),
            ));
        }
        let program = &argv[0];
        let args = argv[1..].to_vec();

        let mut env = BTreeMap::new();
        env.insert("PATH".to_owned(), SAFE_PATH.to_owned());
        env.insert(
            "HOME".to_owned(),
            home_dir
                .to_str()
                .ok_or_else(|| MissionError::Invalid("HOME path is not UTF-8".to_owned()))?
                .to_owned(),
        );
        env.insert(
            "GZMO_DATA_NEXT".to_owned(),
            staging_root
                .to_str()
                .ok_or_else(|| MissionError::Invalid("staging path is not UTF-8".to_owned()))?
                .to_owned(),
        );

        let spec = ProcessSpec::new(
            program,
            args,
            self.config.repo().path(),
            env,
            REFRESH_OUTPUT_CAP_BYTES,
            Duration::from_secs(REFRESH_TIMEOUT_SECS),
        )?;
        Ok(self.runner.run(&spec)?)
    }

    fn validate_staged_and_publish(
        &self,
        staging_root: &Path,
        refresh_start: DateTime<Utc>,
        refresh_end: DateTime<Utc>,
    ) -> Result<Mission, MissionError> {
        let json_rel = self.config.mission().json_rel();
        let md_rel = self.config.mission().markdown_rel();
        let json_path = safe_join_under(staging_root, json_rel)?;
        let md_path = safe_join_under(staging_root, md_rel)?;

        ensure_fresh_regular_file(
            &json_path,
            staging_root,
            refresh_start,
            MAX_MISSION_JSON_BYTES,
        )?;
        ensure_fresh_regular_file(
            &md_path,
            staging_root,
            refresh_start,
            MAX_MISSION_MARKDOWN_BYTES,
        )?;

        let json_bytes = fs::read(&json_path)?;
        let markdown_bytes = fs::read(&md_path)?;
        let markdown = std::str::from_utf8(&markdown_bytes)
            .map_err(|err| MissionError::Invalid(format!("markdown is not UTF-8: {err}")))?
            .to_owned();

        let payload = decode_next_mission(&json_bytes)?;
        validate_payload_fields(&payload)?;
        validate_generated_at(payload.generated_at, refresh_start, refresh_end)?;
        validate_mission_md_path(&payload.mission_md, &md_path)?;
        validate_markdown_sections(&markdown)?;

        // Publish immutable generation. Any failure must leave prior CURRENT.
        let missions_dir = self.config.state_dir().join(MISSIONS_DIR);
        ensure_dir_0700(&missions_dir)?;
        let generations_dir = missions_dir.join(GENERATIONS_DIR);
        ensure_dir_0700(&generations_dir)?;

        let generation_id = Uuid::new_v4().to_string();
        let generation_dir = generations_dir.join(&generation_id);
        if let Err(err) = ensure_dir_0700(&generation_dir) {
            let _ = remove_path_best_effort(&generation_dir);
            return Err(err);
        }

        let published_md = generation_dir.join(GENERATION_MARKDOWN);
        let published_json = generation_dir.join(GENERATION_JSON);
        let content_digest = format!("sha256:{}", sha256_hex(markdown.as_bytes()));

        // Pre-commit phase: any error deletes the unpublished generation.
        let precommit = (|| -> Result<(PublishedMissionV1, Mission), MissionError> {
            write_file_0600(&published_md, markdown.as_bytes())?;
            let published = PublishedMissionV1 {
                schema: payload.schema.clone(),
                generated_at: payload.generated_at,
                ok: true,
                bet_id: payload.bet_id.clone(),
                title: payload.title.clone(),
                score: payload.score,
                ship_bar: true,
                mission_md: published_md
                    .to_str()
                    .ok_or_else(|| {
                        MissionError::Invalid("published markdown path is not UTF-8".to_owned())
                    })?
                    .to_owned(),
                content_digest: content_digest.clone(),
            };
            let canonical = canonical_json_bytes(&published)?;
            write_file_0600(&published_json, &canonical)?;
            fsync_file(&published_md)?;
            fsync_file(&published_json)?;
            fsync_dir(&generation_dir)?;
            let mission = Mission {
                schema: payload.schema.clone(),
                generated_at: payload.generated_at,
                ok: true,
                bet_id: payload.bet_id.clone(),
                title: payload.title.clone(),
                score: payload.score,
                ship_bar: true,
                mission_md: published_md.clone(),
                markdown: markdown.clone(),
                content_digest: content_digest.clone(),
                generation_id: generation_id.clone(),
            };
            Ok((published, mission))
        })();

        let (_published, mission) = match precommit {
            Ok(v) => v,
            Err(err) => {
                let _ = remove_path_best_effort(&generation_dir);
                return Err(err);
            }
        };

        // Commit point: atomic CURRENT rename. After this succeeds the generation
        // must never be deleted by error paths.
        #[cfg(test)]
        let force_durability = self.force_post_rename_durability_error;
        #[cfg(not(test))]
        let force_durability = false;
        match write_current_pointer(&missions_dir, &generation_id, force_durability) {
            Ok(CurrentWriteOutcome::Committed) => Ok(mission),
            Ok(CurrentWriteOutcome::CommittedWithDurabilityError {
                generation_id,
                detail,
            }) => {
                // New pair is readable; surface durability failure so CLI is not success.
                Err(MissionError::Durability(format!(
                    "generation {generation_id}: CURRENT already advanced; durability not confirmed ({detail})"
                )))
            }
            Err(err) => {
                // Rename did not commit: clean unpublished generation, keep prior CURRENT.
                let _ = remove_path_best_effort(&generation_dir);
                Err(err)
            }
        }
    }
}

impl Mission {
    /// Pure conversion: mission + policy + injected baseline/time → prepared candidate.
    pub fn to_prepared_candidate(
        &self,
        config: &RepoEvolverConfig,
        policy: &TrustedPolicy,
        baseline_commit: &str,
        created_at: DateTime<Utc>,
    ) -> Result<PreparedCandidate, MissionError> {
        if !self.ok || !self.ship_bar {
            return Err(MissionError::Invalid(
                "mission must be ok with ship_bar=true".to_owned(),
            ));
        }
        validate_baseline_commit(baseline_commit)?;
        if policy.owner() != config.repo().owner()
            || policy.repository() != config.repo().repository()
        {
            return Err(MissionError::Invalid(
                "policy owner/repository must match config target".to_owned(),
            ));
        }

        let id = build_candidate_id(created_at, &self.bet_id, baseline_commit)?;
        let kind = policy.candidate_kind();
        let authority = kind.authority_tier();
        let candidate_branch = format!("evolve/{}", id.as_str());
        let target = CandidateTarget::Repository {
            owner: config.repo().owner().to_owned(),
            repository: config.repo().repository().to_owned(),
            base_branch: config.repo().base_branch().to_owned(),
            candidate_branch,
        };
        let required_gates: Vec<String> = policy
            .required_hard_floor_names()
            .into_iter()
            .map(str::to_owned)
            .collect();
        if required_gates.is_empty() {
            return Err(MissionError::Invalid(
                "policy must expose at least one hard_floor gate".to_owned(),
            ));
        }
        let protected_paths = policy.protected_paths().protected_paths.clone();
        let budget = policy.budget().clone();
        let baseline_digest = format!("git-sha1:{baseline_commit}");

        let manifest = CandidateManifest {
            schema: CANDIDATE_SCHEMA.to_owned(),
            id,
            mission_id: self.bet_id.clone(),
            kind,
            authority,
            target,
            baseline_digest,
            required_gates,
            protected_paths,
            budget,
            created_at,
        };
        manifest.validate()?;

        let policy_digest = policy
            .digest()
            .map_err(|err| MissionError::Invalid(format!("policy digest: {err}")))?;
        if !policy_digest.starts_with("sha256:") || policy_digest.len() != "sha256:".len() + 64 {
            return Err(MissionError::Invalid(format!(
                "policy digest must be sha256:<64 hex>, got {policy_digest}"
            )));
        }

        // AuthorityTier is fixed by kind; restate for clarity under deny-unknown contracts.
        let _authority_check: AuthorityTier = authority;
        let _kind_check: CandidateKind = kind;

        Ok(PreparedCandidate {
            manifest,
            policy_digest,
        })
    }
}

fn decode_next_mission(bytes: &[u8]) -> Result<NextMissionV1, MissionError> {
    if bytes.len() > MAX_MISSION_JSON_BYTES {
        return Err(MissionError::Invalid(format!(
            "mission json exceeds {MAX_MISSION_JSON_BYTES} bytes"
        )));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|err| MissionError::Invalid(format!("mission json is not UTF-8: {err}")))?;
    serde_json::from_str::<NextMissionV1>(text)
        .map_err(|err| MissionError::Invalid(format!("mission json decode failed: {err}")))
}

fn validate_payload_fields(payload: &NextMissionV1) -> Result<(), MissionError> {
    if payload.schema != NEXT_MISSION_SCHEMA {
        return Err(MissionError::Invalid(format!(
            "schema must be {NEXT_MISSION_SCHEMA}, got {:?}",
            payload.schema
        )));
    }
    if !payload.ok {
        return Err(MissionError::Invalid("mission payload ok=false".to_owned()));
    }
    if !payload.ship_bar {
        return Err(MissionError::Invalid(
            "mission payload ship_bar=false".to_owned(),
        ));
    }
    validate_safe_bet_id(&payload.bet_id)?;
    validate_safe_title(&payload.title)?;
    if payload.advice.len() > MAX_AUX_STRING_BYTES {
        return Err(MissionError::Invalid(format!(
            "advice exceeds {MAX_AUX_STRING_BYTES} bytes"
        )));
    }
    if payload.automation_note.len() > MAX_AUX_STRING_BYTES {
        return Err(MissionError::Invalid(format!(
            "automation_note exceeds {MAX_AUX_STRING_BYTES} bytes"
        )));
    }
    // advice / automation_note intentionally discarded after length checks.
    let _ = (&payload.advice, &payload.automation_note);
    Ok(())
}

fn validate_safe_bet_id(value: &str) -> Result<(), MissionError> {
    if value.is_empty() {
        return Err(MissionError::Invalid("bet_id must be nonempty".to_owned()));
    }
    if value.len() > MAX_BET_ID_BYTES {
        return Err(MissionError::Invalid(format!(
            "bet_id exceeds {MAX_BET_ID_BYTES} bytes"
        )));
    }
    if value != value.trim() {
        return Err(MissionError::Invalid(
            "bet_id must not have leading or trailing whitespace".to_owned(),
        ));
    }
    if value.contains("..") || value.contains('/') || value.contains('\\') {
        return Err(MissionError::Invalid(
            "bet_id must not contain path elements".to_owned(),
        ));
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        let preview: String = value.chars().take(64).collect();
        return Err(MissionError::Invalid(format!(
            "bet_id must be lowercase [a-z0-9-], got {preview:?}"
        )));
    }
    if value.starts_with('-') || value.ends_with('-') {
        return Err(MissionError::Invalid(
            "bet_id must not start or end with hyphen".to_owned(),
        ));
    }
    Ok(())
}

fn validate_safe_title(value: &str) -> Result<(), MissionError> {
    if value.is_empty() {
        return Err(MissionError::Invalid("title must be nonempty".to_owned()));
    }
    if value != value.trim() {
        return Err(MissionError::Invalid(
            "title must not have leading or trailing whitespace".to_owned(),
        ));
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(MissionError::Invalid(
            "title must not contain control characters".to_owned(),
        ));
    }
    if value.len() > 512 {
        return Err(MissionError::Invalid("title exceeds 512 bytes".to_owned()));
    }
    Ok(())
}

fn validate_generated_at(
    generated_at: DateTime<Utc>,
    refresh_start: DateTime<Utc>,
    refresh_end: DateTime<Utc>,
) -> Result<(), MissionError> {
    let latest = refresh_end + chrono::Duration::seconds(GENERATED_AT_FORWARD_TOLERANCE_SECS);
    if generated_at < refresh_start || generated_at > latest {
        return Err(MissionError::Invalid(format!(
            "generated_at {generated_at} outside refresh interval [{refresh_start}, {latest}]"
        )));
    }
    Ok(())
}

fn validate_mission_md_path(
    payload_path: &Path,
    expected_staged: &Path,
) -> Result<(), MissionError> {
    let expected = fs::canonicalize(expected_staged).map_err(|err| {
        MissionError::Io(format!(
            "canonicalize staged markdown {}: {err}",
            expected_staged.display()
        ))
    })?;
    // Producer may emit absolute or relative paths; resolve against filesystem.
    let actual = if payload_path.is_absolute() {
        fs::canonicalize(payload_path).map_err(|err| {
            MissionError::Invalid(format!(
                "mission_md path {} cannot be canonicalized: {err}",
                payload_path.display()
            ))
        })?
    } else {
        return Err(MissionError::Invalid(format!(
            "mission_md must be absolute, got {}",
            payload_path.display()
        )));
    };
    if actual != expected {
        return Err(MissionError::Invalid(format!(
            "mission_md path mismatch: payload {} != staged {}",
            actual.display(),
            expected.display()
        )));
    }
    Ok(())
}

fn validate_markdown_sections(markdown: &str) -> Result<(), MissionError> {
    for heading in ["## Mission", "## Constraints", "## Verify"] {
        if !section_nonempty(markdown, heading) {
            return Err(MissionError::Invalid(format!(
                "markdown missing nonempty {heading} section"
            )));
        }
    }
    Ok(())
}

fn section_nonempty(markdown: &str, heading: &str) -> bool {
    // Scan unfenced lines only; require the trimmed line to equal the heading
    // exactly (rejects `## Missionary` and fenced false positives).
    let mut in_fence = false;
    let mut found_at: Option<usize> = None;
    let mut offset = 0usize;
    for line in markdown.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();
        let trimmed = line.trim_end_matches(['\r', '\n']);
        let probe = trimmed.trim();
        if probe.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if probe == heading {
            found_at = Some(line_start + line.len());
            break;
        }
    }
    let Some(body_start) = found_at else {
        return false;
    };
    // Body runs until the next exact unfenced H2 or EOF.
    let rest = &markdown[body_start..];
    let mut body = String::new();
    let mut in_fence = false;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        let probe = trimmed.trim();
        if probe.starts_with("```") {
            in_fence = !in_fence;
            body.push_str(line);
            continue;
        }
        if !in_fence && probe.starts_with("## ") {
            break;
        }
        body.push_str(line);
    }
    body.chars().any(|c| !c.is_whitespace())
}

fn validate_baseline_commit(commit: &str) -> Result<(), MissionError> {
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(MissionError::Invalid(
            "baseline commit must be 40 lowercase hex characters".to_owned(),
        ));
    }
    Ok(())
}

fn build_candidate_id(
    created_at: DateTime<Utc>,
    bet_id: &str,
    baseline_commit: &str,
) -> Result<CandidateId, MissionError> {
    let ts = created_at
        .format("%Y%m%dt%H%M%Sz")
        .to_string()
        .to_ascii_lowercase();
    let hash8 = &baseline_commit[..8];
    let sanitized = sanitize_bet_for_id(bet_id);
    let prefix = format!("cand-{ts}-");
    let suffix = format!("-{hash8}");
    let budget = MAX_CANDIDATE_ID_BYTES
        .saturating_sub(prefix.len())
        .saturating_sub(suffix.len());
    if budget == 0 {
        return Err(MissionError::Invalid(
            "candidate id budget exhausted before bet portion".to_owned(),
        ));
    }
    let mut bet_part = sanitized;
    if bet_part.len() > budget {
        bet_part.truncate(budget);
        while bet_part.ends_with('-') {
            bet_part.pop();
        }
    }
    if bet_part.is_empty() {
        return Err(MissionError::Invalid(
            "sanitized bet_id empty after truncation".to_owned(),
        ));
    }
    // Avoid edge hyphens after truncation against suffix join.
    let id = format!("{prefix}{bet_part}{suffix}");
    CandidateId::parse(&id).map_err(|err| MissionError::Contract(err.to_string()))
}

fn sanitize_bet_for_id(bet_id: &str) -> String {
    let mut out = String::with_capacity(bet_id.len());
    for b in bet_id.bytes() {
        let c = match b {
            b'A'..=b'Z' => (b + 32) as char,
            b'a'..=b'z' | b'0'..=b'9' => b as char,
            b'-' | b'_' => '-',
            _ => continue,
        };
        if c == '-' && (out.is_empty() || out.ends_with('-')) {
            continue;
        }
        out.push(c);
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn load_generation(generation_dir: &Path, generation_id: &str) -> Result<Mission, MissionError> {
    reject_symlink_path(generation_dir)?;
    let md_path = generation_dir.join(GENERATION_MARKDOWN);
    let json_path = generation_dir.join(GENERATION_JSON);
    ensure_regular_file_bounded(&md_path, MAX_MISSION_MARKDOWN_BYTES)?;
    ensure_regular_file_bounded(&json_path, MAX_MISSION_JSON_BYTES)?;

    let markdown_bytes = fs::read(&md_path)?;
    let markdown = std::str::from_utf8(&markdown_bytes)
        .map_err(|err| MissionError::Invalid(format!("markdown is not UTF-8: {err}")))?
        .to_owned();
    validate_markdown_sections(&markdown)?;

    let json_bytes = fs::read(&json_path)?;
    let published: PublishedMissionV1 = serde_json::from_slice(&json_bytes).map_err(|err| {
        MissionError::Invalid(format!("published mission json decode failed: {err}"))
    })?;
    // Re-check canonical form.
    let canonical = canonical_json_bytes(&published)?;
    if canonical != json_bytes {
        return Err(MissionError::Invalid(
            "published mission json is not canonical".to_owned(),
        ));
    }
    if published.schema != NEXT_MISSION_SCHEMA {
        return Err(MissionError::Invalid(
            "published schema mismatch".to_owned(),
        ));
    }
    if !published.ok || !published.ship_bar {
        return Err(MissionError::Invalid(
            "published mission not shippable".to_owned(),
        ));
    }
    validate_safe_bet_id(&published.bet_id)?;
    validate_safe_title(&published.title)?;

    let actual_digest = format!("sha256:{}", sha256_hex(markdown.as_bytes()));
    if published.content_digest != actual_digest {
        return Err(MissionError::Invalid(
            "published content_digest does not match mission.md bytes".to_owned(),
        ));
    }
    if !published.content_digest.starts_with("sha256:")
        || published.content_digest.len() != "sha256:".len() + 64
    {
        return Err(MissionError::Invalid(
            "published content_digest must be sha256:<64 hex>".to_owned(),
        ));
    }

    let expected_md = md_path
        .to_str()
        .ok_or_else(|| MissionError::Invalid("markdown path not UTF-8".to_owned()))?;
    if published.mission_md != expected_md {
        // Also accept canonicalized equality.
        let published_path = PathBuf::from(&published.mission_md);
        let actual = fs::canonicalize(&published_path)
            .map_err(|err| MissionError::Invalid(format!("published mission_md invalid: {err}")))?;
        let expected = fs::canonicalize(&md_path)?;
        if actual != expected {
            return Err(MissionError::Invalid(
                "published mission_md does not match generation markdown".to_owned(),
            ));
        }
    }

    Ok(Mission {
        schema: published.schema,
        generated_at: published.generated_at,
        ok: true,
        bet_id: published.bet_id,
        title: published.title,
        score: published.score,
        ship_bar: true,
        mission_md: md_path,
        content_digest: published.content_digest,
        markdown,
        generation_id: generation_id.to_owned(),
    })
}

// PublishedMissionV1 needs Deserialize for load_current.
impl<'de> Deserialize<'de> for PublishedMissionV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            schema: String,
            generated_at: DateTime<Utc>,
            ok: bool,
            bet_id: String,
            title: String,
            score: i64,
            ship_bar: bool,
            mission_md: String,
            content_digest: String,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            schema: raw.schema,
            generated_at: raw.generated_at,
            ok: raw.ok,
            bet_id: raw.bet_id,
            title: raw.title,
            score: raw.score,
            ship_bar: raw.ship_bar,
            mission_md: raw.mission_md,
            content_digest: raw.content_digest,
        })
    }
}

/// Result of attempting to publish the CURRENT pointer.
enum CurrentWriteOutcome {
    /// Rename and post-rename durability steps all succeeded.
    Committed,
    /// Rename committed; a later durability step failed. Generation must be kept.
    CommittedWithDurabilityError {
        generation_id: String,
        detail: String,
    },
}

fn write_current_pointer(
    missions_dir: &Path,
    generation_id: &str,
    force_post_rename_durability_error: bool,
) -> Result<CurrentWriteOutcome, MissionError> {
    let tmp = missions_dir.join(format!("CURRENT.{}.tmp", Uuid::new_v4()));
    let final_path = missions_dir.join(CURRENT_POINTER);
    // Complete all fallible pre-rename work on the temp file first.
    write_file_0600(&tmp, generation_id.as_bytes())?;
    fsync_file(&tmp)?;
    set_file_mode_0600(&tmp)?;

    fs::rename(&tmp, &final_path).map_err(|err| {
        let _ = remove_path_best_effort(&tmp);
        MissionError::Io(format!("atomic CURRENT replace failed: {err}"))
    })?;

    // Post-commit: never delete the generation on failure here.
    // `force_post_rename_durability_error` is only ever true via a test-scoped
    // MissionAdapter field; production always passes false.
    if force_post_rename_durability_error {
        return Ok(CurrentWriteOutcome::CommittedWithDurabilityError {
            generation_id: generation_id.to_owned(),
            detail: "injected post-rename durability failure".to_owned(),
        });
    }

    if let Err(err) = set_file_mode_0600(&final_path) {
        return Ok(CurrentWriteOutcome::CommittedWithDurabilityError {
            generation_id: generation_id.to_owned(),
            detail: format!("chmod CURRENT after rename: {err}"),
        });
    }
    if let Err(err) = fsync_dir(missions_dir) {
        return Ok(CurrentWriteOutcome::CommittedWithDurabilityError {
            generation_id: generation_id.to_owned(),
            detail: format!("fsync missions dir after rename: {err}"),
        });
    }
    Ok(CurrentWriteOutcome::Committed)
}

fn ensure_fresh_regular_file(
    path: &Path,
    root: &Path,
    refresh_start: DateTime<Utc>,
    max_bytes: usize,
) -> Result<(), MissionError> {
    ensure_no_symlink_components(root, path)?;
    let meta = fs::symlink_metadata(path).map_err(|err| {
        MissionError::Invalid(format!("missing staged artifact {}: {err}", path.display()))
    })?;
    if meta.file_type().is_symlink() {
        return Err(MissionError::Invalid(format!(
            "staged artifact must not be a symlink: {}",
            path.display()
        )));
    }
    if !meta.file_type().is_file() {
        return Err(MissionError::Invalid(format!(
            "staged artifact must be a regular file: {}",
            path.display()
        )));
    }
    let len = meta.len() as usize;
    if len > max_bytes {
        return Err(MissionError::Invalid(format!(
            "staged artifact {} exceeds {max_bytes} bytes",
            path.display()
        )));
    }
    let modified = meta
        .modified()
        .map_err(|err| MissionError::Io(format!("mtime {}: {err}", path.display())))?;
    let modified_dt = system_time_to_utc(modified)?;
    // Allow equality with refresh_start; reject strictly older.
    if modified_dt < refresh_start - chrono::Duration::seconds(1) {
        // 1s slack for filesystem mtime second resolution vs chrono.
        return Err(MissionError::Invalid(format!(
            "staged artifact {} mtime {modified_dt} is older than refresh start {refresh_start}",
            path.display()
        )));
    }
    // Canonical containment: path resolves under root.
    let canon_file = fs::canonicalize(path)?;
    let canon_root = fs::canonicalize(root)?;
    if !path_is_within(&canon_file, &canon_root) {
        return Err(MissionError::Invalid(format!(
            "staged artifact {} escapes staging root",
            path.display()
        )));
    }
    Ok(())
}

fn ensure_regular_file_bounded(path: &Path, max_bytes: usize) -> Result<(), MissionError> {
    reject_symlink_path(path)?;
    let meta = fs::metadata(path)
        .map_err(|err| MissionError::Invalid(format!("missing file {}: {err}", path.display())))?;
    if !meta.is_file() {
        return Err(MissionError::Invalid(format!(
            "not a regular file: {}",
            path.display()
        )));
    }
    if meta.len() as usize > max_bytes {
        return Err(MissionError::Invalid(format!(
            "file {} exceeds {max_bytes} bytes",
            path.display()
        )));
    }
    Ok(())
}

fn ensure_no_symlink_components(root: &Path, path: &Path) -> Result<(), MissionError> {
    // Walk from root to path, rejecting any symlink component.
    let rel = path.strip_prefix(root).map_err(|_| {
        MissionError::Invalid(format!(
            "path {} is not under {}",
            path.display(),
            root.display()
        ))
    })?;
    let mut cursor = root.to_path_buf();
    reject_symlink_path(&cursor)?;
    for component in rel.components() {
        match component {
            Component::Normal(name) => {
                cursor.push(name);
                if cursor.exists() {
                    reject_symlink_path(&cursor)?;
                }
            }
            Component::CurDir => {}
            other => {
                return Err(MissionError::Invalid(format!(
                    "unsafe path component {other:?} in {}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

fn reject_symlink_path(path: &Path) -> Result<(), MissionError> {
    let meta = fs::symlink_metadata(path)
        .map_err(|err| MissionError::Io(format!("stat {}: {err}", path.display())))?;
    if meta.file_type().is_symlink() {
        return Err(MissionError::Invalid(format!(
            "path must not be a symlink: {}",
            path.display()
        )));
    }
    Ok(())
}

fn safe_join_under(root: &Path, rel: &Path) -> Result<PathBuf, MissionError> {
    if rel.is_absolute() {
        return Err(MissionError::Invalid(format!(
            "relative path expected, got {}",
            rel.display()
        )));
    }
    let mut out = root.to_path_buf();
    for component in rel.components() {
        match component {
            Component::Normal(name) => out.push(name),
            Component::CurDir => {}
            _ => {
                return Err(MissionError::Invalid(format!(
                    "relative path escapes: {}",
                    rel.display()
                )));
            }
        }
    }
    Ok(out)
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    let mut path_iter = path.components();
    for root_c in root.components() {
        match path_iter.next() {
            Some(c) if c == root_c => {}
            _ => return false,
        }
    }
    true
}

fn system_time_to_utc(time: SystemTime) -> Result<DateTime<Utc>, MissionError> {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|err| MissionError::Io(format!("mtime before epoch: {err}")))?;
    Utc.timestamp_opt(duration.as_secs() as i64, duration.subsec_nanos())
        .single()
        .ok_or_else(|| MissionError::Io("invalid mtime".to_owned()))
}

fn validate_generation_basename(name: &str) -> Result<(), MissionError> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(MissionError::Invalid(format!(
            "invalid CURRENT generation basename {name:?}"
        )));
    }
    if !name.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
        // UUID shape is hex + hyphens.
        return Err(MissionError::Invalid(format!(
            "invalid CURRENT generation basename {name:?}"
        )));
    }
    Ok(())
}

fn ensure_dir_0700(path: &Path) -> Result<(), MissionError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true).mode(0o700);
        match builder.create(path) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                let meta = fs::symlink_metadata(path)?;
                if meta.file_type().is_symlink() {
                    return Err(MissionError::Invalid(format!(
                        "expected directory, found symlink at {}",
                        path.display()
                    )));
                }
                if !meta.is_dir() {
                    return Err(MissionError::Invalid(format!(
                        "expected directory at {}",
                        path.display()
                    )));
                }
            }
            Err(err) => return Err(MissionError::Io(err.to_string())),
        }
        // Confirm directory before chmod so we never flip a file to 0700.
        let meta = fs::symlink_metadata(path)?;
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err(MissionError::Invalid(format!(
                "expected directory at {}",
                path.display()
            )));
        }
        set_dir_mode_0700(path)?;
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        if path.exists() && !path.is_dir() {
            return Err(MissionError::Invalid(format!(
                "expected directory at {}",
                path.display()
            )));
        }
        fs::create_dir_all(path)?;
        Ok(())
    }
}

fn set_dir_mode_0700(path: &Path) -> Result<(), MissionError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = fs::symlink_metadata(path)?;
        if meta.file_type().is_symlink() {
            return Err(MissionError::Invalid(format!(
                "directory must not be a symlink: {}",
                path.display()
            )));
        }
        let mut perms = meta.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms)?;
    }
    let _ = path;
    Ok(())
}

fn set_file_mode_0600(path: &Path) -> Result<(), MissionError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
    }
    let _ = path;
    Ok(())
}

fn write_file_0600(path: &Path, bytes: &[u8]) -> Result<(), MissionError> {
    if let Some(parent) = path.parent() {
        ensure_dir_0700(parent)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        set_file_mode_0600(path)?;
        return Ok(());
    }
    #[cfg(not(unix))]
    {
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }
}

fn fsync_file(path: &Path) -> Result<(), MissionError> {
    let file = OpenOptions::new().read(true).write(true).open(path)?;
    file.sync_all()?;
    Ok(())
}

fn fsync_dir(path: &Path) -> Result<(), MissionError> {
    #[cfg(unix)]
    {
        let file = File::open(path)?;
        file.sync_all()?;
    }
    let _ = path;
    Ok(())
}

fn remove_path_best_effort(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let meta = fs::symlink_metadata(path)?;
    if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

/// RAII exclusive liveness lock for a staging root.
struct StagingLivenessLock {
    _file: File,
}

fn acquire_staging_liveness_lock(staging_root: &Path) -> Result<StagingLivenessLock, MissionError> {
    let lock_path = staging_root.join(STAGING_LOCK_NAME);
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(&lock_path)
        .map_err(|err| MissionError::Io(format!("open staging liveness lock: {err}")))?;
    set_file_mode_0600(&lock_path)?;
    file.try_lock_exclusive().map_err(|err| {
        MissionError::Io(format!(
            "acquire staging liveness lock {}: {err}",
            lock_path.display()
        ))
    })?;
    Ok(StagingLivenessLock { _file: file })
}

fn try_acquire_staging_liveness_lock_for_cleanup(staging_root: &Path) -> io::Result<Option<File>> {
    let lock_path = staging_root.join(STAGING_LOCK_NAME);
    if !lock_path.exists() {
        // No lock file: treat as abandoned only if age check already passed.
        return Ok(None);
    }
    let file = OpenOptions::new()
        .create(false)
        .write(true)
        .read(true)
        .open(&lock_path)?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(file)),
        Err(err) if err.raw_os_error() == fs2::lock_contended_error().raw_os_error() => Ok(None),
        Err(err) => Err(err),
    }
}

fn cleanup_abandoned_staging(parent: &Path) -> io::Result<()> {
    if !parent.is_dir() {
        return Ok(());
    }
    let now = SystemTime::now();
    let min_age = Duration::from_secs(ABANDONED_STAGING_SECS);
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let path = entry.path();
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_dir() {
            continue;
        }
        let modified = match meta.modified() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let age = match now.duration_since(modified) {
            Ok(d) => d,
            Err(_) => continue,
        };
        if age < min_age {
            continue;
        }
        // Only remove if we can acquire the liveness lock (or none exists).
        match try_acquire_staging_liveness_lock_for_cleanup(&path) {
            Ok(Some(_guard)) => {
                let _ = remove_path_best_effort(&path);
            }
            Ok(None) if !path.join(STAGING_LOCK_NAME).exists() => {
                let _ = remove_path_best_effort(&path);
            }
            Ok(None) => {
                // Live concurrent refresh still holds the lock — leave it.
            }
            Err(_) => {}
        }
    }
    Ok(())
}

/// Parse a producer-shaped fixture JSON (and companion markdown) into a Mission
/// without going through process refresh. Used by unit tests and load helpers.
pub fn parse_fixture_mission(
    json_bytes: &[u8],
    markdown: &str,
    mission_md_path: PathBuf,
    generation_id: impl Into<String>,
) -> Result<Mission, MissionError> {
    let payload = decode_next_mission(json_bytes)?;
    validate_payload_fields(&payload)?;
    validate_markdown_sections(markdown)?;
    if markdown.len() > MAX_MISSION_MARKDOWN_BYTES {
        return Err(MissionError::Invalid(format!(
            "markdown exceeds {MAX_MISSION_MARKDOWN_BYTES} bytes"
        )));
    }
    Ok(Mission {
        schema: payload.schema,
        generated_at: payload.generated_at,
        ok: true,
        bet_id: payload.bet_id,
        title: payload.title,
        score: payload.score,
        ship_bar: true,
        mission_md: mission_md_path,
        content_digest: format!("sha256:{}", sha256_hex(markdown.as_bytes())),
        markdown: markdown.to_owned(),
        generation_id: generation_id.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RepoEvolverConfig;
    use crate::process::FakeProcessRunner;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Arc;
    use tempfile::TempDir;

    const POLICY_TOML: &str = r#"
schema = "gzmo.repo_evolver.policy/v1"
owner = "maximilianwruhs-cyber"
repository = "GZMO"
candidate_kind = "code"
max_active_candidates = 1
max_repair_attempts = 2
allowed_branch_prefix = "evolve/"

[budget]
wall_seconds = 2700
max_attempts = 1
max_changed_files = 20
max_added_lines = 1500
max_tool_calls = 80
max_input_tokens = 250000
max_output_tokens = 50000
allow_missing_energy_meter = true

[protected_paths]
protected_paths = [
  ".github/workflows/",
  "docs/superpowers/specs/",
  "docs/ADR-",
  "AGENTS.md",
  "Cargo.toml",
  "Cargo.lock",
  "crates/evolution-contracts/",
  "gzmo-evolver/",
  "config/repo-evolver.policy.toml",
]

[[gates]]
name = "format"
class = "hard_floor"
argv = ["cargo", "fmt", "--all", "--", "--check"]
timeout_seconds = 300

[[gates]]
name = "clippy"
class = "hard_floor"
argv = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
timeout_seconds = 900

[[gates]]
name = "tests"
class = "hard_floor"
argv = ["cargo", "test", "--all"]
timeout_seconds = 1800

[[gates]]
name = "opportunity-contract"
class = "hard_floor"
argv = ["bash", "scripts/opportunity-discovery-check.sh"]
timeout_seconds = 300
"#;

    const FIXTURE_MARKDOWN: &str = r#"# Mission card — Felt Use mass + MemRL-style utility (organism not warehouse)

## Mission

**Bet id:** `felt-use-mass-growth`
**Title:** Felt Use mass + MemRL-style utility (organism not warehouse)
**Score:** 23
**Why rare:** See research/opportunities/felt-use-mass-growth.md
**Brain profit:** axis brain_profit=5
**Done when:** Exit criteria in the bet file.

## Constraints

- USP: airgap living (ADR-0004); Brain Feed nutrients preferred
- One overnight writer (ADR-0003)
- No local-intel quests; no Socratic tourism; no public webserver SKU
- Finish-through: implement → verify → commit → push → PR → CI green → stop with PR URL or blocker

## Verify

```bash
bash scripts/opportunity-discovery-check.sh
bash scripts/brain-feed-check.sh
```

## Bet file

`research/opportunities/felt-use-mass-growth.md`
"#;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn load_fixture(name: &str) -> Result<Mission, MissionError> {
        let json_path = fixtures_dir().join(name);
        let json_bytes = fs::read(&json_path).map_err(|err| MissionError::Io(err.to_string()))?;
        let md_name = name.replace(".json", ".md");
        let md_path = fixtures_dir().join(md_name);
        let markdown =
            fs::read_to_string(&md_path).map_err(|err| MissionError::Io(err.to_string()))?;
        // For fixture-only acceptance, bind mission_md to the companion file path.
        parse_fixture_mission(&json_bytes, &markdown, md_path, "fixture-generation")
    }

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 9, 1, 12, 0, 0).unwrap()
    }

    struct Env {
        _root: TempDir,
        config_path: PathBuf,
        repo: PathBuf,
        state_dir: PathBuf,
        config: RepoEvolverConfig,
    }

    impl Env {
        fn new() -> Self {
            let root = TempDir::new().unwrap();
            let repo = root.path().join("repo");
            let state_dir = root.path().join("state");
            let worker = root.path().join("omp");
            fs::create_dir_all(repo.join("config")).unwrap();
            fs::create_dir_all(repo.join("scripts")).unwrap();
            File::create(repo.join("scripts/opportunity-next-mission.sh")).unwrap();
            fs::write(repo.join("config/repo-evolver.policy.toml"), POLICY_TOML).unwrap();
            let mut perms = fs::metadata(&worker).ok().map(|m| m.permissions());
            {
                use std::os::unix::fs::OpenOptionsExt;
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .mode(0o755)
                    .open(&worker)
                    .unwrap();
            }
            let _ = perms;
            let config_path = root.path().join("repo-evolver.toml");
            let body = format!(
                r#"
state_dir = "{state}"
[repo]
path = "{repo}"
remote = "origin"
base_branch = "main"
owner = "maximilianwruhs-cyber"
repository = "GZMO"
[mission]
json_rel = "opportunity-discovery/next-mission.json"
markdown_rel = "opportunity-discovery/next-mission.md"
refresh_argv = ["bash", "scripts/opportunity-next-mission.sh"]
[worker]
executable = "{worker}"
profile = "gzmo-repo-evolver-worker"
[policy]
repo_path = "config/repo-evolver.policy.toml"
"#,
                state = state_dir.display(),
                repo = repo.display(),
                worker = worker.display(),
            );
            fs::write(&config_path, body).unwrap();
            let config = RepoEvolverConfig::load(&config_path).unwrap();
            Self {
                _root: root,
                config_path,
                repo,
                state_dir,
                config,
            }
        }

        fn policy(&self) -> &TrustedPolicy {
            self.config.working_policy()
        }
    }

    fn fixture_json_for_path(md_abs: &Path, generated_at: DateTime<Utc>) -> String {
        serde_json::json!({
            "schema": NEXT_MISSION_SCHEMA,
            "generated_at": generated_at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "ok": true,
            "bet_id": "felt-use-mass-growth",
            "title": "Felt Use mass + MemRL-style utility (organism not warehouse)",
            "score": 23,
            "ship_bar": true,
            "mission_md": md_abs.to_str().unwrap(),
            "advice": "opportunity_next_mission_ok — paste next-mission.md into a new agent (or cron opens Cloud Agent with this file)",
            "automation_note": "Cursor Automations cannot start from bet status alone until a cron/PR trigger runs an agent with this mission card. PR babysitter only runs after a PR exists."
        })
        .to_string()
    }

    fn install_fake_success(
        fake: &FakeProcessRunner,
        staging_writer: impl Fn(&Path, DateTime<Utc>) + Send + Sync + 'static,
    ) {
        let writer = Arc::new(staging_writer);
        fake.set_handler(move |spec| {
            let data_next = spec
                .env
                .get("GZMO_DATA_NEXT")
                .expect("GZMO_DATA_NEXT")
                .clone();
            let root = PathBuf::from(data_next);
            // generated_at chosen by writer via side channel — use Utc::now for OS path;
            // tests pass a closure that knows the clock.
            writer(&root, Utc::now());
            Ok(ProcessOutput {
                status: 0,
                stdout: b"{}\n".to_vec(),
                stderr: Vec::new(),
            })
        });
    }

    #[test]
    fn accepts_only_one_active_ship_bar_mission() {
        let mission = load_fixture("next-mission.json").unwrap();
        assert_eq!(mission.schema, "gzmo.opportunity.next_mission/v1");
        assert!(mission.ok && mission.ship_bar);
        assert_eq!(mission.bet_id, "felt-use-mass-growth");
    }

    #[test]
    fn conversion_binds_injected_baseline_policy_and_target() {
        let env = Env::new();
        let mission = load_fixture("next-mission.json").unwrap();
        let prepared = mission
            .to_prepared_candidate(
                &env.config,
                env.policy(),
                "0123456789012345678901234567890123456789",
                fixed_now(),
            )
            .unwrap();
        assert_eq!(
            prepared.manifest.baseline_digest,
            "git-sha1:0123456789012345678901234567890123456789"
        );
        assert_eq!(prepared.policy_digest, env.policy().digest().unwrap());
        let gates: Vec<String> = env
            .policy()
            .required_hard_floor_names()
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert_eq!(prepared.manifest.required_gates, gates);
        assert_eq!(prepared.manifest.kind, CandidateKind::Code);
        assert_eq!(prepared.manifest.authority, AuthorityTier::Candidate);
        assert_eq!(prepared.manifest.mission_id, "felt-use-mass-growth");
        match &prepared.manifest.target {
            CandidateTarget::Repository {
                owner,
                repository,
                base_branch,
                candidate_branch,
            } => {
                assert_eq!(owner, "maximilianwruhs-cyber");
                assert_eq!(repository, "GZMO");
                assert_eq!(base_branch, "main");
                assert_eq!(
                    candidate_branch,
                    &format!("evolve/{}", prepared.manifest.id.as_str())
                );
            }
            other => panic!("expected repository target, got {other:?}"),
        }
        assert!(prepared
            .manifest
            .id
            .as_str()
            .starts_with("cand-20260901t120000z-"));
        assert!(prepared.manifest.id.as_str().ends_with("-01234567"));
    }

    #[test]
    fn truncates_only_bet_portion_when_id_exceeds_96() {
        let env = Env::new();
        let mut mission = load_fixture("next-mission.json").unwrap();
        // 80-char bet forces truncation while preserving ts + hash suffix.
        mission.bet_id = format!("bet-{}", "a".repeat(80));
        // bet_id validation on conversion uses mission.bet_id as mission_id token —
        // contracts allow broader tokens; ensure sanitize works.
        // Bypass bet_id field rules by calling build directly:
        let id = build_candidate_id(
            fixed_now(),
            &format!("bet-{}", "x".repeat(80)),
            "0123456789012345678901234567890123456789",
        )
        .unwrap();
        assert!(id.as_str().len() <= 96, "len {}", id.as_str().len());
        assert!(id.as_str().starts_with("cand-20260901t120000z-"));
        assert!(id.as_str().ends_with("-01234567"));
        // Still a valid id.
        CandidateId::parse(id.as_str()).unwrap();
        let _ = env;
        let _ = mission;
    }

    #[test]
    fn refresh_publishes_immutable_generation_and_current() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();
        let clock_for_writer = ManualClock::new(fixed_now());
        fake.set_handler(move |spec| {
            assert_eq!(spec.program, PathBuf::from("bash"));
            assert_eq!(
                spec.args,
                vec!["scripts/opportunity-next-mission.sh".to_owned()]
            );
            assert_eq!(spec.env.get("PATH").map(String::as_str), Some(SAFE_PATH));
            assert!(spec.env.get("HOME").is_some());
            assert!(spec.env.get("GZMO_DATA_NEXT").is_some());
            assert_eq!(spec.env.len(), 3, "only PATH/HOME/GZMO_DATA_NEXT allowed");
            let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
            let home = PathBuf::from(spec.env.get("HOME").unwrap());
            assert_eq!(
                home,
                root.join(STAGING_HOME_NAME),
                "HOME must live inside the staging root"
            );
            assert!(
                root.join(STAGING_LOCK_NAME).exists(),
                "liveness lock must exist during producer run"
            );
            assert_eq!(spec.output_cap, REFRESH_OUTPUT_CAP_BYTES);
            assert_eq!(spec.timeout, Duration::from_secs(REFRESH_TIMEOUT_SECS));

            let out = root.join("opportunity-discovery");
            fs::create_dir_all(&out).unwrap();
            let md_path = out.join("next-mission.md");
            let json_path = out.join("next-mission.json");
            fs::write(&md_path, &md_body).unwrap();
            let generated_at = clock_for_writer.now();
            fs::write(&json_path, fixture_json_for_path(&md_path, generated_at)).unwrap();
            Ok(ProcessOutput {
                status: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        });

        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let mission = adapter.refresh_and_load().unwrap();
        assert_eq!(mission.bet_id, "felt-use-mass-growth");
        assert!(mission.ok && mission.ship_bar);
        assert!(mission.mission_md.ends_with(GENERATION_MARKDOWN));
        assert!(mission.content_digest.starts_with("sha256:"));

        let current = env.state_dir.join(MISSIONS_DIR).join(CURRENT_POINTER);
        assert!(current.exists());
        #[cfg(unix)]
        {
            let mode = fs::metadata(&current).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
            let gen_dir = env
                .state_dir
                .join(MISSIONS_DIR)
                .join(GENERATIONS_DIR)
                .join(&mission.generation_id);
            let gmode = fs::metadata(&gen_dir).unwrap().permissions().mode() & 0o777;
            assert_eq!(gmode, 0o700);
            let md_mode = fs::metadata(&mission.mission_md)
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(md_mode, 0o600);
        }

        let loaded = adapter.load_current().unwrap();
        assert_eq!(loaded.bet_id, mission.bet_id);
        assert_eq!(loaded.content_digest, mission.content_digest);
        assert_eq!(loaded.generation_id, mission.generation_id);

        // Staging cleaned up.
        let staging = env.state_dir.join(MISSION_STAGING_DIR);
        if staging.exists() {
            let leftovers: Vec<_> = fs::read_dir(&staging)
                .unwrap()
                .filter_map(|e| e.ok())
                .collect();
            assert!(
                leftovers.is_empty(),
                "staging should be empty after success, got {:?}",
                leftovers.iter().map(|e| e.path()).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn failed_publication_preserves_prior_current() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();

        // First successful publish.
        fake.set_handler({
            let md_body = md_body.clone();
            let clock = ManualClock::new(fixed_now());
            move |spec| {
                let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
                let out = root.join("opportunity-discovery");
                fs::create_dir_all(&out).unwrap();
                let md_path = out.join("next-mission.md");
                fs::write(&md_path, &md_body).unwrap();
                fs::write(
                    out.join("next-mission.json"),
                    fixture_json_for_path(&md_path, clock.now()),
                )
                .unwrap();
                Ok(ProcessOutput {
                    status: 0,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
        });
        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let first = adapter.refresh_and_load().unwrap();
        let prior_current =
            fs::read_to_string(env.state_dir.join(MISSIONS_DIR).join(CURRENT_POINTER)).unwrap();

        // Second refresh: produce valid staging but sabotage generations parent mid-flight
        // by making generations a file so publish fails after staging succeeds.
        fake.set_handler({
            let md_body = md_body.clone();
            let clock = ManualClock::new(fixed_now());
            let state_dir = env.state_dir.clone();
            move |spec| {
                // Replace generations dir with a file to force publish failure.
                let generations = state_dir.join(MISSIONS_DIR).join(GENERATIONS_DIR);
                let _ = fs::remove_dir_all(&generations);
                fs::write(&generations, b"blocked").unwrap();

                let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
                let out = root.join("opportunity-discovery");
                fs::create_dir_all(&out).unwrap();
                let md_path = out.join("next-mission.md");
                fs::write(&md_path, &md_body).unwrap();
                fs::write(
                    out.join("next-mission.json"),
                    fixture_json_for_path(&md_path, clock.now()),
                )
                .unwrap();
                Ok(ProcessOutput {
                    status: 0,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
        });
        let err = adapter.refresh_and_load().unwrap_err();
        assert!(
            matches!(err, MissionError::Io(_) | MissionError::Invalid(_)),
            "{err}"
        );
        let current_after =
            fs::read_to_string(env.state_dir.join(MISSIONS_DIR).join(CURRENT_POINTER)).unwrap();
        assert_eq!(current_after, prior_current);
        assert_eq!(current_after.trim(), first.generation_id);
    }

    #[test]
    fn rejects_stale_generated_at() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();
        fake.set_handler(move |spec| {
            let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
            let out = root.join("opportunity-discovery");
            fs::create_dir_all(&out).unwrap();
            let md_path = out.join("next-mission.md");
            fs::write(&md_path, &md_body).unwrap();
            let stale = fixed_now() - chrono::Duration::hours(2);
            fs::write(
                out.join("next-mission.json"),
                fixture_json_for_path(&md_path, stale),
            )
            .unwrap();
            Ok(ProcessOutput {
                status: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        });
        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let err = adapter.refresh_and_load().unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(msg) if msg.contains("generated_at")),
            "{err}"
        );
    }

    #[test]
    fn rejects_stale_mtime() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();
        fake.set_handler(move |spec| {
            let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
            let out = root.join("opportunity-discovery");
            fs::create_dir_all(&out).unwrap();
            let md_path = out.join("next-mission.md");
            let json_path = out.join("next-mission.json");
            fs::write(&md_path, &md_body).unwrap();
            fs::write(&json_path, fixture_json_for_path(&md_path, fixed_now())).unwrap();
            // Force mtime to epoch.
            std::process::Command::new("touch")
                .args(["-d", "@0"])
                .arg(&md_path)
                .arg(&json_path)
                .status()
                .unwrap();
            Ok(ProcessOutput {
                status: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        });
        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let err = adapter.refresh_and_load().unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(msg) if msg.contains("mtime")),
            "{err}"
        );
    }

    #[test]
    fn rejects_symlink_ancestors() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();
        fake.set_handler(move |spec| {
            let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
            let real = root.join("real-out");
            fs::create_dir_all(&real).unwrap();
            let link = root.join("opportunity-discovery");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            let md_path = link.join("next-mission.md");
            fs::write(&md_path, &md_body).unwrap();
            fs::write(
                link.join("next-mission.json"),
                fixture_json_for_path(&md_path, fixed_now()),
            )
            .unwrap();
            Ok(ProcessOutput {
                status: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        });
        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let err = adapter.refresh_and_load().unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(msg) if msg.to_lowercase().contains("symlink")),
            "{err}"
        );
    }

    #[test]
    fn rejects_payload_path_mismatch() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();
        fake.set_handler(move |spec| {
            let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
            let out = root.join("opportunity-discovery");
            fs::create_dir_all(&out).unwrap();
            let md_path = out.join("next-mission.md");
            fs::write(&md_path, &md_body).unwrap();
            let mut json: serde_json::Value =
                serde_json::from_str(&fixture_json_for_path(&md_path, fixed_now())).unwrap();
            json["mission_md"] = serde_json::json!("/tmp/not-the-staged-file.md");
            fs::write(out.join("next-mission.json"), json.to_string()).unwrap();
            Ok(ProcessOutput {
                status: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        });
        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let err = adapter.refresh_and_load().unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(msg) if msg.contains("mission_md")),
            "{err}"
        );
    }

    #[test]
    fn rejects_missing_sections_and_unsafe_ids() {
        let env = Env::new();
        let bad_md = "# nope\n\n## Mission\n\nonly one section\n";
        let err = parse_fixture_mission(
            fixture_json_for_path(Path::new("/tmp/x.md"), fixed_now()).as_bytes(),
            bad_md,
            PathBuf::from("/tmp/x.md"),
            "g",
        )
        .unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("section")),
            "{err}"
        );

        let mut json: serde_json::Value =
            serde_json::from_str(&fixture_json_for_path(Path::new("/tmp/x.md"), fixed_now()))
                .unwrap();
        json["bet_id"] = serde_json::json!("../etc/passwd");
        let err = parse_fixture_mission(
            json.to_string().as_bytes(),
            FIXTURE_MARKDOWN,
            PathBuf::from("/tmp/x.md"),
            "g",
        )
        .unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("bet_id")),
            "{err}"
        );

        json["bet_id"] = serde_json::json!("felt-use-mass-growth");
        json["ship_bar"] = serde_json::json!(false);
        let err = parse_fixture_mission(
            json.to_string().as_bytes(),
            FIXTURE_MARKDOWN,
            PathBuf::from("/tmp/x.md"),
            "g",
        )
        .unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("ship_bar")),
            "{err}"
        );

        json["ship_bar"] = serde_json::json!(true);
        json["ok"] = serde_json::json!(false);
        // ok=false still requires all fields present for deny_unknown decode of success shape;
        // producer failure payload omits fields — decoder rejects.
        let err = parse_fixture_mission(
            json.to_string().as_bytes(),
            FIXTURE_MARKDOWN,
            PathBuf::from("/tmp/x.md"),
            "g",
        )
        .unwrap_err();
        assert!(matches!(&err, MissionError::Invalid(_)), "{err}");
        let _ = env;
    }

    #[test]
    fn rejects_oversized_json_and_aux_strings() {
        let big_advice = "a".repeat(MAX_AUX_STRING_BYTES + 1);
        let mut json: serde_json::Value =
            serde_json::from_str(&fixture_json_for_path(Path::new("/tmp/x.md"), fixed_now()))
                .unwrap();
        json["advice"] = serde_json::json!(big_advice);
        let err = decode_next_mission(json.to_string().as_bytes())
            .and_then(|p| validate_payload_fields(&p))
            .unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("advice")),
            "{err}"
        );

        let huge = format!("{{\"schema\":\"x\"{}}}", " ".repeat(MAX_MISSION_JSON_BYTES));
        let err = decode_next_mission(huge.as_bytes()).unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("exceeds")),
            "{err}"
        );
    }

    #[test]
    fn rejects_nonzero_producer_exit() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        fake.set_handler(|_| {
            Err(ProcessError::NonZeroExit {
                code: 1,
                stdout: b"nope".to_vec(),
                stderr: Vec::new(),
            })
        });
        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let err = adapter.refresh_and_load().unwrap_err();
        assert!(matches!(
            err,
            MissionError::Process(ProcessError::NonZeroExit { code: 1, .. })
        ));
    }

    #[test]
    fn rejects_unknown_producer_fields() {
        let mut json: serde_json::Value =
            serde_json::from_str(&fixture_json_for_path(Path::new("/tmp/x.md"), fixed_now()))
                .unwrap();
        json["extra_authority"] = serde_json::json!("nope");
        let err = decode_next_mission(json.to_string().as_bytes()).unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("decode")),
            "{err}"
        );
    }

    #[test]
    fn producer_fixture_fields_round_trip_from_disk() {
        // Ensure the checked-in fixture mirrors the live producer success shape.
        let raw = fs::read_to_string(fixtures_dir().join("next-mission.json")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        for key in [
            "schema",
            "generated_at",
            "ok",
            "bet_id",
            "title",
            "score",
            "ship_bar",
            "mission_md",
            "advice",
            "automation_note",
        ] {
            assert!(value.get(key).is_some(), "missing producer field {key}");
        }
        assert_eq!(value["schema"], NEXT_MISSION_SCHEMA);
        assert_eq!(value["ok"], true);
        assert_eq!(value["ship_bar"], true);
        assert_eq!(value["bet_id"], "felt-use-mass-growth");
        assert!(value["advice"]
            .as_str()
            .unwrap()
            .contains("opportunity_next_mission_ok"));
        assert!(value["automation_note"]
            .as_str()
            .unwrap()
            .contains("Cursor Automations"));
    }

    #[test]
    fn ensure_dir_0700_rejects_file_without_changing_mode() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("not-a-dir");
        fs::write(&file_path, b"payload").unwrap();
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&file_path, perms).unwrap();
            let before = fs::metadata(&file_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(before, 0o644);
            let err = ensure_dir_0700(&file_path).unwrap_err();
            assert!(
                matches!(&err, MissionError::Invalid(m) if m.contains("expected directory")),
                "{err}"
            );
            let after = fs::metadata(&file_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(after, 0o644, "file mode must remain unchanged");
            let body = fs::read(&file_path).unwrap();
            assert_eq!(body, b"payload");
        }
    }

    #[test]
    fn exact_unfenced_headings_reject_missionary_and_fenced_false_positives() {
        assert!(section_nonempty(FIXTURE_MARKDOWN, "## Mission"));
        assert!(section_nonempty(FIXTURE_MARKDOWN, "## Constraints"));
        assert!(section_nonempty(FIXTURE_MARKDOWN, "## Verify"));

        let missionary =
            "# card\n\n## Missionary\n\nbody\n\n## Constraints\n\nx\n\n## Verify\n\ny\n";
        assert!(!section_nonempty(missionary, "## Mission"));

        let fenced =
            "# card\n\n```text\n## Mission\n```\n\n## Constraints\n\nx\n\n## Verify\n\ny\n";
        assert!(!section_nonempty(fenced, "## Mission"));

        let err = parse_fixture_mission(
            fixture_json_for_path(Path::new("/tmp/x.md"), fixed_now()).as_bytes(),
            missionary,
            PathBuf::from("/tmp/x.md"),
            "g",
        )
        .unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("## Mission")),
            "{err}"
        );
    }

    #[test]
    fn rejects_oversized_bet_id_without_echoing_full_value() {
        let big = "a".repeat(MAX_BET_ID_BYTES + 8);
        let mut json: serde_json::Value =
            serde_json::from_str(&fixture_json_for_path(Path::new("/tmp/x.md"), fixed_now()))
                .unwrap();
        json["bet_id"] = serde_json::json!(big.clone());
        let err = decode_next_mission(json.to_string().as_bytes())
            .and_then(|p| validate_payload_fields(&p))
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("bet_id exceeds"), "{msg}");
        assert!(!msg.contains(&big), "must not echo full oversized bet_id");
    }

    #[test]
    fn load_rejects_tampered_published_markdown_digest() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();
        fake.set_handler({
            let md_body = md_body.clone();
            let clock = ManualClock::new(fixed_now());
            move |spec| {
                let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
                let out = root.join("opportunity-discovery");
                fs::create_dir_all(&out).unwrap();
                let md_path = out.join("next-mission.md");
                fs::write(&md_path, &md_body).unwrap();
                fs::write(
                    out.join("next-mission.json"),
                    fixture_json_for_path(&md_path, clock.now()),
                )
                .unwrap();
                Ok(ProcessOutput {
                    status: 0,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
        });
        let adapter = MissionAdapter::new(&env.config, &fake, &clock);
        let mission = adapter.refresh_and_load().unwrap();
        // Tamper markdown bytes inside the immutable generation.
        fs::write(
            &mission.mission_md,
            b"# tampered\n\n## Mission\n\nx\n\n## Constraints\n\ny\n\n## Verify\n\nz\n",
        )
        .unwrap();
        let err = adapter.load_current().unwrap_err();
        assert!(
            matches!(&err, MissionError::Invalid(m) if m.contains("content_digest")),
            "{err}"
        );
    }

    #[test]
    fn post_commit_durability_failure_retains_readable_generation() {
        let env = Env::new();
        let clock = ManualClock::new(fixed_now());
        let fake = FakeProcessRunner::new();
        let md_body = FIXTURE_MARKDOWN.to_owned();
        fake.set_handler({
            let md_body = md_body.clone();
            let clock = ManualClock::new(fixed_now());
            move |spec| {
                let root = PathBuf::from(spec.env.get("GZMO_DATA_NEXT").unwrap());
                let out = root.join("opportunity-discovery");
                fs::create_dir_all(&out).unwrap();
                let md_path = out.join("next-mission.md");
                fs::write(&md_path, &md_body).unwrap();
                fs::write(
                    out.join("next-mission.json"),
                    fixture_json_for_path(&md_path, clock.now()),
                )
                .unwrap();
                Ok(ProcessOutput {
                    status: 0,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
        });

        let adapter =
            MissionAdapter::with_forced_post_rename_durability_error(&env.config, &fake, &clock);
        let err = adapter.refresh_and_load().unwrap_err();
        let msg = err.to_string();
        assert!(
            matches!(&err, MissionError::Durability(_)),
            "CLI must not see success after post-commit failure: {err}"
        );
        assert!(
            msg.contains("CURRENT already advanced; durability not confirmed"),
            "operator text must state CURRENT advanced: {msg}"
        );
        assert!(
            msg.contains("injected post-rename durability failure"),
            "{msg}"
        );
        // Bound the message (no huge dump).
        assert!(
            msg.len() < 512,
            "durability message must stay bounded: {msg}"
        );

        // CURRENT points at a readable pair despite the durability error.
        let loaded = adapter.load_current().unwrap();
        assert_eq!(loaded.bet_id, "felt-use-mass-growth");
        assert!(loaded.mission_md.exists());
        assert!(
            msg.contains(&loaded.generation_id),
            "durability error must name committed generation id: {msg}"
        );
        let gen_dir = env
            .state_dir
            .join(MISSIONS_DIR)
            .join(GENERATIONS_DIR)
            .join(&loaded.generation_id);
        assert!(
            gen_dir.is_dir(),
            "generation must not be deleted post-commit"
        );
    }

    #[test]
    fn abandoned_staging_cleanup_preserves_live_locked_roots() {
        let env = Env::new();
        let staging_parent = env.state_dir.join(MISSION_STAGING_DIR);
        ensure_dir_0700(&staging_parent).unwrap();

        // Old unlocked root (> 2× timeout) should be removed.
        let old_root = staging_parent.join("11111111-1111-1111-1111-111111111111");
        ensure_dir_0700(&old_root).unwrap();
        fs::write(old_root.join("marker"), b"old").unwrap();
        // Age the directory.
        std::process::Command::new("touch")
            .args(["-d", "1970-01-01T00:00:00Z"])
            .arg(&old_root)
            .status()
            .unwrap();

        // Live locked root must survive even if aged.
        let live_root = staging_parent.join("22222222-2222-2222-2222-222222222222");
        ensure_dir_0700(&live_root).unwrap();
        let live_lock = acquire_staging_liveness_lock(&live_root).unwrap();
        std::process::Command::new("touch")
            .args(["-d", "1970-01-01T00:00:00Z"])
            .arg(&live_root)
            .status()
            .unwrap();

        cleanup_abandoned_staging(&staging_parent).unwrap();

        assert!(!old_root.exists(), "old unlocked root must be cleaned");
        assert!(live_root.exists(), "live locked root must be preserved");
        drop(live_lock);
    }
}
