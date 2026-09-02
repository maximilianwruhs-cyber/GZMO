//! Coordinator-owned bare mirror and independent no-push candidate workspaces.
//!
//! Every Git invocation is argv-only through [`ProcessRunner`], with a cleared
//! environment and fixed safe variables. Network remotes must identify the
//! configured GitHub owner/repository without credentials; local/file URLs are
//! accepted only for hermetic tests. Candidate clones never share objects with
//! the trusted checkout or mirror and never retain push/fetch authority.

use crate::config::RepoEvolverConfig;
use crate::policy::TrustedPolicy;
use crate::process::{ProcessError, ProcessOutput, ProcessRunner, ProcessSpec};
use crate::state::CandidateRecord;
use chrono::{DateTime, Utc};
use evolution_contracts::{CandidateManifest, CandidateState, PathPolicy};
use fs2::FileExt;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use url::Url;

/// Fixed safe PATH for Git launches.
pub const SAFE_PATH: &str = "/usr/bin:/bin";
/// Coordinator-owned HOME directory name under the state root.
pub const GIT_HOME_NAME: &str = "git-home";
/// Bare mirror directory name under the state root.
pub const MIRROR_NAME: &str = "mirror.git";
/// Staging name for atomic mirror creation/refresh replacement.
pub const MIRROR_STAGING_NAME: &str = "mirror.git.staging";
/// Exclusive mirror lease filename under the state root.
pub const MIRROR_LOCK_NAME: &str = "mirror.lock";
/// Independent workspace parent under the state root.
pub const WORKSPACES_DIR: &str = "workspaces";
/// Wall-clock timeout for trusted Git commands.
pub const GIT_TIMEOUT_SECS: u64 = 300;
/// Combined stdout+stderr capture ceiling for normal Git commands.
pub const GIT_OUTPUT_CAP_BYTES: usize = 16 * 1024 * 1024;
/// Capture ceiling for bounded blob reads (policy / small files).
pub const GIT_BLOB_CAP_BYTES: usize = 1024 * 1024;
/// Capture ceiling for tree listings and diff parsers.
pub const GIT_DIFF_CAP_BYTES: usize = 32 * 1024 * 1024;
/// Maximum accepted UTF-8 path component/length in diff parsers.
pub const MAX_DIFF_PATH_BYTES: usize = 4096;
/// Maximum number of diff path records accepted.
pub const MAX_DIFF_FILES: usize = 10_000;
/// Disabled candidate fetch remote.
pub const NO_FETCH_URL: &str = "no-fetch://candidate-worker";
/// Disabled candidate push remote.
pub const NO_PUSH_URL: &str = "no-push://candidate-worker";
/// Candidate commit author/committer identity.
pub const CANDIDATE_AUTHOR_NAME: &str = "GZMO Evolver Candidate";
/// Candidate commit author/committer email.
pub const CANDIDATE_AUTHOR_EMAIL: &str = "candidate@gzmo.invalid";
/// Maximum terminal/failure reason length echoed to operators.
pub const MAX_REASON_BYTES: usize = 512;

/// Errors raised by trusted Git mirror/workspace operations.
#[derive(Debug, Error)]
pub enum GitError {
    /// Structural or policy rejection before/while running Git.
    #[error("invalid git operation: {0}")]
    Invalid(String),
    /// Filesystem failure under coordinator-owned paths.
    #[error("git io error: {0}")]
    Io(String),
    /// Underlying process seam failure (no secret/output dump).
    #[error("git process error: {0}")]
    Process(String),
    /// Mirror lease held by another process.
    #[error("git mirror lock busy")]
    MirrorLockBusy,
    /// Trusted checkout or mirror failed integrity/layout checks.
    #[error("git trust error: {0}")]
    Trust(String),
    /// Candidate workspace failed independence or content validation.
    #[error("git workspace error: {0}")]
    Workspace(String),
}

impl From<io::Error> for GitError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<ProcessError> for GitError {
    fn from(value: ProcessError) -> Self {
        // Never surface captured stdout/stderr (may contain secrets/paths).
        let msg = match &value {
            ProcessError::Invalid(m) => format!("invalid: {m}"),
            ProcessError::Io(m) => format!("io: {m}"),
            ProcessError::OutputOverflow { cap } => format!("output exceeded {cap} bytes"),
            ProcessError::Timeout { timeout_ms } => format!("timed out after {timeout_ms} ms"),
            ProcessError::NonZeroExit { code, .. } => format!("exited with status {code}"),
            ProcessError::SignalExit { signal } => format!("terminated by signal {signal}"),
        };
        Self::Process(bound_reason(&msg))
    }
}

/// Bounded diff facts for a baseline/candidate pair (no quality judgment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffStats {
    pub files: Vec<DiffFile>,
    pub added_lines: u32,
    pub deleted_lines: u32,
    pub whitespace_ok: bool,
}

/// One changed path record from raw+numstat parsers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFile {
    pub path: String,
    pub old_mode: String,
    pub new_mode: String,
    pub status: char,
    pub added: Option<u32>,
    pub deleted: Option<u32>,
    pub binary: bool,
}

/// Coordinator view of the trusted checkout + bare mirror.
pub struct GitRepository<'a, R: ProcessRunner> {
    config: &'a RepoEvolverConfig,
    runner: &'a R,
    git_program: PathBuf,
    home: PathBuf,
    mirror_path: PathBuf,
    workspaces_dir: PathBuf,
}

/// Independent candidate clone under coordinator state.
#[derive(Debug)]
pub struct GitWorkspace<'a, R: ProcessRunner> {
    config: &'a RepoEvolverConfig,
    runner: &'a R,
    git_program: PathBuf,
    home: PathBuf,
    path: PathBuf,
    candidate_id: String,
    baseline: String,
}
impl<'a, R: ProcessRunner> GitRepository<'a, R> {
    /// Bind config + runner and ensure coordinator Git HOME exists (0700).
    pub fn open(config: &'a RepoEvolverConfig, runner: &'a R) -> Result<Self, GitError> {
        let state = config.state_dir();
        ensure_dir_0700(state)?;
        let home = state.join(GIT_HOME_NAME);
        ensure_dir_0700(&home)?;
        let workspaces_dir = state.join(WORKSPACES_DIR);
        ensure_dir_0700(&workspaces_dir)?;
        let git_program = resolve_git_program()?;
        Ok(Self {
            config,
            runner,
            git_program,
            home,
            mirror_path: state.join(MIRROR_NAME),
            workspaces_dir,
        })
    }

    /// Absolute trusted checkout path.
    pub fn checkout_path(&self) -> &Path {
        self.config.repo().path()
    }

    /// Absolute bare mirror path.
    pub fn mirror_path(&self) -> &Path {
        &self.mirror_path
    }

    /// Absolute workspaces parent.
    pub fn workspaces_dir(&self) -> &Path {
        &self.workspaces_dir
    }

    /// Verify remote identity and trusted checkout hygiene without mutating.
    pub fn verify_trust(&self) -> Result<(), GitError> {
        self.read_and_validate_remote_identity()?;
        self.reject_executable_local_git_config(self.checkout_path())?;
        self.require_clean_checkout()?;
        Ok(())
    }

    /// Refresh the bare mirror and return the exact fetched base-branch OID.
    pub fn refresh(&self) -> Result<String, GitError> {
        let _lease = MirrorLock::acquire(self.config.state_dir())?;
        let remote_url = self.read_and_validate_remote_identity()?;
        self.reject_executable_local_git_config(self.checkout_path())?;
        self.require_clean_checkout()?;

        if self.mirror_path.exists() {
            self.validate_mirror_layout(&self.mirror_path)?;
            self.fetch_base_into_mirror(&self.mirror_path)?;
        } else {
            self.create_mirror_staged(&remote_url)?;
        }

        let baseline =
            self.rev_parse_mirror(&format!("refs/heads/{}", self.config.repo().base_branch()))?;
        validate_oid(&baseline)?;
        self.reject_special_modes_in_tree(&self.mirror_path, &baseline)?;
        Ok(baseline)
    }

    /// Resolve the current mirror base-branch OID without fetching.
    pub fn resolve_baseline(&self) -> Result<String, GitError> {
        if !self.mirror_path.exists() {
            return Err(GitError::Trust("mirror does not exist".to_owned()));
        }
        self.validate_mirror_layout(&self.mirror_path)?;
        let baseline =
            self.rev_parse_mirror(&format!("refs/heads/{}", self.config.repo().base_branch()))?;
        validate_oid(&baseline)?;
        Ok(baseline)
    }

    /// Refresh mirror, require clean checkout HEAD == baseline, return baseline.
    pub fn refresh_and_resolve_baseline(&self) -> Result<String, GitError> {
        let baseline = self.refresh()?;
        let head = self.rev_parse_checkout("HEAD")?;
        validate_oid(&head)?;
        if head != baseline {
            return Err(GitError::Trust(format!(
                "trusted checkout HEAD {} does not match fetched baseline {}",
                redact_oid(&head),
                redact_oid(&baseline)
            )));
        }
        // Working-tree policy must match baseline policy digest.
        let policy_rel = self
            .config
            .policy()
            .repo_path()
            .to_str()
            .ok_or_else(|| GitError::Invalid("policy path must be UTF-8".to_owned()))?
            .replace('\\', "/");
        let baseline_policy = self.read_file_at(&baseline, &policy_rel)?;
        let parsed = TrustedPolicy::parse_toml(&baseline_policy)
            .map_err(|err| GitError::Trust(format!("baseline policy: {err}")))?;
        let baseline_digest = parsed
            .digest()
            .map_err(|err| GitError::Trust(format!("baseline policy digest: {err}")))?;
        if baseline_digest != self.config.working_policy_digest() {
            return Err(GitError::Trust(
                "working-tree policy digest does not match baseline policy digest".to_owned(),
            ));
        }
        Ok(baseline)
    }

    /// Read a blob at `commit:path` from the mirror (bounded).
    pub fn read_file_at(&self, commit: &str, path: &str) -> Result<Vec<u8>, GitError> {
        validate_oid(commit)?;
        validate_repo_rel_path(path)?;
        if !self.mirror_path.exists() {
            return Err(GitError::Trust("mirror does not exist".to_owned()));
        }
        let spec_path = format!("{commit}:{path}");
        let out = self.run_git(
            None,
            &[
                "--git-dir".to_owned(),
                path_to_string(&self.mirror_path)?,
                "cat-file".to_owned(),
                "blob".to_owned(),
                spec_path,
            ],
            GIT_BLOB_CAP_BYTES,
        )?;
        Ok(out.stdout)
    }

    /// Clone an independent candidate workspace for `manifest` from the mirror.
    pub fn prepare(&self, manifest: &CandidateManifest) -> Result<GitWorkspace<'a, R>, GitError> {
        manifest
            .validate()
            .map_err(|err| GitError::Invalid(err.to_string()))?;
        let baseline = match &manifest.baseline_digest {
            d if d.starts_with("git-sha1:") => d["git-sha1:".len()..].to_owned(),
            _ => {
                return Err(GitError::Invalid(
                    "manifest baseline_digest must be git-sha1:<40 hex>".to_owned(),
                ))
            }
        };
        validate_oid(&baseline)?;

        let candidate_id = manifest.id.as_str().to_owned();
        validate_safe_component(&candidate_id)?;
        let final_path = self.workspaces_dir.join(&candidate_id);
        if final_path.exists() {
            return Err(GitError::Workspace(format!(
                "workspace path already exists for {candidate_id}"
            )));
        }
        if final_path
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(GitError::Workspace(
                "workspace path must not be a symlink".to_owned(),
            ));
        }

        let staging_name = format!(
            ".staging-{}-{}",
            candidate_id,
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let staging = self.workspaces_dir.join(&staging_name);
        if staging.exists() {
            return Err(GitError::Workspace(
                "staging workspace path collision".to_owned(),
            ));
        }

        let cleanup_staging = |path: &Path| {
            let _ = remove_path_best_effort(path);
        };

        let result = (|| -> Result<GitWorkspace<'a, R>, GitError> {
            if !self.mirror_path.exists() {
                return Err(GitError::Trust("mirror does not exist".to_owned()));
            }
            self.validate_mirror_layout(&self.mirror_path)?;
            let mirror_base =
                self.rev_parse_mirror(&format!("refs/heads/{}", self.config.repo().base_branch()))?;
            if mirror_base != baseline {
                return Err(GitError::Trust(
                    "mirror base branch does not match manifest baseline".to_owned(),
                ));
            }

            // Clone into staging (independent objects).
            self.run_git(
                Some(self.workspaces_dir.as_path()),
                &[
                    "clone".to_owned(),
                    "--no-local".to_owned(),
                    "--single-branch".to_owned(),
                    "--no-tags".to_owned(),
                    "--branch".to_owned(),
                    self.config.repo().base_branch().to_owned(),
                    path_to_string(&self.mirror_path)?,
                    staging_name.clone(),
                ],
                GIT_OUTPUT_CAP_BYTES,
            )?;

            reject_symlink_path(&staging)?;
            let git_dir = staging.join(".git");
            if !git_dir.is_dir() || git_dir.symlink_metadata()?.file_type().is_symlink() {
                return Err(GitError::Workspace(
                    "workspace must contain a normal in-tree .git directory".to_owned(),
                ));
            }

            let head = self.rev_parse_at(&staging, "HEAD")?;
            if head != baseline {
                return Err(GitError::Workspace(
                    "cloned HEAD does not match baseline".to_owned(),
                ));
            }

            // Branch evolve/<candidate-id>
            let branch = match &manifest.target {
                evolution_contracts::CandidateTarget::Repository {
                    candidate_branch, ..
                } => candidate_branch.clone(),
                _ => {
                    return Err(GitError::Invalid(
                        "manifest target must be repository".to_owned(),
                    ))
                }
            };
            if branch != format!("evolve/{candidate_id}") {
                return Err(GitError::Invalid(
                    "candidate_branch must equal evolve/<id>".to_owned(),
                ));
            }
            self.run_git(
                Some(&staging),
                &["switch".to_owned(), "-c".to_owned(), branch.clone()],
                GIT_OUTPUT_CAP_BYTES,
            )?;

            // Disable remotes and set identity.
            self.run_git(
                Some(&staging),
                &[
                    "remote".to_owned(),
                    "set-url".to_owned(),
                    "origin".to_owned(),
                    NO_FETCH_URL.to_owned(),
                ],
                GIT_OUTPUT_CAP_BYTES,
            )?;
            self.run_git(
                Some(&staging),
                &[
                    "remote".to_owned(),
                    "set-url".to_owned(),
                    "--push".to_owned(),
                    "origin".to_owned(),
                    NO_PUSH_URL.to_owned(),
                ],
                GIT_OUTPUT_CAP_BYTES,
            )?;
            self.run_git(
                Some(&staging),
                &[
                    "config".to_owned(),
                    "user.name".to_owned(),
                    CANDIDATE_AUTHOR_NAME.to_owned(),
                ],
                GIT_OUTPUT_CAP_BYTES,
            )?;
            self.run_git(
                Some(&staging),
                &[
                    "config".to_owned(),
                    "user.email".to_owned(),
                    CANDIDATE_AUTHOR_EMAIL.to_owned(),
                ],
                GIT_OUTPUT_CAP_BYTES,
            )?;

            self.reject_executable_local_git_config(&staging)?;
            self.verify_disabled_remotes(&staging)?;
            self.verify_object_independence(&staging)?;

            // Atomic publish.
            fs::rename(&staging, &final_path).map_err(|err| GitError::Io(err.to_string()))?;
            reject_symlink_path(&final_path)?;

            Ok(GitWorkspace {
                config: self.config,
                runner: self.runner,
                git_program: self.git_program.clone(),
                home: self.home.clone(),
                path: final_path.clone(),
                candidate_id,
                baseline,
            })
        })();

        if result.is_err() {
            cleanup_staging(&staging);
            // If rename failed after partial final, do not delete final unless we created it
            // only from this attempt — rename is atomic; staging remains on failure before rename.
        }
        result
    }

    fn create_mirror_staged(&self, remote_url: &str) -> Result<(), GitError> {
        let staging = self.config.state_dir().join(MIRROR_STAGING_NAME);
        let _ = remove_path_best_effort(&staging);
        // Parent is state_dir (same parent as final mirror) for atomic rename.
        let out = self.run_git_allow_nonzero(
            Some(self.config.state_dir()),
            &[
                "clone".to_owned(),
                "--mirror".to_owned(),
                remote_url.to_owned(),
                MIRROR_STAGING_NAME.to_owned(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        if out.status != 0 {
            let _ = remove_path_best_effort(&staging);
            return Err(GitError::Trust(
                "mirror clone failed without network credential inheritance".to_owned(),
            ));
        }
        if let Err(err) = self.validate_mirror_layout(&staging) {
            let _ = remove_path_best_effort(&staging);
            return Err(err);
        }
        // Fetch explicit base (also establishes origin tracking shape).
        if let Err(err) = self.fetch_base_into_mirror(&staging) {
            let _ = remove_path_best_effort(&staging);
            return Err(err);
        }
        // Atomic rename into place (same parent).
        if self.mirror_path.exists() {
            let _ = remove_path_best_effort(&staging);
            return Err(GitError::Trust(
                "mirror appeared during staged creation".to_owned(),
            ));
        }
        fs::rename(&staging, &self.mirror_path).map_err(|err| {
            let _ = remove_path_best_effort(&staging);
            GitError::Io(err.to_string())
        })?;
        self.validate_mirror_layout(&self.mirror_path)?;
        Ok(())
    }

    fn fetch_base_into_mirror(&self, mirror: &Path) -> Result<(), GitError> {
        let base = self.config.repo().base_branch();
        let refspec = format!("+refs/heads/{base}:refs/heads/{base}");
        self.run_git(
            None,
            &[
                "--git-dir".to_owned(),
                path_to_string(mirror)?,
                "fetch".to_owned(),
                "--prune".to_owned(),
                "--no-tags".to_owned(),
                "origin".to_owned(),
                refspec,
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        Ok(())
    }

    fn validate_mirror_layout(&self, mirror: &Path) -> Result<(), GitError> {
        reject_symlink_path(mirror)?;
        let meta = fs::symlink_metadata(mirror)?;
        if !meta.is_dir() {
            return Err(GitError::Trust("mirror must be a directory".to_owned()));
        }
        // Bare repo: HEAD + objects + refs, no working tree index required.
        for name in ["HEAD", "objects", "refs"] {
            let p = mirror.join(name);
            if !p.exists() {
                return Err(GitError::Trust(format!(
                    "mirror missing required entry {name}"
                )));
            }
            reject_symlink_path(&p)?;
        }
        // Origin URL in mirror config must still match identity (for network remotes).
        let origin = self.config_get_local(mirror, "remote.origin.url")?;
        if let Some(raw) = origin {
            validate_remote_identity(
                &raw,
                self.config.repo().owner(),
                self.config.repo().repository(),
            )?;
        }
        // No alternates.
        let alternates = mirror.join("objects/info/alternates");
        if alternates.exists() {
            return Err(GitError::Trust(
                "mirror must not use objects/info/alternates".to_owned(),
            ));
        }
        Ok(())
    }

    fn read_and_validate_remote_identity(&self) -> Result<String, GitError> {
        let remote_name = self.config.repo().remote();
        let key = format!("remote.{remote_name}.url");
        let raw = self
            .config_get_local(self.checkout_path(), &key)?
            .ok_or_else(|| GitError::Trust(format!("missing local config {key}")))?;
        validate_remote_identity(
            &raw,
            self.config.repo().owner(),
            self.config.repo().repository(),
        )?;
        Ok(raw)
    }

    fn require_clean_checkout(&self) -> Result<(), GitError> {
        let out = self.run_git(
            Some(self.checkout_path()),
            &[
                "status".to_owned(),
                "--porcelain=v1".to_owned(),
                "-z".to_owned(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        if !out.stdout.is_empty() {
            return Err(GitError::Trust(
                "trusted checkout is dirty (tracked or untracked changes)".to_owned(),
            ));
        }
        Ok(())
    }

    fn reject_executable_local_git_config(&self, repo: &Path) -> Result<(), GitError> {
        // Inspect local config keys that reintroduce ambient authority.
        let forbidden_prefixes = ["url.", "include.", "includeIf."];
        let forbidden_exact = [
            "core.hookspath",
            "core.fsmonitor",
            "core.fsmonitorhookversion",
            "core.sshcommand",
            "commit.gpgsign",
            "gpg.program",
            "i18n.commitencoding",
        ];
        let listing = self.run_git_allow_nonzero(
            Some(repo),
            &[
                "config".to_owned(),
                "--local".to_owned(),
                "--list".to_owned(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        if listing.status != 0 && listing.status != 1 {
            return Err(GitError::Trust(
                "unable to list local git config".to_owned(),
            ));
        }
        let text = String::from_utf8_lossy(&listing.stdout);
        for line in text.lines() {
            let key = line
                .split('=')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if key.is_empty() {
                continue;
            }
            if forbidden_exact.contains(&key.as_str()) {
                return Err(GitError::Trust(format!(
                    "local git config enables forbidden key {key}"
                )));
            }
            for prefix in forbidden_prefixes {
                if key.starts_with(prefix) {
                    return Err(GitError::Trust(format!(
                        "local git config enables forbidden prefix {prefix}"
                    )));
                }
            }
            if key.contains("insteadof") {
                return Err(GitError::Trust(
                    "local git config enables URL insteadOf rewrite".to_owned(),
                ));
            }
        }

        // Hooks directory must not contain executable hooks.
        let git_dir = self.resolve_git_dir(repo)?;
        let hooks = git_dir.join("hooks");
        if hooks.is_dir() {
            for entry in fs::read_dir(&hooks)? {
                let entry = entry?;
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".sample") {
                    continue;
                }
                if path.is_file() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mode = fs::symlink_metadata(&path)?.permissions().mode();
                        if mode & 0o111 != 0 {
                            return Err(GitError::Trust(format!(
                                "executable git hook present: {name}"
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn reject_special_modes_in_tree(&self, git_dir: &Path, commit: &str) -> Result<(), GitError> {
        let out = self.run_git(
            None,
            &[
                "--git-dir".to_owned(),
                path_to_string(git_dir)?,
                "ls-tree".to_owned(),
                "-r".to_owned(),
                "-z".to_owned(),
                commit.to_owned(),
            ],
            GIT_DIFF_CAP_BYTES,
        )?;
        parse_ls_tree_reject_special(&out.stdout)?;
        Ok(())
    }

    fn verify_disabled_remotes(&self, repo: &Path) -> Result<(), GitError> {
        let fetch = self
            .config_get_local(repo, "remote.origin.url")?
            .unwrap_or_default();
        let push = self
            .config_get_local(repo, "remote.origin.pushurl")?
            .unwrap_or_else(|| fetch.clone());
        if fetch != NO_FETCH_URL {
            return Err(GitError::Workspace(
                "origin fetch URL was not disabled".to_owned(),
            ));
        }
        if push != NO_PUSH_URL {
            return Err(GitError::Workspace(
                "origin push URL was not disabled".to_owned(),
            ));
        }
        Ok(())
    }

    fn verify_object_independence(&self, workspace: &Path) -> Result<(), GitError> {
        let git_dir = workspace.join(".git");
        let objects = git_dir.join("objects");
        reject_symlink_path(&objects)?;
        if !objects.is_dir() {
            return Err(GitError::Workspace(
                "workspace objects must be a directory".to_owned(),
            ));
        }
        let alternates = objects.join("info/alternates");
        if alternates.exists() {
            let bytes = fs::read(&alternates)?;
            if !bytes.is_empty() {
                return Err(GitError::Workspace(
                    "workspace must not use objects/info/alternates".to_owned(),
                ));
            }
        }
        // git-dir must differ from mirror and trusted.
        let ws_git = canonicalize_path(&git_dir)?;
        let mirror_git = canonicalize_path(&self.mirror_path)?;
        if ws_git == mirror_git {
            return Err(GitError::Workspace(
                "workspace git-dir must differ from mirror".to_owned(),
            ));
        }
        let trusted_git = self.resolve_git_dir(self.checkout_path())?;
        let trusted_git = canonicalize_path(&trusted_git)?;
        if ws_git == trusted_git {
            return Err(GitError::Workspace(
                "workspace git-dir must differ from trusted checkout".to_owned(),
            ));
        }

        // Probe one object file: must not be hardlinked to the mirror object.
        if let Some((ws_obj, mirror_obj)) =
            find_shared_object_pair(&objects, &self.mirror_path.join("objects"))?
        {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let a = fs::metadata(&ws_obj)?;
                let b = fs::metadata(&mirror_obj)?;
                if a.dev() == b.dev() && a.ino() == b.ino() {
                    return Err(GitError::Workspace(
                        "workspace objects share inodes with mirror (hardlink optimization)"
                            .to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn rev_parse_mirror(&self, rev: &str) -> Result<String, GitError> {
        let out = self.run_git(
            None,
            &[
                "--git-dir".to_owned(),
                path_to_string(&self.mirror_path)?,
                "rev-parse".to_owned(),
                rev.to_owned(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        parse_oid_stdout(&out.stdout)
    }

    fn rev_parse_checkout(&self, rev: &str) -> Result<String, GitError> {
        self.rev_parse_at(self.checkout_path(), rev)
    }

    fn rev_parse_at(&self, cwd: &Path, rev: &str) -> Result<String, GitError> {
        let out = self.run_git(
            Some(cwd),
            &["rev-parse".to_owned(), rev.to_owned()],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        parse_oid_stdout(&out.stdout)
    }

    fn resolve_git_dir(&self, repo: &Path) -> Result<PathBuf, GitError> {
        let out = self.run_git(
            Some(repo),
            &["rev-parse".to_owned(), "--absolute-git-dir".to_owned()],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        let text = String::from_utf8_lossy(&out.stdout);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(GitError::Trust("empty git-dir".to_owned()));
        }
        Ok(PathBuf::from(trimmed))
    }

    fn config_get_local(&self, repo: &Path, key: &str) -> Result<Option<String>, GitError> {
        // Bare mirror: use --git-dir; worktree: -C via cwd.
        let is_bare = repo.join("HEAD").exists() && !repo.join(".git").exists();
        let args = if is_bare {
            vec![
                "--git-dir".to_owned(),
                path_to_string(repo)?,
                "config".to_owned(),
                "--local".to_owned(),
                "--get".to_owned(),
                key.to_owned(),
            ]
        } else {
            vec![
                "config".to_owned(),
                "--local".to_owned(),
                "--get".to_owned(),
                key.to_owned(),
            ]
        };
        let cwd = if is_bare { None } else { Some(repo) };
        let out = self.run_git_allow_nonzero(cwd, &args, GIT_OUTPUT_CAP_BYTES)?;
        if out.status == 1 {
            return Ok(None);
        }
        if out.status != 0 {
            return Err(GitError::Trust(format!(
                "git config --get failed for {}",
                sanitize_config_key(key)
            )));
        }
        let text = String::from_utf8_lossy(&out.stdout);
        Ok(Some(text.trim_end_matches(['\n', '\r']).to_owned()))
    }

    fn run_git(
        &self,
        cwd: Option<&Path>,
        args: &[String],
        cap: usize,
    ) -> Result<ProcessOutput, GitError> {
        let out = self.run_git_allow_nonzero(cwd, args, cap)?;
        if out.status != 0 {
            return Err(ProcessError::NonZeroExit {
                code: out.status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
            .into());
        }
        Ok(out)
    }

    fn run_git_allow_nonzero(
        &self,
        cwd: Option<&Path>,
        args: &[String],
        cap: usize,
    ) -> Result<ProcessOutput, GitError> {
        let cwd = cwd.unwrap_or_else(|| self.config.state_dir()).to_path_buf();
        let env = git_env(&self.home)?;
        let spec = ProcessSpec::new(
            &self.git_program,
            args.iter().cloned(),
            cwd,
            env,
            cap,
            Duration::from_secs(GIT_TIMEOUT_SECS),
        )?;
        match self.runner.run(&spec) {
            Ok(out) => Ok(out),
            Err(ProcessError::NonZeroExit {
                code,
                stdout,
                stderr,
            }) => Ok(ProcessOutput {
                status: code,
                stdout,
                stderr,
            }),
            Err(other) => Err(other.into()),
        }
    }
}

impl<'a, R: ProcessRunner> GitWorkspace<'a, R> {
    /// Absolute workspace path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Candidate id bound at prepare time.
    pub fn candidate_id(&self) -> &str {
        &self.candidate_id
    }

    /// Baseline OID bound at prepare time.
    pub fn baseline(&self) -> &str {
        &self.baseline
    }

    /// Absolute `.git` directory inside the workspace.
    pub fn git_dir(&self) -> Result<PathBuf, GitError> {
        let out = self.run_git(
            &["rev-parse".to_owned(), "--absolute-git-dir".to_owned()],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        let text = String::from_utf8_lossy(&out.stdout);
        Ok(PathBuf::from(text.trim()))
    }

    /// Current branch short name.
    pub fn current_branch(&self) -> Result<String, GitError> {
        let out = self.run_git(
            &[
                "rev-parse".to_owned(),
                "--abbrev-ref".to_owned(),
                "HEAD".to_owned(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    }

    /// `git merge-base` of two revisions.
    pub fn merge_base(&self, a: &str, b: &str) -> Result<String, GitError> {
        let out = self.run_git(
            &["merge-base".to_owned(), a.to_owned(), b.to_owned()],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        parse_oid_stdout(&out.stdout)
    }

    /// Fetch URL for a remote (local config).
    pub fn fetch_url(&self, remote: &str) -> Result<String, GitError> {
        let key = format!("remote.{remote}.url");
        let out = self.run_git(
            &[
                "config".to_owned(),
                "--local".to_owned(),
                "--get".to_owned(),
                key,
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        Ok(String::from_utf8_lossy(&out.stdout)
            .trim_end_matches(['\n', '\r'])
            .to_owned())
    }

    /// Push URL for a remote (falls back to fetch URL when unset).
    pub fn push_url(&self, remote: &str) -> Result<String, GitError> {
        let key = format!("remote.{remote}.pushurl");
        let out = self.run_git_allow_nonzero(
            &[
                "config".to_owned(),
                "--local".to_owned(),
                "--get".to_owned(),
                key,
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        if out.status == 0 {
            return Ok(String::from_utf8_lossy(&out.stdout)
                .trim_end_matches(['\n', '\r'])
                .to_owned());
        }
        self.fetch_url(remote)
    }

    /// True when alternates or shared-object hardlinks are detected.
    pub fn uses_alternates_or_shared_objects(&self) -> Result<bool, GitError> {
        let git_dir = self.path.join(".git");
        let alternates = git_dir.join("objects/info/alternates");
        if alternates.exists() {
            let bytes = fs::read(&alternates)?;
            if !bytes.is_empty() {
                return Ok(true);
            }
        }
        let objects = git_dir.join("objects");
        if objects
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Ok(true);
        }
        Ok(false)
    }

    /// Current HEAD commit OID.
    pub fn candidate_commit(&self) -> Result<String, GitError> {
        let out = self.run_git(
            &["rev-parse".to_owned(), "HEAD".to_owned()],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        parse_oid_stdout(&out.stdout)
    }

    /// Bounded raw/numstat/check diff facts between baseline and candidate.
    pub fn diff_stats(
        &self,
        baseline: &str,
        candidate: &str,
        manifest: &CandidateManifest,
    ) -> Result<DiffStats, GitError> {
        validate_oid(baseline)?;
        validate_oid(candidate)?;
        let path_policy = PathPolicy {
            protected_paths: manifest.protected_paths.clone(),
        };
        path_policy
            .validate()
            .map_err(|err| GitError::Invalid(err.to_string()))?;

        let range = format!("{baseline}..{candidate}");
        let raw = self.run_git(
            &[
                "diff".to_owned(),
                "--no-renames".to_owned(),
                "--raw".to_owned(),
                "-z".to_owned(),
                range.clone(),
            ],
            GIT_DIFF_CAP_BYTES,
        )?;
        let numstat = self.run_git(
            &[
                "diff".to_owned(),
                "--no-renames".to_owned(),
                "--numstat".to_owned(),
                "-z".to_owned(),
                range.clone(),
            ],
            GIT_DIFF_CAP_BYTES,
        )?;
        let check = self.run_git_allow_nonzero(
            &[
                "diff".to_owned(),
                "--no-renames".to_owned(),
                "--check".to_owned(),
                range,
            ],
            GIT_DIFF_CAP_BYTES,
        )?;
        let whitespace_ok = match check.status {
            0 => true,
            1 => false,
            code => {
                return Err(GitError::Process(format!(
                    "diff --check exited with status {code}"
                )))
            }
        };

        let raw_files = parse_raw_diff_z(&raw.stdout)?;
        let num_files = parse_numstat_z(&numstat.stdout)?;
        merge_diff_records(raw_files, num_files, &path_policy, manifest).map(|mut stats| {
            stats.whitespace_ok = whitespace_ok;
            stats
        })
    }

    /// Validate worker HEAD, normalize via commit-tree, CAS-update candidate ref.
    pub fn squash_candidate(
        &self,
        baseline: &str,
        mission_id: &str,
        now: DateTime<Utc>,
    ) -> Result<String, GitError> {
        validate_oid(baseline)?;
        validate_safe_mission_id(mission_id)?;
        self.require_clean_including_untracked()?;
        let branch = self.current_branch()?;
        let expected_branch = format!("evolve/{}", self.candidate_id);
        if branch != expected_branch {
            return Err(GitError::Workspace(format!(
                "workspace branch {branch} does not match {expected_branch}"
            )));
        }
        let head = self.candidate_commit()?;
        // HEAD must descend from baseline.
        let mb = self.merge_base("HEAD", baseline)?;
        if mb != baseline {
            return Err(GitError::Workspace(
                "HEAD does not descend from baseline".to_owned(),
            ));
        }
        if head == baseline {
            return Err(GitError::Workspace(
                "candidate has no changes relative to baseline".to_owned(),
            ));
        }
        // No merge commits in baseline..HEAD.
        let merges = self.run_git(
            &[
                "rev-list".to_owned(),
                "--merges".to_owned(),
                format!("{baseline}..HEAD"),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        if !String::from_utf8_lossy(&merges.stdout).trim().is_empty() {
            return Err(GitError::Workspace(
                "merge commits present in baseline..HEAD".to_owned(),
            ));
        }
        // Reject symlink/gitlink in HEAD tree and in the diff against baseline.
        self.reject_special_in_head()?;
        let diff_raw = self.run_git(
            &[
                "diff".to_owned(),
                "--no-renames".to_owned(),
                "--raw".to_owned(),
                "-z".to_owned(),
                format!("{baseline}..HEAD"),
            ],
            GIT_DIFF_CAP_BYTES,
        )?;
        let raw_files = parse_raw_diff_z(&diff_raw.stdout)?;
        if raw_files.is_empty() {
            return Err(GitError::Workspace(
                "candidate diff against baseline is empty".to_owned(),
            ));
        }
        for f in &raw_files {
            reject_special_mode(&f.old_mode)?;
            reject_special_mode(&f.new_mode)?;
        }

        let tree = {
            let out = self.run_git(
                &["rev-parse".to_owned(), "HEAD^{tree}".to_owned()],
                GIT_OUTPUT_CAP_BYTES,
            )?;
            parse_oid_stdout(&out.stdout)?
        };

        let message = format!("evolve({mission_id}): candidate");
        let ts = now.format("%s").to_string();
        let author = format!("{CANDIDATE_AUTHOR_NAME} <{CANDIDATE_AUTHOR_EMAIL}>");

        // commit-tree with injected env timestamps; no hooks/signing via git_env.
        let mut env = git_env(&self.home)?;
        env.insert(
            "GIT_AUTHOR_NAME".to_owned(),
            CANDIDATE_AUTHOR_NAME.to_owned(),
        );
        env.insert(
            "GIT_AUTHOR_EMAIL".to_owned(),
            CANDIDATE_AUTHOR_EMAIL.to_owned(),
        );
        env.insert("GIT_AUTHOR_DATE".to_owned(), ts.clone());
        env.insert(
            "GIT_COMMITTER_NAME".to_owned(),
            CANDIDATE_AUTHOR_NAME.to_owned(),
        );
        env.insert(
            "GIT_COMMITTER_EMAIL".to_owned(),
            CANDIDATE_AUTHOR_EMAIL.to_owned(),
        );
        env.insert("GIT_COMMITTER_DATE".to_owned(), ts);
        // Also pass message via -m; parent baseline.
        let args = vec![
            "commit-tree".to_owned(),
            tree,
            "-p".to_owned(),
            baseline.to_owned(),
            "-m".to_owned(),
            message,
        ];
        let spec = ProcessSpec::new(
            &self.git_program,
            args,
            &self.path,
            env,
            GIT_OUTPUT_CAP_BYTES,
            Duration::from_secs(GIT_TIMEOUT_SECS),
        )?;
        let out = self.runner.run(&spec).map_err(GitError::from)?;
        if out.status != 0 {
            return Err(GitError::Process(format!(
                "commit-tree exited with status {}",
                out.status
            )));
        }
        let new_commit = parse_oid_stdout(&out.stdout)?;

        // CAS update of refs/heads/<branch> from old HEAD to new_commit.
        let update = self.run_git_allow_nonzero(
            &[
                "update-ref".to_owned(),
                format!("refs/heads/{branch}"),
                new_commit.clone(),
                head.clone(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        if update.status != 0 {
            return Err(GitError::Workspace(
                "candidate ref compare-and-swap failed".to_owned(),
            ));
        }

        // Verify one parent, clean worktree, stable tree.
        let parents = self.run_git(
            &[
                "rev-list".to_owned(),
                "--parents".to_owned(),
                "-n".to_owned(),
                "1".to_owned(),
                new_commit.clone(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        let parent_line = String::from_utf8_lossy(&parents.stdout);
        let parts: Vec<&str> = parent_line.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != new_commit || parts[1] != baseline {
            return Err(GitError::Workspace(
                "normalized commit must have exactly one baseline parent".to_owned(),
            ));
        }
        self.require_clean_including_untracked()?;
        let _ = author; // identity enforced via env
        Ok(new_commit)
    }

    /// Remove this workspace only when `record` is terminal and matches.
    pub fn cleanup(&self, record: &CandidateRecord) -> Result<(), GitError> {
        cleanup_workspace(self.config.state_dir(), record, Some(self.path.as_path()))
    }

    fn require_clean_including_untracked(&self) -> Result<(), GitError> {
        let out = self.run_git(
            &[
                "status".to_owned(),
                "--porcelain=v1".to_owned(),
                "-z".to_owned(),
            ],
            GIT_OUTPUT_CAP_BYTES,
        )?;
        if !out.stdout.is_empty() {
            return Err(GitError::Workspace(
                "workspace is dirty (including untracked)".to_owned(),
            ));
        }
        Ok(())
    }

    fn reject_special_in_head(&self) -> Result<(), GitError> {
        let out = self.run_git(
            &[
                "ls-tree".to_owned(),
                "-r".to_owned(),
                "-z".to_owned(),
                "HEAD".to_owned(),
            ],
            GIT_DIFF_CAP_BYTES,
        )?;
        parse_ls_tree_reject_special(&out.stdout)
    }

    fn run_git(&self, args: &[String], cap: usize) -> Result<ProcessOutput, GitError> {
        let out = self.run_git_allow_nonzero(args, cap)?;
        if out.status != 0 {
            return Err(GitError::Process(format!(
                "git exited with status {}",
                out.status
            )));
        }
        Ok(out)
    }

    fn run_git_allow_nonzero(
        &self,
        args: &[String],
        cap: usize,
    ) -> Result<ProcessOutput, GitError> {
        let env = git_env(&self.home)?;
        let spec = ProcessSpec::new(
            &self.git_program,
            args.iter().cloned(),
            &self.path,
            env,
            cap,
            Duration::from_secs(GIT_TIMEOUT_SECS),
        )?;
        match self.runner.run(&spec) {
            Ok(out) => Ok(out),
            Err(ProcessError::NonZeroExit {
                code,
                stdout,
                stderr,
            }) => Ok(ProcessOutput {
                status: code,
                stdout,
                stderr,
            }),
            Err(other) => Err(other.into()),
        }
    }
}

/// Record-bound cleanup that never escapes `<state_dir>/workspaces`.
pub fn cleanup_workspace(
    state_dir: &Path,
    record: &CandidateRecord,
    expected_path: Option<&Path>,
) -> Result<(), GitError> {
    match record.state() {
        CandidateState::Rejected
        | CandidateState::Failed
        | CandidateState::RolledBack
        | CandidateState::Accepted => {}
        CandidateState::ReviewReady => {
            return Err(GitError::Invalid(
                "cleanup refuses ReviewReady workspaces until PR lifecycle owns them".to_owned(),
            ));
        }
        other => {
            return Err(GitError::Invalid(format!(
                "cleanup requires terminal candidate, got {other}"
            )));
        }
    }
    let ws = record
        .workspace()
        .ok_or_else(|| GitError::Invalid("terminal candidate has no workspace path".to_owned()))?;
    if let Some(expected) = expected_path {
        if ws != expected {
            return Err(GitError::Invalid(
                "cleanup path does not match workspace handle".to_owned(),
            ));
        }
    }
    let workspaces = state_dir.join(WORKSPACES_DIR);
    let ws_canon = canonicalize_path(ws)?;
    let parent_canon = canonicalize_path(&workspaces)?;
    if ws_canon.parent().map(Path::to_path_buf).as_ref() != Some(&parent_canon) {
        return Err(GitError::Invalid(
            "workspace is not an immediate descendant of state workspaces/".to_owned(),
        ));
    }
    // Candidate id must match directory name.
    let name = ws_canon
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| GitError::Invalid("workspace basename invalid".to_owned()))?;
    if name != record.id().as_str() {
        return Err(GitError::Invalid(
            "workspace basename does not match candidate id".to_owned(),
        ));
    }
    reject_symlink_path(&ws_canon)?;
    let git_dir = ws_canon.join(".git");
    if !git_dir.is_dir() {
        return Err(GitError::Invalid(
            "workspace missing in-tree .git directory".to_owned(),
        ));
    }
    let git_canon = canonicalize_path(&git_dir)?;
    if !path_is_within(&git_canon, &ws_canon) {
        return Err(GitError::Invalid(
            "workspace .git escapes workspace root".to_owned(),
        ));
    }
    // Optional HEAD match when candidate_digest is set.
    if let Some(digest) = record.candidate_digest() {
        if let Some(oid) = digest.strip_prefix("git-sha1:") {
            // Best-effort HEAD check without ProcessRunner: read .git/HEAD + ref.
            // If unreadable, refuse cleanup rather than deleting blindly.
            let head = read_workspace_head_oid(&ws_canon)?;
            if head != oid {
                return Err(GitError::Invalid(
                    "workspace HEAD does not match candidate_digest".to_owned(),
                ));
            }
        }
    }
    remove_path_best_effort(&ws_canon).map_err(|err| GitError::Io(err.to_string()))?;
    Ok(())
}

/// Prepare orchestration result returned to CLI callers.
#[derive(Debug, Clone)]
pub struct PrepareOutcome {
    pub record: CandidateRecord,
    pub baseline: Option<String>,
    pub reused_active: bool,
}

/// Trust-first / active-first prepare flow.
pub fn prepare_candidate<R: ProcessRunner, C: crate::mission::Clock>(
    config: &RepoEvolverConfig,
    runner: &R,
    clock: &C,
    store: &crate::state::StateStore,
) -> Result<PrepareOutcome, PrepareError> {
    let repository = format!("{}/{}", config.repo().owner(), config.repo().repository());
    if let Some(active) = store
        .active_candidate(&repository)
        .map_err(|e| PrepareError::State(e.to_string()))?
    {
        return Ok(PrepareOutcome {
            record: active,
            baseline: None,
            reused_active: true,
        });
    }

    let git = GitRepository::open(config, runner).map_err(PrepareError::from_git)?;
    let baseline = git
        .refresh_and_resolve_baseline()
        .map_err(PrepareError::from_git)?;
    let policy_rel = config
        .policy()
        .repo_path()
        .to_str()
        .ok_or_else(|| PrepareError::Invalid("policy path must be UTF-8".to_owned()))?
        .replace('\\', "/");
    let policy_bytes = git
        .read_file_at(&baseline, &policy_rel)
        .map_err(PrepareError::from_git)?;
    let policy = TrustedPolicy::parse_toml(&policy_bytes)
        .map_err(|err| PrepareError::Invalid(format!("baseline policy: {err}")))?;
    let policy_digest = policy
        .digest()
        .map_err(|err| PrepareError::Invalid(format!("policy digest: {err}")))?;

    let adapter = crate::mission::MissionAdapter::new(config, runner, clock);
    let mission = adapter
        .refresh_and_load()
        .map_err(|err| PrepareError::Mission(bound_reason(&err.to_string())))?;
    let prepared = mission
        .to_prepared_candidate(config, &policy, &baseline, clock.now())
        .map_err(|err| PrepareError::Mission(bound_reason(&err.to_string())))?;

    let now = clock.now();
    let observed = store
        .create_candidate(&prepared.manifest, &policy_digest, now)
        .map_err(|e| PrepareError::State(e.to_string()))?;

    match git.prepare(&prepared.manifest) {
        Ok(ws) => {
            let record = store
                .transition(
                    observed.id(),
                    CandidateState::Prepared,
                    crate::state::TransitionMetadata::empty().with_workspace(ws.path()),
                    clock.now(),
                )
                .map_err(|e| PrepareError::State(e.to_string()))?;
            Ok(PrepareOutcome {
                record,
                baseline: Some(baseline),
                reused_active: false,
            })
        }
        Err(err) => {
            let reason = bound_reason(&err.to_string());
            // Best-effort remove any staging leftover already handled inside prepare.
            let _ = store.transition(
                observed.id(),
                CandidateState::Failed,
                crate::state::TransitionMetadata::terminal(reason.clone()),
                clock.now(),
            );
            Err(PrepareError::Failed {
                reason,
                candidate_id: observed.id().as_str().to_owned(),
            })
        }
    }
}

/// Errors from the prepare orchestration (CLI-facing, non-secret).
#[derive(Debug, Error)]
pub enum PrepareError {
    #[error("prepare invalid: {0}")]
    Invalid(String),
    #[error("prepare git: {0}")]
    Git(String),
    #[error("prepare mission: {0}")]
    Mission(String),
    #[error("prepare state: {0}")]
    State(String),
    #[error("prepare failed for {candidate_id}: {reason}")]
    Failed {
        candidate_id: String,
        reason: String,
    },
}

impl PrepareError {
    fn from_git(err: GitError) -> Self {
        Self::Git(bound_reason(&err.to_string()))
    }
}

// --- helpers ----------------------------------------------------------------

struct MirrorLock {
    _file: File,
}

impl MirrorLock {
    fn acquire(state_dir: &Path) -> Result<Self, GitError> {
        ensure_dir_0700(state_dir)?;
        let path = state_dir.join(MIRROR_LOCK_NAME);
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|err| GitError::Io(err.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
        }
        file.try_lock_exclusive().map_err(|err| {
            if err.raw_os_error() == fs2::lock_contended_error().raw_os_error() {
                GitError::MirrorLockBusy
            } else {
                GitError::Io(err.to_string())
            }
        })?;
        Ok(Self { _file: file })
    }
}

fn git_env(home: &Path) -> Result<BTreeMap<String, String>, GitError> {
    let mut env = BTreeMap::new();
    env.insert("PATH".to_owned(), SAFE_PATH.to_owned());
    env.insert("HOME".to_owned(), path_to_string(home)?);
    env.insert("LC_ALL".to_owned(), "C".to_owned());
    env.insert("GIT_TERMINAL_PROMPT".to_owned(), "0".to_owned());
    env.insert("GIT_CONFIG_NOSYSTEM".to_owned(), "1".to_owned());
    env.insert("GIT_CONFIG_GLOBAL".to_owned(), "/dev/null".to_owned());
    env.insert("GIT_CONFIG_SYSTEM".to_owned(), "/dev/null".to_owned());
    env.insert("GIT_OPTIONAL_LOCKS".to_owned(), "0".to_owned());
    // Disable hooks/fsmonitor/signing; allow hermetic file transport; deny ext.
    env.insert("GIT_CONFIG_COUNT".to_owned(), "6".to_owned());
    env.insert("GIT_CONFIG_KEY_0".to_owned(), "core.hooksPath".to_owned());
    env.insert("GIT_CONFIG_VALUE_0".to_owned(), "/dev/null".to_owned());
    env.insert("GIT_CONFIG_KEY_1".to_owned(), "core.fsmonitor".to_owned());
    env.insert("GIT_CONFIG_VALUE_1".to_owned(), "false".to_owned());
    env.insert("GIT_CONFIG_KEY_2".to_owned(), "commit.gpgsign".to_owned());
    env.insert("GIT_CONFIG_VALUE_2".to_owned(), "false".to_owned());
    env.insert("GIT_CONFIG_KEY_3".to_owned(), "tag.gpgsign".to_owned());
    env.insert("GIT_CONFIG_VALUE_3".to_owned(), "false".to_owned());
    env.insert(
        "GIT_CONFIG_KEY_4".to_owned(),
        "protocol.file.allow".to_owned(),
    );
    env.insert("GIT_CONFIG_VALUE_4".to_owned(), "always".to_owned());
    env.insert(
        "GIT_CONFIG_KEY_5".to_owned(),
        "protocol.ext.allow".to_owned(),
    );
    env.insert("GIT_CONFIG_VALUE_5".to_owned(), "never".to_owned());
    env.insert("GIT_ASKPASS".to_owned(), String::new());
    env.insert("GCM_INTERACTIVE".to_owned(), "never".to_owned());
    Ok(env)
}

fn resolve_git_program() -> Result<PathBuf, GitError> {
    for candidate in ["/usr/bin/git", "/bin/git"] {
        let p = Path::new(candidate);
        if p.is_file() {
            return Ok(p.to_path_buf());
        }
    }
    Err(GitError::Invalid(
        "git executable not found in fixed PATH".to_owned(),
    ))
}

/// Parse and validate a credential-free remote that identifies owner/repo.
pub fn validate_remote_identity(raw: &str, owner: &str, repository: &str) -> Result<(), GitError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(GitError::Trust("remote URL is empty".to_owned()));
    }
    if raw.contains('\0') {
        return Err(GitError::Trust("remote URL contains NUL".to_owned()));
    }
    // Reject obvious embedded credentials before any parser details leak.
    if raw.contains("://") {
        if let Some(after_scheme) = raw.split("://").nth(1) {
            let authority = after_scheme.split('/').next().unwrap_or("");
            if authority.contains('@') {
                let userinfo = authority.rsplit_once('@').map(|(u, _)| u).unwrap_or("");
                if !userinfo.is_empty() {
                    return Err(GitError::Trust(
                        "remote URL must not embed credentials".to_owned(),
                    ));
                }
            }
        }
    }

    // SCP-like: git@host:path
    if let Some(scp) = parse_scp_syntax(raw)? {
        return match_owner_repo_path(&scp.path, owner, repository, Some(&scp.host), true);
    }

    if let Ok(url) = Url::parse(raw) {
        match url.scheme() {
            "http" | "https" | "ssh" | "git" => {
                if !url.username().is_empty() || url.password().is_some() {
                    return Err(GitError::Trust(
                        "remote URL must not embed credentials".to_owned(),
                    ));
                }
                let host = url
                    .host_str()
                    .ok_or_else(|| GitError::Trust("network remote missing host".to_owned()))?
                    .to_ascii_lowercase();
                if host != "github.com" && host != "www.github.com" {
                    return Err(GitError::Trust(
                        "network remote host must be github.com".to_owned(),
                    ));
                }
                let path = url.path().trim_start_matches('/');
                return match_owner_repo_path(path, owner, repository, Some(&host), true);
            }
            "file" => {
                if !url.username().is_empty() || url.password().is_some() {
                    return Err(GitError::Trust(
                        "file remote must not embed credentials".to_owned(),
                    ));
                }
                if url.path().is_empty() {
                    return Err(GitError::Trust("file remote path empty".to_owned()));
                }
                return Ok(());
            }
            other => {
                return Err(GitError::Trust(format!(
                    "unsupported remote URL scheme {}",
                    sanitize_scheme(other)
                )));
            }
        }
    }

    // Bare local path (hermetic tests).
    let path = Path::new(raw);
    if path.is_absolute() || raw.starts_with('.') || raw.starts_with('/') {
        if raw.contains("://") {
            return Err(GitError::Trust("ambiguous remote URL".to_owned()));
        }
        return Ok(());
    }

    Err(GitError::Trust(
        "remote URL could not be parsed as credential-free identity".to_owned(),
    ))
}

struct ScpRemote {
    host: String,
    path: String,
}

fn parse_scp_syntax(raw: &str) -> Result<Option<ScpRemote>, GitError> {
    // git@github.com:owner/repo.git
    if raw.contains("://") {
        return Ok(None);
    }
    let Some((user_host, path)) = raw.split_once(':') else {
        return Ok(None);
    };
    if path.starts_with('/') {
        // Absolute path after colon is not classic SCP github form; treat as local.
        return Ok(None);
    }
    if !user_host.contains('@') {
        return Ok(None);
    }
    let mut parts = user_host.rsplitn(2, '@');
    let host = parts.next().unwrap_or("").to_ascii_lowercase();
    let user = parts.next().unwrap_or("");
    if user.is_empty() || host.is_empty() {
        return Err(GitError::Trust("malformed SCP remote".to_owned()));
    }
    // SCP always has a user component; reject password-like userinfo (user:pass@host is not SCP).
    if user.contains(':') {
        return Err(GitError::Trust(
            "remote URL must not embed credentials".to_owned(),
        ));
    }
    Ok(Some(ScpRemote {
        host,
        path: path.to_owned(),
    }))
}

fn match_owner_repo_path(
    path: &str,
    owner: &str,
    repository: &str,
    host: Option<&str>,
    require_github: bool,
) -> Result<(), GitError> {
    if require_github {
        let host = host.unwrap_or("").to_ascii_lowercase();
        if host != "github.com" && host != "www.github.com" {
            return Err(GitError::Trust(
                "network remote host must be github.com".to_owned(),
            ));
        }
    }
    let path = path.trim_start_matches('/');
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let expected = format!("{owner}/{repository}");
    // Accept exact suffix match on normalized path.
    let normalized = path.replace('\\', "/");
    if normalized == expected
        || normalized.ends_with(&format!("/{expected}"))
        || normalized.eq_ignore_ascii_case(&expected)
    {
        // Enforce exact case for owner/repo segments when equal length form.
        let segs: Vec<&str> = normalized.rsplitn(2, '/').collect();
        if segs.len() == 2 {
            let repo_seg = segs[0];
            let owner_seg = segs[1].rsplit('/').next().unwrap_or(segs[1]);
            if owner_seg != owner || repo_seg != repository {
                // allow only exact
                if normalized != expected && !normalized.ends_with(&format!("/{expected}")) {
                    return Err(GitError::Trust(
                        "remote path does not match configured owner/repository".to_owned(),
                    ));
                }
            }
        }
        if normalized == expected || normalized.ends_with(&format!("/{expected}")) {
            return Ok(());
        }
    }
    Err(GitError::Trust(
        "remote path does not match configured owner/repository".to_owned(),
    ))
}

fn validate_oid(oid: &str) -> Result<(), GitError> {
    if oid.len() != 40
        || !oid
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        // must be lowercase hex
        if oid.len() != 40 || !oid.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
            return Err(GitError::Invalid(
                "git object id must be 40 lowercase hex characters".to_owned(),
            ));
        }
    }
    if !oid.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        return Err(GitError::Invalid(
            "git object id must be 40 lowercase hex characters".to_owned(),
        ));
    }
    Ok(())
}

fn parse_oid_stdout(bytes: &[u8]) -> Result<String, GitError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| GitError::Invalid("rev-parse output is not UTF-8".to_owned()))?;
    let oid = text.trim();
    validate_oid(oid)?;
    Ok(oid.to_owned())
}

fn validate_repo_rel_path(path: &str) -> Result<(), GitError> {
    if path.is_empty() || path.len() > MAX_DIFF_PATH_BYTES {
        return Err(GitError::Invalid("blob path length invalid".to_owned()));
    }
    if path.starts_with('/') || path.contains('\0') || path.contains("..") {
        return Err(GitError::Invalid("blob path escapes or invalid".to_owned()));
    }
    Ok(())
}

fn validate_safe_component(name: &str) -> Result<(), GitError> {
    if name.is_empty() || name.len() > 96 {
        return Err(GitError::Invalid(
            "component name length invalid".to_owned(),
        ));
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') || name.starts_with('.') {
        return Err(GitError::Invalid("component name unsafe".to_owned()));
    }
    Ok(())
}

fn validate_safe_mission_id(id: &str) -> Result<(), GitError> {
    if id.is_empty() || id.len() > 128 {
        return Err(GitError::Invalid("mission id length invalid".to_owned()));
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GitError::Invalid(
            "mission id has unsafe characters".to_owned(),
        ));
    }
    Ok(())
}

fn parse_ls_tree_reject_special(bytes: &[u8]) -> Result<(), GitError> {
    // records: MODE SP TYPE SP OBJ\tPATH\0
    let mut i = 0;
    let mut count = 0usize;
    while i < bytes.len() {
        if count >= MAX_DIFF_FILES {
            return Err(GitError::Invalid("ls-tree exceeds file ceiling".to_owned()));
        }
        let rest = &bytes[i..];
        let Some(nul) = rest.iter().position(|b| *b == 0) else {
            return Err(GitError::Invalid(
                "ls-tree output truncated or malformed".to_owned(),
            ));
        };
        let rec = &rest[..nul];
        let text = std::str::from_utf8(rec)
            .map_err(|_| GitError::Invalid("ls-tree path not UTF-8".to_owned()))?;
        let mut parts = text.splitn(2, '\t');
        let meta = parts
            .next()
            .ok_or_else(|| GitError::Invalid("ls-tree missing meta".to_owned()))?;
        let path = parts
            .next()
            .ok_or_else(|| GitError::Invalid("ls-tree missing path".to_owned()))?;
        let mut meta_parts = meta.split_whitespace();
        let mode = meta_parts
            .next()
            .ok_or_else(|| GitError::Invalid("ls-tree missing mode".to_owned()))?;
        reject_special_mode(mode)?;
        if path.is_empty() || path.len() > MAX_DIFF_PATH_BYTES {
            return Err(GitError::Invalid("ls-tree path invalid".to_owned()));
        }
        i += nul + 1;
        count = count.saturating_add(1);
    }
    Ok(())
}

fn reject_special_mode(mode: &str) -> Result<(), GitError> {
    // 120000 symlink, 160000 gitlink
    if mode == "120000" || mode == "160000" {
        return Err(GitError::Trust(format!(
            "special git mode {mode} rejected (symlink/gitlink)"
        )));
    }
    // Also reject unknown non-file/dir/exec modes beyond 100644/100755/040000/000000
    match mode {
        "100644" | "100755" | "040000" | "000000" | "" => Ok(()),
        m if m.chars().all(|c| c.is_ascii_digit()) && m.len() == 6 => {
            // Allow only normal blob modes already listed; anything else fails.
            if m.starts_with("10") {
                Ok(())
            } else {
                Err(GitError::Trust(format!("unsupported git mode {m}")))
            }
        }
        other => Err(GitError::Trust(format!("unsupported git mode {other}"))),
    }
}

struct RawDiffEntry {
    old_mode: String,
    new_mode: String,
    status: char,
    path: String,
}

fn parse_raw_diff_z(bytes: &[u8]) -> Result<Vec<RawDiffEntry>, GitError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if out.len() >= MAX_DIFF_FILES {
            return Err(GitError::Invalid(
                "raw diff exceeds file ceiling".to_owned(),
            ));
        }
        if bytes[i] != b':' {
            return Err(GitError::Invalid(
                "raw diff record must start with ':'".to_owned(),
            ));
        }
        // Find first NUL: meta ends, path follows until second NUL for renames — we forbid renames.
        let rest = &bytes[i + 1..];
        let Some(nul1) = rest.iter().position(|b| *b == 0) else {
            return Err(GitError::Invalid("raw diff truncated".to_owned()));
        };
        let meta = std::str::from_utf8(&rest[..nul1])
            .map_err(|_| GitError::Invalid("raw diff meta not UTF-8".to_owned()))?;
        let mut parts = meta.split_whitespace();
        let old_mode = parts
            .next()
            .ok_or_else(|| GitError::Invalid("raw diff missing old mode".to_owned()))?
            .to_owned();
        let new_mode = parts
            .next()
            .ok_or_else(|| GitError::Invalid("raw diff missing new mode".to_owned()))?
            .to_owned();
        let _old = parts
            .next()
            .ok_or_else(|| GitError::Invalid("raw diff missing old oid".to_owned()))?;
        let _new = parts
            .next()
            .ok_or_else(|| GitError::Invalid("raw diff missing new oid".to_owned()))?;
        let status_s = parts
            .next()
            .ok_or_else(|| GitError::Invalid("raw diff missing status".to_owned()))?;
        if parts.next().is_some() {
            return Err(GitError::Invalid(
                "raw diff meta has trailing fields".to_owned(),
            ));
        }
        let status = status_s
            .chars()
            .next()
            .ok_or_else(|| GitError::Invalid("raw diff empty status".to_owned()))?;
        if status_s.len() != 1 || matches!(status, 'R' | 'C') {
            return Err(GitError::Invalid(
                "raw diff rename/copy status rejected".to_owned(),
            ));
        }
        i += 1 + nul1 + 1;
        if i >= bytes.len() {
            return Err(GitError::Invalid("raw diff missing path".to_owned()));
        }
        let rest = &bytes[i..];
        let Some(nul2) = rest.iter().position(|b| *b == 0) else {
            return Err(GitError::Invalid("raw diff path truncated".to_owned()));
        };
        let path = std::str::from_utf8(&rest[..nul2])
            .map_err(|_| GitError::Invalid("raw diff path not UTF-8".to_owned()))?
            .to_owned();
        if path.is_empty() || path.len() > MAX_DIFF_PATH_BYTES {
            return Err(GitError::Invalid("raw diff path invalid".to_owned()));
        }
        i += nul2 + 1;
        reject_special_mode(&old_mode)?;
        reject_special_mode(&new_mode)?;
        out.push(RawDiffEntry {
            old_mode,
            new_mode,
            status,
            path,
        });
    }
    Ok(out)
}

struct NumstatEntry {
    path: String,
    added: Option<u32>,
    deleted: Option<u32>,
    binary: bool,
}

fn parse_numstat_z(bytes: &[u8]) -> Result<Vec<NumstatEntry>, GitError> {
    // With -z: added\tdeleted\tpath\0  (no final newline required)
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if out.len() >= MAX_DIFF_FILES {
            return Err(GitError::Invalid("numstat exceeds file ceiling".to_owned()));
        }
        let rest = &bytes[i..];
        let Some(nul) = rest.iter().position(|b| *b == 0) else {
            return Err(GitError::Invalid("numstat truncated".to_owned()));
        };
        let rec = std::str::from_utf8(&rest[..nul])
            .map_err(|_| GitError::Invalid("numstat not UTF-8".to_owned()))?;
        let mut parts = rec.splitn(3, '\t');
        let added_s = parts
            .next()
            .ok_or_else(|| GitError::Invalid("numstat missing added".to_owned()))?;
        let deleted_s = parts
            .next()
            .ok_or_else(|| GitError::Invalid("numstat missing deleted".to_owned()))?;
        let path = parts
            .next()
            .ok_or_else(|| GitError::Invalid("numstat missing path".to_owned()))?
            .to_owned();
        if path.is_empty() || path.len() > MAX_DIFF_PATH_BYTES {
            return Err(GitError::Invalid("numstat path invalid".to_owned()));
        }
        let (added, deleted, binary) = if added_s == "-" && deleted_s == "-" {
            (None, None, true)
        } else {
            let a: u32 = added_s
                .parse()
                .map_err(|_| GitError::Invalid("numstat added not u32".to_owned()))?;
            let d: u32 = deleted_s
                .parse()
                .map_err(|_| GitError::Invalid("numstat deleted not u32".to_owned()))?;
            (Some(a), Some(d), false)
        };
        out.push(NumstatEntry {
            path,
            added,
            deleted,
            binary,
        });
        i += nul + 1;
    }
    Ok(out)
}

fn merge_diff_records(
    raw: Vec<RawDiffEntry>,
    num: Vec<NumstatEntry>,
    paths: &PathPolicy,
    manifest: &CandidateManifest,
) -> Result<DiffStats, GitError> {
    if raw.len() != num.len() {
        return Err(GitError::Invalid(
            "raw/numstat path count mismatch".to_owned(),
        ));
    }
    let mut seen = BTreeSet::new();
    let mut files = Vec::with_capacity(raw.len());
    let mut added_total: u64 = 0;
    let mut deleted_total: u64 = 0;

    for (r, n) in raw.into_iter().zip(num.into_iter()) {
        if r.path != n.path {
            return Err(GitError::Invalid(
                "raw/numstat path order mismatch".to_owned(),
            ));
        }
        if !seen.insert(r.path.clone()) {
            return Err(GitError::Invalid(format!(
                "duplicate diff path {}",
                sanitize_path_for_error(&r.path)
            )));
        }
        paths
            .check(&r.path)
            .map_err(|err| GitError::Invalid(format!("path policy: {err}")))?;
        if let Some(a) = n.added {
            added_total = added_total
                .checked_add(u64::from(a))
                .ok_or_else(|| GitError::Invalid("added line count overflow".to_owned()))?;
        }
        if let Some(d) = n.deleted {
            deleted_total = deleted_total
                .checked_add(u64::from(d))
                .ok_or_else(|| GitError::Invalid("deleted line count overflow".to_owned()))?;
        }
        files.push(DiffFile {
            path: r.path,
            old_mode: r.old_mode,
            new_mode: r.new_mode,
            status: r.status,
            added: n.added,
            deleted: n.deleted,
            binary: n.binary,
        });
    }

    let file_count = u32::try_from(files.len())
        .map_err(|_| GitError::Invalid("file count overflow".to_owned()))?;
    if file_count > manifest.budget.max_changed_files {
        return Err(GitError::Invalid(format!(
            "changed files {file_count} exceed budget {}",
            manifest.budget.max_changed_files
        )));
    }
    let added_u32 = u32::try_from(added_total)
        .map_err(|_| GitError::Invalid("added lines exceed u32".to_owned()))?;
    if added_u32 > manifest.budget.max_added_lines {
        return Err(GitError::Invalid(format!(
            "added lines {added_u32} exceed budget {}",
            manifest.budget.max_added_lines
        )));
    }
    let deleted_u32 = u32::try_from(deleted_total).unwrap_or(u32::MAX);

    Ok(DiffStats {
        files,
        added_lines: added_u32,
        deleted_lines: deleted_u32,
        whitespace_ok: true,
    })
}

fn find_shared_object_pair(
    ws_objects: &Path,
    mirror_objects: &Path,
) -> Result<Option<(PathBuf, PathBuf)>, GitError> {
    // Walk first fanout directory for a loose object.
    let entries = match fs::read_dir(ws_objects) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.len() != 2 || !name_str.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        for obj in fs::read_dir(&dir).into_iter().flatten().flatten() {
            if !obj.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let ws_obj = obj.path();
            let mirror_obj = mirror_objects.join(&name).join(obj.file_name());
            if mirror_obj.is_file() {
                return Ok(Some((ws_obj, mirror_obj)));
            }
        }
    }
    Ok(None)
}

fn read_workspace_head_oid(workspace: &Path) -> Result<String, GitError> {
    let head_path = workspace.join(".git/HEAD");
    let text = fs::read_to_string(&head_path).map_err(|err| GitError::Io(err.to_string()))?;
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("ref: ") {
        let ref_path = workspace.join(".git").join(rest);
        let oid = fs::read_to_string(&ref_path).map_err(|err| GitError::Io(err.to_string()))?;
        return parse_oid_stdout(oid.trim().as_bytes());
    }
    parse_oid_stdout(text.as_bytes())
}

fn ensure_dir_0700(path: &Path) -> Result<(), GitError> {
    if path.exists() {
        let meta = fs::symlink_metadata(path)?;
        if meta.file_type().is_symlink() {
            return Err(GitError::Io(format!(
                "{} must not be a symlink",
                path.display()
            )));
        }
        if !meta.is_dir() {
            return Err(GitError::Io(format!(
                "{} must be a directory",
                path.display()
            )));
        }
    } else {
        fs::create_dir_all(path)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn reject_symlink_path(path: &Path) -> Result<(), GitError> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return Err(GitError::Trust(format!(
                "path must not be a symlink: {}",
                sanitize_path_for_error(&path.display().to_string())
            )));
        }
    }
    Ok(())
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, GitError> {
    fs::canonicalize(path).map_err(|err| GitError::Io(err.to_string()))
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    let mut path_c = path.components();
    for rc in root.components() {
        match path_c.next() {
            Some(pc) if pc == rc => {}
            _ => return false,
        }
    }
    true
}

fn path_to_string(path: &Path) -> Result<String, GitError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| GitError::Io("path is not valid UTF-8".to_owned()))
}

fn remove_path_best_effort(path: &Path) -> io::Result<()> {
    if !path.exists() && fs::symlink_metadata(path).is_err() {
        return Ok(());
    }
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() || meta.is_file() {
        fs::remove_file(path)
    } else {
        fs::remove_dir_all(path)
    }
}

fn bound_reason(msg: &str) -> String {
    let mut out = String::new();
    for ch in msg.chars() {
        if out.len() >= MAX_REASON_BYTES {
            break;
        }
        // Strip anything that looks like a URL userinfo just in case.
        out.push(ch);
    }
    // Redact crude credential patterns.
    if out.contains("://") && out.contains('@') {
        return "git operation failed (details redacted)".to_owned();
    }
    out
}

fn redact_oid(oid: &str) -> &str {
    if oid.len() >= 8 {
        &oid[..8]
    } else {
        oid
    }
}

fn sanitize_config_key(key: &str) -> String {
    key.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .take(64)
        .collect()
}

fn sanitize_scheme(scheme: &str) -> String {
    scheme
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(16)
        .collect()
}

fn sanitize_path_for_error(path: &str) -> String {
    path.chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '/')
        .take(64)
        .collect()
}

/// Public helper used by standalone refresh: verify Git trust only.
pub fn verify_git_trust<R: ProcessRunner>(
    config: &RepoEvolverConfig,
    runner: &R,
) -> Result<(), GitError> {
    let git = GitRepository::open(config, runner)?;
    git.verify_trust()?;
    // Also ensure remote identity matches even if checkout dirty check is the main gate.
    let _ = git.read_and_validate_remote_identity()?;
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn rejects_embedded_https_credentials() {
        let err = validate_remote_identity(
            "https://user:token@github.com/maximilianwruhs-cyber/GZMO.git",
            "maximilianwruhs-cyber",
            "GZMO",
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("credential") || msg.contains("Trust") || msg.contains("trust"),
            "{msg}"
        );
        assert!(!msg.contains("token"));
        assert!(!msg.contains("user:"));
    }

    #[test]
    fn rejects_non_github_host() {
        let err = validate_remote_identity(
            "https://gitlab.com/maximilianwruhs-cyber/GZMO.git",
            "maximilianwruhs-cyber",
            "GZMO",
        )
        .unwrap_err();
        assert!(err.to_string().contains("github.com"), "{err}");
    }

    #[test]
    fn accepts_scp_github_identity() {
        validate_remote_identity(
            "git@github.com:maximilianwruhs-cyber/GZMO.git",
            "maximilianwruhs-cyber",
            "GZMO",
        )
        .unwrap();
    }

    #[test]
    fn accepts_local_path_for_hermetic_tests() {
        validate_remote_identity("/tmp/origin.git", "maximilianwruhs-cyber", "GZMO").unwrap();
    }

    #[test]
    fn accepts_file_url_for_hermetic_tests() {
        validate_remote_identity("file:///tmp/origin.git", "maximilianwruhs-cyber", "GZMO")
            .unwrap();
    }

    #[test]
    fn parse_raw_and_numstat_round_trip() {
        let raw = b":100644 100644 abcdef012345678901234567890123456789abcd abcdef012345678901234567890123456789abce M\0src/foo.rs\0";
        let num = b"3\t1\tsrc/foo.rs\0";
        let r = parse_raw_diff_z(raw).unwrap();
        let n = parse_numstat_z(num).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(n[0].added, Some(3));
        let policy = PathPolicy {
            protected_paths: vec!["SECRET/".to_owned()],
        };
        let manifest = dummy_manifest();
        let stats = merge_diff_records(r, n, &policy, &manifest).unwrap();
        assert_eq!(stats.added_lines, 3);
        assert_eq!(stats.files[0].path, "src/foo.rs");
    }

    fn dummy_manifest() -> CandidateManifest {
        use chrono::TimeZone;
        use evolution_contracts::{
            AuthorityTier, CandidateId, CandidateKind, CandidateTarget, ResourceBudget,
            CANDIDATE_SCHEMA,
        };
        CandidateManifest {
            schema: CANDIDATE_SCHEMA.to_owned(),
            id: CandidateId::parse("cand-20260901t120000z-bet-01234567").unwrap(),
            mission_id: "felt-use-mass-growth".to_owned(),
            kind: CandidateKind::Code,
            authority: AuthorityTier::Candidate,
            target: CandidateTarget::Repository {
                owner: "maximilianwruhs-cyber".to_owned(),
                repository: "GZMO".to_owned(),
                base_branch: "main".to_owned(),
                candidate_branch: "evolve/cand-20260901t120000z-bet-01234567".to_owned(),
            },
            baseline_digest: "git-sha1:0123456789012345678901234567890123456789".to_owned(),
            required_gates: vec!["tests".to_owned()],
            protected_paths: vec!["SECRET/".to_owned()],
            budget: ResourceBudget {
                wall_seconds: 100,
                max_attempts: 1,
                max_changed_files: 20,
                max_added_lines: 1500,
                max_tool_calls: 10,
                max_input_tokens: 1000,
                max_output_tokens: 1000,
                max_energy_joules: None,
                allow_missing_energy_meter: true,
            },
            created_at: chrono::Utc.with_ymd_and_hms(2026, 9, 1, 12, 0, 0).unwrap(),
        }
    }
}
