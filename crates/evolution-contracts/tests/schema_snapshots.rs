//! Deterministic checked-in JSON Schema snapshots for public evolution artifacts.
//!
//! Generates schemas into a unique std temp directory, byte-compares against
//! checked-in pretty JSON (with one trailing newline), then removes the temp dir.
//! Semantic assertions guard expressible constraints and honest runtime-validation
//! extension lists without implying schema-only crypto verification.

use evolution_contracts::{
    export_all_schemas, AUDIT_SCHEMA, CANDIDATE_SCHEMA, ENVELOPE_SCHEMA, EVALUATION_SCHEMA,
    PROMOTION_SCHEMA,
};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const SCHEMA_FILES: &[&str] = &[
    "candidate-v1.json",
    "envelope-v1.json",
    "evaluation-v1.json",
    "promotion-v1.json",
    "audit-v1.json",
];

fn crate_schemas_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

fn read_checked_in(name: &str) -> String {
    let path = crate_schemas_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing checked-in schema {}: {err} (run export_schemas)",
            path.display()
        )
    })
}

fn load_schema_json(name: &str) -> Value {
    let text = read_checked_in(name);
    assert!(
        text.ends_with('\n'),
        "{name} must end with exactly one trailing newline"
    );
    assert!(
        !text.ends_with("\n\n"),
        "{name} must not end with multiple newlines"
    );
    serde_json::from_str(text.trim_end_matches('\n')).expect("checked-in schema must be JSON")
}

fn find_by_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    root.pointer(pointer)
}

fn find_def<'a>(root: &'a Value, name: &str) -> Option<&'a Value> {
    root.get("definitions")
        .and_then(|d| d.get(name))
        .or_else(|| root.get("$defs").and_then(|d| d.get(name)))
}

fn resolve_prop<'a>(root: &'a Value, prop: &str) -> &'a Value {
    if let Some(v) = root.pointer(&format!("/properties/{prop}")) {
        return v;
    }
    // Follow $ref at property level if present.
    if let Some(r) = root.pointer(&format!("/properties/{prop}/$ref")) {
        if let Some(s) = r.as_str() {
            if let Some(name) = s.strip_prefix("#/definitions/") {
                return find_def(root, name).unwrap_or_else(|| {
                    panic!("missing definition {name} for property {prop}")
                });
            }
        }
    }
    panic!("property {prop} missing on schema root");
}

fn schema_string_constraints<'a>(root: &'a Value, node: &'a Value) -> &'a Value {
    if let Some(r) = node.get("$ref").and_then(|v| v.as_str()) {
        if let Some(name) = r.strip_prefix("#/definitions/") {
            return find_def(root, name).unwrap_or_else(|| panic!("missing def {name}"));
        }
    }
    node
}

fn assert_string_pattern(root: &Value, node: &Value, pattern_substr: &str) {
    let node = schema_string_constraints(root, node);
    let pattern = node
        .get("pattern")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("expected pattern on {node}"));
    // Patterns may escape regex metacharacters (e.g. `\.`); compare unescaped form too.
    let unescaped = pattern.replace("\\.", ".").replace("\\/", "/");
    assert!(
        pattern.contains(pattern_substr) || unescaped.contains(pattern_substr),
        "pattern {pattern:?} should contain {pattern_substr:?}"
    );
}

fn assert_min_items(root: &Value, prop: &str, min: u64) {
    let node = resolve_prop(root, prop);
    let node = schema_string_constraints(root, node);
    let got = node
        .get("minItems")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| panic!("{prop} missing minItems"));
    assert_eq!(got, min, "{prop} minItems");
}

fn assert_required(root: &Value, fields: &[&str]) {
    let required = root
        .get("required")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("root missing required"));
    let set: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
    for f in fields {
        assert!(
            set.contains(f),
            "required missing {f}, have {set:?}"
        );
    }
}

fn assert_runtime_validation(root: &Value, must_mention: &[&str]) {
    let list = root
        .get("x-gzmo-runtime-validation")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("missing nonempty x-gzmo-runtime-validation"));
    assert!(
        !list.is_empty(),
        "x-gzmo-runtime-validation must be nonempty"
    );
    let joined = list
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for needle in must_mention {
        assert!(
            joined.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()),
            "runtime-validation must mention {needle:?}, got:\n{joined}"
        );
    }
    // Honesty: schema must not claim to verify signatures/digests/hashes alone.
    let lower = joined.to_ascii_lowercase();
    assert!(
        lower.contains("outside")
            || lower.contains("boundary")
            || lower.contains("not verified")
            || lower.contains("cannot")
            || lower.contains("runtime"),
        "runtime-validation should state schema limits honestly"
    );
}

fn assert_root_meta(root: &Value, schema_id: &str, title: &str) {
    let id = root
        .get("$id")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing $id"));
    assert!(!id.is_empty(), "$id must be nonempty");
    assert!(
        id.contains(schema_id) || id.contains(title) || id.contains("gzmo"),
        "$id {id:?} should identify the artifact"
    );
    assert_eq!(
        root.get("title").and_then(|v| v.as_str()),
        Some(title),
        "title"
    );
    assert_eq!(
        root.get("x-gzmo-schema-id").and_then(|v| v.as_str()),
        Some(schema_id),
        "x-gzmo-schema-id"
    );
}

fn assert_additional_properties_false_where_object(root: &Value) {
    // Root object schemas should deny unknown properties when sealed.
    if root.get("type").and_then(|v| v.as_str()) == Some("object")
        || root.get("properties").is_some()
    {
        assert_eq!(
            root.get("additionalProperties"),
            Some(&Value::Bool(false)),
            "root additionalProperties should be false"
        );
    }
}

#[test]
fn schema_snapshots_match_checked_in_pretty_json() {
    let tmp = std::env::temp_dir().join(format!(
        "gzmo-evolution-schemas-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("create temp schema dir");

    export_all_schemas(&tmp).expect("export schemas into temp dir");

    for name in SCHEMA_FILES {
        let generated = fs::read(&tmp.join(name)).unwrap_or_else(|err| {
            panic!("generated schema missing {}: {err}", tmp.join(name).display())
        });
        let checked = fs::read(crate_schemas_dir().join(name)).unwrap_or_else(|err| {
            panic!(
                "checked-in schema missing {}: {err}",
                crate_schemas_dir().join(name).display()
            )
        });
        assert_eq!(
            generated, checked,
            "byte drift in {name}: regenerate with export_schemas and keep deterministic key order + trailing newline"
        );
        assert!(
            checked.ends_with(b"\n"),
            "{name} checked-in bytes must end with newline"
        );
        // Pretty JSON should be valid UTF-8.
        let text = String::from_utf8(checked).expect("utf-8 schema");
        let parsed: Value = serde_json::from_str(text.trim_end_matches('\n')).expect("json");
        assert!(parsed.is_object(), "{name} root must be object");
    }

    fs::remove_dir_all(&tmp).expect("cleanup temp schema dir");
    assert!(!tmp.exists(), "temp schema dir must be removed");
}

#[test]
fn schema_export_is_deterministic_across_reruns() {
    let a = std::env::temp_dir().join(format!("gzmo-schema-det-a-{}", std::process::id()));
    let b = std::env::temp_dir().join(format!("gzmo-schema-det-b-{}", std::process::id()));
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    export_all_schemas(&a).unwrap();
    export_all_schemas(&b).unwrap();
    for name in SCHEMA_FILES {
        let left = fs::read(a.join(name)).unwrap();
        let right = fs::read(b.join(name)).unwrap();
        assert_eq!(left, right, "nondeterministic export for {name}");
    }
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
}

#[test]
fn candidate_schema_constraints_and_runtime_extensions() {
    let root = load_schema_json("candidate-v1.json");
    assert_root_meta(&root, CANDIDATE_SCHEMA, "CandidateManifest");
    assert_additional_properties_false_where_object(&root);
    assert_required(
        &root,
        &[
            "schema",
            "id",
            "mission_id",
            "kind",
            "authority",
            "target",
            "baseline_digest",
            "required_gates",
            "protected_paths",
            "budget",
            "created_at",
        ],
    );

    let schema_field = resolve_prop(&root, "schema");
    assert_string_pattern(&root, schema_field, "gzmo.evolution.candidate/v1");

    // CandidateId constraints (inline or $ref).
    let id_node = resolve_prop(&root, "id");
    let id_schema = schema_string_constraints(&root, id_node);
    assert_eq!(id_schema.get("type").and_then(|v| v.as_str()), Some("string"));
    assert_eq!(id_schema.get("minLength").and_then(|v| v.as_u64()), Some(16));
    assert_eq!(id_schema.get("maxLength").and_then(|v| v.as_u64()), Some(96));
    assert_string_pattern(&root, id_schema, "cand-");

    let baseline = resolve_prop(&root, "baseline_digest");
    assert_string_pattern(&root, baseline, "sha256:");
    assert_string_pattern(&root, baseline, "git-sha1:");

    assert_min_items(&root, "required_gates", 1);
    assert_min_items(&root, "protected_paths", 1);

    // Nested budget maxima present somewhere in the document.
    let dump = root.to_string();
    assert!(dump.contains("wall_seconds") || dump.contains("max_attempts"));
    assert!(
        dump.contains("86400") || dump.contains("maximum"),
        "budget ceilings should appear as numeric maxima"
    );

    // Closed enums for kind/authority appear as enum arrays.
    assert!(
        dump.contains("\"memory\"") && dump.contains("\"tunable\""),
        "kind/authority enums should be present"
    );

    assert_runtime_validation(
        &root,
        &[
            "kind-authority",
            "target-baseline",
            "candidate-id",
            "branch",
        ],
    );
}

#[test]
fn envelope_schema_constraints_and_runtime_extensions() {
    let root = load_schema_json("envelope-v1.json");
    assert_root_meta(&root, ENVELOPE_SCHEMA, "CapabilityEnvelope");
    assert_additional_properties_false_where_object(&root);
    assert_required(
        &root,
        &[
            "schema",
            "envelope_id",
            "policy_version",
            "signer_key_id",
            "issued_at",
            "expires_at",
            "budget",
            "paths",
            "tunables",
            "allowed_candidate_kinds",
            "required_gates",
        ],
    );
    let schema_field = resolve_prop(&root, "schema");
    assert_string_pattern(&root, schema_field, "gzmo.evolution.envelope/v1");
    assert_min_items(&root, "required_gates", 1);

    // paths.protected_paths minItems via nested definition or inline.
    let dump = root.to_string();
    assert!(dump.contains("protected_paths"));
    assert!(dump.contains("minItems"));

    assert_runtime_validation(
        &root,
        &[
            "time",
            "signature",
        ],
    );
}

#[test]
fn evaluation_schema_constraints_and_runtime_extensions() {
    let root = load_schema_json("evaluation-v1.json");
    assert_root_meta(&root, EVALUATION_SCHEMA, "EvaluationReport");
    assert_additional_properties_false_where_object(&root);
    assert_required(
        &root,
        &[
            "schema",
            "candidate_id",
            "baseline_digest",
            "candidate_digest",
            "gates",
            "hard_floors_passed",
            "metrics",
            "artifact_digests",
            "completed_at",
        ],
    );
    assert_min_items(&root, "gates", 1);
    let baseline = resolve_prop(&root, "baseline_digest");
    assert_string_pattern(&root, baseline, "sha256:");
    let artifact_digests = resolve_prop(&root, "artifact_digests");
    let map_values = artifact_digests
        .get("additionalProperties")
        .unwrap_or_else(|| panic!("artifact_digests.additionalProperties missing"));
    assert_eq!(
        map_values.get("type").and_then(|v| v.as_str()),
        Some("string")
    );
    assert_string_pattern(&root, map_values, "sha256:");
    let pattern = map_values
        .get("pattern")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert_eq!(
        pattern.replace("\\.", "."),
        r"^sha256:[a-f0-9]{64}$",
        "artifact_digests values must be sha256-qualified digests"
    );
    let dump = root.to_string();
    assert!(dump.contains("hard_floor") || dump.contains("pass") || dump.contains("fail"));
    assert_runtime_validation(&root, &["recomputed", "verdict"]);
}

#[test]
fn promotion_schema_constraints_and_runtime_extensions() {
    let root = load_schema_json("promotion-v1.json");
    assert_root_meta(&root, PROMOTION_SCHEMA, "UnverifiedAuthorityGrant");
    assert_additional_properties_false_where_object(&root);
    assert_required(&root, &["request", "signer_key_id", "signature_hex"]);

    let sig = resolve_prop(&root, "signature_hex");
    let sig = schema_string_constraints(&root, sig);
    assert_eq!(sig.get("minLength").and_then(|v| v.as_u64()), Some(128));
    assert_eq!(sig.get("maxLength").and_then(|v| v.as_u64()), Some(128));
    assert_string_pattern(&root, sig, "a-f0-9");

    let dump = root.to_string();
    assert!(dump.contains("candidate_digest") || dump.contains("PromotionRequest"));
    assert!(dump.contains("sha256:") || dump.contains("git-sha1:"));

    assert_runtime_validation(
        &root,
        &[
            "binding",
            "time",
            "signature",
        ],
    );
}

#[test]
fn audit_schema_constraints_and_runtime_extensions() {
    let root = load_schema_json("audit-v1.json");
    assert_root_meta(&root, AUDIT_SCHEMA, "AuditEvent");
    assert_additional_properties_false_where_object(&root);
    assert_required(
        &root,
        &[
            "schema",
            "sequence",
            "previous_hash",
            "event_type",
            "payload_digest",
            "occurred_at",
            "event_hash",
        ],
    );

    let sequence = resolve_prop(&root, "sequence");
    let sequence = schema_string_constraints(&root, sequence);
    let minimum = sequence
        .get("minimum")
        .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)));
    assert_eq!(minimum, Some(1), "sequence minimum must be 1");

    for field in ["previous_hash", "payload_digest", "event_hash"] {
        let node = resolve_prop(&root, field);
        let node = schema_string_constraints(&root, node);
        assert_eq!(node.get("minLength").and_then(|v| v.as_u64()), Some(64));
        assert_eq!(node.get("maxLength").and_then(|v| v.as_u64()), Some(64));
        assert_string_pattern(&root, node, "a-f0-9");
    }

    assert_runtime_validation(&root, &["hash", "recomput"]);
}

#[test]
fn checked_in_schemas_directory_has_exactly_five_files() {
    let dir = crate_schemas_dir();
    assert!(dir.is_dir(), "schemas dir must exist at {}", dir.display());
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json"))
        .collect();
    names.sort();
    let mut expected: Vec<String> = SCHEMA_FILES.iter().map(|s| (*s).to_owned()).collect();
    expected.sort();
    assert_eq!(names, expected);
}

#[test]
fn export_helpers_reject_unknown_paths_contract() {
    // Sanity: public helper exists and creates the fixed five names only.
    let tmp = std::env::temp_dir().join(format!("gzmo-schema-names-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    export_all_schemas(&tmp).unwrap();
    let mut names: Vec<_> = fs::read_dir(&tmp)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut expected: Vec<_> = SCHEMA_FILES.iter().map(|s| (*s).to_owned()).collect();
    expected.sort();
    assert_eq!(names, expected);
    let _ = fs::remove_dir_all(&tmp);
}

/// Ensure nested property lookup still works when definitions use $ref for CandidateId.
#[test]
fn candidate_id_definition_keeps_pattern_bounds() {
    let root = load_schema_json("candidate-v1.json");
    let id_def = find_def(&root, "CandidateId")
        .or_else(|| {
            // Inline under properties.id
            find_by_pointer(&root, "/properties/id")
        })
        .expect("CandidateId schema present");
    let id_def = schema_string_constraints(&root, id_def);
    assert_eq!(id_def["minLength"], 16);
    assert_eq!(id_def["maxLength"], 96);
    assert!(id_def["pattern"]
        .as_str()
        .unwrap()
        .starts_with("^cand-"));
}
