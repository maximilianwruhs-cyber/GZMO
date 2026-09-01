# Hybrid Continuous Self-Development Program Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a two-stage development system in which a connected repository runner continuously prepares evidence-gated pull requests now, then the air-gapped GZMO appliance later reuses the same candidate contracts without gaining authority over its own safeguards.

**Architecture:** Stage 1 is a trusted, connected `gzmo-evolver` process outside the Living writer. It consumes the existing Opportunity Discovery mission, gives an uncredentialed sandbox worker one isolated worktree, evaluates the result, and opens a PR; it never merges. Stage 2 embeds the same contracts behind GZMO's Constitutional Spine: autonomous memory, envelope-bounded tunables, isolated hard candidates, operator-signed promotion, soak, and rollback.

**Tech Stack:** Rust 2021, Tokio, Serde/Schemars, rusqlite for connected-runner state, Git CLI, GitHub App REST API, hardened system services with separate coordinator/worker identities, PostgreSQL/sqlx for the later appliance evolution ledger, existing GZMO opportunity/quality gates.

**Spec:** `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`

## Global Constraints

- One physical node and one authoritative writer per Living installation; Stage 1's connected development host is not the Living product.
- Runtime airgap: Stage 2 requires no GitHub, public model registry, remote database, or second inference host.
- Candidate workers receive no GitHub credential, operator signing key, production secret, raw block device, or writable evaluator/policy path.
- Exactly one active opportunity and at most one active candidate per repository.
- Stage 1 may push only `evolve/<candidate-id>` branches and open/update its own PR; it never pushes `main`, merges, changes visibility, changes branch rules, or publishes releases.
- Memory may evolve autonomously. Tunables self-promote only inside an operator-signed envelope. Code, schema, model, runtime, evaluator, security, and capability changes require an operator signature.
- Hard floors are conjunctive: faithfulness, one-writer integrity, airgap/provenance, no authority expansion, storage integrity, bounded resources, audit continuity, and exercised rollback.
- Keep the approved North Star baseline and tag immutable. Follow-on design changes use new dated documents.
- Preserve existing `opportunity-sense.sh`, `opportunity-rank.sh`, `opportunity-next-mission.sh`, and the one-active-bet rule until the Rust replacement proves artifact parity.
- No permanent compatibility aliases, dual-write authority, second product, or silent degraded capability.

## Scope Boundary and Prerequisites

This stack implements continuous **development control**, not every North Star subsystem. It does not build the immutable boot appliance, Hardware Inventory/Capability Compiler, role-qualified model runtime, or migrate living memory to PostgreSQL/Qdrant/Neo4j/Redis. Stage 1 can ship independently. Stage 2 remains disabled until those separately planned prerequisites expose PostgreSQL authority, `BootTrust`, `PromotionKernel`, candidate storage, qualified local model roles, and rollback targets named by the approved spec. Executors must block Stage 2 rather than invent temporary SQLite authority, fake promotion adapters, or weaker model/trust paths.

## Plan Stack and Ownership

| Order | Plan | Owns | Independently testable result |
|---|---|---|---|
| 1 | `2026-09-01-evolution-contracts-and-governance.md` | ADR index/0011–0014 plan, `crates/evolution-contracts`, canonical schemas | Candidate/policy/evaluation/audit artifacts round-trip and reject invalid transitions |
| 2 | `2026-09-01-connected-repo-evolver.md` | `gzmo-evolver` mission/state/worktree/worker loop | One fixture mission produces one isolated candidate commit without credentials or main mutation |
| 3 | `2026-09-01-evaluation-and-pr-shepherd.md` | deterministic evaluator, GitHub App adapter, PR/CI shepherd | Passing candidate opens a PR; failing/protected candidate cannot push or open one |
| 4 | `2026-09-01-airgapped-evolution-controller.md` | GZMO internal observer/controller/PostgreSQL evolution ledger | Offline controller autonomously advances Memory/Tunable candidates and blocks hard promotion |
| 5 | `2026-09-01-continuous-evolution-operations.md` | cadence, resource circuit breakers, status, migration, legacy retirement | Daily/weekly loops run unattended, one candidate at a time, with drills for stop and rollback |

Files owned by a later plan may extend but not rename interfaces produced by an earlier plan. If an interface must change, update the producing plan and all consumers before implementation begins.

## Shared Artifact Contract

All stages exchange canonical JSON generated from `evolution-contracts`; free-form Markdown is display-only.

```rust
pub struct CandidateId(pub String);

pub enum AuthorityTier {
    Memory,
    Tunable,
    Candidate,
    Promotion,
    Authority,
}

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

pub struct ResourceBudget {
    pub wall_seconds: u64,
    pub max_attempts: u8,
    pub max_changed_files: u32,
    pub max_added_lines: u32,
    pub max_tool_calls: u32,
    pub max_input_tokens: u64,
    pub max_output_tokens: u64,
    pub max_energy_joules: Option<u64>,
}

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

pub struct CandidateManifest {
    pub schema: String,
    pub id: CandidateId,
    pub mission_id: String,
    pub kind: CandidateKind,
    pub authority: AuthorityTier,
    pub target: CandidateTarget,
    pub baseline_digest: String,
    pub required_gates: Vec<String>,
    pub protected_paths: Vec<String>,
    pub budget: ResourceBudget,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct EvaluationReport {
    pub schema: String,
    pub candidate_id: CandidateId,
    pub baseline_digest: String,
    pub candidate_digest: String,
    pub gates: Vec<GateResult>,
    pub hard_floors_passed: bool,
    pub metrics: std::collections::BTreeMap<String, f64>,
    pub artifact_digests: std::collections::BTreeMap<String, String>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

pub struct AuditEvent {
    pub schema: String,
    pub sequence: u64,
    pub previous_hash: String,
    pub event_type: String,
    pub candidate_id: Option<CandidateId>,
    pub payload_digest: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}
```

Schema identifiers are fixed at `gzmo.evolution.candidate/v1`, `gzmo.evolution.envelope/v1`, `gzmo.evolution.evaluation/v1`, `gzmo.evolution.promotion/v1`, and `gzmo.evolution.audit/v1` until a signed schema migration introduces a new version.

## End-to-End State Flow

```text
Existing opportunity Sense/Rank
  → exactly one active mission
  → CandidateManifest + baseline digest
  → isolated worktree + uncredentialed worker
  → candidate commit
  → deterministic gates + comparative fitness
  → rejected OR ReviewReady
  → trusted GitHub App pushes evolve/* and opens PR
  → CI/review shepherd may repair twice
  → human merge

Later appliance:
Observation snapshot
  → same CandidateManifest
  → isolated builder/evaluator
  → Memory/Tunable bounded path OR PromotionPending
  → operator-signed grant
  → inactive slot / cloned schema
  → soak
  → Accepted or RolledBack
```

## Milestone Gates

### Task 1: Land Contracts Before Automation

**Files:**
- Implement from: `docs/superpowers/plans/2026-09-01-evolution-contracts-and-governance.md`

**Interfaces:**
- Produces: the exact artifact types and schema identifiers above.
- Consumes: approved North Star spec only.

- [ ] **Step 1: Execute the contracts/governance plan through its full test gate**

Run the plan task-by-task. Do not start the runner while schemas are still changing.

- [ ] **Step 2: Verify the contract gate**

Run: `cargo test -p evolution-contracts`

Expected: state-transition, canonical-digest, schema fixture, and policy tests all pass.

- [ ] **Step 3: Verify ADR lineage**

Run: `bash tests/adr-index-test.sh`

Expected: PASS; no active document treats inaccessible external ADR-0001/0002 as authority.

- [ ] **Step 4: Commit the milestone**

```bash
git add Cargo.toml Cargo.lock crates/evolution-contracts docs/ADR-INDEX.md docs/ADR-001{1,2,3,4}-*.md scripts/adr-check.sh tests/adr-index-test.sh
git commit -m "feat: define constitutional evolution contracts"
```

### Task 2: Deliver the Connected Candidate Loop

**Files:**
- Implement from: `docs/superpowers/plans/2026-09-01-connected-repo-evolver.md`

**Interfaces:**
- Consumes: `CandidateManifest`, `CandidateState`, `ResourceBudget`, `AuditEvent`.
- Produces: candidate commit plus local state/audit records; no remote writes.

- [ ] **Step 1: Execute the connected-runner plan**

Complete every task through the fixture candidate commit.

- [ ] **Step 2: Run the hermetic vertical slice**

Run: `cargo test -p gzmo-evolver --test repo_loop -- --nocapture`

Expected: one candidate reaches `Evaluating`; `main` is unchanged; candidate environment contains no GitHub credential.

- [ ] **Step 3: Commit the milestone**

```bash
git add Cargo.toml Cargo.lock gzmo-evolver config/repo-evolver.toml.example
git commit -m "feat: create isolated repository candidate loop"
```

### Task 3: Deliver Evaluation and PR Preparation

**Files:**
- Implement from: `docs/superpowers/plans/2026-09-01-evaluation-and-pr-shepherd.md`

**Interfaces:**
- Consumes: candidate commit and canonical policy.
- Produces: signed/digested `EvaluationReport`, `ReviewReady` state, and a PR owned by the trusted runner.

- [ ] **Step 1: Execute the evaluation/PR plan**

Use a local fake GitHub server until the final opt-in live smoke.

- [ ] **Step 2: Run safety and PR integration tests**

Run: `cargo test -p gzmo-evolver evaluator github shepherd -- --nocapture`

Expected: hard-floor failure blocks push; protected-path mutation blocks PR; passing fixture opens exactly one mock PR; CI repair caps at two attempts.

- [ ] **Step 3: Run the opt-in live dry run**

Run: `GZMO_EVOLVER_LIVE=1 gzmo-evolver run --mission fixture-safe-doc --stop-before-push`

Expected: report reaches `ReviewReady`; no remote branch exists.

- [ ] **Step 4: Commit the milestone**

```bash
git add gzmo-evolver .github/ISSUE_TEMPLATE .github/pull_request_template.md config/repo-evolver.policy.toml.example
git commit -m "feat: gate autonomous candidates before pull requests"
```

### Task 4: Soak Stage 1 Before Any Internal Evolution

**Files:**
- Runtime state only: `/var/lib/gzmo-evolver/coordinator/` and per-candidate worker paths under `/var/lib/gzmo-evolver/worktrees/`.

**Interfaces:**
- Consumes: merged Stage 1 runner.
- Produces: four weekly audit checkpoints and reviewed PR outcomes.

- [ ] **Step 1: Install the Stage 1 timer in disabled mode**

Run: `sudo bash scripts/install-repo-evolver.sh --install-only`

Expected: service/timer installed; timer disabled.

- [ ] **Step 2: Run one manual fixture mission**

Run: `sudo systemctl start gzmo-repo-evolver.service`

Expected: one candidate only; no main push; status names the exact stop reason or PR.

- [ ] **Step 3: Enable the weekly timer after reviewing the manual audit**

Run: `sudo systemctl enable --now gzmo-repo-evolver.timer`

Expected: next trigger is visible; service credentials readable only by the trusted runner account.

- [ ] **Step 4: Collect four consecutive weekly checkpoints**

Run weekly: `gzmo-evolver status --json`

Expected: no overlapping candidate, no protected-path bypass, no automatic merge, and every candidate has a terminal state.

- [ ] **Step 5: Record Stage 1 acceptance**

Create a dated acceptance record under `research/evolution-soak/` containing candidate IDs, PR URLs or rejection reasons, gate results, repair attempts, and operator decisions.

- [ ] **Step 6: Commit the Stage 1 soak evidence**

```bash
git add research/evolution-soak/
git commit -m "docs: record repository evolution soak"
```

---

### Task 5: Implement the Air-Gapped Controller

**Files:**
- Implement from: `docs/superpowers/plans/2026-09-01-airgapped-evolution-controller.md`

**Interfaces:**
- Consumes: shared contracts, PostgreSQL authority, BootTrust/PromotionKernel adapters.
- Produces: offline candidate lifecycle, envelope-bounded tunable apply, and `PromotionPending` hard candidates.

- [ ] **Step 1: Confirm prerequisites**

Run: `gzmo status --json`

Expected: authoritative PostgreSQL, qualified model roles, one owner, audit root, candidate storage, and rollback target all PASS. Otherwise stop.

- [ ] **Step 2: Execute the air-gapped controller plan**

No GitHub credential or network endpoint may be introduced into the appliance path.

- [ ] **Step 3: Run offline controller tests**

Run: `cargo test -p gzmo-core evolution -- --nocapture`

Expected: Memory/Tunable paths progress as permitted; Candidate/Promotion/Authority paths stop without a valid operator grant.

- [ ] **Step 4: Commit the milestone**

```bash
git add Cargo.toml Cargo.lock gzmo-core/src/evolution gzmo-cli/src/evolve_cmd.rs gzmo-cli/src/main.rs gzmo-core/src/config.rs gzmo.toml.example
git commit -m "feat: add constitutional offline evolution controller"
```

### Task 6: Cut Over to One Continuous Operating Model

**Files:**
- Implement from: `docs/superpowers/plans/2026-09-01-continuous-evolution-operations.md`

**Interfaces:**
- Consumes: both stage implementations and shared audit schemas.
- Produces: explicit connected/offline roles, bounded cadence, unified status, stop/rollback drills, and removal of duplicate legacy evolve paths.

- [ ] **Step 1: Execute the operations plan through shadow parity**

Legacy scripts remain authoritative until parity receipts compare equal.

- [ ] **Step 2: Run the complete verification matrix**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
bash scripts/opportunity-discovery-check.sh
bash scripts/evolution-contract-check.sh
bash scripts/evolution-chaos-drill.sh --fixture
```

Expected: all commands exit 0; chaos drill proves candidate stop, credential isolation, projection fallback, and rollback.

- [ ] **Step 3: Remove superseded paths in the same cutover**

Delete only after parity: `gzmo-cli/src/idle_evolve.rs`, its daemon invocation, and redundant daily/weekly mission orchestration. Preserve Opportunity Discovery inputs and historical research.

- [ ] **Step 4: Commit the clean cutover**

```bash
git add -A
git commit -m "feat: operate continuous development through constitutional evolution"
```

## Program Completion Criteria

The program is complete only when:

1. A connected host has produced four consecutive, non-overlapping, evidence-gated candidate outcomes without direct main mutation or automatic merge.
2. The air-gapped appliance produces equivalent candidate/evaluation/audit artifacts without GitHub or network access.
3. Memory and envelope-bounded tunables can advance automatically; every higher tier stops at `PromotionPending` until an operator signature is verified.
4. Candidate, evaluator, promotion, audit, and production identities have mechanically distinct write permissions.
5. Stop-evolve, last-known-good rollback, and projection rebuild drills pass.
6. Status explains the active mission, candidate, budgets, floors, authority needed, and recovery state through both CLI and MCP.
7. Legacy parallel evolve scripts are removed after artifact parity, leaving one implementation of each rule.
