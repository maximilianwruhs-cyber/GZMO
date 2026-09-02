//! Baseline-owned Stage-1 trusted policy for the connected repository evolver.
//!
//! One wire type only: later evaluation/PR work extends this module rather than
//! inventing a second policy contract.

use evolution_contracts::{CandidateKind, GateClass, PathPolicy, ResourceBudget};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

/// Policy schema identifier for Stage-1 repository evolver policy artifacts.
pub const POLICY_SCHEMA: &str = "gzmo.repo_evolver.policy/v1";

/// Exact branch prefix required for Stage-1 candidate branches.
pub const REQUIRED_BRANCH_PREFIX: &str = "evolve/";

/// Maximum repair attempts authorized by Stage-1 policy.
pub const MAX_REPAIR_ATTEMPTS: u8 = 2;

/// Maximum gate command timeout in seconds.
pub const MAX_GATE_TIMEOUT_SECONDS: u64 = 3600;

/// Errors raised while parsing or validating a trusted policy artifact.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PolicyParseError {
    /// TOML bytes could not be decoded into the raw policy form.
    #[error("invalid policy toml: {0}")]
    InvalidToml(String),
    /// Parsed values failed Stage-1 structural or authority checks.
    #[error("invalid policy: {0}")]
    Invalid(String),
    /// Canonical digest could not be computed from the validated policy.
    #[error("policy digest error: {0}")]
    Digest(String),
}

/// One trusted gate command selected by baseline-owned policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GateCommand {
    name: String,
    class: GateClass,
    argv: Vec<String>,
    timeout_seconds: u64,
}

impl GateCommand {
    /// Gate name used in evaluation reports and required-floor lists.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gate class (hard floor or metric).
    pub fn class(&self) -> GateClass {
        self.class
    }

    /// Argument vector executed by the trusted evaluator.
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// Per-gate wall-clock timeout in seconds.
    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}

/// Immutable baseline-owned Stage-1 candidate policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrustedPolicy {
    schema: String,
    owner: String,
    repository: String,
    candidate_kind: CandidateKind,
    max_active_candidates: u8,
    max_repair_attempts: u8,
    allowed_branch_prefix: String,
    budget: ResourceBudget,
    protected_paths: PathPolicy,
    gates: Vec<GateCommand>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTrustedPolicy {
    schema: String,
    owner: String,
    repository: String,
    candidate_kind: CandidateKind,
    max_active_candidates: u8,
    max_repair_attempts: u8,
    allowed_branch_prefix: String,
    budget: ResourceBudget,
    protected_paths: PathPolicy,
    gates: Vec<RawGateCommand>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGateCommand {
    name: String,
    class: GateClass,
    argv: Vec<String>,
    timeout_seconds: u64,
}

impl TrustedPolicy {
    /// Parse and validate a baseline-owned policy TOML document.
    pub fn parse_toml(bytes: &[u8]) -> Result<Self, PolicyParseError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|err| PolicyParseError::InvalidToml(err.to_string()))?;
        let raw: RawTrustedPolicy = toml::from_str(text)
            .map_err(|err| PolicyParseError::InvalidToml(err.to_string()))?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: RawTrustedPolicy) -> Result<Self, PolicyParseError> {
        if raw.schema != POLICY_SCHEMA {
            return Err(PolicyParseError::Invalid(format!(
                "schema must be {POLICY_SCHEMA}, got {:?}",
                raw.schema
            )));
        }
        validate_owner_or_repository("owner", &raw.owner)?;
        validate_owner_or_repository("repository", &raw.repository)?;

        match raw.candidate_kind {
            CandidateKind::Code | CandidateKind::ProceduralSkill => {}
            other => {
                return Err(PolicyParseError::Invalid(format!(
                    "candidate_kind must be code or procedural_skill, got {other}"
                )));
            }
        }

        if raw.max_active_candidates != 1 {
            return Err(PolicyParseError::Invalid(format!(
                "max_active_candidates must be 1, got {}",
                raw.max_active_candidates
            )));
        }
        if raw.max_repair_attempts == 0 || raw.max_repair_attempts > MAX_REPAIR_ATTEMPTS {
            return Err(PolicyParseError::Invalid(format!(
                "max_repair_attempts must be 1..={MAX_REPAIR_ATTEMPTS}, got {}",
                raw.max_repair_attempts
            )));
        }
        if raw.allowed_branch_prefix != REQUIRED_BRANCH_PREFIX {
            return Err(PolicyParseError::Invalid(format!(
                "allowed_branch_prefix must be {REQUIRED_BRANCH_PREFIX:?}, got {:?}",
                raw.allowed_branch_prefix
            )));
        }

        raw.budget
            .validate()
            .map_err(|err| PolicyParseError::Invalid(err.to_string()))?;
        raw.protected_paths
            .validate()
            .map_err(|err| PolicyParseError::Invalid(err.to_string()))?;

        if raw.gates.is_empty() {
            return Err(PolicyParseError::Invalid(
                "gates must contain at least one entry".to_owned(),
            ));
        }

        let mut names = BTreeSet::new();
        let mut hard_floor_count = 0usize;
        let mut gates = Vec::with_capacity(raw.gates.len());
        for gate in raw.gates {
            let command = validate_gate(gate)?;
            if !names.insert(command.name.clone()) {
                return Err(PolicyParseError::Invalid(format!(
                    "duplicate gate name {:?}",
                    command.name
                )));
            }
            if command.class == GateClass::HardFloor {
                hard_floor_count += 1;
            }
            gates.push(command);
        }
        if hard_floor_count == 0 {
            return Err(PolicyParseError::Invalid(
                "gates must include at least one hard_floor".to_owned(),
            ));
        }

        Ok(Self {
            schema: raw.schema,
            owner: raw.owner,
            repository: raw.repository,
            candidate_kind: raw.candidate_kind,
            max_active_candidates: raw.max_active_candidates,
            max_repair_attempts: raw.max_repair_attempts,
            allowed_branch_prefix: raw.allowed_branch_prefix,
            budget: raw.budget,
            protected_paths: raw.protected_paths,
            gates,
        })
    }

    /// Schema identifier.
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Target repository owner.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Target repository name.
    pub fn repository(&self) -> &str {
        &self.repository
    }

    /// Authorized candidate kind.
    pub fn candidate_kind(&self) -> CandidateKind {
        self.candidate_kind
    }

    /// Maximum simultaneously active candidates (Stage 1: always 1).
    pub fn max_active_candidates(&self) -> u8 {
        self.max_active_candidates
    }

    /// Maximum evidence-bounded repair attempts.
    pub fn max_repair_attempts(&self) -> u8 {
        self.max_repair_attempts
    }

    /// Required candidate branch prefix.
    pub fn allowed_branch_prefix(&self) -> &str {
        &self.allowed_branch_prefix
    }

    /// Signed resource budget.
    pub fn budget(&self) -> &ResourceBudget {
        &self.budget
    }

    /// Protected path policy.
    pub fn protected_paths(&self) -> &PathPolicy {
        &self.protected_paths
    }

    /// Trusted gate commands.
    pub fn gates(&self) -> &[GateCommand] {
        &self.gates
    }

    /// Names of every HardFloor gate, in policy order.
    pub fn required_hard_floor_names(&self) -> Vec<&str> {
        self.gates
            .iter()
            .filter(|gate| gate.class == GateClass::HardFloor)
            .map(|gate| gate.name.as_str())
            .collect()
    }

    /// Canonical `sha256:<64 lowercase hex>` digest over validated typed JSON.
    pub fn digest(&self) -> Result<String, PolicyParseError> {
        let json = serde_json::to_vec(self)
            .map_err(|err| PolicyParseError::Digest(err.to_string()))?;
        let hash = Sha256::digest(&json);
        let mut hex = String::with_capacity(64);
        for byte in hash {
            use std::fmt::Write as _;
            let _ = write!(hex, "{byte:02x}");
        }
        Ok(format!("sha256:{hex}"))
    }
}

fn validate_gate(raw: RawGateCommand) -> Result<GateCommand, PolicyParseError> {
    validate_gate_name(&raw.name)?;
    if raw.argv.is_empty() {
        return Err(PolicyParseError::Invalid(format!(
            "gate {:?} argv must be nonempty",
            raw.name
        )));
    }
    if !(1..=MAX_GATE_TIMEOUT_SECONDS).contains(&raw.timeout_seconds) {
        return Err(PolicyParseError::Invalid(format!(
            "gate {:?} timeout_seconds must be 1..={MAX_GATE_TIMEOUT_SECONDS}, got {}",
            raw.name, raw.timeout_seconds
        )));
    }
    for (index, arg) in raw.argv.iter().enumerate() {
        if arg.is_empty() {
            return Err(PolicyParseError::Invalid(format!(
                "gate {:?} argv[{index}] must be nonempty",
                raw.name
            )));
        }
        if arg == "-c" {
            return Err(PolicyParseError::Invalid(format!(
                "gate {:?} argv must not contain -c",
                raw.name
            )));
        }
        if arg.contains('\0') || arg.contains('\n') || arg.contains('\r') {
            return Err(PolicyParseError::Invalid(format!(
                "gate {:?} argv contains control characters",
                raw.name
            )));
        }
        // Reject shell-interpreted command strings: argv is already an array, so
        // a single token that embeds shell metacharacters for composition is denied.
        if index == 0 && looks_like_shell_command_string(arg) {
            return Err(PolicyParseError::Invalid(format!(
                "gate {:?} argv[0] must be a bare executable, not a shell command string",
                raw.name
            )));
        }
        if contains_shell_metacharacters(arg) {
            return Err(PolicyParseError::Invalid(format!(
                "gate {:?} argv must not embed shell metacharacters",
                raw.name
            )));
        }
    }
    Ok(GateCommand {
        name: raw.name,
        class: raw.class,
        argv: raw.argv,
        timeout_seconds: raw.timeout_seconds,
    })
}

fn validate_gate_name(name: &str) -> Result<(), PolicyParseError> {
    if name.is_empty() {
        return Err(PolicyParseError::Invalid(
            "gate name must be nonempty".to_owned(),
        ));
    }
    let bytes = name.as_bytes();
    let first = bytes[0];
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err(PolicyParseError::Invalid(format!(
            "gate name must start with lowercase alphanumeric: {name:?}"
        )));
    }
    if bytes.len() > 1 {
        let last = bytes[bytes.len() - 1];
        if !(last.is_ascii_lowercase() || last.is_ascii_digit()) {
            return Err(PolicyParseError::Invalid(format!(
                "gate name must end with lowercase alphanumeric: {name:?}"
            )));
        }
    }
    if !bytes.iter().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'.' || *b == b'_' || *b == b'-'
    }) {
        return Err(PolicyParseError::Invalid(format!(
            "gate name contains unsafe characters: {name:?}"
        )));
    }
    Ok(())
}

fn validate_owner_or_repository(field: &str, value: &str) -> Result<(), PolicyParseError> {
    if value.is_empty() {
        return Err(PolicyParseError::Invalid(format!(
            "{field} must be nonempty"
        )));
    }
    if value != value.trim() {
        return Err(PolicyParseError::Invalid(format!(
            "{field} must not have leading or trailing whitespace"
        )));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(PolicyParseError::Invalid(format!(
            "{field} must not contain path separators"
        )));
    }
    if value.contains("..") {
        return Err(PolicyParseError::Invalid(format!(
            "{field} must not contain .."
        )));
    }
    if value
        .bytes()
        .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
    {
        return Err(PolicyParseError::Invalid(format!(
            "{field} must not contain whitespace or control characters"
        )));
    }
    if value.starts_with('-') || value.starts_with('.') {
        return Err(PolicyParseError::Invalid(format!(
            "{field} must not begin with '-' or '.'"
        )));
    }
    Ok(())
}

fn looks_like_shell_command_string(arg: &str) -> bool {
    arg.contains(' ') || arg.contains(';') || arg.contains('|') || arg.contains('&')
}

fn contains_shell_metacharacters(arg: &str) -> bool {
    arg.bytes().any(|b| {
        matches!(
            b,
            b';' | b'|' | b'&' | b'>' | b'<' | b'$' | b'`' | b'\n' | b'\r'
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use evolution_contracts::DEFAULT_PROTECTED;

    const VALID_POLICY_A: &str = r#"
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

    const VALID_POLICY_REORDERED: &str = r#"
repository = "GZMO"
owner = "maximilianwruhs-cyber"
schema = "gzmo.repo_evolver.policy/v1"
allowed_branch_prefix = "evolve/"
max_repair_attempts = 2
max_active_candidates = 1
candidate_kind = "code"

[[gates]]
timeout_seconds = 300
class = "hard_floor"
name = "format"
argv = ["cargo", "fmt", "--all", "--", "--check"]

[[gates]]
name = "clippy"
timeout_seconds = 900
argv = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
class = "hard_floor"

[[gates]]
name = "tests"
class = "hard_floor"
timeout_seconds = 1800
argv = ["cargo", "test", "--all"]

[[gates]]
class = "hard_floor"
name = "opportunity-contract"
timeout_seconds = 300
argv = ["bash", "scripts/opportunity-discovery-check.sh"]

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

[budget]
allow_missing_energy_meter = true
max_output_tokens = 50000
max_input_tokens = 250000
max_tool_calls = 80
max_added_lines = 1500
max_changed_files = 20
max_attempts = 1
wall_seconds = 2700
"#;

    #[test]
    fn policy_digest_is_canonical_and_policy_is_bounded() {
        let first = TrustedPolicy::parse_toml(VALID_POLICY_A.as_bytes()).unwrap();
        let second = TrustedPolicy::parse_toml(VALID_POLICY_REORDERED.as_bytes()).unwrap();
        assert_eq!(first.digest().unwrap(), second.digest().unwrap());
        assert!(first.budget().validate().is_ok());
        assert!(first.protected_paths().check("Cargo.toml").is_err());
        assert!(first
            .protected_paths()
            .check("config/repo-evolver.policy.toml")
            .is_err());
        assert_eq!(
            first.required_hard_floor_names(),
            vec!["format", "clippy", "tests", "opportunity-contract"]
        );
        assert_eq!(first.max_active_candidates(), 1);
        assert_eq!(first.max_repair_attempts(), 2);
        assert_eq!(first.allowed_branch_prefix(), REQUIRED_BRANCH_PREFIX);
        assert_eq!(first.owner(), "maximilianwruhs-cyber");
        assert_eq!(first.repository(), "GZMO");
        assert_eq!(first.candidate_kind(), CandidateKind::Code);
        assert!(first.digest().unwrap().starts_with("sha256:"));
        assert_eq!(first.digest().unwrap().len(), "sha256:".len() + 64);

        // Reordered protected path lists still protect the same set, but change
        // typed JSON order and therefore the digest. Canonicalization covers TOML
        // key/table order for an identical typed value.
        for path in DEFAULT_PROTECTED {
            assert!(first.protected_paths().check(path).is_err());
        }
    }

    #[test]
    fn rejects_invalid_policy_variants() {
        assert!(TrustedPolicy::parse_toml(b"not = [toml").is_err());

        let mut bad = VALID_POLICY_A.replace(
            r#"schema = "gzmo.repo_evolver.policy/v1""#,
            r#"schema = "wrong/v1""#,
        );
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace("max_active_candidates = 1", "max_active_candidates = 2");
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace("max_repair_attempts = 2", "max_repair_attempts = 3");
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace(
            r#"allowed_branch_prefix = "evolve/""#,
            r#"allowed_branch_prefix = "feature/""#,
        );
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace(r#"candidate_kind = "code""#, r#"candidate_kind = "memory""#);
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace(
            r#"argv = ["cargo", "fmt", "--all", "--", "--check"]"#,
            r#"argv = ["bash", "-c", "cargo fmt"]"#,
        );
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace("timeout_seconds = 300", "timeout_seconds = 0");
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace(
            "allow_missing_energy_meter = true",
            "allow_missing_energy_meter = false",
        );
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = VALID_POLICY_A.replace(
            r#"class = "hard_floor""#,
            r#"class = "metric""#,
        );
        // All four gates flipped to metric → no hard floor.
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());

        bad = format!(
            "{VALID_POLICY_A}\nunknown_authority = true\n"
        );
        assert!(TrustedPolicy::parse_toml(bad.as_bytes()).is_err());
    }

    #[test]
    fn tracked_default_policy_parses() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../config/repo-evolver.policy.toml"
        ))
        .expect("tracked policy must exist");
        let policy = TrustedPolicy::parse_toml(&bytes).expect("tracked policy must validate");
        assert_eq!(policy.schema(), POLICY_SCHEMA);
        assert_eq!(policy.budget().wall_seconds, 2700);
        assert_eq!(policy.budget().max_attempts, 1);
        assert_eq!(policy.budget().max_changed_files, 20);
        assert_eq!(policy.budget().max_added_lines, 1500);
        assert_eq!(policy.budget().max_tool_calls, 80);
        assert_eq!(policy.budget().max_input_tokens, 250_000);
        assert_eq!(policy.budget().max_output_tokens, 50_000);
        assert!(policy.budget().max_energy_joules.is_none());
        assert!(policy.budget().allow_missing_energy_meter);
        assert!(policy
            .protected_paths()
            .check("config/repo-evolver.policy.toml")
            .is_err());
    }
}
