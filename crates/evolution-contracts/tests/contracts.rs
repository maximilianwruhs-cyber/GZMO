use evolution_contracts::{
    AuthorityTier, CandidateId, CandidateKind, CandidateState, ContractError, CANDIDATE_SCHEMA,
};

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
