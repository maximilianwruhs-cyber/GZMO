//! Canonical JSON digests and hash-linked audit events.
//!
//! Pure domain values only: no filesystem, network, database, or signing.

use crate::{CandidateId, AUDIT_SCHEMA};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Genesis previous-hash for the first event in a chain (64 zero hex digits).
pub const GENESIS_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Errors raised while building or verifying audit contracts.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuditError {
    /// Single event failed structural or hash validation.
    #[error("invalid audit event: {0}")]
    InvalidEvent(String),
    /// Chain linkage or sequencing failed.
    #[error("invalid audit chain: {0}")]
    InvalidChain(String),
    /// Canonical JSON encoding rejected the value.
    #[error("canonical json error: {0}")]
    CanonicalJson(String),
}

/// One tamper-evident audit ledger entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct AuditEvent {
    pub schema: String,
    pub sequence: u64,
    pub previous_hash: String,
    pub event_type: String,
    pub candidate_id: Option<CandidateId>,
    pub payload_digest: String,
    pub occurred_at: DateTime<Utc>,
    pub event_hash: String,
}

#[derive(Deserialize)]
struct RawAuditEvent {
    schema: String,
    sequence: u64,
    previous_hash: String,
    event_type: String,
    candidate_id: Option<CandidateId>,
    payload_digest: String,
    occurred_at: DateTime<Utc>,
    event_hash: String,
}

/// Private preimage hashed into `event_hash` (every field except `event_hash`).
#[derive(Serialize)]
struct AuditPreimage<'a> {
    schema: &'a str,
    sequence: u64,
    previous_hash: &'a str,
    event_type: &'a str,
    candidate_id: Option<&'a CandidateId>,
    payload_digest: &'a str,
    occurred_at: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for AuditEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawAuditEvent::deserialize(deserializer)?;
        let value = Self {
            schema: raw.schema,
            sequence: raw.sequence,
            previous_hash: raw.previous_hash,
            event_type: raw.event_type,
            candidate_id: raw.candidate_id,
            payload_digest: raw.payload_digest,
            occurred_at: raw.occurred_at,
            event_hash: raw.event_hash,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl AuditEvent {
    /// Append the next event, or start a chain when `previous` is `None`.
    ///
    /// Genesis uses sequence `1` and [`GENESIS_HASH`]. A prior event is validated
    /// first; sequence uses checked arithmetic. Payload and event hashes are
    /// derived from canonical JSON only (never delimiter-free concatenation).
    pub fn next(
        previous: Option<&AuditEvent>,
        event_type: impl AsRef<str>,
        candidate_id: Option<CandidateId>,
        payload: &impl Serialize,
    ) -> Result<Self, AuditError> {
        let event_type = event_type.as_ref();
        validate_event_type(event_type)?;

        let (sequence, previous_hash) = match previous {
            None => (1u64, GENESIS_HASH.to_owned()),
            Some(prev) => {
                prev.validate()?;
                let sequence = prev.sequence.checked_add(1).ok_or_else(|| {
                    AuditError::InvalidEvent("sequence overflow on checked_add".to_owned())
                })?;
                (sequence, prev.event_hash.clone())
            }
        };

        let payload_digest = sha256_hex(&canonical_json_bytes(payload)?);
        let mut event = Self {
            schema: AUDIT_SCHEMA.to_owned(),
            sequence,
            previous_hash,
            event_type: event_type.to_owned(),
            candidate_id,
            payload_digest,
            occurred_at: Utc::now(),
            event_hash: String::new(),
        };
        event.event_hash = event.compute_event_hash()?;
        Ok(event)
    }

    /// Structural validation plus exact recomputation of `event_hash`.
    pub fn validate(&self) -> Result<(), AuditError> {
        if self.schema != AUDIT_SCHEMA {
            return Err(AuditError::InvalidEvent(format!(
                "schema must be {AUDIT_SCHEMA}"
            )));
        }
        if self.sequence == 0 {
            return Err(AuditError::InvalidEvent(
                "sequence must be >= 1".to_owned(),
            ));
        }
        validate_event_type(&self.event_type)?;
        validate_hash_hex("previous_hash", &self.previous_hash)?;
        validate_hash_hex("payload_digest", &self.payload_digest)?;
        validate_hash_hex("event_hash", &self.event_hash)?;

        let expected = self.compute_event_hash()?;
        if self.event_hash != expected {
            return Err(AuditError::InvalidEvent(
                "event_hash does not match recomputed preimage digest".to_owned(),
            ));
        }
        Ok(())
    }

    fn preimage(&self) -> AuditPreimage<'_> {
        AuditPreimage {
            schema: &self.schema,
            sequence: self.sequence,
            previous_hash: &self.previous_hash,
            event_type: &self.event_type,
            candidate_id: self.candidate_id.as_ref(),
            payload_digest: &self.payload_digest,
            occurred_at: self.occurred_at,
        }
    }

    fn compute_event_hash(&self) -> Result<String, AuditError> {
        Ok(sha256_hex(&canonical_json_bytes(&self.preimage())?))
    }
}

/// Verify an ordered audit chain.
///
/// An empty slice is a valid uninitialized ledger. A nonempty chain must start
/// at sequence 1 with [`GENESIS_HASH`], increment by exactly one, link each
/// `previous_hash` to the prior `event_hash`, and pass per-event validation.
pub fn verify_chain(events: &[AuditEvent]) -> Result<(), AuditError> {
    if events.is_empty() {
        return Ok(());
    }

    let first = &events[0];
    if first.sequence != 1 {
        return Err(AuditError::InvalidChain(format!(
            "first sequence must be 1, got {}",
            first.sequence
        )));
    }
    if first.previous_hash != GENESIS_HASH {
        return Err(AuditError::InvalidChain(
            "first previous_hash must be the zero genesis hash".to_owned(),
        ));
    }
    first.validate()?;

    for window in events.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        curr.validate()?;

        let expected_seq = prev.sequence.checked_add(1).ok_or_else(|| {
            AuditError::InvalidChain("sequence overflow while verifying chain".to_owned())
        })?;
        if curr.sequence != expected_seq {
            return Err(AuditError::InvalidChain(format!(
                "sequence gap: expected {expected_seq}, got {}",
                curr.sequence
            )));
        }
        if curr.previous_hash != prev.event_hash {
            return Err(AuditError::InvalidChain(
                "previous_hash does not match preceding event_hash".to_owned(),
            ));
        }
    }
    Ok(())
}

/// Recursively sort object keys, preserve array order, compact UTF-8 JSON bytes.
///
/// Rejects non-finite numbers and values that cannot serialize as JSON.
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, AuditError> {
    // serde_json maps NaN/±Inf to JSON null; reject them explicitly first.
    reject_nonfinite(value)?;
    let value = serde_json::to_value(value).map_err(|err| {
        AuditError::CanonicalJson(format!("serialize to json value failed: {err}"))
    })?;
    let canonical = canonicalize_value(value)?;
    serde_json::to_vec(&canonical)
        .map_err(|err| AuditError::CanonicalJson(format!("compact encode failed: {err}")))
}


/// SHA-256 digest as exactly 64 lowercase hex characters.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(char::from_digit((byte >> 4) as u32, 16).expect("nibble"));
        out.push(char::from_digit((byte & 0x0f) as u32, 16).expect("nibble"));
    }
    out
}

fn canonicalize_value(value: Value) -> Result<Value, AuditError> {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut sorted = Map::with_capacity(entries.len());
            for (key, child) in entries {
                sorted.insert(key, canonicalize_value(child)?);
            }
            Ok(Value::Object(sorted))
        }
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(canonicalize_value(item)?);
            }
            Ok(Value::Array(out))
        }
        Value::Number(number) => {
            if let Some(float) = number.as_f64() {
                if !float.is_finite() {
                    return Err(AuditError::CanonicalJson(
                        "non-finite JSON number rejected".to_owned(),
                    ));
                }
            }
            Ok(Value::Number(number))
        }
        other => Ok(other),
    }
}

fn validate_event_type(event_type: &str) -> Result<(), AuditError> {
    let len = event_type.len();
    if !(1..=128).contains(&len) {
        return Err(AuditError::InvalidEvent(format!(
            "event_type length must be 1..=128, got {len}"
        )));
    }
    if !event_type.bytes().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'_' || b == b'-'
    }) {
        return Err(AuditError::InvalidEvent(format!(
            "event_type must be lowercase ascii [a-z0-9._-], got {event_type:?}"
        )));
    }
    Ok(())
}

fn validate_hash_hex(field: &str, hex: &str) -> Result<(), AuditError> {
    if hex.len() != 64 {
        return Err(AuditError::InvalidEvent(format!(
            "{field} must be exactly 64 hex chars, got length {}",
            hex.len()
        )));
    }
    if !hex
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(AuditError::InvalidEvent(format!(
            "{field} must be lowercase 0-9a-f, got {hex:?}"
        )));
    }
    Ok(())
}

fn reject_nonfinite<T: Serialize + ?Sized>(value: &T) -> Result<(), AuditError> {
    value.serialize(FiniteChecker)
}

/// Serde serializer that only validates finiteness of floating-point leaves.
struct FiniteChecker;

impl serde::Serializer for FiniteChecker {
    type Ok = ();
    type Error = AuditError;
    type SerializeSeq = FiniteList;
    type SerializeTuple = FiniteList;
    type SerializeTupleStruct = FiniteList;
    type SerializeTupleVariant = FiniteList;
    type SerializeMap = FiniteMap;
    type SerializeStruct = FiniteMap;
    type SerializeStructVariant = FiniteMap;

    fn serialize_bool(self, _: bool) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), AuditError> {
        if v.is_finite() {
            Ok(())
        } else {
            Err(AuditError::CanonicalJson(
                "non-finite f32 rejected".to_owned(),
            ))
        }
    }
    fn serialize_f64(self, v: f64) -> Result<(), AuditError> {
        if v.is_finite() {
            Ok(())
        } else {
            Err(AuditError::CanonicalJson(
                "non-finite f64 rejected".to_owned(),
            ))
        }
    }
    fn serialize_char(self, _: char) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn serialize_unit(self) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), AuditError> {
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<FiniteList, AuditError> {
        Ok(FiniteList)
    }
    fn serialize_tuple(self, _: usize) -> Result<FiniteList, AuditError> {
        Ok(FiniteList)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<FiniteList, AuditError> {
        Ok(FiniteList)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<FiniteList, AuditError> {
        Ok(FiniteList)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<FiniteMap, AuditError> {
        Ok(FiniteMap)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<FiniteMap, AuditError> {
        Ok(FiniteMap)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<FiniteMap, AuditError> {
        Ok(FiniteMap)
    }
}

struct FiniteList;
struct FiniteMap;

impl serde::ser::SerializeSeq for FiniteList {
    type Ok = ();
    type Error = AuditError;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}
impl serde::ser::SerializeTuple for FiniteList {
    type Ok = ();
    type Error = AuditError;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}
impl serde::ser::SerializeTupleStruct for FiniteList {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}
impl serde::ser::SerializeTupleVariant for FiniteList {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}
impl serde::ser::SerializeMap for FiniteMap {
    type Ok = ();
    type Error = AuditError;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), AuditError> {
        key.serialize(FiniteChecker)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}
impl serde::ser::SerializeStruct for FiniteMap {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}
impl serde::ser::SerializeStructVariant for FiniteMap {
    type Ok = ();
    type Error = AuditError;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _: &'static str,
        value: &T,
    ) -> Result<(), AuditError> {
        value.serialize(FiniteChecker)
    }
    fn end(self) -> Result<(), AuditError> {
        Ok(())
    }
}

impl serde::ser::Error for AuditError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        AuditError::CanonicalJson(msg.to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn genesis_constant_is_64_zeros() {
        assert_eq!(GENESIS_HASH.len(), 64);
        assert!(GENESIS_HASH.chars().all(|c| c == '0'));
    }

    #[test]
    fn next_links_and_verify_empty() {
        assert!(verify_chain(&[]).is_ok());
        let first = AuditEvent::next(None, "candidate.observed", None, &serde_json::json!({"a":1}))
            .unwrap();
        assert_eq!(first.sequence, 1);
        assert_eq!(first.previous_hash, GENESIS_HASH);
        let second = AuditEvent::next(Some(&first), "candidate.prepared", None, &2u8).unwrap();
        assert!(verify_chain(&[first, second]).is_ok());
    }

    #[test]
    fn rejects_nonfinite_floats() {
        #[derive(Serialize)]
        struct Nasty {
            value: f64,
        }
        assert!(canonical_json_bytes(&Nasty { value: f64::NAN }).is_err());
        assert!(canonical_json_bytes(&Nasty {
            value: f64::INFINITY
        })
        .is_err());
        assert!(canonical_json_bytes(&Nasty {
            value: f64::NEG_INFINITY
        })
        .is_err());
    }

    #[test]
    fn preimage_excludes_event_hash_field_name() {
        let event = AuditEvent {
            schema: AUDIT_SCHEMA.to_owned(),
            sequence: 1,
            previous_hash: GENESIS_HASH.to_owned(),
            event_type: "t".to_owned(),
            candidate_id: None,
            payload_digest: "aa".repeat(32),
            occurred_at: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
            event_hash: "bb".repeat(32),
        };
        let bytes = canonical_json_bytes(&event.preimage()).unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(!text.contains("event_hash"));
        assert!(text.contains("payload_digest"));
    }
}

