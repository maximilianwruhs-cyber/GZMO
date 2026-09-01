use chrono::{TimeZone, Utc};
use evolution_contracts::{
    AuthorityTier, CandidateId, CandidateKind, CandidateState, CapabilityEnvelope, ContractError,
    PathPolicy, PolicyDecision, PolicyError, ResourceBudget, ResourceUsage, TunableRule,
    CANDIDATE_SCHEMA, ENVELOPE_SCHEMA, MAX_ADDED_LINES, MAX_ATTEMPTS, MAX_CHANGED_FILES,
    MAX_ENERGY_JOULES, MAX_INPUT_TOKENS, MAX_OUTPUT_TOKENS, MAX_TOOL_CALLS, MAX_WALL_SECONDS,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn public_contract_has_stable_v1_names() {
    assert_eq!(CANDIDATE_SCHEMA, "gzmo.evolution.candidate/v1");
    assert_eq!(
        CandidateId::parse("cand-20260901t070000z-felt-use-a1b2c3")
            .unwrap()
            .as_str(),
        "cand-20260901t070000z-felt-use-a1b2c3"
    );
    assert_eq!(CandidateKind::Code.authority_tier(), AuthorityTier::Candidate);
    assert_eq!(CandidateState::Observed.to_string(), "observed");
}

#[test]
fn candidate_id_rejects_invalid_forms() {
    let cases = [
        "",
        "cand",
        "cand-",
        "CAND-20260901t070000z-felt-use-a1b2c3",
        "cand-UPPER",
        "cand-has_underscore",
        "cand-has space",
        "cand-has..dots",
        "cand--leading-hyphen-suffix",
        "cand-trailing-hyphen-",
        "cand-short",
        &format!("cand-{}", "a".repeat(93)), // 5 + 93 = 98 > 96
    ];

    for raw in cases {
        let err = CandidateId::parse(raw).expect_err(raw);
        assert!(
            matches!(err, ContractError::InvalidCandidateId(_)),
            "expected InvalidCandidateId for {raw:?}, got {err:?}"
        );
    }
}

#[test]
fn enums_serialize_as_snake_case() {
    let kind = serde_json::to_string(&CandidateKind::ProceduralSkill).unwrap();
    assert_eq!(kind, "\"procedural_skill\"");

    let tier = serde_json::to_string(&AuthorityTier::Candidate).unwrap();
    assert_eq!(tier, "\"candidate\"");

    let state = serde_json::to_string(&CandidateState::ReviewReady).unwrap();
    assert_eq!(state, "\"review_ready\"");

    let id = CandidateId::parse("cand-20260901t070000z-felt-use-a1b2c3").unwrap();
    let id_json = serde_json::to_string(&id).unwrap();
    assert_eq!(id_json, "\"cand-20260901t070000z-felt-use-a1b2c3\"");
}

#[test]
fn candidate_id_json_round_trip_valid() {
    let raw = "\"cand-20260901t070000z-felt-use-a1b2c3\"";
    let id: CandidateId = serde_json::from_str(raw).unwrap();
    assert_eq!(id.as_str(), "cand-20260901t070000z-felt-use-a1b2c3");
    assert_eq!(serde_json::to_string(&id).unwrap(), raw);
}

#[test]
fn candidate_id_json_rejects_invalid_forms() {
    let cases = [
        "\"cand-UPPERCASEXXXX1\"",
        "\"cand-path/../traverse\"",
        "\"cand-has..dotsxxxxx\"",
        "\"not-a-candidate-idxx\"",
        "\"cand-short\"",
        "\"cand--edge-hyphenxx\"",
        "\"cand-trailing-hyphen-\"",
        "\"CAND-20260901t070000z-felt-use-a1b2c3\"",
    ];
    for raw in cases {
        assert!(
            serde_json::from_str::<CandidateId>(raw).is_err(),
            "expected JSON rejection for {raw}"
        );
    }
}

#[test]
fn candidate_id_schema_constrains_length_and_pattern() {
    let schema = schemars::schema_for!(CandidateId);
    let value = serde_json::to_value(schema).unwrap();
    assert_eq!(value["type"], "string");
    assert_eq!(value["minLength"], 16);
    assert_eq!(value["maxLength"], 96);
    assert_eq!(
        value["pattern"],
        r"^cand-[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$"
    );
}

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

fn fixture_envelope() -> CapabilityEnvelope {
    let mut tunables = BTreeMap::new();
    tunables.insert(
        "context.archive_threshold".to_owned(),
        TunableRule::FloatRange {
            min: 0.75,
            max: 0.95,
        },
    );
    tunables.insert(
        "metabolism.max_batch_items".to_owned(),
        TunableRule::IntegerRange { min: 1, max: 100 },
    );
    tunables.insert(
        "feature.enabled".to_owned(),
        TunableRule::Boolean,
    );
    tunables.insert(
        "recall.mode".to_owned(),
        TunableRule::EnumSet {
            values: ["fast", "thorough"].into_iter().map(str::to_owned).collect(),
        },
    );

    let envelope = CapabilityEnvelope {
        schema: ENVELOPE_SCHEMA.to_owned(),
        envelope_id: "env-stage1-fixture".to_owned(),
        policy_version: "v1".to_owned(),
        signer_key_id: "operator-key-1".to_owned(),
        issued_at: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
        expires_at: Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap(),
        budget: valid_budget(),
        paths: PathPolicy::stage1_default(),
        tunables,
        allowed_candidate_kinds: BTreeSet::from([CandidateKind::Memory, CandidateKind::Tunable]),
        required_gates: vec![
            "tests".to_owned(),
            "faithfulness".to_owned(),
            "bounded_resources".to_owned(),
        ],
    };
    envelope.validate().expect("fixture envelope must validate");
    envelope
}

#[test]
fn envelope_never_authorizes_code_or_protected_paths() {
    let envelope = fixture_envelope();
    assert!(envelope
        .authorize_tunable("context.archive_threshold", 0.85)
        .is_ok());
    assert!(envelope
        .authorize_tunable("context.archive_threshold", 0.30)
        .is_err());
    assert!(envelope
        .authorize_candidate_kind(CandidateKind::Code)
        .is_err());
    assert!(envelope
        .paths
        .check("docs/ADR-0014-constitutional-evolution.md")
        .is_err());
}

#[test]
fn resource_budget_enforces_every_absolute_ceiling() {
    let cases: &[(&str, Box<dyn Fn(&mut ResourceBudget)>)] = &[
        (
            "wall_seconds",
            Box::new(|b| b.wall_seconds = MAX_WALL_SECONDS + 1),
        ),
        ("max_attempts", Box::new(|b| b.max_attempts = MAX_ATTEMPTS + 1)),
        (
            "max_changed_files",
            Box::new(|b| b.max_changed_files = MAX_CHANGED_FILES + 1),
        ),
        (
            "max_added_lines",
            Box::new(|b| b.max_added_lines = MAX_ADDED_LINES + 1),
        ),
        (
            "max_tool_calls",
            Box::new(|b| b.max_tool_calls = MAX_TOOL_CALLS + 1),
        ),
        (
            "max_input_tokens",
            Box::new(|b| b.max_input_tokens = MAX_INPUT_TOKENS + 1),
        ),
        (
            "max_output_tokens",
            Box::new(|b| b.max_output_tokens = MAX_OUTPUT_TOKENS + 1),
        ),
        (
            "max_energy_joules",
            Box::new(|b| b.max_energy_joules = Some(MAX_ENERGY_JOULES + 1)),
        ),
    ];

    for (name, mutate) in cases {
        let mut budget = valid_budget();
        mutate(&mut budget);
        let err = budget
            .validate()
            .expect_err(&format!("expected ceiling failure for {name}"));
        assert!(
            matches!(err, PolicyError::InvalidBudget(_)),
            "{name}: {err:?}"
        );
    }

    // Exact ceilings are accepted.
    let mut budget = valid_budget();
    budget.wall_seconds = MAX_WALL_SECONDS;
    budget.max_attempts = MAX_ATTEMPTS;
    budget.max_changed_files = MAX_CHANGED_FILES;
    budget.max_added_lines = MAX_ADDED_LINES;
    budget.max_tool_calls = MAX_TOOL_CALLS;
    budget.max_input_tokens = MAX_INPUT_TOKENS;
    budget.max_output_tokens = MAX_OUTPUT_TOKENS;
    budget.max_energy_joules = Some(MAX_ENERGY_JOULES);
    assert!(budget.validate().is_ok());

    // Zero work limits rejected.
    for mutate in [
        Box::new(|b: &mut ResourceBudget| b.wall_seconds = 0) as Box<dyn Fn(&mut ResourceBudget)>,
        Box::new(|b| b.max_attempts = 0),
        Box::new(|b| b.max_changed_files = 0),
        Box::new(|b| b.max_added_lines = 0),
        Box::new(|b| b.max_tool_calls = 0),
        Box::new(|b| b.max_input_tokens = 0),
        Box::new(|b| b.max_output_tokens = 0),
        Box::new(|b| b.max_energy_joules = Some(0)),
    ] {
        let mut budget = valid_budget();
        mutate(&mut budget);
        assert!(budget.validate().is_err());
    }
}

#[test]
fn missing_energy_meter_is_signed_allowance_not_unlimited() {
    let mut budget = valid_budget();
    budget.max_energy_joules = None;
    budget.allow_missing_energy_meter = false;
    assert!(matches!(
        budget.validate(),
        Err(PolicyError::InvalidBudget(_))
    ));

    budget.allow_missing_energy_meter = true;
    assert!(budget.validate().is_ok());

    let usage_with_energy = ResourceUsage {
        wall_seconds: 1,
        attempts: 1,
        changed_files: 1,
        added_lines: 1,
        tool_calls: 1,
        input_tokens: 1,
        output_tokens: 1,
        energy_joules: Some(1),
    };
    assert!(
        !usage_with_energy.fits(&budget),
        "missing meter must not grant unlimited energy"
    );
    let usage_without = ResourceUsage {
        energy_joules: None,
        ..usage_with_energy.clone()
    };
    assert!(usage_without.fits(&budget));

    // When a meter ceiling is present, usage above it fails and at/under passes.
    budget.max_energy_joules = Some(10);
    budget.allow_missing_energy_meter = false;
    assert!(ResourceUsage {
        energy_joules: Some(10),
        ..usage_with_energy.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        energy_joules: Some(11),
        ..usage_with_energy
    }
    .fits(&budget));
}

#[test]
fn tunable_rules_reject_non_finite_empty_and_reversed() {
    assert!(TunableRule::FloatRange {
        min: f64::NAN,
        max: 1.0
    }
    .validate()
    .is_err());
    assert!(TunableRule::FloatRange {
        min: 0.0,
        max: f64::INFINITY
    }
    .validate()
    .is_err());
    assert!(TunableRule::FloatRange { min: 1.0, max: 0.0 }
        .validate()
        .is_err());
    assert!(TunableRule::IntegerRange { min: 5, max: 1 }
        .validate()
        .is_err());
    assert!(TunableRule::EnumSet {
        values: BTreeSet::new()
    }
    .validate()
    .is_err());
    assert!(TunableRule::EnumSet {
        values: BTreeSet::from([String::new()])
    }
    .validate()
    .is_err());

    let ok_float = TunableRule::FloatRange {
        min: 0.75,
        max: 0.95,
    };
    assert!(ok_float.validate().is_ok());
    assert!(ok_float.authorize_f64(0.85).is_ok());
    assert!(ok_float.authorize_f64(0.30).is_err());
    assert!(ok_float.authorize_f64(f64::NAN).is_err());
}

#[test]
fn candidate_kind_allowlist_accepts_only_memory_and_tunable() {
    let envelope = fixture_envelope();
    assert!(envelope
        .authorize_candidate_kind(CandidateKind::Memory)
        .is_ok());
    assert!(envelope
        .authorize_candidate_kind(CandidateKind::Tunable)
        .is_ok());
    for kind in [
        CandidateKind::Code,
        CandidateKind::Schema,
        CandidateKind::Model,
        CandidateKind::Runtime,
        CandidateKind::Evaluator,
        CandidateKind::Security,
        CandidateKind::Authority,
        CandidateKind::ProceduralSkill,
    ] {
        assert!(
            envelope.authorize_candidate_kind(kind).is_err(),
            "kind {kind} must be denied"
        );
    }

    let mut bad = fixture_envelope();
    bad.allowed_candidate_kinds.insert(CandidateKind::Code);
    assert!(matches!(
        bad.validate(),
        Err(PolicyError::InvalidEnvelope(_))
    ));
}

#[test]
fn path_policy_handles_escape_case_and_separators() {
    let paths = PathPolicy::stage1_default();

    // Protected via default patterns.
    assert!(paths.check("docs/ADR-0014-constitutional-evolution.md").is_err());
    assert!(paths.check("docs/superpowers/specs/foo.md").is_err());
    assert!(paths.check(".github/workflows/ci.yml").is_err());
    assert!(paths.check("AGENTS.md").is_err());
    assert!(paths.check("Cargo.toml").is_err());
    assert!(paths.check("Cargo.lock").is_err());
    assert!(paths.check("crates/evolution-contracts/src/lib.rs").is_err());
    assert!(paths.check("gzmo-evolver/src/main.rs").is_err());

    // Separator + case folding.
    assert!(paths.check(r"docs\ADR-0014-x.md").is_err());
    assert!(paths.check("Docs/ADR-0014-x.md").is_err());
    assert!(paths.check("CRATES/Evolution-Contracts/foo.rs").is_err());
    assert!(paths.check("agents.md").is_err());

    // Escapes and absolutes.
    assert!(paths.check("../secrets").is_err());
    assert!(paths.check("foo/../../etc/passwd").is_err());
    assert!(paths.check("/etc/passwd").is_err());
    assert!(paths.check(r"C:\Windows\system32").is_err());
    assert!(paths.check("//server/share").is_err());
    assert!(paths.check("foo//bar").is_err());
    assert!(paths.check("").is_err());

    // Allowed ordinary paths.
    assert!(paths.check("src/main.rs").is_ok());
    assert!(paths.check("crates/other/src/lib.rs").is_ok());
    assert!(paths.check("docs/notes.md").is_ok());
}

#[test]
fn policy_types_serde_round_trip() {
    let envelope = fixture_envelope();
    let json = serde_json::to_string_pretty(&envelope).unwrap();
    let decoded: CapabilityEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, envelope);
    decoded.validate().unwrap();

    let decision = PolicyDecision::Denied {
        reason: "blocked".to_owned(),
    };
    let d_json = serde_json::to_string(&decision).unwrap();
    let d_decoded: PolicyDecision = serde_json::from_str(&d_json).unwrap();
    assert_eq!(d_decoded, decision);
    assert_eq!(
        serde_json::to_string(&PolicyDecision::Allowed).unwrap(),
        "\"allowed\""
    );

    let rule = TunableRule::FloatRange {
        min: 0.75,
        max: 0.95,
    };
    let r_json = serde_json::to_string(&rule).unwrap();
    let r_decoded: TunableRule = serde_json::from_str(&r_json).unwrap();
    assert_eq!(r_decoded, rule);

    // Envelope rejects bad validity edges.
    let mut expired = fixture_envelope();
    expired.expires_at = expired.issued_at;
    assert!(expired.validate().is_err());
    expired.expires_at = expired.issued_at - chrono::Duration::seconds(1);
    assert!(expired.validate().is_err());

    let mut empty_signer = fixture_envelope();
    empty_signer.signer_key_id.clear();
    assert!(empty_signer.validate().is_err());

    let mut empty_gates = fixture_envelope();
    empty_gates.required_gates.clear();
    assert!(empty_gates.validate().is_err());

    let mut bad_schema = fixture_envelope();
    bad_schema.schema = "nope".to_owned();
    assert!(bad_schema.validate().is_err());
}

#[test]
fn resource_usage_fits_rejects_any_over_budget_field() {
    let budget = valid_budget();
    let base = ResourceUsage {
        wall_seconds: budget.wall_seconds,
        attempts: budget.max_attempts,
        changed_files: budget.max_changed_files,
        added_lines: budget.max_added_lines,
        tool_calls: budget.max_tool_calls,
        input_tokens: budget.max_input_tokens,
        output_tokens: budget.max_output_tokens,
        energy_joules: budget.max_energy_joules,
    };
    assert!(base.fits(&budget));

    assert!(!ResourceUsage {
        wall_seconds: budget.wall_seconds + 1,
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        attempts: budget.max_attempts + 1,
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        changed_files: budget.max_changed_files + 1,
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        added_lines: budget.max_added_lines + 1,
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        tool_calls: budget.max_tool_calls + 1,
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        input_tokens: budget.max_input_tokens + 1,
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        output_tokens: budget.max_output_tokens + 1,
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        energy_joules: Some(budget.max_energy_joules.unwrap() + 1),
        ..base
    }
    .fits(&budget));
}
