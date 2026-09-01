//! Non-compensable evaluation reports and gate semantics.
//!
//! Pure domain values only: no signature verification, filesystem, or I/O.

use crate::{CandidateId, EVALUATION_SCHEMA};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Maximum UTF-8 byte length for a gate detail string.
pub const MAX_GATE_DETAIL_BYTES: usize = 4096;

/// Errors raised while validating evaluation contracts.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    /// Gate payload failed structural validation.
    #[error("invalid gate result: {0}")]
    InvalidGate(String),
    /// Evaluation report failed structural validation.
    #[error("invalid evaluation report: {0}")]
    InvalidReport(String),
}

/// Classification of an evaluation gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GateClass {
    HardFloor,
    Metric,
}

/// Outcome of a single evaluation gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Pass,
    Fail,
    Unavailable,
}

/// One gate observation attached to an evaluation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields, title = "GateResult")]
pub struct GateResult {
    #[schemars(length(min = 1), regex(pattern = r"^[a-z0-9](?:[a-z0-9._-]*[a-z0-9])?$"))]
    pub name: String,
    pub class: GateClass,
    pub status: GateStatus,
    #[schemars(length(max = 4096))]
    pub detail: String,
    #[schemars(regex(pattern = r"^sha256:[a-f0-9]{64}$"))]
    pub artifact_digest: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGateResult {
    name: String,
    class: GateClass,
    status: GateStatus,
    detail: String,
    artifact_digest: Option<String>,
}

impl<'de> Deserialize<'de> for GateResult {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawGateResult::deserialize(deserializer)?;
        let value = Self {
            name: raw.name,
            class: raw.class,
            status: raw.status,
            detail: raw.detail,
            artifact_digest: raw.artifact_digest,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl GateResult {
    /// Hard-floor pass helper used by fixtures and evaluators.
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            class: GateClass::HardFloor,
            status: GateStatus::Pass,
            detail: String::new(),
            artifact_digest: None,
        }
    }

    /// Hard-floor fail helper used by fixtures and evaluators.
    pub fn fail(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            class: GateClass::HardFloor,
            status: GateStatus::Fail,
            detail: detail.into(),
            artifact_digest: None,
        }
    }

    /// Structural validation for a single gate observation.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        validate_safe_gate_name(&self.name)?;
        if self.detail.len() > MAX_GATE_DETAIL_BYTES {
            return Err(EvaluationError::InvalidGate(format!(
                "detail must be at most {MAX_GATE_DETAIL_BYTES} bytes, got {}",
                self.detail.len()
            )));
        }
        if let Some(digest) = &self.artifact_digest {
            validate_sha256_digest("gate.artifact_digest", digest)?;
        }
        Ok(())
    }
}

/// Comparative evaluation of a candidate against its baseline.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[schemars(deny_unknown_fields, title = "EvaluationReport")]
pub struct EvaluationReport {
    #[schemars(regex(pattern = r"^gzmo\.evolution\.evaluation/v1$"))]
    pub schema: String,
    pub candidate_id: CandidateId,
    #[schemars(regex(pattern = r"^(sha256:[a-f0-9]{64}|git-sha1:[a-f0-9]{40})$"))]
    pub baseline_digest: String,
    #[schemars(regex(pattern = r"^(sha256:[a-f0-9]{64}|git-sha1:[a-f0-9]{40})$"))]
    pub candidate_digest: String,
    #[schemars(length(min = 1))]
    pub gates: Vec<GateResult>,
    pub hard_floors_passed: bool,
    pub metrics: BTreeMap<String, f64>,
    /// Map of artifact name → `sha256:<64 lowercase hex>` digest.
    #[schemars(schema_with = "sha256_digest_map_schema")]
    pub artifact_digests: BTreeMap<String, String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvaluationReport {
    schema: String,
    candidate_id: CandidateId,
    baseline_digest: String,
    candidate_digest: String,
    gates: Vec<GateResult>,
    hard_floors_passed: bool,
    metrics: BTreeMap<String, f64>,
    artifact_digests: BTreeMap<String, String>,
    completed_at: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for EvaluationReport {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawEvaluationReport::deserialize(deserializer)?;
        let value = Self {
            schema: raw.schema,
            candidate_id: raw.candidate_id,
            baseline_digest: raw.baseline_digest,
            candidate_digest: raw.candidate_digest,
            gates: raw.gates,
            hard_floors_passed: raw.hard_floors_passed,
            metrics: raw.metrics,
            artifact_digests: raw.artifact_digests,
            completed_at: raw.completed_at,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl EvaluationReport {
    /// Recompute whether every hard floor passed.
    ///
    /// Returns false when no hard-floor gates are present. A hard-floor `Fail`
    /// or `Unavailable` always yields false. Metric gates never compensate.
    pub fn hard_floors_pass(&self) -> bool {
        let mut saw_hard_floor = false;
        for gate in &self.gates {
            if gate.class != GateClass::HardFloor {
                continue;
            }
            saw_hard_floor = true;
            if gate.status != GateStatus::Pass {
                return false;
            }
        }
        saw_hard_floor
    }


    /// Structural validation for an external evaluation report payload.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.schema != EVALUATION_SCHEMA {
            return Err(EvaluationError::InvalidReport(format!(
                "schema must be {EVALUATION_SCHEMA}"
            )));
        }
        validate_algorithm_qualified_digest("baseline_digest", &self.baseline_digest)?;
        validate_algorithm_qualified_digest("candidate_digest", &self.candidate_digest)?;

        if self.gates.is_empty() {
            return Err(EvaluationError::InvalidReport(
                "gates must be nonempty".to_owned(),
            ));
        }

        let mut seen_names = BTreeSet::new();
        let mut hard_floor_count = 0usize;
        for gate in &self.gates {
            gate.validate()?;
            if !seen_names.insert(gate.name.as_str()) {
                return Err(EvaluationError::InvalidReport(format!(
                    "gates contains duplicate name {:?}",
                    gate.name
                )));
            }
            if gate.class == GateClass::HardFloor {
                hard_floor_count += 1;
            }
        }
        if hard_floor_count == 0 {
            return Err(EvaluationError::InvalidReport(
                "gates must include at least one hard floor".to_owned(),
            ));
        }

        for (key, value) in &self.metrics {
            validate_safe_metric_name(key)?;
            if !value.is_finite() {
                return Err(EvaluationError::InvalidReport(format!(
                    "metric {key:?} must be finite, got {value}"
                )));
            }
        }

        for (name, digest) in &self.artifact_digests {
            validate_safe_artifact_name(name)?;
            validate_sha256_digest("artifact_digests value", digest)?;
        }

        let recomputed = self.hard_floors_pass();
        if self.hard_floors_passed != recomputed {
            return Err(EvaluationError::InvalidReport(format!(
                "hard_floors_passed={} mismatches recomputed hard_floors_pass()={recomputed} (forged or stale verdict)",
                self.hard_floors_passed
            )));
        }
        Ok(())
    }
}

fn validate_safe_gate_name(name: &str) -> Result<(), EvaluationError> {
    validate_safe_dotted_identifier("gate name", name).map_err(EvaluationError::InvalidGate)
}

fn validate_safe_metric_name(name: &str) -> Result<(), EvaluationError> {
    validate_safe_dotted_identifier("metric name", name).map_err(EvaluationError::InvalidReport)
}


fn validate_safe_artifact_name(name: &str) -> Result<(), EvaluationError> {
    if name.is_empty() {
        return Err(EvaluationError::InvalidReport(
            "artifact_digests key must be nonempty".to_owned(),
        ));
    }
    if name != name.trim() {
        return Err(EvaluationError::InvalidReport(format!(
            "artifact_digests key must not have edge whitespace: {name:?}"
        )));
    }
    if name.contains("..") || name.contains('\\') || name.starts_with('/') {
        return Err(EvaluationError::InvalidReport(format!(
            "artifact_digests key is unsafe: {name:?}"
        )));
    }
    if name.chars().any(|c| c.is_control() || (c.is_whitespace() && c != ' ')) {
        return Err(EvaluationError::InvalidReport(format!(
            "artifact_digests key contains control characters: {name:?}"
        )));
    }
    Ok(())
}

fn validate_safe_dotted_identifier(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must be nonempty"));
    }
    if value != value.trim() {
        return Err(format!("{field} must not have leading or trailing whitespace"));
    }
    if value.contains("..") {
        return Err(format!("{field} must not contain .."));
    }
    if value.starts_with('.')
        || value.ends_with('.')
        || value.starts_with('-')
        || value.ends_with('-')
    {
        return Err(format!(
            "{field} must not start or end with '.' or '-', got {value:?}"
        ));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(format!("{field} must not contain path separators"));
    }
    if !value.bytes().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_' || b == b'.'
    }) {
        return Err(format!(
            "{field} must be lowercase ascii [a-z0-9._-], got {value:?}"
        ));
    }
    Ok(())
}

fn validate_algorithm_qualified_digest(field: &str, digest: &str) -> Result<(), EvaluationError> {
    if let Some(hex) = digest.strip_prefix("sha256:") {
        return validate_hex(field, hex, 64);
    }
    if let Some(hex) = digest.strip_prefix("git-sha1:") {
        return validate_hex(field, hex, 40);
    }
    Err(EvaluationError::InvalidReport(format!(
        "{field} must be algorithm-qualified sha256:<64 hex> or git-sha1:<40 hex>, got {digest:?}"
    )))
}

fn validate_sha256_digest(field: &str, digest: &str) -> Result<(), EvaluationError> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(EvaluationError::InvalidReport(format!(
            "{field} must start with sha256:, got {digest:?}"
        )));
    };
    validate_hex(field, hex, 64)
}

fn validate_hex(field: &str, hex: &str, expected_len: usize) -> Result<(), EvaluationError> {
    if hex.len() != expected_len {
        return Err(EvaluationError::InvalidReport(format!(
            "{field} hex length must be {expected_len}, got {}",
            hex.len()
        )));
    }
    if !hex
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(EvaluationError::InvalidReport(format!(
            "{field} hex must be lowercase 0-9a-f, got {hex:?}"
        )));
    }
    Ok(())
}

/// JSON Schema for `artifact_digests`: object whose values are sha256-qualified digests.
fn sha256_digest_map_schema(
    _gen: &mut schemars::gen::SchemaGenerator,
) -> schemars::schema::Schema {
    use schemars::schema::{
        InstanceType, ObjectValidation, SchemaObject, StringValidation,
    };
    SchemaObject {
        instance_type: Some(InstanceType::Object.into()),
        object: Some(Box::new(ObjectValidation {
            additional_properties: Some(Box::new(
                SchemaObject {
                    instance_type: Some(InstanceType::String.into()),
                    string: Some(Box::new(StringValidation {
                        pattern: Some(r"^sha256:[a-f0-9]{64}$".to_owned()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_id() -> CandidateId {
        CandidateId::parse("cand-20260901t070000z-felt-use-a1b2c3").unwrap()
    }

    fn sha(fill: u8) -> String {
        format!(
            "sha256:{}",
            (0..32).map(|_| format!("{fill:02x}")).collect::<String>()
        )
    }


    #[test]
    fn hard_floor_fail_is_non_compensable() {
        let report = EvaluationReport {
            schema: EVALUATION_SCHEMA.to_owned(),
            candidate_id: sample_id(),
            baseline_digest: sha(1),
            candidate_digest: sha(2),
            gates: vec![
                GateResult::pass("tests"),
                GateResult::fail("faithfulness", "low"),
            ],
            hard_floors_passed: false,
            metrics: BTreeMap::from([("gain".into(), 999.0)]),
            artifact_digests: BTreeMap::new(),
            completed_at: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        };
        assert!(!report.hard_floors_pass());
        assert!(report.validate().is_ok());
    }

    #[test]
    fn forged_verdict_rejected() {
        let mut report = EvaluationReport {
            schema: EVALUATION_SCHEMA.to_owned(),
            candidate_id: sample_id(),
            baseline_digest: sha(1),
            candidate_digest: sha(2),
            gates: vec![GateResult::fail("tests", "x")],
            hard_floors_passed: false,
            metrics: BTreeMap::new(),
            artifact_digests: BTreeMap::new(),
            completed_at: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        };
        report.hard_floors_passed = true;
        assert!(report.validate().is_err());
    }
}
