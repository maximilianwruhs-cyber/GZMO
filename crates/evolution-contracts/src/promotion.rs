//! Detached promotion requests and unverified authority grants.
//!
//! Pure domain values only: no cryptographic verification, filesystem, or I/O.
//! `VerifiedAuthorityGrant` is intentionally absent from this crate.

use crate::{CandidateId, PROMOTION_SCHEMA};
use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Maximum lifetime of a promotion request after issuance.
pub const MAX_PROMOTION_TTL: Duration = Duration::hours(24);

/// Errors raised while validating promotion contracts.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PromotionError {
    /// Promotion request failed structural validation.
    #[error("invalid promotion request: {0}")]
    InvalidRequest(String),
    /// Unverified grant failed structural validation.
    #[error("invalid authority grant: {0}")]
    InvalidGrant(String),
    /// Binding check failed against supplied digests/target/time.
    #[error("promotion binding mismatch: {0}")]
    BindingMismatch(String),
}

/// Operator-facing promotion binding payload (unsigned).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PromotionRequest {
    pub schema: String,
    pub candidate_id: CandidateId,
    pub candidate_digest: String,
    pub evaluation_digest: String,
    pub policy_digest: String,
    pub target: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub nonce: String,
}

#[derive(Deserialize)]
struct RawPromotionRequest {
    schema: String,
    candidate_id: CandidateId,
    candidate_digest: String,
    evaluation_digest: String,
    policy_digest: String,
    target: String,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    nonce: String,
}

impl<'de> Deserialize<'de> for PromotionRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawPromotionRequest::deserialize(deserializer)?;
        let value = Self {
            schema: raw.schema,
            candidate_id: raw.candidate_id,
            candidate_digest: raw.candidate_digest,
            evaluation_digest: raw.evaluation_digest,
            policy_digest: raw.policy_digest,
            target: raw.target,
            issued_at: raw.issued_at,
            expires_at: raw.expires_at,
            nonce: raw.nonce,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl PromotionRequest {
    /// Structural validation for an external promotion request payload.
    pub fn validate(&self) -> Result<(), PromotionError> {
        if self.schema != PROMOTION_SCHEMA {
            return Err(PromotionError::InvalidRequest(format!(
                "schema must be {PROMOTION_SCHEMA}"
            )));
        }
        validate_algorithm_qualified_digest("candidate_digest", &self.candidate_digest)?;
        validate_sha256_digest("evaluation_digest", &self.evaluation_digest)?;
        validate_sha256_digest("policy_digest", &self.policy_digest)?;
        validate_safe_identifier("target", &self.target)?;
        validate_safe_identifier("nonce", &self.nonce)?;

        if self.expires_at <= self.issued_at {
            return Err(PromotionError::InvalidRequest(
                "expires_at must be strictly after issued_at".to_owned(),
            ));
        }
        let ttl = self.expires_at - self.issued_at;
        if ttl > MAX_PROMOTION_TTL {
            return Err(PromotionError::InvalidRequest(format!(
                "expires_at - issued_at must be <= 24h, got {ttl}"
            )));
        }
        Ok(())
    }

    /// Bind this request to concrete digests/target and reject expiry at `now`.
    pub fn validate_binding(
        &self,
        candidate_digest: &str,
        evaluation_digest: &str,
        policy_digest: &str,
        target: &str,
        now: DateTime<Utc>,
    ) -> Result<(), PromotionError> {
        self.validate()?;
        if self.candidate_digest != candidate_digest {
            return Err(PromotionError::BindingMismatch(
                "candidate_digest mismatch".to_owned(),
            ));
        }
        if self.evaluation_digest != evaluation_digest {
            return Err(PromotionError::BindingMismatch(
                "evaluation_digest mismatch".to_owned(),
            ));
        }
        if self.policy_digest != policy_digest {
            return Err(PromotionError::BindingMismatch(
                "policy_digest mismatch".to_owned(),
            ));
        }
        if self.target != target {
            return Err(PromotionError::BindingMismatch(
                "target mismatch".to_owned(),
            ));
        }
        if now >= self.expires_at {
            return Err(PromotionError::BindingMismatch(format!(
                "request expired at {} (now={now})",
                self.expires_at
            )));
        }
        Ok(())
    }
}

/// Wire-format authority grant prior to cryptographic verification.
///
/// This crate never promotes an unverified grant to a trusted type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct UnverifiedAuthorityGrant {
    pub request: PromotionRequest,
    pub signer_key_id: String,
    /// Ed25519 signature as 128 lowercase hex characters (encoding only).
    pub signature_hex: String,
}

#[derive(Deserialize)]
struct RawUnverifiedAuthorityGrant {
    request: PromotionRequest,
    signer_key_id: String,
    signature_hex: String,
}

impl<'de> Deserialize<'de> for UnverifiedAuthorityGrant {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawUnverifiedAuthorityGrant::deserialize(deserializer)?;
        let value = Self {
            request: raw.request,
            signer_key_id: raw.signer_key_id,
            signature_hex: raw.signature_hex,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl UnverifiedAuthorityGrant {
    /// Structural validation for an external unverified grant payload.
    pub fn validate(&self) -> Result<(), PromotionError> {
        self.request.validate()?;
        validate_safe_identifier("signer_key_id", &self.signer_key_id)
            .map_err(|err| PromotionError::InvalidGrant(err.to_string()))?;
        validate_signature_hex(&self.signature_hex)?;
        Ok(())
    }
}

fn validate_safe_identifier(field: &str, value: &str) -> Result<(), PromotionError> {
    if value.is_empty() {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must be nonempty"
        )));
    }
    if value != value.trim() {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must not have leading or trailing whitespace"
        )));
    }
    if value.contains("..") {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must not contain .."
        )));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must not contain path separators"
        )));
    }
    if value.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must not contain control or whitespace characters"
        )));
    }
    // Allow mixed-case tokens (e.g. system-B) with conservative punctuation.
    if !value.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
    }) {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must be ascii alphanumeric plus [._-], got {value:?}"
        )));
    }
    if value.starts_with('-') || value.ends_with('-') {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must not start or end with hyphen"
        )));
    }
    Ok(())
}

fn validate_algorithm_qualified_digest(field: &str, digest: &str) -> Result<(), PromotionError> {
    if let Some(hex) = digest.strip_prefix("sha256:") {
        return validate_hex(field, hex, 64);
    }
    if let Some(hex) = digest.strip_prefix("git-sha1:") {
        return validate_hex(field, hex, 40);
    }
    Err(PromotionError::InvalidRequest(format!(
        "{field} must be algorithm-qualified sha256:<64 hex> or git-sha1:<40 hex>, got {digest:?}"
    )))
}

fn validate_sha256_digest(field: &str, digest: &str) -> Result<(), PromotionError> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} must start with sha256:, got {digest:?}"
        )));
    };
    validate_hex(field, hex, 64)
}

fn validate_hex(field: &str, hex: &str, expected_len: usize) -> Result<(), PromotionError> {
    if hex.len() != expected_len {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} hex length must be {expected_len}, got {}",
            hex.len()
        )));
    }
    if !hex
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(PromotionError::InvalidRequest(format!(
            "{field} hex must be lowercase 0-9a-f, got {hex:?}"
        )));
    }
    Ok(())
}

fn validate_signature_hex(signature_hex: &str) -> Result<(), PromotionError> {
    if signature_hex.len() != 128 {
        return Err(PromotionError::InvalidGrant(format!(
            "signature_hex must be exactly 128 lowercase hex chars, got length {}",
            signature_hex.len()
        )));
    }
    if !signature_hex
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(PromotionError::InvalidGrant(
            "signature_hex must be lowercase 0-9a-f".to_owned(),
        ));
    }
    Ok(())
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
            (0..32)
                .map(|_| format!("{fill:02x}"))
                .collect::<String>()
        )
    }

    fn request() -> PromotionRequest {
        PromotionRequest {
            schema: PROMOTION_SCHEMA.to_owned(),
            candidate_id: sample_id(),
            candidate_digest: sha(1),
            evaluation_digest: sha(2),
            policy_digest: sha(3),
            target: "system-B".to_owned(),
            issued_at: Utc.with_ymd_and_hms(2026, 9, 1, 10, 0, 0).unwrap(),
            expires_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
            nonce: "nonce-1".to_owned(),
        }
    }

    #[test]
    fn binding_checks_digests_and_expiry() {
        let req = request();
        let now = Utc.with_ymd_and_hms(2026, 9, 1, 12, 0, 0).unwrap();
        assert!(req
            .validate_binding(&sha(1), &sha(2), &sha(3), "system-B", now)
            .is_ok());
        assert!(req
            .validate_binding("other", &sha(2), &sha(3), "system-B", now)
            .is_err());
        assert!(req
            .validate_binding(&sha(1), &sha(2), &sha(3), "system-B", req.expires_at)
            .is_err());
    }
}
