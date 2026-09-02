//! Machine-local placement configuration for the connected repository evolver.
//!
//! Configuration carries placement only. Budgets, gates, branch prefix, candidate
//! kind, and worker authority live in [`crate::policy::TrustedPolicy`] or fixed code.

use crate::policy::{PolicyParseError, TrustedPolicy};
use serde::Deserialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

/// Errors raised while loading machine-local evolver configuration.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// Configuration file could not be read.
    #[error("config read error: {0}")]
    Io(String),
    /// TOML bytes could not be decoded into the raw config form.
    #[error("invalid config toml: {0}")]
    InvalidToml(String),
    /// Parsed placement values failed validation.
    #[error("invalid config: {0}")]
    Invalid(String),
    /// Working-tree policy failed to parse or mismatched configured identity.
    #[error("policy error: {0}")]
    Policy(String),
}

impl From<PolicyParseError> for ConfigError {
    fn from(value: PolicyParseError) -> Self {
        Self::Policy(value.to_string())
    }
}

/// Fully validated machine placement plus loaded working-tree policy.
#[derive(Debug, Clone)]
pub struct RepoEvolverConfig {
    repo: RepoConfig,
    state_dir: PathBuf,
    mission: MissionConfig,
    worker: WorkerConfig,
    policy: PolicyConfig,
    working_policy: TrustedPolicy,
    working_policy_digest: String,
}

/// Repository identity and checkout placement.
#[derive(Debug, Clone)]
pub struct RepoConfig {
    path: PathBuf,
    remote: String,
    base_branch: String,
    owner: String,
    repository: String,
}

/// Mission artifact relative paths and refresh argv.
#[derive(Debug, Clone)]
pub struct MissionConfig {
    json_rel: PathBuf,
    markdown_rel: PathBuf,
    refresh_argv: Vec<String>,
}

/// Fixed worker executable and profile (no arbitrary argv).
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    executable: PathBuf,
    profile: String,
}

/// Relative path to the baseline-owned policy file under the repository.
#[derive(Debug, Clone)]
pub struct PolicyConfig {
    repo_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRepoEvolverConfig {
    state_dir: String,
    repo: RawRepoConfig,
    mission: RawMissionConfig,
    worker: RawWorkerConfig,
    policy: RawPolicyConfig,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRepoConfig {
    path: String,
    remote: String,
    base_branch: String,
    owner: String,
    repository: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMissionConfig {
    json_rel: String,
    markdown_rel: String,
    refresh_argv: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorkerConfig {
    executable: String,
    profile: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicyConfig {
    repo_path: String,
}

impl RepoEvolverConfig {
    /// Load, validate, and seal machine placement configuration.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|err| ConfigError::Io(err.to_string()))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|err| ConfigError::InvalidToml(err.to_string()))?;
        let raw: RawRepoEvolverConfig = toml::from_str(text)
            .map_err(|err| ConfigError::InvalidToml(err.to_string()))?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: RawRepoEvolverConfig) -> Result<Self, ConfigError> {
        let repo_path = require_existing_absolute_path("repo.path", &raw.repo.path)?;
        let worker_executable =
            require_existing_absolute_path("worker.executable", &raw.worker.executable)?;

        let state_dir = parse_absolute_lexical_path("state_dir", &raw.state_dir)?;
        ensure_state_outside_repo(&state_dir, &repo_path)?;

        let remote = validate_safe_identifier("repo.remote", &raw.repo.remote)?;
        let base_branch = validate_safe_git_ref("repo.base_branch", &raw.repo.base_branch)?;
        let owner = validate_safe_identifier("repo.owner", &raw.repo.owner)?;
        let repository = validate_safe_identifier("repo.repository", &raw.repo.repository)?;
        let profile = validate_safe_identifier("worker.profile", &raw.worker.profile)?;

        let json_rel = normalize_relative_path("mission.json_rel", &raw.mission.json_rel)?;
        let markdown_rel =
            normalize_relative_path("mission.markdown_rel", &raw.mission.markdown_rel)?;
        let refresh_argv = validate_refresh_argv(&raw.mission.refresh_argv)?;
        let policy_repo_path =
            normalize_relative_path("policy.repo_path", &raw.policy.repo_path)?;

        let policy_abs = repo_path.join(&policy_repo_path);
        if !policy_abs.is_file() {
            return Err(ConfigError::Invalid(format!(
                "policy.repo_path does not exist in repository working tree: {}",
                policy_abs.display()
            )));
        }
        let policy_bytes =
            fs::read(&policy_abs).map_err(|err| ConfigError::Io(err.to_string()))?;
        let working_policy = TrustedPolicy::parse_toml(&policy_bytes)?;
        if working_policy.owner() != owner {
            return Err(ConfigError::Invalid(format!(
                "config repo.owner {owner:?} does not match policy owner {:?}",
                working_policy.owner()
            )));
        }
        if working_policy.repository() != repository {
            return Err(ConfigError::Invalid(format!(
                "config repo.repository {repository:?} does not match policy repository {:?}",
                working_policy.repository()
            )));
        }
        let working_policy_digest = working_policy.digest()?;

        Ok(Self {
            repo: RepoConfig {
                path: repo_path,
                remote,
                base_branch,
                owner,
                repository,
            },
            state_dir,
            mission: MissionConfig {
                json_rel,
                markdown_rel,
                refresh_argv,
            },
            worker: WorkerConfig {
                executable: worker_executable,
                profile,
            },
            policy: PolicyConfig {
                repo_path: policy_repo_path,
            },
            working_policy,
            working_policy_digest,
        })
    }

    /// Repository placement and identity.
    pub fn repo(&self) -> &RepoConfig {
        &self.repo
    }

    /// Absolute coordinator state directory (need not exist yet).
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Mission artifact placement and refresh argv.
    pub fn mission(&self) -> &MissionConfig {
        &self.mission
    }

    /// Fixed worker executable and profile.
    pub fn worker(&self) -> &WorkerConfig {
        &self.worker
    }

    /// Relative policy path under the repository.
    pub fn policy(&self) -> &PolicyConfig {
        &self.policy
    }

    /// Working-tree trusted policy loaded during config validation.
    pub fn working_policy(&self) -> &TrustedPolicy {
        &self.working_policy
    }

    /// Canonical digest of the working-tree trusted policy.
    pub fn working_policy_digest(&self) -> &str {
        &self.working_policy_digest
    }
}

impl RepoConfig {
    /// Canonical absolute repository path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Git remote name.
    pub fn remote(&self) -> &str {
        &self.remote
    }

    /// Base branch name.
    pub fn base_branch(&self) -> &str {
        &self.base_branch
    }

    /// GitHub owner.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// GitHub repository name.
    pub fn repository(&self) -> &str {
        &self.repository
    }
}

impl MissionConfig {
    /// Mission JSON path relative to `GZMO_DATA_NEXT`.
    pub fn json_rel(&self) -> &Path {
        &self.json_rel
    }

    /// Mission Markdown path relative to `GZMO_DATA_NEXT`.
    pub fn markdown_rel(&self) -> &Path {
        &self.markdown_rel
    }

    /// Validated refresh argument vector.
    pub fn refresh_argv(&self) -> &[String] {
        &self.refresh_argv
    }
}

impl WorkerConfig {
    /// Canonical absolute OMP executable path.
    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Worker profile name.
    pub fn profile(&self) -> &str {
        &self.profile
    }
}

impl PolicyConfig {
    /// Policy path relative to the repository root.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }
}

fn require_existing_absolute_path(field: &str, raw: &str) -> Result<PathBuf, ConfigError> {
    if raw.trim().is_empty() {
        return Err(ConfigError::Invalid(format!("{field} must be nonempty")));
    }
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(ConfigError::Invalid(format!(
            "{field} must be an absolute path, got {raw:?}"
        )));
    }
    if raw.contains('\0') {
        return Err(ConfigError::Invalid(format!(
            "{field} must not contain NUL"
        )));
    }
    let meta = fs::metadata(&path).map_err(|err| {
        ConfigError::Invalid(format!(
            "{field} must exist and be accessible ({raw}): {err}"
        ))
    })?;
    if path
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        // Still canonicalize below; parent components are resolved only when the path exists.
    }
    let canonical = fs::canonicalize(&path).map_err(|err| {
        ConfigError::Invalid(format!(
            "{field} must canonicalize to an absolute path ({raw}): {err}"
        ))
    })?;
    if !canonical.is_absolute() {
        return Err(ConfigError::Invalid(format!(
            "{field} canonical path is not absolute: {}",
            canonical.display()
        )));
    }
    // Keep files and directories; caller decides which.
    let _ = meta;
    Ok(canonical)
}

fn parse_absolute_lexical_path(field: &str, raw: &str) -> Result<PathBuf, ConfigError> {
    if raw.trim().is_empty() {
        return Err(ConfigError::Invalid(format!("{field} must be nonempty")));
    }
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(ConfigError::Invalid(format!(
            "{field} must be an absolute path, got {raw:?}"
        )));
    }
    if raw.contains('\0') {
        return Err(ConfigError::Invalid(format!(
            "{field} must not contain NUL"
        )));
    }
    let normalized = normalize_absolute_lexical(&path).map_err(|err| {
        ConfigError::Invalid(format!("{field} is not lexically safe: {err}"))
    })?;
    Ok(normalized)
}

fn normalize_absolute_lexical(path: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::new();
    let mut components = path.components();
    match components.next() {
        Some(Component::RootDir) => out.push(Component::RootDir.as_os_str()),
        Some(Component::Prefix(prefix)) => {
            out.push(prefix.as_os_str());
            match components.next() {
                Some(Component::RootDir) => out.push(Component::RootDir.as_os_str()),
                Some(other) => {
                    return Err(format!(
                        "absolute path must have root after prefix, got {other:?}"
                    ));
                }
                None => {
                    return Err("absolute path prefix missing root".to_owned());
                }
            }
        }
        other => {
            return Err(format!("expected absolute path, got {other:?}"));
        }
    }
    for component in components {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err("path must not contain '..'".to_owned());
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("path must not contain additional roots/prefixes".to_owned());
            }
        }
    }
    if out.as_os_str().is_empty() {
        return Err("normalized path is empty".to_owned());
    }
    Ok(out)
}

fn ensure_state_outside_repo(state_dir: &Path, repo_path: &Path) -> Result<(), ConfigError> {
    if state_dir == repo_path {
        return Err(ConfigError::Invalid(
            "state_dir must be outside the repository path".to_owned(),
        ));
    }
    if state_dir.starts_with(repo_path) {
        return Err(ConfigError::Invalid(format!(
            "state_dir {} is inside repository {}",
            state_dir.display(),
            repo_path.display()
        )));
    }
    Ok(())
}

fn normalize_relative_path(field: &str, raw: &str) -> Result<PathBuf, ConfigError> {
    if raw.trim().is_empty() {
        return Err(ConfigError::Invalid(format!("{field} must be nonempty")));
    }
    if raw.starts_with('/')
        || raw.starts_with('\\')
        || raw.contains('\0')
        || looks_like_windows_abs(raw)
    {
        return Err(ConfigError::Invalid(format!(
            "{field} must be a relative path without absolute/parent/symlink syntax, got {raw:?}"
        )));
    }
    if raw.contains("://") {
        return Err(ConfigError::Invalid(format!(
            "{field} must not be a URL-like path, got {raw:?}"
        )));
    }
    let path = Path::new(raw);
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let text = part.to_string_lossy();
                if text.contains('\0') {
                    return Err(ConfigError::Invalid(format!(
                        "{field} component contains NUL"
                    )));
                }
                out.push(part);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(ConfigError::Invalid(format!(
                    "{field} must not contain '..'"
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(ConfigError::Invalid(format!(
                    "{field} must be relative, got {raw:?}"
                )));
            }
        }
    }
    if out.as_os_str().is_empty() {
        return Err(ConfigError::Invalid(format!(
            "{field} normalized to empty path"
        )));
    }
    // Reject symlink-looking path tokens (no leading/trailing slash tricks).
    if raw.ends_with('/') || raw.ends_with('\\') {
        return Err(ConfigError::Invalid(format!(
            "{field} must not end with a path separator"
        )));
    }
    Ok(out)
}

fn looks_like_windows_abs(raw: &str) -> bool {
    let bytes = raw.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn validate_refresh_argv(argv: &[String]) -> Result<Vec<String>, ConfigError> {
    if argv.len() != 2 {
        return Err(ConfigError::Invalid(
            "mission.refresh_argv must be exactly [\"bash\", \"scripts/opportunity-next-mission.sh\"]"
                .to_owned(),
        ));
    }
    if argv[0] != "bash" {
        return Err(ConfigError::Invalid(format!(
            "mission.refresh_argv[0] must be \"bash\", got {:?}",
            argv[0]
        )));
    }
    if argv[1] != "scripts/opportunity-next-mission.sh" {
        return Err(ConfigError::Invalid(format!(
            "mission.refresh_argv[1] must be \"scripts/opportunity-next-mission.sh\", got {:?}",
            argv[1]
        )));
    }
    if argv.iter().any(|a| a == "-c") {
        return Err(ConfigError::Invalid(
            "mission.refresh_argv must not contain -c".to_owned(),
        ));
    }
    Ok(argv.to_vec())
}

fn validate_safe_identifier(field: &str, value: &str) -> Result<String, ConfigError> {
    if value.is_empty() {
        return Err(ConfigError::Invalid(format!("{field} must be nonempty")));
    }
    if value != value.trim() {
        return Err(ConfigError::Invalid(format!(
            "{field} must not have leading or trailing whitespace"
        )));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(ConfigError::Invalid(format!(
            "{field} must not contain path separators"
        )));
    }
    if value.contains("..") {
        return Err(ConfigError::Invalid(format!(
            "{field} must not contain .."
        )));
    }
    if value
        .bytes()
        .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
    {
        return Err(ConfigError::Invalid(format!(
            "{field} must not contain whitespace or control characters"
        )));
    }
    if value.starts_with('-') || value.starts_with('.') {
        return Err(ConfigError::Invalid(format!(
            "{field} must not begin with '-' or '.'"
        )));
    }
    Ok(value.to_owned())
}

fn validate_safe_git_ref(field: &str, value: &str) -> Result<String, ConfigError> {
    if value.is_empty() {
        return Err(ConfigError::Invalid(format!("{field} must be nonempty")));
    }
    if value != value.trim() {
        return Err(ConfigError::Invalid(format!(
            "{field} must not have leading or trailing whitespace"
        )));
    }
    if value.starts_with('/') || value.ends_with('/') || value.contains("//") {
        return Err(ConfigError::Invalid(format!(
            "{field} has invalid slash placement"
        )));
    }
    if value.contains("..") || value.contains("@{") || value == "@" {
        return Err(ConfigError::Invalid(format!(
            "{field} contains unsafe git-ref syntax"
        )));
    }
    for component in value.split('/') {
        if component.is_empty()
            || component.starts_with('.')
            || component.starts_with('-')
            || component.ends_with('.')
            || component.ends_with(".lock")
        {
            return Err(ConfigError::Invalid(format!(
                "{field} component is unsafe: {component:?}"
            )));
        }
        if component
            .bytes()
            .any(|b| b.is_ascii_whitespace() || b.is_ascii_control() || matches!(b, b'\\' | b'~' | b'^' | b':' | b'?' | b'*' | b'[' | b']'))
        {
            return Err(ConfigError::Invalid(format!(
                "{field} component contains unsafe characters: {component:?}"
            )));
        }
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    use std::os::unix::fs::OpenOptionsExt;
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

    struct ConfigFixture {
        _root: TempDir,
        repo: PathBuf,
        state_dir: PathBuf,
        worker: PathBuf,
        valid_config: PathBuf,
        policy_path: PathBuf,
    }

    impl ConfigFixture {
        fn new() -> Self {
            let root = TempDir::new().expect("tempdir");
            let repo = root.path().join("repo");
            let state_dir = root.path().join("state");
            let worker = root.path().join("omp");
            fs::create_dir_all(repo.join("config")).unwrap();
            fs::create_dir_all(repo.join("scripts")).unwrap();
            File::create(repo.join("scripts/opportunity-next-mission.sh")).unwrap();
            let policy_path = repo.join("config/repo-evolver.policy.toml");
            fs::write(&policy_path, POLICY_TOML).unwrap();

            let mut opts = fs::OpenOptions::new();
            opts.write(true).create(true).mode(0o755);
            opts.open(&worker).unwrap();

            let valid_config = root.path().join("repo-evolver.toml");
            let fixture = Self {
                _root: root,
                repo,
                state_dir,
                worker,
                valid_config,
                policy_path,
            };
            fixture.write_valid_config();
            fixture
        }

        fn write_valid_config(&self) {
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
                state = self.state_dir.display(),
                repo = self.repo.display(),
                worker = self.worker.display(),
            );
            fs::write(&self.valid_config, body).unwrap();
        }

        fn write_config_with_state(&self, state: PathBuf) {
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
                state = state.display(),
                repo = self.repo.display(),
                worker = self.worker.display(),
            );
            fs::write(&self.valid_config, body).unwrap();
        }

        fn write_config_with_unknown_worker_field(&self, key: &str, value: &str) {
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
{key} = "{value}"

[policy]
repo_path = "config/repo-evolver.policy.toml"
"#,
                state = self.state_dir.display(),
                repo = self.repo.display(),
                worker = self.worker.display(),
                key = key,
                value = value,
            );
            fs::write(&self.valid_config, body).unwrap();
        }

        fn rewrite_raw(&self, mutate: impl FnOnce(&mut String)) {
            let mut body = fs::read_to_string(&self.valid_config).unwrap();
            mutate(&mut body);
            fs::write(&self.valid_config, body).unwrap();
        }
    }

    #[test]
    fn rejects_state_inside_repo_and_arbitrary_worker_argv() {
        let fixture = ConfigFixture::new();
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_ok());

        fixture.write_config_with_state(fixture.repo.join("data-next/evolver"));
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());

        fixture.write_valid_config();
        fixture.write_config_with_unknown_worker_field("argv", "bash -c echo bad");
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());
    }

    #[test]
    fn rejects_lexical_escape_missing_paths_and_unsafe_refresh() {
        let fixture = ConfigFixture::new();

        fixture.write_config_with_state(PathBuf::from("/tmp/gzmo-evolver-state/../escape"));
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());

        fixture.write_valid_config();
        fixture.rewrite_raw(|body| {
            *body = body.replace(
                &format!("executable = \"{}\"", fixture.worker.display()),
                "executable = \"/definitely/missing/omp-binary\"",
            );
        });
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());

        fixture.write_valid_config();
        fs::remove_file(&fixture.policy_path).unwrap();
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());
        fs::write(&fixture.policy_path, POLICY_TOML).unwrap();

        fixture.write_valid_config();
        fixture.rewrite_raw(|body| {
            *body = body.replace(
                r#"refresh_argv = ["bash", "scripts/opportunity-next-mission.sh"]"#,
                r#"refresh_argv = ["bash", "-c", "echo bad"]"#,
            );
        });
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());

        fixture.write_valid_config();
        fixture.rewrite_raw(|body| {
            *body = body.replace(
                r#"json_rel = "opportunity-discovery/next-mission.json""#,
                r#"json_rel = "../escape/next-mission.json""#,
            );
        });
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());

        fixture.write_valid_config();
        fixture.rewrite_raw(|body| {
            *body = body.replace(
                r#"owner = "maximilianwruhs-cyber""#,
                r#"owner = "someone-else""#,
            );
        });
        assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());
    }
    #[test]
    fn loads_normalized_paths_and_policy_digest() {
        let fixture = ConfigFixture::new();
        let cfg = RepoEvolverConfig::load(&fixture.valid_config).unwrap();
        assert!(cfg.repo().path().is_absolute());
        assert!(cfg.worker().executable().is_absolute());
        assert!(cfg.state_dir().is_absolute());
        assert!(!cfg.state_dir().starts_with(cfg.repo().path()));
        assert_eq!(
            cfg.mission().refresh_argv(),
            &[
                "bash".to_owned(),
                "scripts/opportunity-next-mission.sh".to_owned()
            ]
        );
        assert_eq!(cfg.repo().owner(), "maximilianwruhs-cyber");
        assert_eq!(cfg.repo().repository(), "GZMO");
        assert_eq!(
            cfg.working_policy_digest(),
            cfg.working_policy().digest().unwrap()
        );
        assert_eq!(
            cfg.working_policy().required_hard_floor_names(),
            vec!["format", "clippy", "tests", "opportunity-contract"]
        );
    }
}
