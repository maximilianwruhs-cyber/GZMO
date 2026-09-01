//! Candidate identity, kind, authority tier, lifecycle state, target, and manifest.

use crate::{ResourceBudget, CANDIDATE_SCHEMA};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use thiserror::Error;

/// Errors raised while validating evolution contract values.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// Candidate identifier failed structural validation.
    #[error("invalid candidate id: {0}")]
    InvalidCandidateId(String),
    /// Candidate target fields failed structural validation.
    #[error("invalid candidate target: {0}")]
    InvalidTarget(String),
    /// Candidate manifest failed structural validation.
    #[error("invalid candidate manifest: {0}")]
    InvalidManifest(String),
}

/// Strict candidate identifier: `cand-` + lowercase ASCII/digit/hyphen body.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct CandidateId(String);

impl CandidateId {
    /// Parse and validate a candidate identifier.
    pub fn parse(raw: impl AsRef<str>) -> Result<Self, ContractError> {
        let raw = raw.as_ref();
        if !is_valid_candidate_id(raw) {
            return Err(ContractError::InvalidCandidateId(raw.to_owned()));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Borrow the validated identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CandidateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CandidateId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw).map_err(serde::de::Error::custom)
    }
}

impl JsonSchema for CandidateId {
    fn schema_name() -> String {
        "CandidateId".to_owned()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{InstanceType, SchemaObject, StringValidation};
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                min_length: Some(16),
                max_length: Some(96),
                // Body is non-empty alnum edges; hyphens only inside; no dots.
                pattern: Some(r"^cand-[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$".to_owned()),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

fn is_valid_candidate_id(raw: &str) -> bool {
    let len = raw.len();
    if !(16..=96).contains(&len) {
        return false;
    }
    if !raw.starts_with("cand-") {
        return false;
    }
    if raw.contains("..") {
        return false;
    }
    if !raw
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return false;
    }

    // Body after the required `cand-` prefix: no empty body, no edge hyphens.
    let body = &raw["cand-".len()..];
    if body.is_empty() {
        return false;
    }
    if body.starts_with('-') || body.ends_with('-') {
        return false;
    }
    true
}

/// Authority classification for a candidate kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityTier {
    Memory,
    Tunable,
    Candidate,
    Promotion,
    Authority,
}

impl fmt::Display for AuthorityTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Memory => "memory",
            Self::Tunable => "tunable",
            Self::Candidate => "candidate",
            Self::Promotion => "promotion",
            Self::Authority => "authority",
        };
        f.write_str(s)
    }
}

/// Kind of evolutionary change being proposed.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CandidateKind {
    Memory,
    Tunable,
    ProceduralSkill,
    Code,
    Schema,
    Model,
    Runtime,
    Evaluator,
    Security,
    Authority,
}

impl CandidateKind {
    /// Map kind onto its fixed authority tier.
    pub fn authority_tier(self) -> AuthorityTier {
        match self {
            Self::Memory => AuthorityTier::Memory,
            Self::Tunable => AuthorityTier::Tunable,
            Self::Authority | Self::Evaluator | Self::Security => AuthorityTier::Authority,
            Self::ProceduralSkill | Self::Code | Self::Schema | Self::Model | Self::Runtime => {
                AuthorityTier::Candidate
            }
        }
    }
}

impl fmt::Display for CandidateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Memory => "memory",
            Self::Tunable => "tunable",
            Self::ProceduralSkill => "procedural_skill",
            Self::Code => "code",
            Self::Schema => "schema",
            Self::Model => "model",
            Self::Runtime => "runtime",
            Self::Evaluator => "evaluator",
            Self::Security => "security",
            Self::Authority => "authority",
        };
        f.write_str(s)
    }
}

/// Base lifecycle states for a candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CandidateState {
    Observed,
    Prepared,
    Building,
    Evaluating,
    Rejected,
    ReviewReady,
    PromotionPending,
    Soaking,
    Accepted,
    RolledBack,
    Failed,
}

impl CandidateState {
    /// Exact legal lifecycle edges. Terminal states never leave; evaluation,
    /// promotion, and soak cannot be skipped.
    pub fn can_transition_to(self, next: Self) -> bool {
        use CandidateState::*;
        matches!(
            (self, next),
            (Observed, Prepared | Failed)
                | (Prepared, Building | Failed)
                | (Building, Evaluating | Failed)
                | (Evaluating, Rejected | ReviewReady | Failed)
                | (ReviewReady, PromotionPending | Rejected | Failed)
                | (PromotionPending, Soaking | Rejected | Failed)
                | (Soaking, Accepted | RolledBack | Failed)
        )
    }
}

impl fmt::Display for CandidateState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Observed => "observed",
            Self::Prepared => "prepared",
            Self::Building => "building",
            Self::Evaluating => "evaluating",
            Self::Rejected => "rejected",
            Self::ReviewReady => "review_ready",
            Self::PromotionPending => "promotion_pending",
            Self::Soaking => "soaking",
            Self::Accepted => "accepted",
            Self::RolledBack => "rolled_back",
            Self::Failed => "failed",
        };
        f.write_str(s)
    }
}

/// Where a candidate will be built and evaluated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
#[schemars(deny_unknown_fields, title = "CandidateTarget")]
pub enum CandidateTarget {
    Repository {
        #[schemars(length(min = 1))]
        owner: String,
        #[schemars(length(min = 1))]
        repository: String,
        #[schemars(length(min = 1))]
        base_branch: String,
        #[schemars(
            length(min = 1),
            regex(pattern = r"^evolve/cand-[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$")
        )]
        candidate_branch: String,
    },
    Appliance {
        #[schemars(
            length(min = 1),
            regex(pattern = r"^[a-z0-9](?:[a-z0-9_-]*[a-z0-9])?$")
        )]
        node_id: String,
        #[schemars(
            length(min = 1),
            regex(pattern = r"^[a-z0-9](?:[a-z0-9_-]*[a-z0-9])?$")
        )]
        target_class: String,
        inactive_target: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum RawCandidateTarget {
    Repository {
        owner: String,
        repository: String,
        base_branch: String,
        candidate_branch: String,
    },
    Appliance {
        node_id: String,
        target_class: String,
        inactive_target: Option<String>,
    },
}

impl<'de> Deserialize<'de> for CandidateTarget {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawCandidateTarget::deserialize(deserializer)?;
        let value = match raw {
            RawCandidateTarget::Repository {
                owner,
                repository,
                base_branch,
                candidate_branch,
            } => Self::Repository {
                owner,
                repository,
                base_branch,
                candidate_branch,
            },
            RawCandidateTarget::Appliance {
                node_id,
                target_class,
                inactive_target,
            } => Self::Appliance {
                node_id,
                target_class,
                inactive_target,
            },
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl CandidateTarget {
    /// Structural validation independent of a concrete candidate id binding.
    pub fn validate(&self) -> Result<(), ContractError> {
        match self {
            Self::Repository {
                owner,
                repository,
                base_branch,
                candidate_branch,
            } => {
                validate_safe_owner_or_repository("owner", owner)?;
                validate_safe_owner_or_repository("repository", repository)?;
                validate_safe_git_ref("base_branch", base_branch)?;
                validate_safe_git_ref("candidate_branch", candidate_branch)?;
                let Some(id_part) = candidate_branch.strip_prefix("evolve/") else {
                    return Err(ContractError::InvalidTarget(
                        "candidate_branch must start with evolve/".to_owned(),
                    ));
                };
                if id_part.contains('/') {
                    return Err(ContractError::InvalidTarget(
                        "candidate_branch must be evolve/<candidate-id> with no extra path"
                            .to_owned(),
                    ));
                }
                CandidateId::parse(id_part).map_err(|_| {
                    ContractError::InvalidTarget(format!(
                        "candidate_branch must bind a valid candidate id, got {candidate_branch:?}"
                    ))
                })?;
                Ok(())
            }
            Self::Appliance {
                node_id,
                target_class,
                inactive_target,
            } => {
                validate_safe_lower_identifier("node_id", node_id)?;
                validate_safe_lower_identifier("target_class", target_class)?;
                if let Some(inactive) = inactive_target {
                    validate_safe_relative_identifier("inactive_target", inactive)?;
                }
                Ok(())
            }
        }
    }

    /// Validate target fields and repository branch binding to `id`.
    pub fn validate_for_candidate(&self, id: &CandidateId) -> Result<(), ContractError> {
        self.validate()?;
        if let Self::Repository {
            candidate_branch, ..
        } = self
        {
            let expected = format!("evolve/{}", id.as_str());
            if candidate_branch != &expected {
                return Err(ContractError::InvalidTarget(format!(
                    "candidate_branch must equal {expected:?}, got {candidate_branch:?}"
                )));
            }
        }
        Ok(())
    }
}

/// Immutable candidate declaration consumed by later evolution stages.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields, title = "CandidateManifest")]
pub struct CandidateManifest {
    #[schemars(regex(pattern = r"^gzmo\.evolution\.candidate/v1$"))]
    pub schema: String,
    pub id: CandidateId,
    #[schemars(length(min = 1))]
    pub mission_id: String,
    pub kind: CandidateKind,
    pub authority: AuthorityTier,
    pub target: CandidateTarget,
    #[schemars(regex(pattern = r"^(sha256:[a-f0-9]{64}|git-sha1:[a-f0-9]{40})$"))]
    pub baseline_digest: String,
    #[schemars(length(min = 1))]
    pub required_gates: Vec<String>,
    #[schemars(length(min = 1))]
    pub protected_paths: Vec<String>,
    pub budget: ResourceBudget,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCandidateManifest {
    schema: String,
    id: CandidateId,
    mission_id: String,
    kind: CandidateKind,
    authority: AuthorityTier,
    target: CandidateTarget,
    baseline_digest: String,
    required_gates: Vec<String>,
    protected_paths: Vec<String>,
    budget: ResourceBudget,
    created_at: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for CandidateManifest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawCandidateManifest::deserialize(deserializer)?;
        let value = Self {
            schema: raw.schema,
            id: raw.id,
            mission_id: raw.mission_id,
            kind: raw.kind,
            authority: raw.authority,
            target: raw.target,
            baseline_digest: raw.baseline_digest,
            required_gates: raw.required_gates,
            protected_paths: raw.protected_paths,
            budget: raw.budget,
            created_at: raw.created_at,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl CandidateManifest {
    /// Structural validation for an external candidate manifest payload.
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.schema != CANDIDATE_SCHEMA {
            return Err(ContractError::InvalidManifest(format!(
                "schema must be {CANDIDATE_SCHEMA}"
            )));
        }
        validate_safe_mission_id(&self.mission_id)?;
        if self.authority != self.kind.authority_tier() {
            return Err(ContractError::InvalidManifest(format!(
                "authority {:?} does not match kind {} tier {:?}",
                self.authority,
                self.kind,
                self.kind.authority_tier()
            )));
        }
        self.target.validate_for_candidate(&self.id)?;
        validate_baseline_digest(&self.baseline_digest, &self.target)?;
        validate_unique_nonempty_gates(&self.required_gates)?;
        validate_unique_protected_paths(&self.protected_paths)?;
        self.budget
            .validate()
            .map_err(|err| ContractError::InvalidManifest(err.to_string()))?;
        Ok(())
    }
}

fn validate_safe_mission_id(value: &str) -> Result<(), ContractError> {
    if value.is_empty() {
        return Err(ContractError::InvalidManifest(
            "mission_id must be nonempty".to_owned(),
        ));
    }
    validate_safe_token("mission_id", value).map_err(|msg| ContractError::InvalidManifest(msg))
}

/// Owner/repository names: no path separators; cannot repoint the target.
///
/// Allows the exact GitHub special name `.github` (community health files) while
/// still rejecting leading `-`, slash repointing, `..`, trailing dots, and unsafe
/// ref characters for every other value.
fn validate_safe_owner_or_repository(field: &str, value: &str) -> Result<(), ContractError> {
    if value.contains('/') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain '/'"
        )));
    }
    if value == ".github" {
        return Ok(());
    }
    validate_safe_git_ref_component(field, value)
}

/// Branch/ref names: slash-separated components, each independently safe.
fn validate_safe_git_ref(field: &str, value: &str) -> Result<(), ContractError> {
    if value.is_empty() {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must be nonempty"
        )));
    }
    if value != value.trim() {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not have leading or trailing whitespace"
        )));
    }
    if value.starts_with('/') || value.ends_with('/') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not have leading or trailing slash"
        )));
    }
    if value.contains("//") {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain empty components (consecutive slashes)"
        )));
    }
    // Whole-ref checks that also apply across component boundaries.
    if value.contains("..") {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain .."
        )));
    }
    if value.contains("@{") {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain @{{"
        )));
    }
    if value == "@" {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not be bare @"
        )));
    }
    for component in value.split('/') {
        validate_safe_git_ref_component(field, component)?;
    }
    Ok(())
}

fn validate_safe_git_ref_component(field: &str, component: &str) -> Result<(), ContractError> {
    if component.is_empty() {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain empty components"
        )));
    }
    if component == "@" {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain bare @"
        )));
    }
    if component.starts_with('.') || component.starts_with('-') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} component must not begin with '.' or '-': {component:?}"
        )));
    }
    if component.ends_with('.') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} component must not end with '.': {component:?}"
        )));
    }
    if component.ends_with(".lock") {
        return Err(ContractError::InvalidTarget(format!(
            "{field} component must not end with .lock"
        )));
    }
    if component.contains("..") {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain .."
        )));
    }
    if component.contains("@{") {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain @{{"
        )));
    }
    for ch in component.chars() {
        if ch.is_control()
            || ch.is_whitespace()
            || ch == '\\'
            || ch == ':'
            || ch == '~'
            || ch == '^'
            || ch == '?'
            || ch == '*'
            || ch == '['
            || ch == '/'
        {
            return Err(ContractError::InvalidTarget(format!(
                "{field} contains forbidden character {ch:?} in {component:?}"
            )));
        }
    }
    Ok(())
}

fn validate_safe_token(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must be nonempty"));
    }
    if value != value.trim() {
        return Err(format!(
            "{field} must not have leading or trailing whitespace"
        ));
    }
    if value.starts_with('/') || value.ends_with('/') {
        return Err(format!("{field} must not have leading or trailing slash"));
    }
    if value.contains("..") {
        return Err(format!("{field} must not contain .."));
    }
    if value.contains("@{") {
        return Err(format!("{field} must not contain @{{"));
    }
    if value.ends_with(".lock") || value.split('/').any(|part| part.ends_with(".lock")) {
        return Err(format!("{field} must not end with .lock"));
    }
    for ch in value.chars() {
        if ch.is_control()
            || ch.is_whitespace()
            || ch == '\\'
            || ch == ':'
            || ch == '~'
            || ch == '^'
            || ch == '?'
            || ch == '*'
            || ch == '['
        {
            return Err(format!(
                "{field} contains forbidden character {ch:?} in {value:?}"
            ));
        }
    }
    Ok(())
}

fn validate_safe_lower_identifier(field: &str, value: &str) -> Result<(), ContractError> {
    if value.is_empty() {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must be nonempty"
        )));
    }
    if value != value.trim() {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not have leading or trailing whitespace"
        )));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain path separators"
        )));
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must be lowercase ascii [a-z0-9_-], got {value:?}"
        )));
    }
    if value.starts_with('-') || value.ends_with('-') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not start or end with hyphen"
        )));
    }
    Ok(())
}

fn validate_safe_relative_identifier(field: &str, value: &str) -> Result<(), ContractError> {
    if value.is_empty() {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must be nonempty when present"
        )));
    }
    if value.starts_with('/')
        || value.starts_with('\\')
        || value.ends_with('/')
        || value.ends_with('\\')
    {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must be a safe relative identifier, got {value:?}"
        )));
    }
    if value.contains("..") || value.contains('\0') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not escape via .. or NUL, got {value:?}"
        )));
    }
    if value != value.trim() || value.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain control or whitespace characters"
        )));
    }
    if value.contains('\\') || value.contains(':') {
        return Err(ContractError::InvalidTarget(format!(
            "{field} must not contain backslash or colon"
        )));
    }
    let unified = value.replace('\\', "/");
    for part in unified.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(ContractError::InvalidTarget(format!(
                "{field} has empty or relative component in {value:?}"
            )));
        }
        if !part
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
        {
            return Err(ContractError::InvalidTarget(format!(
                "{field} components must be lowercase ascii [a-z0-9_-], got {value:?}"
            )));
        }
    }
    Ok(())
}

fn validate_baseline_digest(digest: &str, target: &CandidateTarget) -> Result<(), ContractError> {
    let (prefix, expected_len) = match target {
        CandidateTarget::Repository { .. } => ("git-sha1:", 40usize),
        CandidateTarget::Appliance { .. } => ("sha256:", 64usize),
    };
    let Some(hex) = digest.strip_prefix(prefix) else {
        return Err(ContractError::InvalidManifest(format!(
            "baseline_digest must start with {prefix} for this target, got {digest:?}"
        )));
    };
    if hex.len() != expected_len {
        return Err(ContractError::InvalidManifest(format!(
            "baseline_digest hex length must be {expected_len}, got {}",
            hex.len()
        )));
    }
    if !hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(ContractError::InvalidManifest(format!(
            "baseline_digest hex must be lowercase 0-9a-f, got {digest:?}"
        )));
    }
    Ok(())
}

fn validate_unique_nonempty_gates(gates: &[String]) -> Result<(), ContractError> {
    if gates.is_empty() {
        return Err(ContractError::InvalidManifest(
            "required_gates must be nonempty".to_owned(),
        ));
    }
    let mut seen = BTreeSet::new();
    for gate in gates {
        if gate.trim().is_empty() {
            return Err(ContractError::InvalidManifest(
                "required_gates entries must be nonempty".to_owned(),
            ));
        }
        if gate != gate.trim() {
            return Err(ContractError::InvalidManifest(format!(
                "required_gates entry must not have edge whitespace: {gate:?}"
            )));
        }
        if !seen.insert(gate.as_str()) {
            return Err(ContractError::InvalidManifest(format!(
                "required_gates contains duplicate {gate:?}"
            )));
        }
    }
    Ok(())
}

fn validate_unique_protected_paths(paths: &[String]) -> Result<(), ContractError> {
    if paths.is_empty() {
        return Err(ContractError::InvalidManifest(
            "protected_paths must be nonempty".to_owned(),
        ));
    }
    let mut seen = BTreeSet::new();
    for raw in paths {
        let normalized = normalize_manifest_protected_path(raw)?;
        if !seen.insert(normalized) {
            return Err(ContractError::InvalidManifest(format!(
                "protected_paths contains duplicate normalized path for {raw:?}"
            )));
        }
    }
    Ok(())
}

fn normalize_manifest_protected_path(path: &str) -> Result<String, ContractError> {
    if path.is_empty() {
        return Err(ContractError::InvalidManifest(
            "protected_paths entries must be nonempty".to_owned(),
        ));
    }
    if path.starts_with('/')
        || path.starts_with('\\')
        || path.starts_with("//")
        || path.starts_with("\\\\")
        || path_has_windows_drive(path)
    {
        return Err(ContractError::InvalidManifest(format!(
            "absolute protected path rejected: {path:?}"
        )));
    }
    let unified = path.replace('\\', "/");
    if unified.contains('\0') {
        return Err(ContractError::InvalidManifest(
            "protected path contains NUL byte".to_owned(),
        ));
    }
    let trailing_slash = unified.ends_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for component in unified.split('/').filter(|c| !c.is_empty()) {
        match component {
            "." => {}
            ".." => {
                return Err(ContractError::InvalidManifest(format!(
                    "protected path escapes via ..: {path:?}"
                )));
            }
            other => {
                if other.contains(':') {
                    return Err(ContractError::InvalidManifest(format!(
                        "protected path component forbidden: {other:?}"
                    )));
                }
                let trimmed = other.trim_end_matches(|c: char| c == ' ' || c == '\t' || c == '.');
                if trimmed != other || trimmed.is_empty() {
                    return Err(ContractError::InvalidManifest(format!(
                        "protected path component forbidden: {other:?}"
                    )));
                }
                parts.push(other);
            }
        }
    }
    if parts.is_empty() {
        return Err(ContractError::InvalidManifest(format!(
            "protected path resolves empty: {path:?}"
        )));
    }
    let mut normalized = parts.join("/");
    if trailing_slash {
        normalized.push('/');
    }
    // Case-fold for uniqueness so Windows-style aliases collide.
    Ok(normalized.chars().flat_map(char::to_lowercase).collect())
}

fn path_has_windows_drive(path: &str) -> bool {
    let mut chars = path.chars();
    match (chars.next(), chars.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_boundary_lengths() {
        // prefix 5 + body 11 = 16
        let min = format!("cand-{}", "a".repeat(11));
        assert_eq!(min.len(), 16);
        assert!(CandidateId::parse(&min).is_ok());

        // prefix 5 + body 91 = 96
        let max = format!("cand-{}", "a".repeat(91));
        assert_eq!(max.len(), 96);
        assert!(CandidateId::parse(&max).is_ok());
    }
}
