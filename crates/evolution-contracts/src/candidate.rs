//! Candidate identity, kind, authority tier, and lifecycle state.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors raised while validating evolution contract values.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// Candidate identifier failed structural validation.
    #[error("invalid candidate id: {0}")]
    InvalidCandidateId(String),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
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
            Self::ProceduralSkill
            | Self::Code
            | Self::Schema
            | Self::Model
            | Self::Runtime => AuthorityTier::Candidate,
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

/// Base lifecycle states for a candidate (transitions added in a later task).
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
