//! Signed capability envelopes, resource budgets, path policy, and tunables.
//!
//! Pure domain values only: no signature verification, filesystem, or I/O.

use crate::{CandidateKind, ENVELOPE_SCHEMA};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Absolute compile-time ceilings for signed resource budgets.
pub const MAX_WALL_SECONDS: u64 = 86_400;
pub const MAX_ATTEMPTS: u8 = 5;
pub const MAX_CHANGED_FILES: u32 = 100;
pub const MAX_ADDED_LINES: u32 = 10_000;
pub const MAX_TOOL_CALLS: u32 = 500;
pub const MAX_INPUT_TOKENS: u64 = 5_000_000;
pub const MAX_OUTPUT_TOKENS: u64 = 1_000_000;
pub const MAX_ENERGY_JOULES: u64 = 10_000_000;

/// Stage-1 default protected path prefixes and files.
pub const DEFAULT_PROTECTED: &[&str] = &[
    ".github/workflows/",
    "docs/superpowers/specs/",
    "docs/ADR-",
    "AGENTS.md",
    "Cargo.toml",
    "Cargo.lock",
    "crates/evolution-contracts/",
    "gzmo-evolver/",
];

/// Errors raised while validating or applying policy contracts.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PolicyError {
    /// Resource budget failed structural or ceiling checks.
    #[error("invalid resource budget: {0}")]
    InvalidBudget(String),
    /// Path was absolute, escaped, or matched a protected pattern.
    #[error("path policy violation: {0}")]
    InvalidPath(String),
    /// Tunable rule failed structural checks.
    #[error("invalid tunable rule: {0}")]
    InvalidTunable(String),
    /// Envelope failed structural checks.
    #[error("invalid capability envelope: {0}")]
    InvalidEnvelope(String),
    /// Authorization or usage check denied the request.
    #[error("policy denied: {0}")]
    Denied(String),
}

/// Absolute ceilings for a single candidate attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields, title = "ResourceBudget")]
pub struct ResourceBudget {
    #[schemars(range(min = 1, max = 86_400))]
    pub wall_seconds: u64,
    #[schemars(range(min = 1, max = 5))]
    pub max_attempts: u8,
    #[schemars(range(min = 1, max = 100))]
    pub max_changed_files: u32,
    #[schemars(range(min = 1, max = 10_000))]
    pub max_added_lines: u32,
    #[schemars(range(min = 1, max = 500))]
    pub max_tool_calls: u32,
    #[schemars(range(min = 1, max = 5_000_000))]
    pub max_input_tokens: u64,
    #[schemars(range(min = 1, max = 1_000_000))]
    pub max_output_tokens: u64,
    /// When `None`, the signed profile acknowledges a missing energy meter.
    /// That is never unlimited energy; see [`ResourceBudget::allow_missing_energy_meter`].
    #[schemars(range(min = 1, max = 10_000_000))]
    pub max_energy_joules: Option<u64>,
    /// Explicit signed allowance that energy metering may be absent.
    pub allow_missing_energy_meter: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawResourceBudget {
    wall_seconds: u64,
    max_attempts: u8,
    max_changed_files: u32,
    max_added_lines: u32,
    max_tool_calls: u32,
    max_input_tokens: u64,
    max_output_tokens: u64,
    max_energy_joules: Option<u64>,
    allow_missing_energy_meter: bool,
}

impl<'de> Deserialize<'de> for ResourceBudget {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawResourceBudget::deserialize(deserializer)?;
        let value = Self {
            wall_seconds: raw.wall_seconds,
            max_attempts: raw.max_attempts,
            max_changed_files: raw.max_changed_files,
            max_added_lines: raw.max_added_lines,
            max_tool_calls: raw.max_tool_calls,
            max_input_tokens: raw.max_input_tokens,
            max_output_tokens: raw.max_output_tokens,
            max_energy_joules: raw.max_energy_joules,
            allow_missing_energy_meter: raw.allow_missing_energy_meter,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl ResourceBudget {
    /// Validate work limits against zero and compile-time ceilings.
    pub fn validate(&self) -> Result<(), PolicyError> {
        fn nonzero_u64(name: &str, value: u64, ceiling: u64) -> Result<(), PolicyError> {
            if value == 0 {
                return Err(PolicyError::InvalidBudget(format!(
                    "{name} must be greater than zero"
                )));
            }
            if value > ceiling {
                return Err(PolicyError::InvalidBudget(format!(
                    "{name} {value} exceeds ceiling {ceiling}"
                )));
            }
            Ok(())
        }
        fn nonzero_u32(name: &str, value: u32, ceiling: u32) -> Result<(), PolicyError> {
            if value == 0 {
                return Err(PolicyError::InvalidBudget(format!(
                    "{name} must be greater than zero"
                )));
            }
            if value > ceiling {
                return Err(PolicyError::InvalidBudget(format!(
                    "{name} {value} exceeds ceiling {ceiling}"
                )));
            }
            Ok(())
        }
        fn nonzero_u8(name: &str, value: u8, ceiling: u8) -> Result<(), PolicyError> {
            if value == 0 {
                return Err(PolicyError::InvalidBudget(format!(
                    "{name} must be greater than zero"
                )));
            }
            if value > ceiling {
                return Err(PolicyError::InvalidBudget(format!(
                    "{name} {value} exceeds ceiling {ceiling}"
                )));
            }
            Ok(())
        }

        nonzero_u64("wall_seconds", self.wall_seconds, MAX_WALL_SECONDS)?;
        nonzero_u8("max_attempts", self.max_attempts, MAX_ATTEMPTS)?;
        nonzero_u32(
            "max_changed_files",
            self.max_changed_files,
            MAX_CHANGED_FILES,
        )?;
        nonzero_u32("max_added_lines", self.max_added_lines, MAX_ADDED_LINES)?;
        nonzero_u32("max_tool_calls", self.max_tool_calls, MAX_TOOL_CALLS)?;
        nonzero_u64("max_input_tokens", self.max_input_tokens, MAX_INPUT_TOKENS)?;
        nonzero_u64(
            "max_output_tokens",
            self.max_output_tokens,
            MAX_OUTPUT_TOKENS,
        )?;

        match self.max_energy_joules {
            None => {
                if !self.allow_missing_energy_meter {
                    return Err(PolicyError::InvalidBudget(
                        "max_energy_joules is missing without allow_missing_energy_meter"
                            .to_owned(),
                    ));
                }
            }
            Some(0) => {
                return Err(PolicyError::InvalidBudget(
                    "max_energy_joules must be greater than zero when present".to_owned(),
                ));
            }
            Some(joules) if joules > MAX_ENERGY_JOULES => {
                return Err(PolicyError::InvalidBudget(format!(
                    "max_energy_joules {joules} exceeds ceiling {MAX_ENERGY_JOULES}"
                )));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Observed resource consumption to compare against a signed budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(deny_unknown_fields, title = "ResourceUsage")]
pub struct ResourceUsage {
    pub wall_seconds: u64,
    pub attempts: u8,
    pub changed_files: u32,
    pub added_lines: u32,
    pub tool_calls: u32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub energy_joules: Option<u64>,
}

impl ResourceUsage {
    /// Return true when every used field is within the signed maximum.
    ///
    /// Energy rules:
    /// - `max_energy_joules = None` never means unlimited energy: any positive
    ///   observed energy fails.
    /// - Against a finite ceiling, `energy_joules = None` is accepted only when
    ///   `allow_missing_energy_meter` is true (explicit signed allowance).
    pub fn fits(&self, budget: &ResourceBudget) -> bool {
        if self.wall_seconds > budget.wall_seconds
            || self.attempts > budget.max_attempts
            || self.changed_files > budget.max_changed_files
            || self.added_lines > budget.max_added_lines
            || self.tool_calls > budget.max_tool_calls
            || self.input_tokens > budget.max_input_tokens
            || self.output_tokens > budget.max_output_tokens
        {
            return false;
        }

        match budget.max_energy_joules {
            None => match self.energy_joules {
                None | Some(0) => true,
                Some(_) => false,
            },
            Some(max) => match self.energy_joules {
                None => budget.allow_missing_energy_meter,
                Some(used) => used <= max,
            },
        }
    }
}

/// Protected-path rules for candidate diffs and writes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields, title = "PathPolicy")]
pub struct PathPolicy {
    #[schemars(length(min = 1))]
    pub protected_paths: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPathPolicy {
    protected_paths: Vec<String>,
}

impl<'de> Deserialize<'de> for PathPolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawPathPolicy::deserialize(deserializer)?;
        let value = Self {
            protected_paths: raw.protected_paths,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl PathPolicy {
    /// Stage-1 default protected paths.
    pub fn stage1_default() -> Self {
        Self {
            protected_paths: DEFAULT_PROTECTED.iter().map(|p| (*p).to_owned()).collect(),
        }
    }

    /// Normalize separators and reject absolute/`..`/empty-escape paths, then
    /// deny case-folded matches against protected patterns.
    pub fn check(&self, path: &str) -> Result<(), PolicyError> {
        let normalized = normalize_relative_path(path)?;
        let folded = case_fold(&normalized);
        for raw_pattern in &self.protected_paths {
            let pattern = normalize_protected_pattern(raw_pattern)?;
            let folded_pattern = case_fold(&pattern);
            if path_matches_protected(&folded, &folded_pattern) {
                return Err(PolicyError::InvalidPath(format!(
                    "path {path:?} matches protected pattern {raw_pattern:?}"
                )));
            }
        }
        Ok(())
    }

    /// Validate that every configured protected pattern is well-formed.
    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.protected_paths.is_empty() {
            return Err(PolicyError::InvalidPath(
                "protected_paths must be nonempty".to_owned(),
            ));
        }
        for raw in &self.protected_paths {
            normalize_protected_pattern(raw)?;
        }
        Ok(())
    }
}

fn case_fold(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).collect()
}

fn normalize_relative_path(path: &str) -> Result<String, PolicyError> {
    if path.is_empty() {
        return Err(PolicyError::InvalidPath("path must be nonempty".to_owned()));
    }
    if path.starts_with('/')
        || path.starts_with('\\')
        || path_has_windows_drive(path)
        || path.starts_with("\\\\")
        || path.starts_with("//")
    {
        return Err(PolicyError::InvalidPath(format!(
            "absolute path rejected: {path:?}"
        )));
    }

    let unified = path.replace('\\', "/");
    if unified.contains('\0') {
        return Err(PolicyError::InvalidPath(
            "path contains NUL byte".to_owned(),
        ));
    }

    let mut out: Vec<&str> = Vec::new();
    for component in unified.split('/') {
        match component {
            "" => {
                // Reject empty components ("a//b") as symlink/escape ambiguity.
                return Err(PolicyError::InvalidPath(format!(
                    "path has empty component: {path:?}"
                )));
            }
            "." => {
                // Drop current-dir markers after separator normalization.
            }
            ".." => {
                return Err(PolicyError::InvalidPath(format!(
                    "path escapes via ..: {path:?}"
                )));
            }
            other => {
                if component_is_forbidden(other) {
                    return Err(PolicyError::InvalidPath(format!(
                        "path component forbidden (colon or trailing dot/space): {other:?} in {path:?}"
                    )));
                }
                out.push(other);
            }
        }
    }
    if out.is_empty() {
        return Err(PolicyError::InvalidPath(format!(
            "path resolves empty: {path:?}"
        )));
    }
    Ok(out.join("/"))
}

/// Reject NTFS ADS / drive markers and Windows-trimmed trailing dots/spaces.
fn component_is_forbidden(component: &str) -> bool {
    if component.contains(':') {
        return true;
    }
    let trimmed = component.trim_end_matches(|c: char| c == ' ' || c == '\t' || c == '.');
    trimmed != component || trimmed.is_empty()
}

fn normalize_protected_pattern(pattern: &str) -> Result<String, PolicyError> {
    if pattern.is_empty() {
        return Err(PolicyError::InvalidPath(
            "protected pattern must be nonempty".to_owned(),
        ));
    }
    if pattern.starts_with('/')
        || pattern.starts_with('\\')
        || path_has_windows_drive(pattern)
        || pattern.starts_with("//")
        || pattern.starts_with("\\\\")
    {
        return Err(PolicyError::InvalidPath(format!(
            "absolute protected pattern rejected: {pattern:?}"
        )));
    }
    let unified = pattern.replace('\\', "/");
    let trailing_slash = unified.ends_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for component in unified.split('/').filter(|c| !c.is_empty()) {
        match component {
            "." => {}
            ".." => {
                return Err(PolicyError::InvalidPath(format!(
                    "protected pattern escapes via ..: {pattern:?}"
                )));
            }
            other => {
                if component_is_forbidden(other) {
                    return Err(PolicyError::InvalidPath(format!(
                        "protected pattern component forbidden: {other:?}"
                    )));
                }
                parts.push(other);
            }
        }
    }
    if parts.is_empty() {
        return Err(PolicyError::InvalidPath(format!(
            "protected pattern resolves empty: {pattern:?}"
        )));
    }
    let mut normalized = parts.join("/");
    if trailing_slash {
        normalized.push('/');
    }
    Ok(normalized)
}

fn path_has_windows_drive(path: &str) -> bool {
    let mut chars = path.chars();
    match (chars.next(), chars.next()) {
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic() => true,
        _ => false,
    }
}

fn path_matches_protected(path: &str, pattern: &str) -> bool {
    if path == pattern {
        return true;
    }
    if pattern.ends_with('/') {
        return path.starts_with(pattern);
    }
    // Prefix token such as `docs/ADR-`.
    if pattern.ends_with('-') {
        return path.starts_with(pattern);
    }
    // Bare filename: match root or any nested segment.
    if !pattern.contains('/') {
        return path == pattern || path.ends_with(&format!("/{pattern}"));
    }
    // Directory-like pattern without trailing slash: boundary-aware prefix.
    path.starts_with(pattern)
        && (path.len() == pattern.len()
            || path.as_bytes().get(pattern.len()) == Some(&b'/')
            || path.as_bytes().get(pattern.len()) == Some(&b'-'))
}

/// JSON Schema for tunables map: nonempty keys with dotted-identifier pattern.
fn tunables_map_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    use schemars::schema::{InstanceType, ObjectValidation, SchemaObject, StringValidation};
    let value_schema = gen.subschema_for::<TunableRule>();
    SchemaObject {
        instance_type: Some(InstanceType::Object.into()),
        object: Some(Box::new(ObjectValidation {
            additional_properties: Some(Box::new(value_schema)),
            property_names: Some(Box::new(
                SchemaObject {
                    instance_type: Some(InstanceType::String.into()),
                    string: Some(Box::new(StringValidation {
                        min_length: Some(1),
                        pattern: Some(r"^[A-Za-z0-9][A-Za-z0-9._-]*$".to_owned()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into(),
            )),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

/// Tunable map keys must match frozen envelope schema `propertyNames`.
///
/// Pattern: `^[A-Za-z0-9][A-Za-z0-9._-]*$` (safe dotted identifiers; no leading
/// underscore/punctuation, no whitespace).
fn validate_tunable_map_key(key: &str) -> Result<(), PolicyError> {
    if key.is_empty() {
        return Err(PolicyError::InvalidTunable(
            "tunable key must be nonempty".to_owned(),
        ));
    }
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return Err(PolicyError::InvalidTunable(
            "tunable key must be nonempty".to_owned(),
        ));
    };
    if !first.is_ascii_alphanumeric() {
        return Err(PolicyError::InvalidTunable(format!(
            "tunable key must match ^[A-Za-z0-9][A-Za-z0-9._-]*$, got {key:?}"
        )));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-') {
        return Err(PolicyError::InvalidTunable(format!(
            "tunable key must match ^[A-Za-z0-9][A-Za-z0-9._-]*$, got {key:?}"
        )));
    }
    Ok(())
}


fn nonempty_unique_string_array_schema(
    _gen: &mut schemars::gen::SchemaGenerator,
) -> schemars::schema::Schema {
    use schemars::schema::{ArrayValidation, InstanceType, SchemaObject, StringValidation};
    let item: schemars::schema::Schema = SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        string: Some(Box::new(StringValidation {
            min_length: Some(1),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into();
    SchemaObject {
        instance_type: Some(InstanceType::Array.into()),
        array: Some(Box::new(ArrayValidation {
            items: Some(item.into()),
            min_items: Some(1),
            unique_items: Some(true),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

fn nonempty_unique_candidate_kinds_schema(
    gen: &mut schemars::gen::SchemaGenerator,
) -> schemars::schema::Schema {
    use schemars::schema::{ArrayValidation, InstanceType, SchemaObject};
    let item = gen.subschema_for::<CandidateKind>();
    SchemaObject {
        instance_type: Some(InstanceType::Array.into()),
        array: Some(Box::new(ArrayValidation {
            items: Some(item.into()),
            min_items: Some(1),
            unique_items: Some(true),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

/// Typed bounds for an operator-signed tunable key.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
#[schemars(deny_unknown_fields, title = "TunableRule")]
pub enum TunableRule {
    IntegerRange {
        min: i64,
        max: i64,
    },
    FloatRange {
        min: f64,
        max: f64,
    },
    EnumSet {
        #[schemars(length(min = 1))]
        values: BTreeSet<String>,
    },
    Boolean,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", deny_unknown_fields)]
enum RawTunableRule {
    IntegerRange { min: i64, max: i64 },
    FloatRange { min: f64, max: f64 },
    EnumSet { values: BTreeSet<String> },
    Boolean,
}

impl<'de> Deserialize<'de> for TunableRule {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawTunableRule::deserialize(deserializer)?;
        let value = match raw {
            RawTunableRule::IntegerRange { min, max } => Self::IntegerRange { min, max },
            RawTunableRule::FloatRange { min, max } => Self::FloatRange { min, max },
            RawTunableRule::EnumSet { values } => Self::EnumSet { values },
            RawTunableRule::Boolean => Self::Boolean,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl TunableRule {
    /// Validate range order, finiteness, and nonempty enum sets.
    pub fn validate(&self) -> Result<(), PolicyError> {
        match self {
            Self::IntegerRange { min, max } => {
                if min > max {
                    return Err(PolicyError::InvalidTunable(format!(
                        "integer range min {min} > max {max}"
                    )));
                }
            }
            Self::FloatRange { min, max } => {
                if !min.is_finite() || !max.is_finite() {
                    return Err(PolicyError::InvalidTunable(
                        "float range bounds must be finite".to_owned(),
                    ));
                }
                if min > max {
                    return Err(PolicyError::InvalidTunable(format!(
                        "float range min {min} > max {max}"
                    )));
                }
            }
            Self::EnumSet { values } => {
                if values.is_empty() {
                    return Err(PolicyError::InvalidTunable(
                        "enum set must be nonempty".to_owned(),
                    ));
                }
                if values.iter().any(|v| v.is_empty()) {
                    return Err(PolicyError::InvalidTunable(
                        "enum set values must be nonempty".to_owned(),
                    ));
                }
            }
            Self::Boolean => {}
        }
        Ok(())
    }

    /// Authorize a floating-point assignment against this rule.
    pub fn authorize_f64(&self, value: f64) -> Result<(), PolicyError> {
        match self {
            Self::FloatRange { min, max } => {
                if !value.is_finite() {
                    return Err(PolicyError::Denied(
                        "tunable float value must be finite".to_owned(),
                    ));
                }
                if value < *min || value > *max {
                    return Err(PolicyError::Denied(format!(
                        "value {value} outside float range [{min}, {max}]"
                    )));
                }
                Ok(())
            }
            other => Err(PolicyError::Denied(format!(
                "tunable rule {other:?} does not accept float values"
            ))),
        }
    }

    /// Authorize an integer assignment against this rule.
    pub fn authorize_i64(&self, value: i64) -> Result<(), PolicyError> {
        match self {
            Self::IntegerRange { min, max } => {
                if value < *min || value > *max {
                    return Err(PolicyError::Denied(format!(
                        "value {value} outside integer range [{min}, {max}]"
                    )));
                }
                Ok(())
            }
            other => Err(PolicyError::Denied(format!(
                "tunable rule {other:?} does not accept integer values"
            ))),
        }
    }

    /// Authorize a boolean assignment against this rule.
    pub fn authorize_bool(&self, _value: bool) -> Result<(), PolicyError> {
        match self {
            Self::Boolean => Ok(()),
            other => Err(PolicyError::Denied(format!(
                "tunable rule {other:?} does not accept boolean values"
            ))),
        }
    }

    /// Authorize an enum-string assignment against this rule.
    pub fn authorize_enum(&self, value: &str) -> Result<(), PolicyError> {
        match self {
            Self::EnumSet { values } => {
                if values.contains(value) {
                    Ok(())
                } else {
                    Err(PolicyError::Denied(format!(
                        "value {value:?} not in enum set"
                    )))
                }
            }
            other => Err(PolicyError::Denied(format!(
                "tunable rule {other:?} does not accept enum values"
            ))),
        }
    }
}

/// Operator-facing allow/deny decision with an explicit denial reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[schemars(deny_unknown_fields, title = "PolicyDecision")]
pub enum PolicyDecision {
    Allowed,
    Denied { reason: String },
}

impl PolicyDecision {
    /// Convert a policy result into an allow/deny decision.
    pub fn from_result(result: Result<(), PolicyError>) -> Self {
        match result {
            Ok(()) => Self::Allowed,
            Err(err) => Self::Denied {
                reason: err.to_string(),
            },
        }
    }

    /// True when the decision is [`PolicyDecision::Allowed`].
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// Signed capability envelope (signature verification lives outside this crate).
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields, title = "CapabilityEnvelope")]
pub struct CapabilityEnvelope {
    #[schemars(regex(pattern = r"^gzmo\.evolution\.envelope/v1$"))]
    pub schema: String,
    #[schemars(length(min = 1))]
    pub envelope_id: String,
    #[schemars(length(min = 1))]
    pub policy_version: String,
    #[schemars(length(min = 1))]
    pub signer_key_id: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub budget: ResourceBudget,
    pub paths: PathPolicy,
    /// Tunable allowlist keyed by dotted identifiers (runtime validates keys).
    #[schemars(schema_with = "tunables_map_schema")]
    pub tunables: BTreeMap<String, TunableRule>,
    #[schemars(
        length(min = 1),
        schema_with = "nonempty_unique_candidate_kinds_schema"
    )]
    pub allowed_candidate_kinds: BTreeSet<CandidateKind>,
    #[schemars(length(min = 1), schema_with = "nonempty_unique_string_array_schema")]
    pub required_gates: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCapabilityEnvelope {
    schema: String,
    envelope_id: String,
    policy_version: String,
    signer_key_id: String,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    budget: ResourceBudget,
    paths: PathPolicy,
    tunables: BTreeMap<String, TunableRule>,
    allowed_candidate_kinds: BTreeSet<CandidateKind>,
    required_gates: Vec<String>,
}

impl<'de> Deserialize<'de> for CapabilityEnvelope {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawCapabilityEnvelope::deserialize(deserializer)?;
        let value = Self {
            schema: raw.schema,
            envelope_id: raw.envelope_id,
            policy_version: raw.policy_version,
            signer_key_id: raw.signer_key_id,
            issued_at: raw.issued_at,
            expires_at: raw.expires_at,
            budget: raw.budget,
            paths: raw.paths,
            tunables: raw.tunables,
            allowed_candidate_kinds: raw.allowed_candidate_kinds,
            required_gates: raw.required_gates,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl CapabilityEnvelope {
    /// Structural validation for a signed envelope payload.
    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.schema != ENVELOPE_SCHEMA {
            return Err(PolicyError::InvalidEnvelope(format!(
                "schema must be {ENVELOPE_SCHEMA}"
            )));
        }
        if self.envelope_id.trim().is_empty() {
            return Err(PolicyError::InvalidEnvelope(
                "envelope_id must be nonempty".to_owned(),
            ));
        }
        if self.policy_version.trim().is_empty() {
            return Err(PolicyError::InvalidEnvelope(
                "policy_version must be nonempty".to_owned(),
            ));
        }
        if self.signer_key_id.trim().is_empty() {
            return Err(PolicyError::InvalidEnvelope(
                "signer_key_id must be nonempty".to_owned(),
            ));
        }
        if self.issued_at >= self.expires_at {
            return Err(PolicyError::InvalidEnvelope(
                "issued_at must be strictly before expires_at".to_owned(),
            ));
        }
        if self.required_gates.is_empty() || self.required_gates.iter().any(|g| g.trim().is_empty())
        {
            return Err(PolicyError::InvalidEnvelope(
                "required_gates must be nonempty with nonempty names".to_owned(),
            ));
        }
        {
            let mut seen = BTreeSet::new();
            for gate in &self.required_gates {
                if !seen.insert(gate.as_str()) {
                    return Err(PolicyError::InvalidEnvelope(format!(
                        "required_gates contains duplicate {gate:?}"
                    )));
                }
            }
        }
        self.budget.validate()?;
        self.paths.validate()?;
        for (key, rule) in &self.tunables {
            validate_tunable_map_key(key)?;
            rule.validate()?;
        }
        if self.allowed_candidate_kinds.is_empty() {
            return Err(PolicyError::InvalidEnvelope(
                "allowed_candidate_kinds must be nonempty".to_owned(),
            ));
        }
        for kind in &self.allowed_candidate_kinds {
            match kind {
                CandidateKind::Memory | CandidateKind::Tunable => {}
                other => {
                    return Err(PolicyError::InvalidEnvelope(format!(
                        "allowed_candidate_kinds may only contain Memory or Tunable, found {other}"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Reject requests outside the signed validity window.
    ///
    /// Accepts `issued_at <= now < expires_at`. Callers must supply wall-clock
    /// time; authorization entry points never omit it.
    pub fn validate_at(&self, now: DateTime<Utc>) -> Result<(), PolicyError> {
        self.validate()?;
        if now < self.issued_at {
            return Err(PolicyError::Denied(format!(
                "envelope not yet valid until {} (now={now})",
                self.issued_at
            )));
        }
        if now >= self.expires_at {
            return Err(PolicyError::Denied(format!(
                "envelope expired at {} (now={now})",
                self.expires_at
            )));
        }
        Ok(())
    }

    /// Authorize a float tunable assignment by key within the validity window.
    pub fn authorize_tunable(
        &self,
        key: &str,
        value: f64,
        now: DateTime<Utc>,
    ) -> Result<(), PolicyError> {
        self.validate_at(now)?;
        let rule = self.tunables.get(key).ok_or_else(|| {
            PolicyError::Denied(format!("tunable key {key:?} is not in envelope"))
        })?;
        rule.authorize_f64(value)
    }

    /// Authorize a candidate kind against the signed allowlist within the window.
    pub fn authorize_candidate_kind(
        &self,
        kind: CandidateKind,
        now: DateTime<Utc>,
    ) -> Result<(), PolicyError> {
        self.validate_at(now)?;
        if self.allowed_candidate_kinds.contains(&kind) {
            Ok(())
        } else {
            Err(PolicyError::Denied(format!(
                "candidate kind {kind} is not allowed by envelope"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn valid_budget() -> ResourceBudget {
        ResourceBudget {
            wall_seconds: 3_600,
            max_attempts: 2,
            max_changed_files: 10,
            max_added_lines: 400,
            max_tool_calls: 50,
            max_input_tokens: 100_000,
            max_output_tokens: 20_000,
            max_energy_joules: Some(1_000),
            allow_missing_energy_meter: false,
        }
    }

    #[test]
    fn budget_rejects_zero_and_over_ceiling() {
        let mut budget = valid_budget();
        budget.wall_seconds = 0;
        assert!(budget.validate().is_err());
        budget = valid_budget();
        budget.wall_seconds = MAX_WALL_SECONDS + 1;
        assert!(budget.validate().is_err());
        budget = valid_budget();
        budget.max_attempts = MAX_ATTEMPTS + 1;
        assert!(budget.validate().is_err());
    }

    #[test]
    fn missing_energy_requires_explicit_allowance() {
        let mut budget = valid_budget();
        budget.max_energy_joules = None;
        budget.allow_missing_energy_meter = false;
        assert!(budget.validate().is_err());
        budget.allow_missing_energy_meter = true;
        assert!(budget.validate().is_ok());
    }

    #[test]
    fn usage_fit_respects_energy_semantics() {
        let mut budget = valid_budget();
        let usage = ResourceUsage {
            wall_seconds: 10,
            attempts: 1,
            changed_files: 1,
            added_lines: 1,
            tool_calls: 1,
            input_tokens: 1,
            output_tokens: 1,
            energy_joules: Some(500),
        };
        assert!(usage.fits(&budget));
        let over = ResourceUsage {
            energy_joules: Some(1_001),
            ..usage.clone()
        };
        assert!(!over.fits(&budget));

        // Finite ceiling rejects missing energy unless allow_missing_energy_meter.
        let missing = ResourceUsage {
            energy_joules: None,
            ..usage.clone()
        };
        assert!(!missing.fits(&budget));
        budget.allow_missing_energy_meter = true;
        assert!(missing.fits(&budget));

        budget.max_energy_joules = None;
        budget.allow_missing_energy_meter = true;
        assert!(!usage.fits(&budget));
        assert!(missing.fits(&budget));
    }

    #[test]
    fn path_policy_blocks_protected_and_escapes() {
        let paths = PathPolicy::stage1_default();
        assert!(paths
            .check("docs/ADR-0014-constitutional-evolution.md")
            .is_err());
        assert!(paths.check("Docs\\ADR-0014-x.md").is_err());
        assert!(paths.check("../Cargo.toml").is_err());
        assert!(paths.check("/etc/passwd").is_err());
        assert!(paths.check("C:\\Windows\\system32").is_err());
        assert!(paths.check("src/lib.rs").is_ok());
        assert!(paths.check("crates/other/src/lib.rs").is_ok());
    }

    #[test]
    fn issued_before_expiry() {
        let issued = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
        let expires = Utc.with_ymd_and_hms(2026, 9, 2, 0, 0, 0).unwrap();
        let envelope = CapabilityEnvelope {
            schema: ENVELOPE_SCHEMA.to_owned(),
            envelope_id: "env-unit".to_owned(),
            policy_version: "v1".to_owned(),
            signer_key_id: "key-1".to_owned(),
            issued_at: issued,
            expires_at: expires,
            budget: valid_budget(),
            paths: PathPolicy::stage1_default(),
            tunables: BTreeMap::new(),
            allowed_candidate_kinds: BTreeSet::from([CandidateKind::Memory]),
            required_gates: vec!["tests".to_owned()],
        };
        assert!(envelope.validate().is_ok());

        let mut inverted = envelope.clone();
        inverted.issued_at = expires;
        inverted.expires_at = issued;
        assert!(inverted.validate().is_err());

        let mut equal = envelope.clone();
        equal.expires_at = equal.issued_at;
        assert!(equal.validate().is_err());

        // Validity window: issued_at inclusive, expires_at exclusive.
        assert!(envelope.validate_at(issued).is_ok());
        assert!(envelope
            .validate_at(issued + chrono::Duration::hours(12))
            .is_ok());
        assert!(envelope
            .validate_at(issued - chrono::Duration::seconds(1))
            .is_err());
        assert!(envelope.validate_at(expires).is_err());
        assert!(envelope
            .validate_at(expires + chrono::Duration::seconds(1))
            .is_err());
    }
}
