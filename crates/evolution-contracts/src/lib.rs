//! Pure domain contracts for GZMO evolution artifacts.
//!
//! No filesystem, network, database, or process side effects in domain types.
//! Schema export helpers write deterministic JSON Schema documents only.

pub mod audit;
pub mod candidate;
pub mod evaluation;
pub mod policy;
pub mod promotion;
pub mod schema_meta;

pub use audit::*;
pub use candidate::*;
pub use evaluation::*;
pub use policy::*;
pub use promotion::*;

pub const CANDIDATE_SCHEMA: &str = "gzmo.evolution.candidate/v1";
pub const ENVELOPE_SCHEMA: &str = "gzmo.evolution.envelope/v1";
pub const EVALUATION_SCHEMA: &str = "gzmo.evolution.evaluation/v1";
pub const PROMOTION_SCHEMA: &str = "gzmo.evolution.promotion/v1";
pub const AUDIT_SCHEMA: &str = "gzmo.evolution.audit/v1";

use schemars::schema::RootSchema;
use std::path::Path;

/// Cross-field invariants JSON Schema cannot express for candidate manifests.
pub const CANDIDATE_RUNTIME_VALIDATION: &[&str] = &[
    "kind-authority: authority must equal kind.authority_tier(); not expressible as a pure schema correlation",
    "target-baseline: baseline_digest algorithm depends on target mode (repository→git-sha1, appliance→sha256)",
    "candidate-id-branch: repository candidate_branch must equal evolve/<id> bound to manifest id",
    "runtime Deserialize/validate remains authoritative; schema does not verify digests or bindings",
];

/// Cross-field invariants for capability envelopes.
pub const ENVELOPE_RUNTIME_VALIDATION: &[&str] = &[
    "time-window: issued_at must be strictly before expires_at; authorize_* requires issued_at <= now < expires_at via validate_at",
    "allowed_candidate_kinds: stage-1 allowlist is Memory|Tunable only (runtime enum subset check; schema enum is wider)",
    "energy-meter: max_energy_joules absence requires allow_missing_energy_meter",
    "signature-verification-boundary: envelope signature verification is outside this crate and outside schema validation",
];

/// Cross-field invariants for evaluation reports.
pub const EVALUATION_RUNTIME_VALIDATION: &[&str] = &[
    "recomputed-evaluation-verdict: hard_floors_passed must equal hard_floors_pass() over gate observations",
    "covers-required-gates: every demanded name must exist exactly once as HardFloor with Pass; empty/duplicate/unsafe required lists reject",
    "gates must include at least one hard_floor class and unique names (partially beyond minItems)",
    "baseline/candidate digest algorithms must match (sha256 with sha256, git-sha1 with git-sha1)",
    "runtime Deserialize/validate remains authoritative; schema does not recompute verdicts or required-gate coverage",
    "GateResult.detail maxLength in schema is character-oriented; runtime caps UTF-8 bytes at MAX_GATE_DETAIL_BYTES",
];

/// Cross-field invariants for unverified authority grants / promotion.
pub const PROMOTION_RUNTIME_VALIDATION: &[&str] = &[
    "promotion-binding: request digests and target must match supplied binding values at use time",
    "promotion-time-window: now must satisfy issued_at <= now < expires_at and TTL <= 24h",
    "signature-verification-boundary: signature_hex is encoding-only (128 lowercase hex); cryptographic verification is outside schema and this crate",
    "schema validation alone never verifies a signature or digest binding",
];

/// Cross-field invariants for audit events.
pub const AUDIT_RUNTIME_VALIDATION: &[&str] = &[
    "audit-hash-recomputation: event_hash must equal sha256 of canonical JSON preimage (all fields except event_hash); runtime validate/recompute_event_hash recomputes this",
    "chain linkage (sequence+previous_hash) is verified by verify_chain, not by single-event schema",
    "signature/digest boundary: schema validation alone never verifies an event hash or chain integrity; runtime validate/recompute_event_hash remains authoritative",
];

fn seal_for<T: schemars::JsonSchema>(
    id_uri: &str,
    title: &str,
    schema_id: &str,
    runtime: &[&str],
) -> RootSchema {
    let root = schemars::schema_for!(T);
    schema_meta::seal_root_schema(root, id_uri, title, schema_id, runtime)
}

/// Build the sealed root schema for a public artifact type.
pub fn sealed_schema_for_candidate() -> RootSchema {
    seal_for::<CandidateManifest>(
        "https://gzmo.dev/schemas/evolution/candidate-v1.json",
        "CandidateManifest",
        CANDIDATE_SCHEMA,
        CANDIDATE_RUNTIME_VALIDATION,
    )
}

pub fn sealed_schema_for_envelope() -> RootSchema {
    seal_for::<CapabilityEnvelope>(
        "https://gzmo.dev/schemas/evolution/envelope-v1.json",
        "CapabilityEnvelope",
        ENVELOPE_SCHEMA,
        ENVELOPE_RUNTIME_VALIDATION,
    )
}

pub fn sealed_schema_for_evaluation() -> RootSchema {
    seal_for::<EvaluationReport>(
        "https://gzmo.dev/schemas/evolution/evaluation-v1.json",
        "EvaluationReport",
        EVALUATION_SCHEMA,
        EVALUATION_RUNTIME_VALIDATION,
    )
}

pub fn sealed_schema_for_promotion() -> RootSchema {
    seal_for::<UnverifiedAuthorityGrant>(
        "https://gzmo.dev/schemas/evolution/promotion-v1.json",
        "UnverifiedAuthorityGrant",
        PROMOTION_SCHEMA,
        PROMOTION_RUNTIME_VALIDATION,
    )
}

pub fn sealed_schema_for_audit() -> RootSchema {
    seal_for::<AuditEvent>(
        "https://gzmo.dev/schemas/evolution/audit-v1.json",
        "AuditEvent",
        AUDIT_SCHEMA,
        AUDIT_RUNTIME_VALIDATION,
    )
}

/// Write all five deterministic schemas into `out_dir` in fixed order.
///
/// Creates `out_dir` if needed. Filenames are fixed:
/// candidate-v1.json, envelope-v1.json, evaluation-v1.json, promotion-v1.json, audit-v1.json.
pub fn export_all_schemas(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs_create(out_dir)?;
    let files: [(&str, RootSchema); 5] = [
        ("candidate-v1.json", sealed_schema_for_candidate()),
        ("envelope-v1.json", sealed_schema_for_envelope()),
        ("evaluation-v1.json", sealed_schema_for_evaluation()),
        ("promotion-v1.json", sealed_schema_for_promotion()),
        ("audit-v1.json", sealed_schema_for_audit()),
    ];
    for (name, root) in files {
        schema_meta::write_schema_value(&out_dir.join(name), root)?;
    }
    Ok(())
}

fn fs_create(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out_dir)?;
    Ok(())
}
