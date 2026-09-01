use chrono::{TimeZone, Utc};
use evolution_contracts::{
    AuthorityTier, CandidateId, CandidateKind, CandidateManifest, CandidateState, CandidateTarget,
    CapabilityEnvelope, ContractError, EvaluationError, EvaluationReport, GateClass, GateResult,
    GateStatus, PathPolicy, PolicyDecision, PolicyError, PromotionRequest, ResourceBudget,
    ResourceUsage, TunableRule, UnverifiedAuthorityGrant, CANDIDATE_SCHEMA, ENVELOPE_SCHEMA,
    EVALUATION_SCHEMA, MAX_ADDED_LINES, MAX_ATTEMPTS, MAX_CHANGED_FILES, MAX_ENERGY_JOULES,
    MAX_INPUT_TOKENS, MAX_OUTPUT_TOKENS, MAX_TOOL_CALLS, MAX_WALL_SECONDS, PROMOTION_SCHEMA,
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

    let base = ResourceUsage {
        wall_seconds: 1,
        attempts: 1,
        changed_files: 1,
        added_lines: 1,
        tool_calls: 1,
        input_tokens: 1,
        output_tokens: 1,
        energy_joules: Some(1),
    };
    // No ceiling: positive energy never fits; missing energy does.
    assert!(!base.fits(&budget));
    assert!(ResourceUsage {
        energy_joules: None,
        ..base.clone()
    }
    .fits(&budget));

    // Finite ceiling + missing usage energy requires allow_missing_energy_meter.
    budget.max_energy_joules = Some(10);
    budget.allow_missing_energy_meter = false;
    assert!(!ResourceUsage {
        energy_joules: None,
        ..base.clone()
    }
    .fits(&budget));
    budget.allow_missing_energy_meter = true;
    assert!(ResourceUsage {
        energy_joules: None,
        ..base.clone()
    }
    .fits(&budget));

    // Reported energy still compared to the ceiling either way.
    budget.allow_missing_energy_meter = false;
    assert!(ResourceUsage {
        energy_joules: Some(10),
        ..base.clone()
    }
    .fits(&budget));
    assert!(!ResourceUsage {
        energy_joules: Some(11),
        ..base
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

    // Colon / ADS and Windows trailing-dot/space components.
    assert!(paths.check("Cargo.toml:x").is_err());
    assert!(paths.check("Cargo.toml::$DATA").is_err());
    assert!(paths.check("Cargo.toml.").is_err());
    assert!(paths.check("AGENTS.md ").is_err());
    assert!(paths.check("src/foo:bar.rs").is_err());
    assert!(paths.check("src/trailing. ").is_err());

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

    // Envelope rejects bad validity edges via validate().
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
fn policy_json_deserialize_rejects_invalid_contracts() {
    let good = serde_json::to_value(fixture_envelope()).unwrap();

    // Code / Authority kinds cannot sneak in via JSON allowlist.
    for kinds in [
        serde_json::json!(["code"]),
        serde_json::json!(["authority"]),
        serde_json::json!(["memory", "code"]),
        serde_json::json!(["tunable", "authority"]),
    ] {
        let mut value = good.clone();
        value["allowed_candidate_kinds"] = kinds.clone();
        assert!(
            serde_json::from_value::<CapabilityEnvelope>(value).is_err(),
            "expected reject for kinds {kinds}"
        );
    }

    // Over-ceiling budget rejected on standalone and nested deserialize.
    let mut over = serde_json::to_value(valid_budget()).unwrap();
    over["wall_seconds"] = serde_json::json!(MAX_WALL_SECONDS + 1);
    assert!(serde_json::from_value::<ResourceBudget>(over.clone()).is_err());
    let mut env_over = good.clone();
    env_over["budget"] = over;
    assert!(serde_json::from_value::<CapabilityEnvelope>(env_over).is_err());

    // Empty protected paths rejected.
    let empty_paths = serde_json::json!({ "protected_paths": [] });
    assert!(serde_json::from_value::<PathPolicy>(empty_paths).is_err());
    let mut env_paths = good.clone();
    env_paths["paths"] = serde_json::json!({ "protected_paths": [] });
    assert!(serde_json::from_value::<CapabilityEnvelope>(env_paths).is_err());

    // Invalid tunables: reversed integer range and empty enum set.
    let reversed = serde_json::json!({ "type": "integer_range", "min": 5, "max": 1 });
    assert!(serde_json::from_value::<TunableRule>(reversed).is_err());
    let empty_enum = serde_json::json!({ "type": "enum_set", "values": [] });
    assert!(serde_json::from_value::<TunableRule>(empty_enum).is_err());
    let mut env_tunable = good.clone();
    env_tunable["tunables"] = serde_json::json!({
        "bad": { "type": "float_range", "min": 1.0, "max": 0.0 }
    });
    assert!(serde_json::from_value::<CapabilityEnvelope>(env_tunable).is_err());

    // Invalid timestamps: issued_at >= expires_at.
    let mut env_time = good.clone();
    env_time["expires_at"] = env_time["issued_at"].clone();
    assert!(serde_json::from_value::<CapabilityEnvelope>(env_time.clone()).is_err());
    env_time["expires_at"] = serde_json::json!("2020-01-01T00:00:00Z");
    assert!(serde_json::from_value::<CapabilityEnvelope>(env_time).is_err());

    // Missing energy without allowance rejected on budget JSON.
    let mut missing_energy = serde_json::to_value(valid_budget()).unwrap();
    missing_energy["max_energy_joules"] = serde_json::Value::Null;
    missing_energy["allow_missing_energy_meter"] = serde_json::json!(false);
    assert!(serde_json::from_value::<ResourceBudget>(missing_energy).is_err());
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
        ..base.clone()
    }
    .fits(&budget));
    // Finite ceiling without allow_missing rejects absent energy reading.
    assert!(!ResourceUsage {
        energy_joules: None,
        ..base
    }
    .fits(&budget));
}

#[test]
fn candidate_cannot_skip_evaluation_or_signature() {
    assert!(CandidateState::Observed.can_transition_to(CandidateState::Prepared));
    assert!(CandidateState::Prepared.can_transition_to(CandidateState::Building));
    assert!(CandidateState::Building.can_transition_to(CandidateState::Evaluating));
    assert!(!CandidateState::Building.can_transition_to(CandidateState::ReviewReady));
    assert!(CandidateState::Building.can_transition_to(CandidateState::Failed));
    assert!(!CandidateState::ReviewReady.can_transition_to(CandidateState::Accepted));
    assert!(CandidateState::ReviewReady.can_transition_to(CandidateState::PromotionPending));
    assert!(CandidateState::ReviewReady.can_transition_to(CandidateState::Rejected));
}

#[test]
fn hard_candidate_is_never_tunable_authority() {
    assert_eq!(CandidateKind::Code.authority_tier(), AuthorityTier::Candidate);
    assert_eq!(CandidateKind::Schema.authority_tier(), AuthorityTier::Candidate);
    assert_eq!(CandidateKind::Tunable.authority_tier(), AuthorityTier::Tunable);
}

#[test]
fn candidate_state_every_legal_and_illegal_transition() {
    use CandidateState::*;
    let legal: &[(CandidateState, CandidateState)] = &[
        (Observed, Prepared),
        (Observed, Failed),
        (Prepared, Building),
        (Prepared, Failed),
        (Building, Evaluating),
        (Building, Failed),
        (Evaluating, Rejected),
        (Evaluating, ReviewReady),
        (Evaluating, Failed),
        (ReviewReady, PromotionPending),
        (ReviewReady, Rejected),
        (ReviewReady, Failed),
        (PromotionPending, Soaking),
        (PromotionPending, Rejected),
        (PromotionPending, Failed),
        (Soaking, Accepted),
        (Soaking, RolledBack),
        (Soaking, Failed),
    ];
    let all = [
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
    ];
    for &from in &all {
        for &to in &all {
            let expected = legal.iter().any(|&(f, t)| f == from && t == to);
            assert_eq!(
                from.can_transition_to(to),
                expected,
                "{from:?} -> {to:?} expected {expected}"
            );
        }
    }
    // Terminal states never leave.
    for terminal in [Rejected, Accepted, RolledBack, Failed] {
        for &to in &all {
            assert!(
                !terminal.can_transition_to(to),
                "{terminal:?} must not transition to {to:?}"
            );
        }
    }
}

fn fixture_candidate_id() -> CandidateId {
    CandidateId::parse("cand-20260901t070000z-felt-use-a1b2c3").unwrap()
}

fn fixture_repo_target(id: &CandidateId) -> CandidateTarget {
    CandidateTarget::Repository {
        owner: "gzmo-org".to_owned(),
        repository: "gzmo".to_owned(),
        base_branch: "main".to_owned(),
        candidate_branch: format!("evolve/{}", id.as_str()),
    }
}

fn fixture_appliance_target() -> CandidateTarget {
    CandidateTarget::Appliance {
        node_id: "ct101".to_owned(),
        target_class: "living-appliance".to_owned(),
        inactive_target: Some("slot-b".to_owned()),
    }
}

fn fixture_repo_manifest() -> CandidateManifest {
    let id = fixture_candidate_id();
    CandidateManifest {
        schema: CANDIDATE_SCHEMA.to_owned(),
        id: id.clone(),
        mission_id: "mission-felt-use-20260901".to_owned(),
        kind: CandidateKind::Code,
        authority: AuthorityTier::Candidate,
        target: fixture_repo_target(&id),
        baseline_digest: format!("git-sha1:{}", "a".repeat(40)),
        required_gates: vec!["unit".to_owned(), "airgap".to_owned()],
        protected_paths: vec![
            "docs/ADR-0014-constitutional-evolution.md".to_owned(),
            ".github/workflows/".to_owned(),
        ],
        budget: valid_budget(),
        created_at: Utc.with_ymd_and_hms(2026, 9, 1, 7, 0, 0).unwrap(),
    }
}

fn fixture_appliance_manifest() -> CandidateManifest {
    CandidateManifest {
        schema: CANDIDATE_SCHEMA.to_owned(),
        id: fixture_candidate_id(),
        mission_id: "mission-appliance-slot".to_owned(),
        kind: CandidateKind::Runtime,
        authority: AuthorityTier::Candidate,
        target: fixture_appliance_target(),
        baseline_digest: format!("sha256:{}", "b".repeat(64)),
        required_gates: vec!["bundle-verify".to_owned()],
        protected_paths: vec!["boot.sh".to_owned()],
        budget: valid_budget(),
        created_at: Utc.with_ymd_and_hms(2026, 9, 1, 8, 0, 0).unwrap(),
    }
}

#[test]
fn candidate_manifest_serde_round_trips_valid_targets() {
    let repo = fixture_repo_manifest();
    assert!(repo.validate().is_ok());
    let repo_json = serde_json::to_string_pretty(&repo).unwrap();
    let repo_decoded: CandidateManifest = serde_json::from_str(&repo_json).unwrap();
    assert_eq!(repo_decoded, repo);

    let appliance = fixture_appliance_manifest();
    assert!(appliance.validate().is_ok());
    let app_json = serde_json::to_string_pretty(&appliance).unwrap();
    let app_decoded: CandidateManifest = serde_json::from_str(&app_json).unwrap();
    assert_eq!(app_decoded, appliance);

    // Target-only round trip.
    let target = fixture_repo_target(&fixture_candidate_id());
    let t_json = serde_json::to_string(&target).unwrap();
    let t_decoded: CandidateTarget = serde_json::from_str(&t_json).unwrap();
    assert_eq!(t_decoded, target);
}

#[test]
fn candidate_manifest_rejects_kind_authority_mismatch() {
    let mut bad = fixture_repo_manifest();
    bad.authority = AuthorityTier::Tunable;
    assert!(bad.validate().is_err());

    let mut value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["authority"] = serde_json::json!("tunable");
    assert!(serde_json::from_value::<CandidateManifest>(value.clone()).is_err());

    value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["kind"] = serde_json::json!("memory");
    // authority remains candidate while kind is memory
    assert!(serde_json::from_value::<CandidateManifest>(value).is_err());
}

#[test]
fn candidate_manifest_rejects_digest_algorithm_length_and_case() {
    let mut repo = fixture_repo_manifest();
    // Wrong algorithm for repository.
    repo.baseline_digest = format!("sha256:{}", "a".repeat(64));
    assert!(repo.validate().is_err());
    // Wrong length.
    repo.baseline_digest = format!("git-sha1:{}", "a".repeat(39));
    assert!(repo.validate().is_err());
    repo.baseline_digest = format!("git-sha1:{}", "a".repeat(41));
    assert!(repo.validate().is_err());
    // Uppercase hex rejected.
    repo.baseline_digest = format!("git-sha1:{}", "A".repeat(40));
    assert!(repo.validate().is_err());
    // Non-hex.
    repo.baseline_digest = format!("git-sha1:{}", "g".repeat(40));
    assert!(repo.validate().is_err());

    let mut app = fixture_appliance_manifest();
    app.baseline_digest = format!("git-sha1:{}", "a".repeat(40));
    assert!(app.validate().is_err());
    app.baseline_digest = format!("sha256:{}", "a".repeat(63));
    assert!(app.validate().is_err());
    app.baseline_digest = format!("sha256:{}", "A".repeat(64));
    assert!(app.validate().is_err());

    let mut value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["baseline_digest"] = serde_json::json!(format!("git-sha1:{}", "A".repeat(40)));
    assert!(serde_json::from_value::<CandidateManifest>(value).is_err());
}

#[test]
fn candidate_manifest_binds_repo_branch_to_candidate_id() {
    let mut bad = fixture_repo_manifest();
    bad.target = CandidateTarget::Repository {
        owner: "gzmo-org".to_owned(),
        repository: "gzmo".to_owned(),
        base_branch: "main".to_owned(),
        candidate_branch: "evolve/cand-20260901t070000z-other-id-zzzz".to_owned(),
    };
    assert!(bad.validate().is_err());

    let mut value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["target"]["candidate_branch"] =
        serde_json::json!("evolve/cand-20260901t070000z-other-id-zzzz");
    assert!(serde_json::from_value::<CandidateManifest>(value).is_err());
}

#[test]
fn candidate_target_rejects_unsafe_ref_and_path_syntax() {
    let id = fixture_candidate_id();
    let expected_branch = format!("evolve/{}", id.as_str());
    let unsafe_values = [
        "",
        " leading",
        "trailing ",
        "has space",
        "has..dots",
        "has@{upstream}",
        "has\\slash",
        "has:colon",
        "has~tilde",
        "has^caret",
        "has?question",
        "has*star",
        "has[bracket",
        "ends.lock",
        "/leading-slash",
        "trailing-slash/",
        "has\ncontrol",
    ];

    for bad in unsafe_values {
        for field in ["owner", "repository", "base_branch"] {
            let mut value = serde_json::json!({
                "mode": "repository",
                "owner": "gzmo-org",
                "repository": "gzmo",
                "base_branch": "main",
                "candidate_branch": expected_branch,
            });
            value[field] = serde_json::json!(bad);
            assert!(
                serde_json::from_value::<CandidateTarget>(value).is_err(),
                "expected reject {field}={bad:?}"
            );
        }
    }

    // candidate_branch must be evolve/<valid-id> shape even standalone.
    let mut bad_branch = serde_json::json!({
        "mode": "repository",
        "owner": "gzmo-org",
        "repository": "gzmo",
        "base_branch": "main",
        "candidate_branch": "feature/not-evolve",
    });
    assert!(serde_json::from_value::<CandidateTarget>(bad_branch.clone()).is_err());
    bad_branch["candidate_branch"] = serde_json::json!("evolve/not-a-valid-id");
    assert!(serde_json::from_value::<CandidateTarget>(bad_branch).is_err());
}

#[test]
fn candidate_target_rejects_ref_component_and_owner_path_tricks() {
    let id = fixture_candidate_id();
    let expected_branch = format!("evolve/{}", id.as_str());

    let base_branch_invalid = [
        ".hidden",
        "main.",
        "a//b",
        "@",
        "--upload-pack=x",
        "feature/.hidden",
        "feature/-evil",
        "feature/main.",
        "feature/@",
    ];
    for bad in base_branch_invalid {
        let value = serde_json::json!({
            "mode": "repository",
            "owner": "gzmo-org",
            "repository": "gzmo",
            "base_branch": bad,
            "candidate_branch": expected_branch,
        });
        assert!(
            serde_json::from_value::<CandidateTarget>(value.clone()).is_err(),
            "expected reject base_branch={bad:?}"
        );

        let mut manifest = fixture_repo_manifest();
        if let CandidateTarget::Repository {
            ref mut base_branch, ..
        } = manifest.target
        {
            *base_branch = bad.to_owned();
        }
        assert!(
            manifest.validate().is_err(),
            "expected validate reject base_branch={bad:?}"
        );
    }

    for (field, bad) in [("owner", "owner/evil"), ("repository", "repo/evil")] {
        let mut value = serde_json::json!({
            "mode": "repository",
            "owner": "gzmo-org",
            "repository": "gzmo",
            "base_branch": "main",
            "candidate_branch": expected_branch,
        });
        value[field] = serde_json::json!(bad);
        assert!(
            serde_json::from_value::<CandidateTarget>(value).is_err(),
            "expected reject {field}={bad:?}"
        );

        let mut manifest = fixture_repo_manifest();
        match &mut manifest.target {
            CandidateTarget::Repository {
                owner,
                repository,
                ..
            } => {
                if field == "owner" {
                    *owner = bad.to_owned();
                } else {
                    *repository = bad.to_owned();
                }
            }
            _ => unreachable!(),
        }
        assert!(
            manifest.validate().is_err(),
            "expected validate reject {field}={bad:?}"
        );
    }

    // Valid slash-separated base branch with safe components.
    let good = serde_json::json!({
        "mode": "repository",
        "owner": "gzmo-org",
        "repository": "gzmo",
        "base_branch": "feature/foo-bar",
        "candidate_branch": expected_branch,
    });
    let decoded: CandidateTarget = serde_json::from_value(good).unwrap();
    assert_eq!(
        decoded,
        CandidateTarget::Repository {
            owner: "gzmo-org".to_owned(),
            repository: "gzmo".to_owned(),
            base_branch: "feature/foo-bar".to_owned(),
            candidate_branch: expected_branch.clone(),
        }
    );

    let mut manifest = fixture_repo_manifest();
    if let CandidateTarget::Repository {
        ref mut base_branch, ..
    } = manifest.target
    {
        *base_branch = "feature/foo-bar".to_owned();
    }
    assert!(manifest.validate().is_ok());
}

#[test]
fn candidate_appliance_rejects_bad_identifiers() {
    let cases = [
        serde_json::json!({
            "mode": "appliance",
            "node_id": "CT101",
            "target_class": "living-appliance",
            "inactive_target": "slot-b"
        }),
        serde_json::json!({
            "mode": "appliance",
            "node_id": "ct101",
            "target_class": "Living-Appliance",
            "inactive_target": "slot-b"
        }),
        serde_json::json!({
            "mode": "appliance",
            "node_id": "",
            "target_class": "living-appliance",
            "inactive_target": "slot-b"
        }),
        serde_json::json!({
            "mode": "appliance",
            "node_id": "ct101",
            "target_class": "living-appliance",
            "inactive_target": "../escape"
        }),
        serde_json::json!({
            "mode": "appliance",
            "node_id": "ct101",
            "target_class": "living-appliance",
            "inactive_target": "/absolute"
        }),
        serde_json::json!({
            "mode": "appliance",
            "node_id": "ct 101",
            "target_class": "living-appliance",
            "inactive_target": null
        }),
    ];
    for value in cases {
        assert!(
            serde_json::from_value::<CandidateTarget>(value.clone()).is_err(),
            "expected reject {value}"
        );
    }

    let good = serde_json::json!({
        "mode": "appliance",
        "node_id": "ct101",
        "target_class": "living-appliance",
        "inactive_target": null
    });
    assert!(serde_json::from_value::<CandidateTarget>(good).is_ok());
}

#[test]
fn candidate_manifest_rejects_duplicate_empty_gates_and_protected_paths() {
    let mut empty_gates = fixture_repo_manifest();
    empty_gates.required_gates.clear();
    assert!(empty_gates.validate().is_err());

    let mut blank_gate = fixture_repo_manifest();
    blank_gate.required_gates = vec!["unit".to_owned(), "  ".to_owned()];
    assert!(blank_gate.validate().is_err());

    let mut dup_gates = fixture_repo_manifest();
    dup_gates.required_gates = vec!["unit".to_owned(), "unit".to_owned()];
    assert!(dup_gates.validate().is_err());

    let mut empty_paths = fixture_repo_manifest();
    empty_paths.protected_paths.clear();
    assert!(empty_paths.validate().is_err());

    let mut blank_path = fixture_repo_manifest();
    blank_path.protected_paths = vec!["src/lib.rs".to_owned(), "".to_owned()];
    assert!(blank_path.validate().is_err());

    let mut dup_paths = fixture_repo_manifest();
    dup_paths.protected_paths = vec!["docs/a.md".to_owned(), "docs\\a.md".to_owned()];
    assert!(dup_paths.validate().is_err());

    let mut value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["required_gates"] = serde_json::json!([]);
    assert!(serde_json::from_value::<CandidateManifest>(value.clone()).is_err());
    value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["protected_paths"] = serde_json::json!(["docs/a.md", "docs/a.md"]);
    assert!(serde_json::from_value::<CandidateManifest>(value).is_err());
}

#[test]
fn candidate_manifest_rejects_invalid_nested_budget_and_schema() {
    let mut bad_budget = fixture_repo_manifest();
    bad_budget.budget.wall_seconds = 0;
    assert!(bad_budget.validate().is_err());

    let mut value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["budget"]["wall_seconds"] = serde_json::json!(0);
    assert!(serde_json::from_value::<CandidateManifest>(value.clone()).is_err());

    value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["budget"]["wall_seconds"] = serde_json::json!(MAX_WALL_SECONDS + 1);
    assert!(serde_json::from_value::<CandidateManifest>(value.clone()).is_err());

    value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["schema"] = serde_json::json!("gzmo.evolution.candidate/v0");
    assert!(serde_json::from_value::<CandidateManifest>(value.clone()).is_err());

    value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["mission_id"] = serde_json::json!("");
    assert!(serde_json::from_value::<CandidateManifest>(value.clone()).is_err());

    value = serde_json::to_value(fixture_repo_manifest()).unwrap();
    value["mission_id"] = serde_json::json!("bad mission id with spaces");
    assert!(serde_json::from_value::<CandidateManifest>(value).is_err());
}


// Local hex helper so tests stay dependency-light if hex crate is absent.
mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(64);
        for b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0xf) as usize] as char);
        }
        out
    }

    pub fn encode_n(n: usize, fill: u8) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(n * 2);
        for _ in 0..n {
            out.push(HEX[(fill >> 4) as usize] as char);
            out.push(HEX[(fill & 0xf) as usize] as char);
        }
        out
    }
}

fn fixture_candidate_digest() -> String {
    format!("sha256:{}", hex::encode([0x11; 32]))
}

fn fixture_evaluation_digest() -> String {
    format!("sha256:{}", hex::encode([0x22; 32]))
}

fn fixture_policy_digest() -> String {
    format!("sha256:{}", hex::encode([0x33; 32]))
}

fn fixture_baseline_digest() -> String {
    format!("sha256:{}", hex::encode([0x44; 32]))
}

fn fixture_signature_hex() -> String {
    hex::encode_n(64, 0xab)
}

fn report_with(
    gates: Vec<GateResult>,
    metrics: BTreeMap<String, f64>,
) -> EvaluationReport {
    let mut artifact_digests = BTreeMap::new();
    artifact_digests.insert(
        "report.json".to_owned(),
        format!("sha256:{}", hex::encode([0x55; 32])),
    );
    let mut report = EvaluationReport {
        schema: EVALUATION_SCHEMA.to_owned(),
        candidate_id: fixture_candidate_id(),
        baseline_digest: fixture_baseline_digest(),
        candidate_digest: fixture_candidate_digest(),
        gates,
        hard_floors_passed: false,
        metrics,
        artifact_digests,
        completed_at: Utc.with_ymd_and_hms(2026, 9, 1, 12, 0, 0).unwrap(),
    };
    report.hard_floors_passed = report.hard_floors_pass();
    report.validate().expect("report_with fixture must validate");
    report
}

fn report_unchecked(
    gates: Vec<GateResult>,
    metrics: BTreeMap<String, f64>,
) -> EvaluationReport {
    let mut report = EvaluationReport {
        schema: EVALUATION_SCHEMA.to_owned(),
        candidate_id: fixture_candidate_id(),
        baseline_digest: fixture_baseline_digest(),
        candidate_digest: fixture_candidate_digest(),
        gates,
        hard_floors_passed: false,
        metrics,
        artifact_digests: BTreeMap::new(),
        completed_at: Utc.with_ymd_and_hms(2026, 9, 1, 12, 0, 0).unwrap(),
    };
    report.hard_floors_passed = report.hard_floors_pass();
    report
}


fn fixture_promotion_request() -> PromotionRequest {
    let request = PromotionRequest {
        schema: PROMOTION_SCHEMA.to_owned(),
        candidate_id: fixture_candidate_id(),
        candidate_digest: fixture_candidate_digest(),
        evaluation_digest: fixture_evaluation_digest(),
        policy_digest: fixture_policy_digest(),
        target: "system-B".to_owned(),
        issued_at: Utc.with_ymd_and_hms(2026, 9, 1, 10, 0, 0).unwrap(),
        expires_at: Utc.with_ymd_and_hms(2026, 9, 1, 18, 0, 0).unwrap(),
        nonce: "nonce-fixture-001".to_owned(),
    };
    request
        .validate()
        .expect("fixture promotion request must validate");
    request
}

fn fixture_unverified_grant() -> UnverifiedAuthorityGrant {
    let grant = UnverifiedAuthorityGrant {
        request: fixture_promotion_request(),
        signer_key_id: "operator-key-1".to_owned(),
        signature_hex: fixture_signature_hex(),
    };
    grant.validate().expect("fixture grant must validate");
    grant
}

#[test]
fn one_failed_hard_gate_rejects_any_metric_gain() {
    let report = report_with(
        vec![
            GateResult::pass("tests"),
            GateResult::fail("faithfulness", "0.79 < 0.90"),
        ],
        [("throughput_gain_pct".into(), 300.0)].into(),
    );
    assert!(!report.hard_floors_pass());
}

#[test]
fn grant_binds_candidate_evaluation_policy_target_and_expiry() {
    let request = fixture_promotion_request();
    let now = Utc.with_ymd_and_hms(2026, 9, 1, 12, 0, 0).unwrap();
    assert!(request
        .validate_binding(
            &fixture_candidate_digest(),
            &fixture_evaluation_digest(),
            &fixture_policy_digest(),
            "system-B",
            now,
        )
        .is_ok());
    assert!(request
        .validate_binding(
            "other",
            &fixture_evaluation_digest(),
            &fixture_policy_digest(),
            "system-B",
            now,
        )
        .is_err());
}

#[test]
fn hard_floor_unavailable_also_fails_regardless_of_metrics() {
    let report = report_with(
        vec![
            GateResult::pass("tests"),
            GateResult {
                name: "integrity".to_owned(),
                class: GateClass::HardFloor,
                status: GateStatus::Unavailable,
                detail: "runner offline".to_owned(),
                artifact_digest: None,
            },
            GateResult {
                name: "throughput".to_owned(),
                class: GateClass::Metric,
                status: GateStatus::Pass,
                detail: String::new(),
                artifact_digest: None,
            },
        ],
        [("latency_improvement_pct".into(), 50.0)].into(),
    );
    assert!(!report.hard_floors_pass());
    assert!(!report.hard_floors_passed);
}

#[test]
fn all_hard_floors_pass_with_metric_noise() {
    let report = report_with(
        vec![
            GateResult::pass("tests"),
            GateResult::pass("faithfulness"),
            GateResult {
                name: "throughput".to_owned(),
                class: GateClass::Metric,
                status: GateStatus::Fail,
                detail: "slower".to_owned(),
                artifact_digest: None,
            },
        ],
        BTreeMap::new(),
    );
    assert!(report.hard_floors_pass());
    assert!(report.hard_floors_passed);
}

#[test]
fn evaluation_rejects_zero_hard_floors() {
    let mut report = report_with(
        vec![
            GateResult::pass("tests"),
            GateResult::pass("faithfulness"),
        ],
        BTreeMap::new(),
    );
    report.gates = vec![GateResult {
        name: "throughput".to_owned(),
        class: GateClass::Metric,
        status: GateStatus::Pass,
        detail: String::new(),
        artifact_digest: None,
    }];
    assert!(!report.hard_floors_pass());
    report.hard_floors_passed = true;
    assert!(report.validate().is_err());
}

#[test]
fn hard_floors_pass_false_for_empty_and_metric_only_reports() {
    let empty = report_unchecked(Vec::new(), BTreeMap::new());
    assert!(!empty.hard_floors_pass());

    let metric_only = report_unchecked(
        vec![GateResult {
            name: "throughput".to_owned(),
            class: GateClass::Metric,
            status: GateStatus::Pass,
            detail: String::new(),
            artifact_digest: None,
        }],
        [("gain".into(), 1.0)].into(),
    );
    assert!(!metric_only.hard_floors_pass());

    let passed = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    assert!(passed.hard_floors_pass());

    let failed = report_with(
        vec![GateResult::fail("tests", "nope")],
        BTreeMap::new(),
    );
    assert!(!failed.hard_floors_pass());

    let unavailable = report_with(
        vec![GateResult {
            name: "tests".to_owned(),
            class: GateClass::HardFloor,
            status: GateStatus::Unavailable,
            detail: "offline".to_owned(),
            artifact_digest: None,
        }],
        BTreeMap::new(),
    );
    assert!(!unavailable.hard_floors_pass());
}


#[test]
fn evaluation_rejects_duplicate_and_unsafe_gate_names() {
    let mut report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.gates = vec![
        GateResult::pass("tests"),
        GateResult::pass("tests"),
    ];
    report.hard_floors_passed = true;
    assert!(report.validate().is_err());

    report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.gates = vec![GateResult::pass("../escape")];
    report.hard_floors_passed = true;
    assert!(report.validate().is_err());

    report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.gates = vec![GateResult::pass("has space")];
    report.hard_floors_passed = true;
    assert!(report.validate().is_err());

    report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.gates = vec![GateResult::pass("")];
    report.hard_floors_passed = true;
    assert!(report.validate().is_err());
}

#[test]
fn evaluation_rejects_oversized_detail() {
    let mut report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.gates = vec![GateResult::fail("tests", "x".repeat(4097))];
    report.hard_floors_passed = false;
    let err = report.validate().expect_err("detail over 4096 bytes");
    assert!(
        matches!(
            err,
            EvaluationError::InvalidGate(_) | EvaluationError::InvalidReport(_)
        ),
        "unexpected error: {err:?}"
    );
}


#[test]
fn evaluation_rejects_invalid_and_mixed_digest_algorithms() {
    let mut report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.baseline_digest = "md5:deadbeef".to_owned();
    assert!(report.validate().is_err());

    report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.candidate_digest = format!("git-sha1:{}", "g".repeat(40));
    assert!(report.validate().is_err());

    report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.candidate_digest = format!("sha256:{}", "A".repeat(64));
    assert!(report.validate().is_err());

    report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.artifact_digests.insert(
        "bundle.tar".to_owned(),
        format!("git-sha1:{}", "a".repeat(40)),
    );
    assert!(report.validate().is_err());

    report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.gates[0].artifact_digest = Some(format!("git-sha1:{}", "b".repeat(40)));
    assert!(report.validate().is_err());
}

#[test]
fn evaluation_rejects_nonfinite_metrics() {
    let mut report = report_with(vec![GateResult::pass("tests")], BTreeMap::new());
    report.metrics.insert("gain".to_owned(), f64::NAN);
    assert!(report.validate().is_err());
    report.metrics.insert("gain".to_owned(), f64::INFINITY);
    assert!(report.validate().is_err());
    report.metrics.insert("gain".to_owned(), f64::NEG_INFINITY);
    assert!(report.validate().is_err());
}

#[test]
fn evaluation_rejects_forged_hard_floor_verdict() {
    let mut report = report_with(
        vec![
            GateResult::pass("tests"),
            GateResult::fail("faithfulness", "below floor"),
        ],
        BTreeMap::new(),
    );
    assert!(!report.hard_floors_pass());
    report.hard_floors_passed = true;
    assert!(report.validate().is_err());

    let mut value = serde_json::to_value(
        report_with(
            vec![
                GateResult::pass("tests"),
                GateResult::fail("faithfulness", "below floor"),
            ],
            BTreeMap::new(),
        ),
    )
    .unwrap();
    value["hard_floors_passed"] = serde_json::json!(true);
    let err = serde_json::from_value::<EvaluationReport>(value).unwrap_err();
    assert!(
        err.to_string().contains("hard_floors_passed")
            || err.to_string().contains("forged")
            || err.to_string().contains("mismatch"),
        "unexpected error: {err}"
    );
}

#[test]
fn evaluation_serde_round_trip_valid() {
    let report = report_with(
        vec![
            GateResult::pass("tests"),
            GateResult::pass("faithfulness"),
            GateResult {
                name: "holdout.memory_lifecycle".to_owned(),
                class: GateClass::HardFloor,
                status: GateStatus::Pass,
                detail: String::new(),
                artifact_digest: Some(format!("sha256:{}", hex::encode([0x66; 32]))),
            },
            GateResult {
                name: "throughput".to_owned(),
                class: GateClass::Metric,
                status: GateStatus::Fail,
                detail: "no gain".to_owned(),
                artifact_digest: None,
            },
        ],
        [("throughput_gain_pct".into(), 12.5)].into(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let back: EvaluationReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back, report);
    assert!(back.hard_floors_pass());
}

#[test]
fn promotion_rejects_malformed_signature_signer_nonce_target() {
    let mut grant = fixture_unverified_grant();
    grant.signature_hex = "ab".repeat(63); // 126 chars
    assert!(grant.validate().is_err());

    grant = fixture_unverified_grant();
    grant.signature_hex = "AB".repeat(64); // uppercase
    assert!(grant.validate().is_err());

    grant = fixture_unverified_grant();
    grant.signature_hex.push('0'); // 129
    assert!(grant.validate().is_err());

    grant = fixture_unverified_grant();
    grant.signer_key_id.clear();
    assert!(grant.validate().is_err());

    grant = fixture_unverified_grant();
    grant.signer_key_id = "bad signer".to_owned();
    assert!(grant.validate().is_err());

    let mut request = fixture_promotion_request();
    request.nonce.clear();
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.nonce = "has space".to_owned();
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.target.clear();
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.target = "../slot".to_owned();
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.evaluation_digest = format!("git-sha1:{}", "a".repeat(40));
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.policy_digest = format!("sha256:{}", "G".repeat(64));
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.candidate_digest = "not-a-digest".to_owned();
    assert!(request.validate().is_err());
}

#[test]
fn promotion_rejects_equal_inverted_and_over_24h_expiry() {
    let issued = Utc.with_ymd_and_hms(2026, 9, 1, 10, 0, 0).unwrap();

    let mut request = fixture_promotion_request();
    request.issued_at = issued;
    request.expires_at = issued;
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.issued_at = issued;
    request.expires_at = issued - chrono::Duration::hours(1);
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.issued_at = issued;
    request.expires_at = issued + chrono::Duration::hours(24) + chrono::Duration::seconds(1);
    assert!(request.validate().is_err());

    request = fixture_promotion_request();
    request.issued_at = issued;
    request.expires_at = issued + chrono::Duration::hours(24);
    assert!(request.validate().is_ok());
}

#[test]
fn promotion_binding_rejects_expiry_at_supplied_now_and_mismatches() {
    let request = fixture_promotion_request();
    let cand = fixture_candidate_digest();
    let eval = fixture_evaluation_digest();
    let pol = fixture_policy_digest();
    let target = "system-B";

    // Time window: before / equal-issued / inside / equal-expiry / after.
    assert!(request
        .validate_binding(
            &cand,
            &eval,
            &pol,
            target,
            request.issued_at - chrono::Duration::seconds(1),
        )
        .is_err());
    assert!(request
        .validate_binding(&cand, &eval, &pol, target, request.issued_at)
        .is_ok());
    assert!(request
        .validate_binding(
            &cand,
            &eval,
            &pol,
            target,
            request.issued_at + chrono::Duration::hours(1),
        )
        .is_ok());
    assert!(request
        .validate_binding(
            &cand,
            &eval,
            &pol,
            target,
            request.expires_at - chrono::Duration::seconds(1),
        )
        .is_ok());
    assert!(request
        .validate_binding(&cand, &eval, &pol, target, request.expires_at)
        .is_err());
    assert!(request
        .validate_binding(
            &cand,
            &eval,
            &pol,
            target,
            request.expires_at + chrono::Duration::seconds(1),
        )
        .is_err());

    assert!(request
        .validate_binding("wrong-cand", &eval, &pol, target, request.issued_at)
        .is_err());
    assert!(request
        .validate_binding(&cand, "wrong-eval", &pol, target, request.issued_at)
        .is_err());
    assert!(request
        .validate_binding(&cand, &eval, "wrong-pol", target, request.issued_at)
        .is_err());
    assert!(request
        .validate_binding(&cand, &eval, &pol, "other-target", request.issued_at)
        .is_err());
}


#[test]
fn promotion_serde_round_trip_valid() {
    let grant = fixture_unverified_grant();
    let json = serde_json::to_string(&grant).unwrap();
    let back: UnverifiedAuthorityGrant = serde_json::from_str(&json).unwrap();
    assert_eq!(back, grant);

    let request = fixture_promotion_request();
    let json = serde_json::to_string(&request).unwrap();
    let back: PromotionRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back, request);
}

#[test]
fn promotion_json_deserialize_rejects_invalid_contracts() {
    let mut value = serde_json::to_value(fixture_unverified_grant()).unwrap();
    value["signature_hex"] = serde_json::json!("00".repeat(63));
    assert!(serde_json::from_value::<UnverifiedAuthorityGrant>(value).is_err());

    let mut value = serde_json::to_value(fixture_promotion_request()).unwrap();
    value["schema"] = serde_json::json!("gzmo.evolution.promotion/v0");
    assert!(serde_json::from_value::<PromotionRequest>(value.clone()).is_err());

    value = serde_json::to_value(fixture_promotion_request()).unwrap();
    value["expires_at"] = serde_json::json!(fixture_promotion_request().issued_at);
    assert!(serde_json::from_value::<PromotionRequest>(value).is_err());
}

#[test]
fn promotion_schema_constant_is_stable() {
    assert_eq!(PROMOTION_SCHEMA, "gzmo.evolution.promotion/v1");
    assert_eq!(EVALUATION_SCHEMA, "gzmo.evolution.evaluation/v1");
}

