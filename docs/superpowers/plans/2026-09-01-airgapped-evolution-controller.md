# Air-Gapped Constitutional Evolution Controller Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement GZMO's offline `Observe → Hypothesize → Build → Evaluate → Archive → Promote/Soak/Rollback` controller so memory and signed-envelope tunables can advance autonomously while code, schema, model, runtime, evaluator, security, and authority changes stop for operator signature.

**Architecture:** `gzmo-core::evolution` consumes the shared evolution-contracts and persists the authoritative lifecycle in PostgreSQL. Deterministic observers emit bounded opportunities; a local role-qualified model may draft hard candidates but receives no tool or write authority. Hermetic builders/evaluators run candidates against cloned state and immutable suites. The controller can apply Memory/Tunable actions through narrow ports; hard candidates end at `PromotionPending` until a trusted `PromotionKernel` verifies an external grant.

**Tech Stack:** Rust/Tokio, evolution-contracts, sqlx/PostgreSQL, local ModelRuntimeSupervisor/Gateway, systemd transient sandbox units, SHA-256, Serde/Schemars, existing memory/metabolism/health gates.

**Spec:** `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`

## Prerequisites

- ADR-0011–0014 and evolution-contracts are implemented.
- The ADR-0013 PostgreSQL authority exists for living facts/evidence/outbox; do not merge a second evolution-only authority beside SQLite.
- Qualified `extract_verify` and `code_candidate` role records exist; code generation remains unavailable when the latter is missing.
- Immutable evaluator files and candidate volume are mounted separately by the appliance layer.
- This plan may be unit-developed with test doubles before those prerequisites, but no production enablement or temporary dual-authority path is allowed.

## Global Constraints

- No GitHub client, Git credential, public network URL, package download, or external model registry in `gzmo-core::evolution`.
- One controller holds the same Living owner claim; no separate overnight writer.
- Observer output is evidence, not authority. Candidate generation cannot modify observations, policy, evaluator, audit, active envelope, or last-known-good state.
- A local LLM may produce candidate artifacts only; it cannot execute tools or call `PromotionKernel`.
- Build/test commands run with network disabled, read-only source/evaluator mounts, writable candidate/output mounts, CPU/RAM/PID/time caps, and `cargo --offline --locked`.
- Hard floors fail closed on `Fail` or `Unavailable`.
- Memory changes use existing verify/promote/supersede semantics. Tunables are typed and constrained by a verified envelope.
- Hard candidates require a detached operator grant bound to candidate, evaluation, policy, target, and expiry.
- Runtime status always reports enabled authority tier, budgets, active candidate, last audit hash, and stop reason.

## File Structure

| Path | Responsibility |
|---|---|
| `gzmo-core/migrations/0014_evolution.sql` | Evolution observation/candidate/evaluation/envelope/audit tables |
| `gzmo-core/src/evolution/mod.rs` | Public controller interface only |
| `gzmo-core/src/evolution/store.rs` | PostgreSQL lifecycle transactions and audit append |
| `gzmo-core/src/evolution/observer.rs` | Structured health/metabolism/memory/resource observations |
| `gzmo-core/src/evolution/opportunity.rs` | Deduplicate/rank observations under hard floors |
| `gzmo-core/src/evolution/circuit_breaker.rs` | Stop, cooldown, budget, thermal/error protections |
| `gzmo-core/src/evolution/generator.rs` | Local-model candidate drafting without tools |
| `gzmo-core/src/evolution/sandbox.rs` | Candidate filesystem and transient-unit builder/evaluator |
| `gzmo-core/src/evolution/controller.rs` | State orchestration and authority dispatch |
| `gzmo-core/src/evolution/tunables.rs` | Typed envelope-bounded runtime overlay |
| `gzmo-core/src/evolution/promotion.rs` | Trusted verifier interface; no private-key handling |
| `gzmo-core/src/evolution/status.rs` | Unified structured status |
| `gzmo-core/src/config.rs` | `EvolutionConfig`; no mutable authority fields from unsigned TOML |
| `gzmo-cli/src/evolve_cmd.rs` | Inspect/run/export/import-grant/abort/rollback commands |
| `gzmo-cli/src/main.rs` | Thin command registration |
| `gzmo-core/tests/evolution_postgres.rs` | PostgreSQL transactional integration tests |
| `gzmo-core/tests/evolution_sandbox.rs` | Network/write/limit escape tests |

---

### Task 1: Create the Authoritative Evolution Ledger

**Files:**
- Modify: `Cargo.toml`
- Modify: `gzmo-core/Cargo.toml`
- Create: `gzmo-core/migrations/0014_evolution.sql`
- Create: `gzmo-core/src/evolution/mod.rs`
- Create: `gzmo-core/src/evolution/store.rs`
- Modify: `gzmo-core/src/lib.rs`
- Test: `gzmo-core/tests/evolution_postgres.rs`

**Interfaces:**
- Produces: `PgEvolutionStore::{record_observation,create_candidate,transition,store_evaluation,active_candidate,append_audit,verify_audit}`.
- Consumes: `sqlx::PgPool`, evolution-contracts.

- [ ] **Step 1: Write a transactional lifecycle test**

```rust
#[sqlx::test(migrations = "migrations")]
async fn transition_and_audit_commit_together(pool: PgPool) {
    let store = PgEvolutionStore::new(pool);
    let id = store.create_candidate(manifest()).await.unwrap();
    store.transition(&id, CandidateState::Prepared, reason("selected")).await.unwrap();
    assert_eq!(store.load(&id).await.unwrap().state, CandidateState::Prepared);
    assert!(store.verify_audit().await.is_ok());
    assert!(store.create_candidate(second_manifest()).await.is_err());
}
```

- [ ] **Step 2: Run to verify failure**

Run with a disposable PostgreSQL service: `cargo test -p gzmo-core --test evolution_postgres transition_and_audit`

Expected: FAIL: migration/module missing.

- [ ] **Step 3: Add PostgreSQL dependencies**

Add workspace:

```toml
sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid", "json", "migrate"] }
```

Add `sqlx` and `evolution-contracts` to `gzmo-core`.

- [ ] **Step 4: Create normalized tables**

Migration creates:

```sql
CREATE SCHEMA IF NOT EXISTS evolution;
CREATE TABLE evolution.observations (
  id uuid PRIMARY KEY,
  kind text NOT NULL,
  source_version text NOT NULL,
  payload jsonb NOT NULL,
  payload_sha256 text NOT NULL,
  observed_at timestamptz NOT NULL,
  UNIQUE(kind, payload_sha256)
);
CREATE TABLE evolution.candidates (
  id text PRIMARY KEY,
  mission_id text NOT NULL,
  authority text NOT NULL,
  state text NOT NULL,
  manifest jsonb NOT NULL,
  manifest_sha256 text NOT NULL,
  artifact_sha256 text,
  active boolean NOT NULL DEFAULT true,
  terminal_reason text,
  created_at timestamptz NOT NULL,
  updated_at timestamptz NOT NULL
);
CREATE UNIQUE INDEX evolution_one_active ON evolution.candidates(active) WHERE active;
CREATE TABLE evolution.evaluations (
  candidate_id text PRIMARY KEY REFERENCES evolution.candidates(id),
  report jsonb NOT NULL,
  report_sha256 text NOT NULL,
  completed_at timestamptz NOT NULL
);
CREATE TABLE evolution.envelopes (
  digest text PRIMARY KEY,
  envelope jsonb NOT NULL,
  verified_signer text NOT NULL,
  active boolean NOT NULL DEFAULT false,
  valid_from timestamptz NOT NULL,
  expires_at timestamptz NOT NULL
);
CREATE UNIQUE INDEX evolution_one_envelope ON evolution.envelopes(active) WHERE active;
CREATE TABLE evolution.active_tunables (
  key text PRIMARY KEY,
  value jsonb NOT NULL,
  envelope_digest text NOT NULL REFERENCES evolution.envelopes(digest),
  candidate_id text NOT NULL REFERENCES evolution.candidates(id),
  applied_at timestamptz NOT NULL
);
CREATE TABLE evolution.audit_events (
  sequence bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  event jsonb NOT NULL,
  event_hash text NOT NULL UNIQUE,
  previous_hash text NOT NULL,
  occurred_at timestamptz NOT NULL
);
```

- [ ] **Step 5: Implement transaction boundaries**

Use `SELECT ... FOR UPDATE` on candidate and active-envelope rows. Transition, evaluation insert, tunable apply, and audit append occur in one transaction. Enforce legal transition in Rust and a database check on state vocabulary.

- [ ] **Step 6: Verify canonical bytes after read**

On every load, canonicalize stored JSON and match its digest before use. Audit sequence/hash verification runs at boot and before promotion.

- [ ] **Step 7: Run store tests**

Run: `cargo test -p gzmo-core --test evolution_postgres`

Expected: PASS including race for one active candidate, transaction rollback, duplicate observation, expired envelope, and audit tamper.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock gzmo-core
git commit -m "feat: persist constitutional evolution in postgres"
```

---

### Task 2: Observe Structured Signals and Rank One Opportunity

**Files:**
- Create: `gzmo-core/src/evolution/observer.rs`
- Create: `gzmo-core/src/evolution/opportunity.rs`
- Test: `gzmo-core/src/evolution/observer.rs`
- Test: `gzmo-core/src/evolution/opportunity.rs`

**Interfaces:**
- Produces: `ObservationSnapshot`, `EvolutionOpportunity`, `OpportunitySelector::select_one`.
- Consumes: read-only ports for MemoryKernel, metabolism board, projection health, capability status, resource telemetry.

- [ ] **Step 1: Write deterministic selection tests**

```rust
#[test]
fn hard_floor_failure_creates_repair_only_and_blocks_optimization() {
    let snapshot = fixture_snapshot().with_floor_failure("projection_authority");
    let selected = select_one(&snapshot, &policy()).unwrap();
    assert_eq!(selected.kind, OpportunityKind::Repair);
    assert_eq!(selected.authority, AuthorityTier::Candidate);
}

#[test]
fn stable_snapshot_deduplicates_same_opportunity() {
    assert_eq!(fingerprint(&opportunity()), fingerprint(&opportunity()));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-core evolution::opportunity`

Expected: FAIL: observer/opportunity missing.

- [ ] **Step 3: Define observation inputs**

Record only structured, versioned data:

- last metabolism job outcomes and ages;
- keep-quality/faithfulness/felt-use results;
- PostgreSQL and projection watermarks/digests;
- failure cases and lifecycle conflicts;
- role qualification and model pin digests;
- CPU/RAM/disk/thermal/joule budgets;
- prior candidate outcomes and cooldowns.

Raw prompts, secrets, hidden reasoning, GitHub data, and mutable wiki text are not control inputs.

- [ ] **Step 4: Implement priority classes**

Order: `SafetyRepair` → `CorrectnessRepair` → `ReliabilityRepair` → `ResourceOptimization` → `QualityOptimization` → `ResearchCandidate`. A hard-floor failure suppresses optimization/research. Within a class, rank deterministic severity, affected real-use count, reproducibility, then stable fingerprint.

- [ ] **Step 5: Exclude synthetic self-reward**

Only observations originating from real sessions, signed evals, deterministic probes, or operator-imported fixtures may improve fitness. Candidate-generated traffic and memory-gym sessions carry `synthetic=true` and cannot satisfy progress floors.

- [ ] **Step 6: Persist snapshot before selection**

Store one immutable snapshot digest and bind every resulting manifest to it. Rerunning the same snapshot returns the same opportunity or deduplicates it.

- [ ] **Step 7: Run observer tests**

Run: `cargo test -p gzmo-core evolution::observer evolution::opportunity`

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add gzmo-core/src/evolution
git commit -m "feat: derive one evolution opportunity from trusted signals"
```

---

### Task 3: Enforce Resource and Authority Circuit Breakers

**Files:**
- Create: `gzmo-core/src/evolution/circuit_breaker.rs`
- Modify: `gzmo-core/src/config.rs`
- Modify: `gzmo.toml.example`
- Test: `gzmo-core/src/evolution/circuit_breaker.rs`

**Interfaces:**
- Produces: `CircuitBreaker::preflight`, `CircuitLease::checkpoint`, `StopReason`.
- Consumes: signed active envelope, profile capability manifest, resource telemetry, candidate history.

- [ ] **Step 1: Write stop-order tests**

```rust
#[test]
fn candidate_work_stops_before_living_recall() {
    let mut lease = lease_with_limits();
    assert_eq!(lease.checkpoint(sample().thermal_over_limit()), Err(StopReason::Thermal));
    assert_eq!(lease.action(), StopAction::KillCandidatePreserveRecall);
}

#[test]
fn stop_file_and_error_streak_block_new_candidate() {
    assert_eq!(preflight(context().stop_file(true)), Err(StopReason::OperatorStop));
    assert_eq!(preflight(context().consecutive_failures(3)), Err(StopReason::FailureStreak));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-core evolution::circuit_breaker`

Expected: FAIL: circuit breaker missing.

- [ ] **Step 3: Add configuration that cannot grant authority**

```rust
pub struct EvolutionConfig {
    pub enabled: bool,
    pub observe_cron_hour: u32,
    pub observe_cron_minute: u32,
    pub candidate_cooldown_hours: u32,
    pub stop_file: PathBuf,
    pub candidate_root: PathBuf,
    pub evaluator_root: PathBuf,
}
```

TOML may disable, schedule, and locate files. Budgets, protected paths, tunable ranges, and authority come only from the verified envelope/database, never unsigned TOML.

- [ ] **Step 4: Implement preflight order**

Check: feature enabled → stop file absent → one owner → audit valid → active envelope valid → no nonterminal candidate → cooldown → required roles qualified → state snapshot capacity → disk reserve → thermal/power headroom → budget available.

- [ ] **Step 5: Implement runtime checkpoints**

At least every tool/build phase and every 5 seconds: wall, process tree, RSS, output bytes, token usage, energy if meter required, disk, temperature, operator stop, owner lease, envelope expiry. On failure kill candidate process tree and append audit before any new work.

- [ ] **Step 6: Run circuit-breaker tests**

Run: `cargo test -p gzmo-core evolution::circuit_breaker`

Expected: PASS including unavailable required meter and envelope expiry mid-run.

- [ ] **Step 7: Commit**

```bash
git add gzmo-core/src/evolution gzmo-core/src/config.rs gzmo.toml.example
git commit -m "feat: bound autonomous evolution resources and authority"
```

---

### Task 4: Apply Autonomous Memory and Envelope-Bounded Tunables

**Files:**
- Create: `gzmo-core/src/evolution/tunables.rs`
- Modify: `gzmo-core/src/evolution/controller.rs`
- Modify: `gzmo-core/src/context.rs`
- Modify selected existing modules only when a real tunable is connected; initial keys below
- Test: `gzmo-core/tests/evolution_postgres.rs`

**Interfaces:**
- Produces: `TunableSnapshot`, `TunableProvider`, `MemoryEvolutionPort`.
- Consumes: verified envelope, evaluation report, current last-known-good tunables.

- [ ] **Step 1: Write authority tests**

```rust
#[tokio::test]
async fn bounded_tunable_can_apply_but_code_cannot() {
    let ctl = harness_with_envelope("context.archive_threshold", 0.75..=0.95).await;
    assert!(ctl.apply_tunable(candidate(0.85), passing_report()).await.is_ok());
    assert!(ctl.apply_tunable(candidate(0.50), passing_report()).await.is_err());
    assert!(ctl.promote_hard(code_candidate(), passing_report(), None).await.is_err());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-core --test evolution_postgres bounded_tunable`

Expected: FAIL: tunable/controller missing.

- [ ] **Step 3: Start with three typed tunables**

- `context.archive_threshold`: float 0.75–0.95;
- `metabolism.max_batch_items`: integer 1–100;
- `recall.utility_lambda`: float 0.0–0.20.

The envelope may narrow but never widen compiled absolute limits. Unknown keys fail. Values are stored in PostgreSQL with candidate/envelope/evaluation digests.

- [ ] **Step 4: Inject immutable runtime snapshots**

Load verified tunables at owner startup and after an audited apply. Modules receive `Arc<TunableSnapshot>`; no module queries PostgreSQL per hot-path operation. Existing defaults remain when no signed override exists.

- [ ] **Step 5: Apply memory actions through a narrow port**

```rust
#[async_trait]
pub trait MemoryEvolutionPort: Send + Sync {
    async fn promote_verified(&self, receipt: VerifiedMemoryReceipt) -> Result<MemoryMutationReceipt>;
    async fn supersede_verified(&self, receipt: VerifiedSupersessionReceipt) -> Result<MemoryMutationReceipt>;
    async fn record_outcome(&self, receipt: OutcomeReceipt) -> Result<MemoryMutationReceipt>;
}
```

The PostgreSQL MemoryKernel adapter enforces evidence/lifecycle in the same authority transaction. The controller cannot issue raw SQL or direct projection writes.

- [ ] **Step 6: Implement automatic rollback**

During Tunable soak, any hard-floor failure atomically restores the prior tunable snapshot, marks candidate RolledBack, and appends audit. Memory facts use supersession/valid-time correction, not destructive rollback.

- [ ] **Step 7: Run authority tests**

Run: `cargo test -p gzmo-core evolution::tunables --test evolution_postgres`

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add gzmo-core
git commit -m "feat: evolve memory and signed-envelope tunables"
```

---

### Task 5: Generate Hard Candidates Without Tool Authority

**Files:**
- Create: `gzmo-core/src/evolution/generator.rs`
- Create: `gzmo-core/src/evolution/sandbox.rs`
- Create: `gzmo-core/tests/evolution_sandbox.rs`

**Interfaces:**
- Produces: content-addressed `CandidateBundle` and local `EvaluationReport`.
- Consumes: immutable observation snapshot, manifest, qualified `code_candidate` role, source snapshot, evaluator suites.

- [ ] **Step 1: Write candidate escape tests**

Test generated artifacts attempting network, `/dev` access, production database/socket access, evaluator modification, root/key reads, writes outside candidate volume, fork bomb, disk flood, and package download. Each must fail structurally.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-core --test evolution_sandbox`

Expected: FAIL: sandbox missing.

- [ ] **Step 3: Generate a structured proposal only**

Call the local `code_candidate` model with no tool registry. Require strict JSON containing problem evidence digest, intended files, patch in unified-diff form, expected observable behavior, test additions, risks, and rollback class. Reject paths/payload outside manifest policy before writing.

- [ ] **Step 4: Materialize a candidate source snapshot**

Copy/export the signed source commit to the candidate volume, verify digest, apply patch there, and record resulting tree digest. Production source/system slots remain read-only.

- [ ] **Step 5: Build in a transient sandbox**

Use systemd transient units or equivalent with `DynamicUser=yes`, `PrivateNetwork=yes`, `ProtectSystem=strict`, `ProtectHome=yes`, `PrivateDevices=yes`, `NoNewPrivileges=yes`, dropped capabilities, read-only source/evaluator/toolchain/dependency cache, writable candidate/target/temp only, and CPU/memory/PID/runtime quotas. Run `cargo build --offline --locked` and trusted evaluator commands.

- [ ] **Step 6: Archive, never promote directly**

Store bundle/report by digest, transition to Rejected or ReviewReady then PromotionPending. Generator/builder/evaluator has no `PromotionKernel` handle.

- [ ] **Step 7: Run sandbox tests**

Run: `cargo test -p gzmo-core --test evolution_sandbox -- --nocapture`

Expected: all escape attempts fail; passing fixture creates a digested bundle/report and stops at PromotionPending.

- [ ] **Step 8: Commit**

```bash
git add gzmo-core/src/evolution gzmo-core/tests/evolution_sandbox.rs
git commit -m "feat: build hard evolution candidates in offline sandboxes"
```

---

### Task 6: Verify Detached Grants and Control Promotion State

**Files:**
- Create: `gzmo-core/src/evolution/promotion.rs`
- Modify: `gzmo-core/src/evolution/controller.rs`
- Test: `gzmo-core/tests/evolution_postgres.rs`

**Interfaces:**
- Produces: `PromotionVerifier::verify(UnverifiedAuthorityGrant) -> VerifiedAuthorityGrant`, `PromotionRequest`.
- Consumes: operator public keys from immutable trust store; no signing private keys.

- [ ] **Step 1: Write binding/replay tests**

Cover wrong candidate/report/policy/target, expired grant, unknown signer, invalid signature, reused nonce, downgraded minimum version, and candidate trying to sign itself.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-core evolution::promotion`

Expected: FAIL: verifier missing.

- [ ] **Step 3: Implement verification-only key handling**

Load operator public keys and minimum policy epoch from immutable BootTrust mount. Verify detached Ed25519 signature over canonical promotion request. Store nonce/digest before returning verified grant to prevent replay.

- [ ] **Step 4: Separate targets**

- Tunable target: already handled by envelope path; hard grant not required.
- Code/runtime target: inactive system/application slot via external BootTrust adapter.
- Schema target: isolated DB clone + backup/compatibility receipt; external data-plane promotion adapter.
- Model target: content-addressed model slot + qualification record.
- Evaluator/security/authority target: never auto-run; requires dedicated Authority grant class and separate review artifact.

- [ ] **Step 5: Keep controller powerless without adapter**

If the appliance layer has no concrete promotion adapter for target class, remain PromotionPending and report `adapter_unavailable`; never shell out to a guessed installer.

- [ ] **Step 6: Run promotion tests**

Run: `cargo test -p gzmo-core evolution::promotion --test evolution_postgres`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add gzmo-core/src/evolution
git commit -m "feat: require detached authority grants for hard promotion"
```

---

### Task 7: Add CLI and Read-Only MCP Status

**Files:**
- Create: `gzmo-core/src/evolution/status.rs`
- Create: `gzmo-cli/src/evolve_cmd.rs`
- Modify: `gzmo-cli/src/main.rs`
- Modify: `gzmo-core/src/mcp/serve.rs`
- Test: `gzmo-cli/src/evolve_cmd.rs`
- Test: `gzmo-core/src/mcp/serve.rs`

**Interfaces:**
- Produces: `gzmo evolve observe|run-once|status|inspect|export|import-grant|abort|rollback`; MCP `gzmo_evolution_status` only.
- Consumes: controller/store/status and promotion verifier.

- [ ] **Step 1: Write CLI safety tests**

Assert status/inspect/export are read-only; `run-once` requires config enabled and owner; `import-grant` validates before storing; abort cannot erase artifacts/audit; rollback targets only active Tunable soak or calls a trusted adapter.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-cli evolve_cmd`

Expected: FAIL: command missing.

- [ ] **Step 3: Add thin command parsing/dispatch**

Follow existing `workflow_skill_cmd.rs` pattern: exact usage, argument-vector parsing, errors via `anyhow`, all logic in core. Add `Command::Evolve(Vec<String>)` to log filter, identity exclusion, and dispatch.

- [ ] **Step 4: Define status JSON**

Include schema, enabled/profile, authority tiers, active envelope digest/expiry, observation/candidate IDs, state, resource budget used/remaining, hard floors, evaluation digest, audit head, required operator action, stop reason, last-known-good, and next eligible run.

- [ ] **Step 5: Add read-only MCP tool**

`gzmo_evolution_status` exposes the same status. It must not accept a candidate, grant, apply, abort, or rollback. Mutation stays operator CLI/local physical workflow.

- [ ] **Step 6: Run CLI/MCP tests**

Run: `cargo test -p gzmo-cli evolve_cmd && cargo test -p gzmo-core evolution_status`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add gzmo-cli gzmo-core/src/mcp/serve.rs gzmo-core/src/evolution/status.rs
git commit -m "feat: expose constitutional evolution status and controls"
```

---

### Task 8: Prove the Offline Authority Matrix

**Files:**
- Modify: `gzmo-core/tests/evolution_postgres.rs`
- Modify: `gzmo-core/tests/evolution_sandbox.rs`
- Create: `tests/evolution-offline-smoke.sh`

**Interfaces:**
- Produces: execution gate for the Stage 2 controller.
- Consumes: all internal controller paths.

- [ ] **Step 1: Add an offline integration fixture**

Fixture starts local PostgreSQL and mock local model endpoints, disconnects network namespace, loads signed test envelope/public key, runs one memory, one tunable, and one code candidate.

- [ ] **Step 2: Assert authority outcomes**

Expected:

- Memory candidate reaches Accepted with evidence/audit.
- In-envelope Tunable reaches Soaking then Accepted; forced floor failure rolls back.
- Code candidate reaches PromotionPending and cannot proceed without a valid external grant.
- Authority/evaluator candidate never self-applies.
- No outbound socket, GitHub string, or cloud endpoint occurs.

- [ ] **Step 3: Run the full offline gate**

```bash
cargo fmt --all -- --check
cargo clippy -p gzmo-core -p gzmo-cli --all-targets -- -D warnings
cargo test -p gzmo-core evolution -- --nocapture
cargo test -p gzmo-cli evolve_cmd
bash tests/evolution-offline-smoke.sh
```

Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add gzmo-core gzmo-cli tests/evolution-offline-smoke.sh
git commit -m "test: prove offline constitutional evolution authority"
```
