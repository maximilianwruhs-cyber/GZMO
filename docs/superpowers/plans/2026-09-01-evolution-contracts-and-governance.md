# Evolution Contracts and Governance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the constitutional ADR lineage and one versioned Rust contract for missions, candidates, resource envelopes, evaluations, promotions, and append-only audit across connected and air-gapped evolution.

**Architecture:** `crates/evolution-contracts` is a pure domain crate: Serde/Schemars data, transition validation, canonical digests, and no filesystem/network/database process. The connected runner and later appliance controller both depend on it. ADR-0011–0014 explain authority; machine-readable envelopes enforce it.

**Tech Stack:** Rust 2021, Serde, serde_json, Schemars, Chrono, UUID, SHA-256, Bash/Python documentation checks.

**Spec:** `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`

## Global Constraints

- Preserve the approved spec and `north-star-design-approved-2026-08-31` tag unchanged.
- ADR bodies 0003–0010 remain historical; change only their status/lineage headers when the new ADRs are accepted.
- Record that GZMO never issued ADR-0001/0002; inaccessible external references are non-authoritative.
- Domain types contain no execution, GitHub, filesystem, database, or signature-private-key logic.
- A candidate cannot create an `AuthorityGrant`; verification consumes a detached signed grant from outside the candidate write set.
- Hard gates are booleans combined with logical AND; no aggregate score overrides a failure.
- State transitions are explicit and reject skipped authority stages.
- Every serialized top-level artifact carries an exact `gzmo.evolution.* /v1` schema identifier.

## File Structure

| Path | Responsibility |
|---|---|
| `docs/ADR-INDEX.md` | Canonical decision/implementation status and precedence |
| `docs/ADR-0011-self-developing-living-database.md` | Product constitution |
| `docs/ADR-0012-hardware-adaptive-immutable-appliance.md` | Hardware, boot, trust, model qualification |
| `docs/ADR-0013-authoritative-full-stack-data-plane.md` | PostgreSQL authority and derived accelerators |
| `docs/ADR-0014-constitutional-evolution.md` | Authority tiers, evaluation, promotion, rollback |
| `scripts/adr-check.sh` | Deterministic ADR-index consistency check |
| `tests/adr-index-test.sh` | Hermetic broken-lineage regression test |
| `crates/evolution-contracts/src/candidate.rs` | Candidate identity, kind, state, transition graph |
| `crates/evolution-contracts/src/policy.rs` | Resource budget, protected paths, capability envelope |
| `crates/evolution-contracts/src/evaluation.rs` | Gate and comparative fitness report |
| `crates/evolution-contracts/src/promotion.rs` | Detached grant and promotion request/result |
| `crates/evolution-contracts/src/audit.rs` | Canonical JSON digest and hash-linked events |
| `crates/evolution-contracts/src/lib.rs` | Public interface and schema constants |
| `crates/evolution-contracts/src/bin/export_schemas.rs` | Reproducible JSON Schema exporter |
| `crates/evolution-contracts/schemas/*.json` | Checked-in generated schemas |
| `crates/evolution-contracts/tests/contracts.rs` | Cross-module round-trip and invariant tests |

---

### Task 1: Make ADR Authority Legible

**Files:**
- Create: `docs/ADR-INDEX.md`
- Create: `docs/ADR-0011-self-developing-living-database.md`
- Create: `docs/ADR-0012-hardware-adaptive-immutable-appliance.md`
- Create: `docs/ADR-0013-authoritative-full-stack-data-plane.md`
- Create: `docs/ADR-0014-constitutional-evolution.md`
- Create: `scripts/adr-check.sh`
- Test: `tests/adr-index-test.sh`
- Modify later in this task: status lines only in `docs/ADR-0003-*.md` through `docs/ADR-0010-*.md`

**Interfaces:**
- Consumes: approved North Star spec sections 4–12 and 17.
- Produces: one canonical authority order and four narrow accepted decisions.

- [ ] **Step 1: Write the failing lineage test**

Create `tests/adr-index-test.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INDEX="$ROOT/docs/ADR-INDEX.md"

[[ -f "$INDEX" ]] || { echo "FAIL missing ADR index"; exit 1; }
for n in 0011 0012 0013 0014; do
  file=("$ROOT"/docs/ADR-${n}-*.md)
  [[ -f "${file[0]}" ]] || { echo "FAIL missing ADR-$n"; exit 1; }
  grep -q '^\*\*Decision status:\*\* Accepted' "${file[0]}" || {
    echo "FAIL ADR-$n not Accepted"; exit 1;
  }
done

grep -q 'ADR-0001/0002 were never issued in GZMO' "$INDEX" || {
  echo "FAIL missing 0001/0002 provenance"; exit 1;
}
if grep -RIl 'little-tools-lab/docs/adr/000[12]-' \
  "$ROOT/AGENTS.md" "$ROOT/MACHINE.md" "$ROOT/README.md" "$ROOT/docs/SPINE_FOCUS.md"; then
  echo "FAIL active authority depends on inaccessible LTL ADR"; exit 1
fi

echo "PASS ADR index"
```

- [ ] **Step 2: Run the test to prove the index is absent**

Run: `bash tests/adr-index-test.sh`

Expected: FAIL with `missing ADR index`.

- [ ] **Step 3: Write the ADR index**

`docs/ADR-INDEX.md` must define:

```markdown
# GZMO Architecture Decision Index

Authority order:
1. ADR-0011 constitutional invariants
2. ADR-0012/0013/0014 narrow target architecture
3. Accepted current-runtime ADRs not yet superseded in implementation
4. Operational docs
5. Research and proposed records

ADR-0001/0002 were never issued in GZMO. Historical links to sibling
little-tools-lab records are provenance only and are non-authoritative.

Decision status: Proposed | Accepted | Rejected | Superseded
Implementation status: Not started | In progress | Implemented | Retired
```

Include rows 0003–0014 with explicit `Superseded by` links and separate implementation state. ADR-0006 remains decision Accepted / implementation Implemented until the replacement owner path cuts over.

- [ ] **Step 4: Write four focused ADRs**

Each ADR must use:

```markdown
# ADR-NNNN — Title

**Decision status:** Accepted (YYYY-MM-DD)
**Implementation status:** Not started
**Supersedes:** ...
**Spec:** ...

## Context
## Decision
## Invariants
## Consequences
## Rejected alternatives
## Verification
```

Copy decisions—not implementation task lists—from the approved spec:

- 0011: one node, one writer, airgap, evidence-before-memory, no self-issued authority, reversible change, honest capability, one product.
- 0012: Thor-first ladder, installer/recovery courier, BootTrust A/B, encrypted internal NVMe, HIR→CM→catalog→qualification.
- 0013: PostgreSQL+pgvector sole authority; transactional outbox; Qdrant/Neo4j/Redis mandatory full-Living accelerators but rebuildable and correctness-neutral.
- 0014: M/T/C/P/A authority tiers, immutable evaluator, signed promotion, audit, soak, rollback.

- [ ] **Step 5: Update historical status headers only**

Apply the disposition table from spec §17. Do not rewrite historical context or rationale. Add `Decision status` and `Implementation status` lines while retaining original status text as `Historical status`.

- [ ] **Step 6: Write the ADR checker**

`scripts/adr-check.sh` checks unique numbers, required headings for 0011–0014, valid status vocabulary, target existence for every `Superseded by`, and absence of inaccessible LTL authority in active entry-point docs. Emit `gzmo.adr.check/v1` JSON to `data-next/adr-check/latest.json`.

- [ ] **Step 7: Run the ADR tests**

```bash
bash tests/adr-index-test.sh
bash scripts/adr-check.sh
```

Expected: both PASS; JSON `ok=true`.

- [ ] **Step 8: Commit**

```bash
git add docs/ADR-INDEX.md docs/ADR-00*.md scripts/adr-check.sh tests/adr-index-test.sh
git commit -m "docs: establish constitutional ADR authority"
```

---

### Task 2: Create the Pure Evolution Contracts Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/evolution-contracts/Cargo.toml`
- Create: `crates/evolution-contracts/src/lib.rs`
- Create: `crates/evolution-contracts/src/candidate.rs`
- Test: `crates/evolution-contracts/tests/contracts.rs`

**Interfaces:**
- Consumes: none beyond workspace libraries.
- Produces: `CandidateId`, `AuthorityTier`, `CandidateKind`, base `CandidateState`, schema constants.

- [ ] **Step 1: Write a failing public-interface test**

```rust
use evolution_contracts::{
    AuthorityTier, CandidateId, CandidateKind, CandidateState, CANDIDATE_SCHEMA,
};

#[test]
fn public_contract_has_stable_v1_names() {
    assert_eq!(CANDIDATE_SCHEMA, "gzmo.evolution.candidate/v1");
    assert_eq!(CandidateId::parse("cand-20260901t070000z-felt-use-a1b2c3").unwrap().as_str(),
               "cand-20260901t070000z-felt-use-a1b2c3");
    assert_eq!(CandidateKind::Code.authority_tier(), AuthorityTier::Candidate);
    assert_eq!(CandidateState::Observed.to_string(), "observed");
}
```

- [ ] **Step 2: Run the test to verify the crate is missing**

Run: `cargo test -p evolution-contracts`

Expected: FAIL: package not found.

- [ ] **Step 3: Add workspace dependencies and member**

Add `crates/evolution-contracts` to `workspace.members`. Add workspace dependencies:

```toml
schemars = { version = "0.8", features = ["chrono", "uuid1"] }
```

Create crate manifest using workspace `serde`, `serde_json`, `chrono`, `uuid`, `sha2`, `thiserror`, and `schemars`.

- [ ] **Step 4: Add the public module surface**

```rust
pub mod candidate;

pub use candidate::*;

pub const CANDIDATE_SCHEMA: &str = "gzmo.evolution.candidate/v1";
pub const ENVELOPE_SCHEMA: &str = "gzmo.evolution.envelope/v1";
pub const EVALUATION_SCHEMA: &str = "gzmo.evolution.evaluation/v1";
pub const AUDIT_SCHEMA: &str = "gzmo.evolution.audit/v1";
```

- [ ] **Step 5: Implement strict identifiers and base enums**

Implement `CandidateId` parsing plus `AuthorityTier`, `CandidateKind`, and `CandidateState` Serde/Schemars/Display representations. Accept only ASCII lowercase/digits/hyphens, prefix `cand-`, length 16–96, no `..`, and no leading/trailing hyphen after the prefix. Return `ContractError::InvalidCandidateId` otherwise. `CandidateManifest`, target fields, and legal transitions are deliberately added only after Task 4 creates `ResourceBudget`.

- [ ] **Step 6: Run the interface test**

Run: `cargo test -p evolution-contracts public_contract_has_stable_v1_names`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/evolution-contracts
git commit -m "feat: add shared evolution contract crate"
```

---

### Task 3: Define Candidate State and Legal Transitions

**Files:**
- Modify: `crates/evolution-contracts/src/candidate.rs`
- Modify: `crates/evolution-contracts/tests/contracts.rs`

**Interfaces:**
- Produces: `CandidateManifest`, `CandidateState::can_transition_to`, `CandidateKind::authority_tier`.
- Consumes: `ResourceBudget` from Task 4. Execute Task 4 before Task 3 as recorded in the SDD ledger.

- [ ] **Step 1: Write transition tests**

```rust
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
```

- [ ] **Step 2: Run and confirm failure**

Run: `cargo test -p evolution-contracts candidate_cannot_skip`

Expected: FAIL: transition method missing.

- [ ] **Step 3: Implement the exact state graph**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
    pub fn authority_tier(self) -> AuthorityTier {
        match self {
            Self::Memory => AuthorityTier::Memory,
            Self::Tunable => AuthorityTier::Tunable,
            Self::Authority | Self::Evaluator | Self::Security => AuthorityTier::Authority,
            Self::ProceduralSkill | Self::Code | Self::Schema | Self::Model | Self::Runtime => {
                AuthorityTier::Candidate
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CandidateTarget {
    Repository {
        owner: String,
        repository: String,
        base_branch: String,
        candidate_branch: String,
    },
    Appliance {
        node_id: String,
        target_class: String,
        inactive_target: Option<String>,
    },
}

pub fn can_transition_to(self, next: Self) -> bool {
    use CandidateState::*;
    matches!((self, next),
        (Observed, Prepared | Failed) |
        (Prepared, Building | Failed) |
        (Building, Evaluating | Failed) |
        (Evaluating, Rejected | ReviewReady | Failed) |
        (ReviewReady, PromotionPending | Rejected | Failed) |
        (PromotionPending, Soaking | Rejected | Failed) |
        (Soaking, Accepted | RolledBack | Failed)
    )
}
```

Memory candidates may use the same graph with an internally generated bounded grant; they still cannot skip evaluation/audit.

- [ ] **Step 4: Implement `CandidateManifest::validate`**

Require exact schema, nonempty mission, nonempty required gates, authority matching `CandidateKind`, and an algorithm-qualified baseline digest (`git-sha1:<40 hex>` or `sha256:<64 hex>`). `CandidateTarget::Repository` requires fixed owner/repository/base branch plus `candidate_branch = evolve/<candidate-id>`; `CandidateTarget::Appliance` requires node ID, target class, and no Git branch.

- [ ] **Step 5: Run all candidate tests**

Run: `cargo test -p evolution-contracts candidate`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/evolution-contracts
git commit -m "feat: encode legal evolution candidate transitions"
```

---

### Task 4: Define Signed Envelopes and Resource Budgets

**Files:**
- Create: `crates/evolution-contracts/src/policy.rs`
- Modify: `crates/evolution-contracts/src/lib.rs`
- Modify: `crates/evolution-contracts/tests/contracts.rs`

**Interfaces:**
- Produces: `CapabilityEnvelope`, `ResourceBudget`, `PathPolicy`, `TunableRule`, `PolicyDecision`.
- Consumes: candidate authority and kind.

- [ ] **Step 1: Write hard-boundary tests**

```rust
#[test]
fn envelope_never_authorizes_code_or_protected_paths() {
    let envelope = fixture_envelope();
    assert!(envelope.authorize_tunable("context.archive_threshold", 0.85).is_ok());
    assert!(envelope.authorize_tunable("context.archive_threshold", 0.30).is_err());
    assert!(envelope.authorize_candidate_kind(CandidateKind::Code).is_err());
    assert!(envelope.paths.check("docs/ADR-0014-constitutional-evolution.md").is_err());
}
```

- [ ] **Step 2: Run and confirm failure**

Run: `cargo test -p evolution-contracts envelope_never_authorizes`

Expected: FAIL: policy types missing.

- [ ] **Step 3: Implement resource budget validation**

Reject zero wall time, zero attempts, unbounded file/line/tool/token values, and any used amount greater than its signed maximum. `max_energy_joules=None` means the meter is unavailable and the profile must explicitly allow that absence; it never means unlimited energy.

- [ ] **Step 4: Implement path policy**

Default protected paths for Stage 1:

```rust
const DEFAULT_PROTECTED: &[&str] = &[
    ".github/workflows/",
    "docs/superpowers/specs/",
    "docs/ADR-",
    "AGENTS.md",
    "Cargo.toml",
    "Cargo.lock",
    "crates/evolution-contracts/",
    "gzmo-evolver/",
];
```

Normalize separators; reject absolute paths, `..`, symlink escape, and case-folded matches on case-insensitive hosts.

- [ ] **Step 5: Implement tunable rules**

Rules are typed `IntegerRange`, `FloatRange`, `EnumSet`, or `Boolean`. The envelope digest covers policy version, expiry, rules, budgets, protected paths, required gates, and signer key ID. Signature verification belongs outside this pure crate; this crate exposes canonical bytes.

Then add `pub mod policy; pub use policy::*;` to `lib.rs`; do not expose a module before its implementation compiles.

- [ ] **Step 6: Run policy tests**

Run: `cargo test -p evolution-contracts policy`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/evolution-contracts
git commit -m "feat: define bounded evolution capability envelopes"
```

---

### Task 5: Define Non-Compensable Evaluation and Promotion

**Files:**
- Create: `crates/evolution-contracts/src/evaluation.rs`
- Create: `crates/evolution-contracts/src/promotion.rs`
- Modify: `crates/evolution-contracts/src/lib.rs`
- Modify: `crates/evolution-contracts/tests/contracts.rs`

**Interfaces:**
- Produces: `GateResult`, `EvaluationReport::hard_floors_pass`, `PromotionRequest`, and `UnverifiedAuthorityGrant`.
- Consumes: candidate ID/state, canonical artifact digests.

- [ ] **Step 1: Write floor and grant tests**

```rust
#[test]
fn one_failed_hard_gate_rejects_any_metric_gain() {
    let report = report_with(
        vec![GateResult::pass("tests"), GateResult::fail("faithfulness", "0.79 < 0.90")],
        [("throughput_gain_pct".into(), 300.0)].into(),
    );
    assert!(!report.hard_floors_pass());
}

#[test]
fn grant_binds_candidate_evaluation_policy_target_and_expiry() {
    let request = fixture_promotion_request();
    assert!(request.validate_binding(
        "candidate-digest", "evaluation-digest", "policy-digest", "system-B"
    ).is_ok());
    assert!(request.validate_binding(
        "other", "evaluation-digest", "policy-digest", "system-B"
    ).is_err());
}
```

- [ ] **Step 2: Run and confirm failure**

Run: `cargo test -p evolution-contracts one_failed_hard_gate`

Expected: FAIL: evaluation types missing.

- [ ] **Step 3: Implement gate semantics**

`GateResult` carries name, class (`HardFloor` or `Metric`), status (`Pass`, `Fail`, `Unavailable`), detail, and artifact digest. A hard-floor `Fail` or `Unavailable` returns false. Metrics are reported but cannot change the result.

- [ ] **Step 4: Implement detached promotion binding**

`AuthorityGrant` contains signer key ID, candidate/evaluation/policy digests, target, issued/expiry time, and detached signature bytes. It cannot be constructed by a deserialized candidate without later verification; name the raw type `UnverifiedAuthorityGrant` and return `VerifiedAuthorityGrant` only from the trusted verifier crate in later plans.

Then add `pub mod evaluation; pub mod promotion; pub use evaluation::*; pub use promotion::*;` to `lib.rs` after both modules compile.

- [ ] **Step 5: Run evaluation and promotion tests**

Run: `cargo test -p evolution-contracts evaluation promotion`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/evolution-contracts
git commit -m "feat: bind hard fitness floors to promotion requests"
```

---

### Task 6: Implement Canonical Digests and Hash-Linked Audit

**Files:**
- Create: `crates/evolution-contracts/src/audit.rs`
- Modify: `crates/evolution-contracts/src/lib.rs`
- Modify: `crates/evolution-contracts/tests/contracts.rs`

**Interfaces:**
- Produces: `canonical_json_bytes`, `sha256_hex`, `AuditEvent::next`, `verify_chain`.
- Consumes: serializable payloads.

- [ ] **Step 1: Write deterministic-chain tests**

```rust
#[test]
fn audit_hash_is_stable_and_tamper_evident() {
    let first = AuditEvent::next(None, "candidate.observed", Some(candidate_id()), &payload()).unwrap();
    let second = AuditEvent::next(Some(&first), "candidate.prepared", Some(candidate_id()), &payload()).unwrap();
    assert!(verify_chain(&[first.clone(), second.clone()]).is_ok());
    let mut tampered = second;
    tampered.payload_digest = "00".repeat(32);
    assert!(verify_chain(&[first, tampered]).is_err());
}
```

- [ ] **Step 2: Run and confirm failure**

Run: `cargo test -p evolution-contracts audit_hash_is_stable`

Expected: FAIL: audit implementation missing.

- [ ] **Step 3: Implement canonical JSON**

Recursively sort JSON object keys, preserve array order, reject non-finite floats before serialization, and serialize without whitespace. Hash `schema || sequence || previous_hash || event_type || candidate_id || payload_digest || occurred_at` with SHA-256.

Then add `pub mod audit; pub use audit::*;` to `lib.rs` after the audit implementation compiles.

- [ ] **Step 4: Implement chain verification**

Require sequence starts at one, increments by one, every `previous_hash` equals the previous event hash, and each event hash recomputes exactly.

- [ ] **Step 5: Run audit tests**

Run: `cargo test -p evolution-contracts audit`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/evolution-contracts
git commit -m "feat: add tamper-evident evolution audit contracts"
```

---

### Task 7: Export and Freeze JSON Schemas

**Files:**
- Create: `crates/evolution-contracts/src/bin/export_schemas.rs`
- Create: `crates/evolution-contracts/schemas/candidate-v1.json`
- Create: `crates/evolution-contracts/schemas/envelope-v1.json`
- Create: `crates/evolution-contracts/schemas/evaluation-v1.json`
- Create: `crates/evolution-contracts/schemas/audit-v1.json`
- Create: `crates/evolution-contracts/tests/schema_snapshots.rs`

**Interfaces:**
- Produces: checked-in schemas used by non-Rust workers and offline bundle validation.
- Consumes: Schemars derives on all public artifacts.

- [ ] **Step 1: Write the schema drift test**

The test generates each schema to a temporary directory and byte-compares it with the checked-in pretty JSON plus one final newline.

- [ ] **Step 2: Run and confirm failure**

Run: `cargo test -p evolution-contracts schema_snapshots`

Expected: FAIL: exporter/snapshots missing.

- [ ] **Step 3: Implement the exporter**

```rust
fn write_schema<T: schemars::JsonSchema>(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let schema = schemars::schema_for!(T);
    let text = serde_json::to_string_pretty(&schema)? + "\n";
    std::fs::write(path, text)?;
    Ok(())
}
```

Accept `--out <directory>`, create the directory, and write exactly the four filenames above.

- [ ] **Step 4: Generate snapshots**

Run: `cargo run -p evolution-contracts --bin export_schemas -- --out crates/evolution-contracts/schemas`

Expected: four JSON files.

- [ ] **Step 5: Run the complete contract suite**

```bash
cargo fmt --all -- --check
cargo clippy -p evolution-contracts --all-targets -- -D warnings
cargo test -p evolution-contracts
bash tests/adr-index-test.sh
bash scripts/adr-check.sh
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/evolution-contracts
git commit -m "feat: freeze evolution artifact schemas"
```
