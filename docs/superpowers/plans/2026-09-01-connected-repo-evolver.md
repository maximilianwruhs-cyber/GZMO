# Connected Repository Evolver Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing Opportunity Discovery output into one isolated, budgeted repository candidate at a time, without allowing the worker to push, merge, modify `main`, or access GitHub credentials.

**Architecture:** A new `gzmo-evolver` binary is the trusted connected-host coordinator. It reads the existing `next-mission.json/md`, persists candidate state in a local SQLite database outside the repository, creates an independent Git clone from a coordinator-owned mirror, invokes OMP as an uncredentialed worker, records the normalized candidate commit and audit chain, and stops at `Evaluating`. Remote push/PR behavior belongs to the next plan.

**Tech Stack:** Rust 2021, Tokio, Clap, rusqlite, fs2, Serde/TOML, evolution-contracts, Git CLI, OMP v18+ non-interactive CLI, systemd.

**Spec:** `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`

## Global Constraints

- Stage 1 runs on a connected development host and is never the Living writer.
- Reuse `scripts/opportunity-next-mission.sh`; do not create a competing ranker.
- Exactly one active opportunity and one nonterminal candidate per repository.
- Worker target is `CandidateTarget::Repository`; its branch is `evolve/<candidate-id>` and baseline is `git-sha1:<40 hex>` captured before work begins.
- Trusted coordinator and OMP worker run as different OS identities. The worker cannot read the coordinator state directory, GitHub App key, or trusted raw logs.
- Worker receives no `GH_TOKEN`, `GITHUB_TOKEN`, SSH key directory, Git credential helper, cloud provider environment key, operator signing key, or production config.
- The OMP worker profile points only to a qualified local `code_candidate` model served inside the worker's private network namespace. No provider token or cloud auth broker is present; if the local role is unavailable, stop.
- Worker egress is structurally absent. Its private namespace contains loopback plus the local model service only; candidate shell tools cannot reach the host network, GitHub, or the public internet.
- Independent candidate workspace fetch/push URLs are `no-fetch://candidate-worker` and `no-push://candidate-worker`; only the later trusted GitHub adapter may push the evaluated commit.
- Worker may edit only its independent candidate workspace. Protected paths and resource budgets come from the baseline-owned trusted policy artifact.
- No direct main update, PR creation, merge, release, repository setting, or visibility mutation in this plan.
- A crash is resumable from state; it must not create a second candidate.

## File Structure

| Path | Responsibility |
|---|---|
| `gzmo-evolver/Cargo.toml` | Connected runner crate |
| `gzmo-evolver/src/lib.rs` | Public crate surface used by integration tests and thin binary |
| `gzmo-evolver/src/main.rs` | CLI only |
| `gzmo-evolver/src/config.rs` | Typed config and safe defaults |
| `gzmo-evolver/src/policy.rs` | Baseline-owned trusted Stage-1 policy and canonical digest |
| `gzmo-evolver/src/state.rs` | Local coordinator SQLite and audit persistence |
| `gzmo-evolver/src/mission.rs` | Existing Opportunity artifact adapter |
| `gzmo-evolver/src/git.rs` | Trusted mirror and independent candidate clone operations |
| `gzmo-evolver/src/worker.rs` | OMP process invocation and environment isolation |
| `gzmo-evolver/src/runner.rs` | Candidate state orchestration |
| `gzmo-evolver/src/process.rs` | Internal `ProcessRunner` seam (real + test fake) |
| `gzmo-evolver/tests/repo_loop.rs` | Hermetic bare-remote/independent-clone vertical slice |
| `gzmo-evolver/tests/fixtures/fake-worker.sh` | Deterministic candidate edit/commit |
| `config/repo-evolver.toml.example` | Connected-host configuration |
| `config/repo-evolver.policy.toml` | Tracked baseline-owned candidate policy |

---

### Task 1: Bootstrap `gzmo-evolver` with Fail-Closed Configuration

**Files:**
- Modify: `Cargo.toml`
- Create: `gzmo-evolver/Cargo.toml`
- Create: `gzmo-evolver/src/lib.rs`
- Create: `gzmo-evolver/src/main.rs`
- Create: `gzmo-evolver/src/config.rs`
- Create: `gzmo-evolver/src/policy.rs`
- Create: `config/repo-evolver.toml.example`
- Create: `config/repo-evolver.policy.toml`
- Test: `gzmo-evolver/src/config.rs`
- Test: `gzmo-evolver/src/policy.rs`

**Interfaces:**
- Consumes: `evolution_contracts::{CandidateKind, GateClass, PathPolicy, ResourceBudget}`.
- Produces: `RepoEvolverConfig::load(path)`, `TrustedPolicy::parse_toml(bytes)`, `TrustedPolicy::digest()`, normalized repository/state/mission paths, and fixed worker executable/profile configuration.

- [ ] **Step 1: Write failing configuration and policy tests**

```rust
#[test]
fn rejects_state_inside_repo_and_arbitrary_worker_argv() {
    let fixture = ConfigFixture::new();
    assert!(RepoEvolverConfig::load(&fixture.valid_config).is_ok());

    fixture.write_config_with_state(fixture.repo.join("data-next/evolver"));
    assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());

    fixture.write_config_with_unknown_worker_field("argv", "bash -c echo bad");
    assert!(RepoEvolverConfig::load(&fixture.valid_config).is_err());
}

#[test]
fn policy_digest_is_canonical_and_policy_is_bounded() {
    let first = TrustedPolicy::parse_toml(VALID_POLICY_A.as_bytes()).unwrap();
    let second = TrustedPolicy::parse_toml(VALID_POLICY_REORDERED.as_bytes()).unwrap();
    assert_eq!(first.digest().unwrap(), second.digest().unwrap());
    assert!(first.budget().validate().is_ok());
    assert!(first.protected_paths().check("Cargo.toml").is_err());
}
```

- [ ] **Step 2: Run tests to verify the package is absent**

Run: `cargo test -p gzmo-evolver config && cargo test -p gzmo-evolver policy`

Expected: FAIL because package `gzmo-evolver` does not exist.

- [ ] **Step 3: Add the workspace member and dependencies**

Add `gzmo-evolver` to `workspace.members`. Add:

```toml
clap = { version = "4", features = ["derive"] }
```

to workspace dependencies. The crate depends on `evolution-contracts`, `tokio`, `serde`, `serde_json`, `toml`, `chrono`, `uuid`, `rusqlite`, `fs2`, `sha2`, `anyhow`, `thiserror`, `tracing`, `tracing-subscriber`, and `clap`; add `tempfile = "3"` as a dev-dependency and inherit workspace lints.

`src/lib.rs` exports only `config` and `policy` in this task. `src/main.rs` is a thin binary over the library.

- [ ] **Step 4: Define configuration without unsigned authority knobs**

```rust
#[derive(Debug, Clone)]
pub struct RepoEvolverConfig {
    repo: RepoConfig,
    state_dir: PathBuf,
    mission: MissionConfig,
    worker: WorkerConfig,
    policy: PolicyConfig,
    working_policy: TrustedPolicy,
    working_policy_digest: String,
}

#[derive(Debug, Clone)]
pub struct RepoConfig {
    path: PathBuf,
    remote: String,
    base_branch: String,
    owner: String,
    repository: String,
}

#[derive(Debug, Clone)]
pub struct MissionConfig {
    json_rel: PathBuf,
    markdown_rel: PathBuf,
    refresh_argv: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    executable: PathBuf,
    profile: String,
}

#[derive(Debug, Clone)]
pub struct PolicyConfig {
    repo_path: PathBuf,
}
```

Unsigned configuration cannot set wall time, tool/token/energy budgets, protected paths, required gates, OMP tool list, arbitrary worker argv, candidate authority, or branch prefix. Those values live in `TrustedPolicy` or fixed code.

Validation rules:

- repository path and worker executable exist and canonicalize to absolute paths;
- `state_dir` is absolute, lexically normalized without `..`, and outside the repository; it need not exist during read-only config validation;
- mission JSON/Markdown are normalized relative paths beneath an injected `GZMO_DATA_NEXT` root, with no absolute/parent/symlink syntax; the policy repo path is a normalized relative path under the repository and its tracked working-tree file must exist;
- remote/base/owner/repository/profile are safe nonempty identifiers; configured owner/repository must match the policy target;
- refresh argv is an argument vector beginning `bash` plus exactly the tracked `scripts/opportunity-next-mission.sh`, with no `-c` or extra shell code;
- worker config cannot carry additional fields such as `argv`, `args`, `max_time`, extensions, tools, environment, or additional directories.

- [ ] **Step 5: Define the one Stage-1 policy wire type**

`gzmo-evolver/src/policy.rs` owns the same `TrustedPolicy` that the evaluation/PR plan later extends with diff execution. Do not create a second policy contract later.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct TrustedPolicy {
    schema: String,
    owner: String,
    repository: String,
    candidate_kind: CandidateKind,
    max_active_candidates: u8,
    max_repair_attempts: u8,
    allowed_branch_prefix: String,
    budget: ResourceBudget,
    protected_paths: PathPolicy,
    gates: Vec<GateCommand>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateCommand {
    name: String,
    class: GateClass,
    argv: Vec<String>,
    timeout_seconds: u64,
}
```

`RepoEvolverConfig::load` and `TrustedPolicy::parse_toml` deserialize private `#[serde(deny_unknown_fields)]` Raw types, validate once, and construct these private-field values. Expose read-only getters plus `required_hard_floor_names()`; no public mutable fields or unchecked constructors.

Require schema `gzmo.repo_evolver.policy/v1`, exact owner/repository, `candidate_kind` in `Code|ProceduralSkill`, one active candidate, repair attempts `<= 2`, branch prefix exactly `evolve/`, valid budget/path policy, at least one uniquely named HardFloor gate, no duplicate gate names, timeout `1..=3600`, nonempty argv, no `-c`, and no shell-interpreted command string. Compute `sha256:<64 lowercase hex>` over canonical JSON of the validated typed policy, so TOML key order and formatting do not change the digest.

- [ ] **Step 6: Write tracked policy and machine-local config examples**

Create tracked `config/repo-evolver.policy.toml` with the exact default policy from `2026-09-01-evaluation-and-pr-shepherd.md`: one active candidate, two repairs, branch `evolve/`, budget 2700s/1 attempt/20 files/1500 lines/80 tools/250000 input/50000 output, `max_energy_joules` absent plus `allow_missing_energy_meter=true`, the protected path set including `config/repo-evolver.policy.toml`, and format/clippy/tests/opportunity-contract HardFloor argv/timeouts.

`config/repo-evolver.toml.example` contains only machine placement:

```toml
state_dir = "/var/lib/gzmo-evolver/coordinator"

[repo]
path = "/srv/gzmo/GZMO"
remote = "origin"
base_branch = "main"
owner = "maximilianwruhs-cyber"
repository = "GZMO"

[mission]
json_rel = "opportunity-discovery/next-mission.json"
markdown_rel = "opportunity-discovery/next-mission.md"
refresh_argv = ["bash", "scripts/opportunity-next-mission.sh"]

[worker]
executable = "/usr/local/bin/omp"
profile = "gzmo-repo-evolver-worker"

[policy]
repo_path = "config/repo-evolver.policy.toml"
```

The installer expands operator-provided paths once and writes absolute paths; runtime never relies on shell `~` expansion. Fixture configs use temporary absolute paths.

- [ ] **Step 7: Implement only a real `config-check` CLI**

```text
gzmo-evolver --config <absolute-path> config-check [--json]
```

It loads/validates config plus the working-tree policy, prints normalized paths, target owner/repository, policy digest, budget, protected-path count, and required HardFloor names, and exits nonzero on any failure. Do not add placeholder handlers for future commands. Later tasks add `refresh`, `prepare`, hidden `worker`, `run`, `resume`, `status`, and `abort` only when their implementations exist.

- [ ] **Step 8: Run focused config and policy tests**

Run: `cargo test -p gzmo-evolver config && cargo test -p gzmo-evolver policy`

Expected: PASS, including unknown-field, state/repo overlap, lexical escape, missing executable/policy, unsafe refresh argv, invalid policy, and canonical digest cases.

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock gzmo-evolver config/repo-evolver.toml.example config/repo-evolver.policy.toml
git commit -m "feat: bootstrap connected repository evolver"
```

---
### Task 2: Persist One Candidate and Its Audit Chain

**Files:**
- Create: `gzmo-evolver/src/state.rs`
- Modify: `gzmo-evolver/src/lib.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Test: `gzmo-evolver/src/state.rs`

**Interfaces:**
- Produces: `StateStore::{open,open_existing_readonly,open_in_memory,create_candidate,transition,active_candidate,verify_audit_chain}`, `CoordinatorLock::try_acquire`, `CandidateRecord`, and `TransitionMetadata`.
- Consumes: `CandidateManifest`, `CandidateState`, `AuditEvent`, canonical JSON/digests.

- [ ] **Step 1: Write one-active-candidate and atomic-transition tests**

```rust
#[test]
fn repository_allows_only_one_nonterminal_candidate() {
    let store = StateStore::open_in_memory().unwrap();
    let now = fixed_now();
    store.create_candidate(&manifest("cand-20260901t070000z-one-aaaa1111"), &sha(1), now).unwrap();
    assert!(store.create_candidate(&manifest("cand-20260901t080000z-two-bbbb2222"), &sha(1), now).is_err());
    store.transition(
        &id("cand-20260901t070000z-one-aaaa1111"),
        CandidateState::Failed,
        TransitionMetadata::terminal("operator abort"),
        now,
    ).unwrap();
    assert!(store.create_candidate(&manifest("cand-20260901t080000z-two-bbbb2222"), &sha(1), now).is_ok());
    assert!(store.verify_audit_chain().is_ok());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver one_nonterminal`

Expected: FAIL because `StateStore` does not exist.

- [ ] **Step 3: Create the SQLite schema**

```sql
CREATE TABLE candidates (
  id TEXT PRIMARY KEY,
  repository TEXT NOT NULL,
  manifest_json TEXT NOT NULL,
  manifest_digest TEXT NOT NULL,
  policy_digest TEXT NOT NULL,
  state TEXT NOT NULL CHECK (state IN (
    'observed','prepared','building','evaluating','rejected',
    'review_ready','promotion_pending','soaking','accepted','rolled_back','failed'
  )),
  workspace TEXT,
  candidate_digest TEXT,
  terminal_reason TEXT,
  worker_receipt_json TEXT,
  receipt_digest TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX one_active_candidate
ON candidates(repository)
WHERE state NOT IN ('rejected','accepted','rolled_back','failed');

CREATE TABLE audit_events (
  sequence INTEGER PRIMARY KEY,
  event_json TEXT NOT NULL,
  event_hash TEXT NOT NULL UNIQUE
);
```

Enable WAL, foreign keys, and `busy_timeout=5000`. Store canonical manifest JSON and `sha256:<digest>`. Derive the repository key from `CandidateTarget::Repository`; reject appliance targets in this connected runner.

- [ ] **Step 4: Separate read-only state access from the coordinator lease**

`StateStore::open` creates the state directory/database with directory mode 0700 and file mode 0600 on Unix but does not hold the process lock. `StateStore::open_existing_readonly` opens an existing database read-only and returns `None` when state/database is absent, without creating any path; `status` uses only this method. `CoordinatorLock::try_acquire(<state_dir>/runner.lock)` uses `fs2` exclusive locking and is held by Task 6 for a complete mutating run. Tests use a unique temporary state directory; `open_in_memory` requires no lock.

- [ ] **Step 5: Enforce candidate creation and transitions transactionally**

`create_candidate(manifest, policy_digest, now)` validates both artifacts, inserts state `Observed`, and appends `candidate.observed` in one SQLite transaction. `transition(id, next, metadata, now)` loads the current record, calls `can_transition_to`, applies optional workspace/candidate digest/terminal reason from `TransitionMetadata`, appends a canonical `AuditEvent::next_at`, and commits state plus audit atomically. A terminal transition sets no separate active flag; the partial index derives activeness from state.

`TransitionMetadata` is a validated local value with optional absolute workspace, optional algorithm-qualified candidate digest, optional canonical worker-receipt JSON plus matching `sha256:` digest, and optional terminal reason capped at 4096 bytes. Workspace/candidate/receipt metadata may be set only once and cannot change after set. Receipt JSON and digest must appear together. Terminal reason is required only for `Rejected|RolledBack|Failed` and forbidden for nonterminal/success states.

- [ ] **Step 6: Provide resumable data, not process decisions**

`CandidateRecord` exposes validated manifest, policy digest, state, workspace, candidate digest, opaque canonical receipt JSON/digest, terminal reason, and timestamps. `active_candidate(repository)` and `load(id)` return records after verifying stored JSON/digests and the full audit chain. Task 6—not StateStore—parses `WorkerReceipt` and decides whether a Prepared/Building candidate can resume after inspecting workspace and receipt. Do not add PID or worker-specific logic here.

- [ ] **Step 7: Add a real read-only `status` CLI**

Add `status [--json]` only after `StateStore` exists. It uses `open_existing_readonly`; absent state returns `initialized=false` without filesystem changes, while existing state prints the validated current record/audit head. Retain Task 1 `config-check`; other future commands remain absent.

- [ ] **Step 8: Run focused state tests**

Run: `cargo test -p gzmo-evolver state`

Expected: PASS for one-active uniqueness, legal/illegal transitions, workspace/candidate/receipt metadata immutability, receipt pair/digest checks, terminal-reason rules, concurrent insert race, transaction rollback, stored-digest tamper, audit tamper, missing-state status with zero filesystem changes, read-only status while lock held, and lock exclusion.

- [ ] **Step 9: Commit**

```bash
git add gzmo-evolver/src
git commit -m "feat: persist one audited evolution candidate"
```

---
### Task 3: Adapt the Existing Opportunity Mission

**Files:**
- Create: `gzmo-evolver/src/process.rs`
- Create: `gzmo-evolver/src/mission.rs`
- Modify: `gzmo-evolver/src/lib.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Modify: `Cargo.toml`
- Modify: `gzmo-evolver/Cargo.toml`
- Modify if dependency resolution changes: `Cargo.lock`
- Create: `gzmo-evolver/tests/fixtures/next-mission.json`
- Create: `gzmo-evolver/tests/fixtures/next-mission.md`
- Test: `gzmo-evolver/src/mission.rs`

**Interfaces:**
- Produces: sync `ProcessRunner`, `MissionAdapter::refresh_and_load() -> Mission`, `Mission::to_prepared_candidate(...) -> PreparedCandidate`.
- Consumes: `gzmo.opportunity.next_mission/v1`, `RepoEvolverConfig`, validated `TrustedPolicy`, injected repository baseline commit, and injected UTC time.

- [ ] **Step 1: Write malformed, stale, and pure-conversion tests**

```rust
#[test]
fn accepts_only_one_active_ship_bar_mission() {
    let mission = load_fixture("next-mission.json").unwrap();
    assert_eq!(mission.schema, "gzmo.opportunity.next_mission/v1");
    assert!(mission.ok && mission.ship_bar);
    assert_eq!(mission.bet_id, "felt-use-mass-growth");
}

#[test]
fn conversion_binds_injected_baseline_policy_and_target() {
    let prepared = mission().to_prepared_candidate(
        &config(),
        &policy(),
        "0123456789012345678901234567890123456789",
        fixed_now(),
    ).unwrap();
    assert_eq!(prepared.manifest.baseline_digest, "git-sha1:0123456789012345678901234567890123456789");
    assert_eq!(prepared.policy_digest, policy().digest().unwrap());
    assert_eq!(prepared.manifest.required_gates, policy().required_hard_floor_names());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver mission`

Expected: FAIL because mission/process modules do not exist.

- [ ] **Step 3: Define a narrow synchronous process seam**

`ProcessRunner::run(ProcessSpec) -> ProcessOutput` is internal and accepts an executable, argument vector, cwd, cleared/explicit environment, a combined stdout+stderr byte cap, and timeout. `SystemProcessRunner` uses `std::process::Command`, drains both pipes concurrently without buffering beyond the cap, and terminates/reaps the process group on Unix (the connected production platform) on overflow or timeout; add `libc` as a direct dependency only for Unix process-group termination. Tests use `FakeProcessRunner`. Mission refresh uses fixed trusted limits of 300 seconds and 1 MiB total output. No method accepts a shell command string. Task 4 reuses this seam for Git; Task 5 has a separate async/cgroup worker launcher.

- [ ] **Step 4: Define the strict mission input**

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NextMissionV1 {
    schema: String,
    generated_at: DateTime<Utc>,
    ok: bool,
    bet_id: String,
    title: String,
    score: i64,
    ship_bar: bool,
    mission_md: PathBuf,
    advice: String,
    automation_note: String,
}
```

The two final string fields are part of the payload emitted today by `scripts/opportunity-next-mission.sh`; validate their UTF-8 byte lengths and discard them rather than turning them into authority. Reject schema mismatch, `ok=false`, `ship_bar=false`, blank/unsafe IDs or titles, oversized JSON (64 KiB), Markdown (256 KiB), or auxiliary strings (4 KiB each), timestamps outside the actual refresh start/end interval (with at most five seconds forward clock tolerance), path escape, payload `mission_md` not resolving exactly to the staged configured Markdown path, or Markdown without nonempty `## Mission`, `## Constraints`, and `## Verify` sections. Inject a clock into the adapter so both interval endpoints are deterministic in tests. Markdown is untrusted content; parsing never turns it into commands or policy.

- [ ] **Step 5: Refresh via exact argv without a shell**

Create a unique coordinator-owned 0700 staging root under `<state_dir>/mission-staging/`, with its 0700 `HOME` and an exclusively locked liveness file inside that root; set only a fixed safe `PATH`, that `HOME`, and `GZMO_DATA_NEXT=<staging-root>`, then invoke the validated `bash scripts/opportunity-next-mission.sh` argv through `ProcessRunner`. Cleanup may remove only roots older than twice the fixed refresh timeout whose liveness lock can be acquired, never every sibling. Read `json_rel`/`markdown_rel` beneath staging. Require exit zero, bounded stdout/stderr, both staged artifacts present, no symlink at any path component, canonical containment, modification times at or after refresh start, and JSON `generated_at` inside the actual refresh interval. After full validation, create a 0700 immutable generation under `<state_dir>/missions/generations/<uuid>/` containing 0600 `mission.md` plus canonical sanitized `mission.json`; bind the JSON to both the final absolute Markdown path and its `sha256:` content digest. Fsync the files and generation directory before publishing. Atomically replace a 0600 `<state_dir>/missions/CURRENT` pointer containing only the validated generation basename; readers resolve and revalidate path, JSON, and Markdown digest through that pointer. The rename is the publication commit point: every pre-rename error deletes the unpublished generation and leaves the prior pointer untouched; after rename, no error path may delete the referenced generation, and post-rename durability errors retain a readable new pair while reporting the durability failure. Fsync the missions directory after rename. On all paths remove only this invocation's staging root.

- [ ] **Step 6: Convert mission plus trusted inputs into a prepared candidate**

```rust
pub struct PreparedCandidate {
    pub manifest: CandidateManifest,
    pub policy_digest: String,
}
```

`to_prepared_candidate` receives the 40-lowercase-hex baseline commit and does no Git I/O. Build lowercase ID `cand-<UTC compact lowercase>-<sanitized-bet-id>-<first-8-commit>`, set CandidateKind from policy, authority from kind, repository target from config, `baseline_digest=git-sha1:<commit>`, hard-floor names/budget/protected paths from policy, and injected `created_at`. Validate manifest and `sha256:` policy digest before returning. If the generated ID exceeds 96 bytes, truncate only the sanitized bet portion and preserve timestamp/hash suffix.

- [ ] **Step 7: Add the real `refresh` CLI**

Add `refresh [--json]`. It validates config/current policy, executes refresh, and prints only validated mission metadata and content digest. It may update the coordinator-owned mission snapshot, but it does not open/mutate the candidate database, acquire the coordinator lease, resolve Git, or prepare a candidate. Retain existing commands.

- [ ] **Step 8: Run focused mission/process tests**

Run: `cargo test -p gzmo-evolver mission && cargo test -p gzmo-evolver process`

Expected: PASS for valid refresh/conversion and rejection of stale timestamps/files, hard combined-output overflow, children that close pipes but exceed timeout, timeout with process-group reaping, nonzero exit, actual producer payload mismatch, payload/config path mismatch, symlink ancestors, unsafe/oversized IDs/titles, oversized artifacts, exact unfenced Markdown headings, published Markdown digest tamper, shell-string attempts, concurrent live staging preservation, pre-commit publication failure preserving prior `CURRENT`, post-commit failure retaining the new readable generation, and overlong candidate IDs.

- [ ] **Step 9: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: convert active opportunity into candidate manifest"
```

---
### Task 4: Create a Safe Independent Git Workspace Without Push Authority

**Files:**
- Modify: `gzmo-evolver/src/process.rs`
- Create: `gzmo-evolver/src/git.rs`
- Modify: `gzmo-evolver/src/lib.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Modify: `Cargo.toml`
- Modify: `gzmo-evolver/Cargo.toml`
- Modify if dependency resolution changes: `Cargo.lock`
- Test: `gzmo-evolver/tests/repo_loop.rs`

**Interfaces:**
- Produces: `GitRepository::{refresh,resolve_baseline,read_file_at}`, `GitWorkspace::{prepare,candidate_commit,diff_stats,squash_candidate,cleanup}`.
- Consumes: trusted repo config, coordinator state root, exact repository CandidateManifest, and synchronous `ProcessRunner`.

- [ ] **Step 1: Write a bare-remote independent-clone test**

First close the Task 3 breaker ruling in `process.rs`: both Unix timeout tests use one bounded identity death-poll before sampling and identity-scoped cleanup, so asynchronous SIGKILL/reaping cannot flake the hard-floor gate. Then create a temporary bare origin and trusted checkout with `main`. Assert:

```rust
let baseline = git_repo.refresh_and_resolve_baseline().unwrap();
let policy = git_repo.read_file_at(&baseline, "config/repo-evolver.policy.toml").unwrap();
let ws = git_repo.prepare(&manifest_for(baseline)).unwrap();
assert_eq!(ws.current_branch().unwrap(), manifest_branch());
assert_eq!(ws.merge_base("HEAD", baseline).unwrap(), baseline);
assert_eq!(ws.fetch_url("origin").unwrap(), "no-fetch://candidate-worker");
assert_eq!(ws.push_url("origin").unwrap(), "no-push://candidate-worker");
assert_ne!(ws.git_dir().unwrap(), trusted_checkout_git_dir());
assert!(!ws.uses_alternates_or_shared_objects().unwrap());
assert!(!trusted_main_changed());
assert!(!policy.is_empty());
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test repo_loop independent_workspace`

Expected: FAIL because Git repository/workspace modules do not exist.

- [ ] **Step 3: Maintain a coordinator-owned bare mirror**

Read the raw `remote.origin.url` from the trusted checkout without applying `url.*.insteadOf`; use a direct `url` crate dependency plus strict handling for SCP syntax, and accept only credential-free GitHub HTTPS or SSH URLs whose normalized path identifies the configured owner/repository. The product library has no local/file remote mode. Hermetic tests give the checkout a GitHub-shaped URL and inject a test-only `ProcessRunner` in `tests/repo_loop.rs` that maps that exact transport to a temporary local origin; production command construction and identity validation remain unchanged. Authenticated/private fetch is intentionally deferred to the later trusted GitHub adapter rather than inheriting ambient credentials. Every Git invocation clears the environment and sets only fixed `PATH`, coordinator-owned 0700 `HOME`, `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, disabled hooks/fsmonitor/signing, and command-specific values. Fixed trusted limits: 900 seconds for clone/fetch, 120 seconds otherwise, 8 MiB combined output. Serialize mirror mutation with a distinct fs2 `<state_dir>/mirror.lock` (not the candidate `CoordinatorLock`). Under `<state_dir>/mirror.git`, create a new mirror in a unique 0700 sibling staging directory, fully validate it, then atomically rename it; on refresh verify the existing path is a real nonsymlink bare repository with the expected raw origin before fetching the configured base ref explicitly with prune/no-tags. No credential is embedded, inherited, logged, or forwarded to workers:

```text
git clone --mirror <trusted-remote-url> <state_dir>/mirror.git
git --git-dir <mirror> fetch --prune --no-tags origin +refs/heads/<base_branch>:refs/heads/<base_branch>
git --git-dir <mirror> rev-parse refs/heads/<base_branch>
git --git-dir <mirror> cat-file blob <baseline>:<policy_repo_path>
```

Refuse a dirty trusted checkout (tracked or untracked), a checkout HEAD other than the freshly fetched remote baseline, a non-40-lowercase-hex baseline, missing base branch/policy blob, repository identity mismatch, executable local Git config hooks/fsmonitor/URL rewrites, symlink or submodule/gitlink entries anywhere in the baseline tree, malformed mirror layout, or baseline not equal to the explicitly fetched remote base. Require the working-tree policy digest loaded by config to equal the parsed baseline policy digest. Mirror and coordinator state remain unreadable to the worker OS identity.

- [ ] **Step 4: Clone an independent candidate repository**

Do **not** use `git worktree`; linked worktrees share the common Git directory and would let the worker mutate trusted refs/config. Require the final path absent, clone into a unique nonsymlink sibling staging directory, validate completely, then atomically rename to `<state_dir>/workspaces/<candidate-id>`; every error removes only that staging directory. Create it with:

```text
git clone --no-local --single-branch --no-tags --branch <base_branch> <mirror> <workspace>
git -C <workspace> rev-parse HEAD
git -C <workspace> switch -c evolve/<candidate-id>
git -C <workspace> remote set-url origin no-fetch://candidate-worker
git -C <workspace> remote set-url --push origin no-push://candidate-worker
git -C <workspace> config user.name "GZMO Evolver Candidate"
git -C <workspace> config user.email "candidate@gzmo.invalid"
```

Verify objects are copied independently: no `.git/objects/info/alternates`, no objects/info/alternates environment, no `objects` symlink/hardlink optimization, and workspace git-dir differs from mirror/trusted git-dir. Require a normal in-workspace `.git` directory and scrub/verify all remote URLs and executable local Git config before the path can be recorded. Only after preparation succeeds does the coordinator transfer workspace ownership to the worker identity in the later system-service adapter.

- [ ] **Step 5: Validate and normalize the post-worker candidate commit**

Require the recorded branch, clean workspace (including untracked files), HEAD descended from the exact baseline, no merge commits in `baseline..HEAD`, no symlink or submodule/gitlink entries/changes, and at least one change. The trusted coordinator creates the normalized commit with `git commit-tree` from the validated HEAD tree and exact baseline parent—never by invoking hooks—using author/committer `GZMO Evolver Candidate <candidate@gzmo.invalid>`, message `evolve(<mission-id>): candidate`, disabled signing, and injected UTC timestamp; update the candidate ref only with an expected-old-value check. Verify one parent, the resulting `git-sha1:` digest, clean worktree, and stable tree/diff. Worker-local commits are untrusted inputs and are never pushed directly.

- [ ] **Step 6: Inspect bounded diff facts without evaluating quality**

`diff_stats` uses `git diff --no-renames --raw -z`, `--numstat -z`, and `--check` through argument vectors against the exact baseline/candidate pair. It reports UTF-8 paths, modes, binary markers, checked added/deleted line totals, and whitespace status. It rejects malformed/truncated output, duplicate/inconsistent path records, special modes, path traversal, and paths that fail the manifest `PathPolicy`; it enforces manifest file/added-line ceilings before returning. `--check` exit 1 means `whitespace_ok=false`; other Git failures remain errors. The deeper trusted gate/holdout evaluator remains in the next plan.

- [ ] **Step 7: Implement safe cleanup**

Cleanup accepts a validated terminal `CandidateRecord`. It removes only an independent workspace whose canonical path is an immediate descendant of `<state_dir>/workspaces`, whose repository candidate ID/HEAD matches the record, and whose `.git` directory is inside that workspace. It never removes the trusted checkout, mirror, coordinator state, a symlink, a nonterminal candidate, or a workspace after `ReviewReady` until the later PR plan owns remote lifecycle.

- [ ] **Step 8: Add a real `prepare` CLI**

`prepare [--json]` acquires `CoordinatorLock`, opens validated state, and first returns any existing active candidate without refreshing mission/mirror or creating files. Otherwise it refreshes the mirror, requires the clean trusted checkout HEAD and working policy to match that baseline, loads/parses policy bytes from the baseline commit, and only then executes/validates the baseline-owned mission producer. It creates `PreparedCandidate`, inserts `Observed`, creates the independent workspace, and transitions to `Prepared` with immutable workspace metadata. A handled workspace/preparation failure cleans partial paths and transitions the new candidate to `Failed` with a bounded reason; a crash leaves resumable `Observed` data for Task 6. Update standalone `refresh` now that Git exists: verify mirror baseline, checkout HEAD, and baseline policy before executing the producer; it still does not open candidate state or acquire its lease. Repeated prepare with an active candidate returns it and creates nothing else.

- [ ] **Step 9: Run focused Git/workspace tests**

Run: `cargo test -p gzmo-evolver --test repo_loop`

Expected: PASS for bounded identity death-wait, injected hermetic transport over a production-shaped GitHub URL, independent object storage, no trusted-ref/config mutation, disabled remotes, exact baseline/policy read, deterministic squash, diff limits, failure-to-Failed handling, transient mirror contention remaining resumable, early active-candidate return, and cleanup; reject dirty/non-baseline checkout, embedded credentials, wrong host, every local/file/git transport in product APIs, stale/unknown base, policy mismatch, symlink/gitlink, shared objects/alternates, existing or staged path collisions, executable Git config, merge commit, empty/dirty workspace, malformed diff output, and cleanup mismatch.

- [ ] **Step 10: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: isolate candidates in independent no-push clones"
```

---
### Task 5: Invoke OMP as an Uncredentialed Worker

**Files:**
- Modify: `gzmo-evolver/Cargo.toml`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `gzmo-evolver/src/worker.rs`
- Modify: `gzmo-evolver/src/lib.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Create: `gzmo-evolver/tests/fixtures/fake-worker.sh`
- Modify: `gzmo-evolver/tests/repo_loop.rs`
- Test: `gzmo-evolver/src/worker.rs`

**Interfaces:**
- Produces: async `WorkerLauncher`, `SystemdWorkerLauncher`, worker-side `run_worker_request`, validated `WorkerRequest`, and `WorkerReceipt`.
- Consumes: independent candidate workspace, rendered mission, manifest/policy digests, signed budget, and fixed local OMP executable/profile.

- [ ] **Step 1: Write environment, request, and receipt rejection tests**

The fake worker records names—not values—of visible variables. Assert none of:

```rust
const FORBIDDEN_ENV: &[&str] = &[
    "GH_TOKEN", "GITHUB_TOKEN", "COPILOT_GITHUB_TOKEN", "SSH_AUTH_SOCK",
    "AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY", "GEMINI_API_KEY", "HTTP_PROXY", "HTTPS_PROXY",
];
```

Assert the OMP child environment equals one fixed allowlist—dedicated HOME, `/usr/bin:/bin` PATH, locale, null Git config/noninteractive guards, and loopback-only `NO_PROXY`—and contains no other inherited name; model endpoint/provider configuration lives only in the sealed read-only profile, not invented environment variables. The listed forbidden names are explicit regression sentinels, not the whole defense. Reject unknown request/receipt fields, digest or canonical-byte mismatch, wrong candidate/UID/GID/owner, mutable or worker-owned request, symlink/path escape, output outside the fixed candidate output root, untrusted/writable OMP executable or profile, OMP version mismatch, missing or inconsistent JSONL usage/tool events, over-budget usage, and malformed OMP JSON.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver worker`

Expected: FAIL because worker module/command does not exist.

- [ ] **Step 3: Define sealed request and bounded receipt**

```rust
#[derive(Serialize)]
pub struct WorkerRequest {
    schema: String,
    candidate_id: CandidateId,
    manifest_digest: String,
    policy_digest: String,
    policy_toml_digest: String,
    mission_digest: String,
    system_prompt_digest: String,
    omp_config_digest: String,
    workspace: PathBuf,
    mission_markdown: PathBuf,
    system_prompt: PathBuf,
    omp_config: PathBuf,
    output_dir: PathBuf,
    omp_executable: PathBuf,
    omp_profile: String,
    profile_digest: String,
    omp_version: String,
    coordinator_uid: u32,
    expected_uid: u32,
    expected_gid: u32,
    budget: ResourceBudget,
    issued_at: DateTime<Utc>,
    deadline: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WorkerReceipt {
    schema: String,
    candidate_id: CandidateId,
    manifest_digest: String,
    policy_digest: String,
    omp_version: String,
    started_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
    exit_code: i32,
    output_digest: String,
    worker_head_digest: Option<String>,
    usage: ResourceUsage,
}
```

Use schemas `gzmo.repo_evolver.worker_request/v1` and `gzmo.repo_evolver.worker_receipt/v1`. Fixed production roots are `/run/gzmo-evolver` (sealed requests), `/var/lib/gzmo-evolver-worker/output` (worker outputs/homes), `/var/lib/gzmo-evolver-worker/profiles` (trusted profiles), and `/run/netns/gzmo-evolver-model` (local-model network namespace); tests supply isolated roots through private `#[cfg(test)]` helpers, never a production-exported ownership authority. Fields remain private with getters and validated constructors; each type uses a private `#[serde(deny_unknown_fields)]` Raw form plus custom validated `Deserialize`. Intrinsic checks cover exact schema, algorithm-qualified typed-policy/raw-policy/profile digests, OMP version exactly matching probed `v18.x`, safe profile, absolute UTF-8 lexically normalized paths (the normalized value must equal the input), distinct nonzero coordinator/worker IDs, valid budget, checked `deadline == issued_at + budget.wall_seconds`, monotonic timestamps, and nonnegative exit representation. Operations/Task 6 provision the output root, profile root, `<output-root>/<candidate-id>`, and its `home` with their specified owners/modes; the coordinator may create/chmod only its request root and request staging, otherwise it verifies real nonsymlink directories and never mutates foreign/root-owned roots. `seal_worker_bundle` strictly validates and canonically digests the installed profile tree, then atomically publishes a coordinator-owned 0750/group-worker request directory under `<request-root>/<candidate-id>` containing 0440 `request.json`, canonical `manifest.json`, baseline `policy.toml`, `system-prompt.md`, `mission.md`, and strict `omp-overlay.yml`; every companion and the external profile tree is request-bound. Production `load_sealed_request` always uses real effective identity/lstat; injected ownership helpers are private test-only. It rejects every symlink component and checks canonical immediate-child roots, request/companion owner=`coordinator_uid`, group=`expected_gid`, exact modes, current effective UID/GID, preprovisioned output/home, workspace placement, and an OMP executable/profile owned by root/coordinator and not worker/group/world-writable. `load_worker_receipt` caps size, checks worker ownership/mode, canonical JSON, and raw-output digest. `WorkerReceipt::validate_against(request, actual_head)` verifies candidate/digest/version binding, exit zero, completion inside `[issued_at, deadline]`, required HEAD equality, and `usage.fits`; deserialization alone never claims external checks.

- [ ] **Step 4: Implement the hidden worker command under the worker identity**

Add Unix-target dependency `nix` with `user` and `fs` features. Make global `--config` optional in Clap and require it only for public coordinator commands; the hidden worker never reads coordinator config. Hidden CLI:

```text
gzmo-evolver worker --request /run/gzmo-evolver/<candidate-id>/request.json
```

It requires request path under fixed `/run/gzmo-evolver`, verifies effective UID/GID against the sealed request, rejects UID 0 and the coordinator UID, validates all companions/executable/profile/workspace branch+baseline/output directory, and takes a per-output exclusive worker lease before spawning OMP. Production systemd supplies the workspace/profile mounts; tests inject roots/identity without requiring live OS users.

- [ ] **Step 5: Construct fixed OMP argv in code**

The request may choose only the sealed executable/profile; no arbitrary args. Probe `omp --version`, require the sealed v18.x value, then build exactly:

```text
<omp> -p --mode json --no-session --no-title --no-prewalk --no-pty
  --model @code_candidate
  --profile <profile>
  --cwd <independent-workspace>
  --max-time <budget.wall_seconds>s
  --approval-mode yolo
  --no-extensions --no-skills --no-rules
  --tools read,bash,edit,write,grep,glob,lsp
  --config <sealed-omp-overlay.yml>
  --append-system-prompt <sealed-system-prompt.md>
  @<untrusted-mission.md>
```

Clear the environment and set exactly the fixed allowlist tested in Step 1. `HOME=<output-dir>/home` and its OMP runtime directories are preprovisioned worker-owned 0700 paths. On every attempt, before OMP starts, the hidden worker symlink-safely removes every prior runtime entry beneath `<home>/.omp/profiles/<profile>/agent` except the two file mount points, so a previous attempt cannot persist hooks/tools/context or poison caches. The transient unit bind-mounts installed `agent/config.yml` and `agent/models.yml` individually read-only onto those exact destination files; code never copies or reuses worker-writable configuration. The hidden worker revalidates both destination files' trusted owner/mode/content and the request-bound profile digest immediately before spawning OMP; parent/runtime directories remain worker-owned/writable for `agent.db`, blobs, and model cache. Parse installed YAML strictly: `modelRoles` contains exactly one entry, `code_candidate = "<provider>/<model>"`; exactly that provider/model exists, uses `auth: none`, contains no exact credential-bearing keys (`apiKey`, `headers`, `authHeader`, `token`, `password`, `secret`, `credential`), and has an `http` base URL whose parsed host is loopback; benign schema keys such as `maxTokens` remain valid. The sealed settings overlay uses only real OMP keys: the same string model-role pin, `mcp.enableProjectConfig: false`, and the exact complete replacement discovery-source list `native, omp-plugins, claude, agent-plugins, codex, agents, claude-plugins, gemini, opencode, cursor, windsurf, cline, github, vscode, agents-md, mcp-json, ssh-json`; argv also disables extensions/skills/rules and selects `@code_candidate`. No provider/API/Git/SSH/proxy credential survives. The safe PATH deliberately excludes the OMP install directory; the parent is launched by its validated absolute path.

- [ ] **Step 6: Render the bounded mission outside the worker**

The coordinator renders two bounded artifacts, never one mixed-authority prompt. `system-prompt.md` is trusted policy: exact candidate/baseline/workspace, protected paths, required gates, signed budgets, no remote/main/credential/policy/evaluator changes, commit required, one candidate, then stop. `mission.md` contains only the approved untrusted Opportunity Markdown under a data label. OMP receives the first through `--append-system-prompt` and the second as the sole user `@file`, so later untrusted text cannot become system policy. Both are coordinator-owned 0440 companions bound by the request.

- [ ] **Step 7: Enforce limits and parse OMP JSON fail-closed**

`SystemdWorkerLauncher` uses argv-only bounded `systemd-run --unit=gzmo-evolver-worker@<candidate-id>.service --no-block --service-type=exec --property=RemainAfterExit=yes` rather than trusting a mutable template. It pins numeric User/Group, `UMask=0077`, `NoNewPrivileges`, strict system/home/device/kernel protections, a fixed prevalidated local-model `NetworkNamespacePath`, read-only request/executable mounts, separate read-only source-to-destination binds for installed `agent/config.yml` and `agent/models.yml`, read-write workspace/output mounts, `MemoryMax=8G`, `TasksMax=128`, and `RuntimeMaxSec=<signed wall seconds>`, then executes the current trusted `gzmo-evolver worker --request …`. Do not use `--collect` before status is read. Poll exact `LoadState`, `ActiveState`, and `SubState`; `activating|active/running|deactivating` are nonterminal, `active/exited` or `failed` trigger validation of `Result` and `ExecMainStatus`. Missing/collected status is failure. Only after capturing success/failure does cleanup run `systemctl stop` and `reset-failed`; deadline runs stop then `systemctl kill --kill-whom=all --signal=KILL` and verifies inactive. System command output is capped at 1 MiB. Fixed identities, preprovisioned roots/output/home/runtime mount-point parents, profile, namespace, and transient-unit privilege are operations prerequisites. Test-only fake launchers live in test code or behind `#[cfg(test)]`, never in the default library surface; no live systemd/OMP process is required here.

Raw OMP stdout is capped at 8 MiB and stored worker-only. Parse the official OMP v18 `--mode json` JSONL contract: one `type=session, version=3` header; unique paired `tool_execution_start/end` IDs; assistant `message_end.message.usage` fields `input`, `output`, `cacheRead`, `cacheWrite`, `totalTokens`, and optional orchestration buckets; a terminal non-error `agent_end`. Count tool starts once; checked-sum every assistant turn's noncached+cache+orchestration input and output buckets and require consistency with `totalTokens`. Missing counters, malformed/truncated lines, duplicate/unpaired tools, nonterminal/error stop, overflow, or unknown numeric shapes fail rather than become zero. After OMP exits, fixed Git argv requires the recorded branch, clean committed workspace, HEAD different from and descended from baseline, and computes changed-file/added-line facts. Set attempts=1, wall time rounded up, energy absent only when signed policy allows, require `usage.fits`, and write the receipt atomically.

- [ ] **Step 8: Require a verified receipt before accepting the candidate**

The coordinator treats receipt and raw output as untrusted: cap/read nonsymlink files, validate real ownership/modes, recompute the raw-output digest, require canonical receipt JSON, compare request/candidate/manifest/typed-policy/raw-policy/OMP version and actual workspace HEAD/branch/diff facts independently, and require the signed budget before Task 6 may persist receipt/candidate metadata. A nonzero OMP exit may produce diagnostic files but never a valid success receipt.

- [ ] **Step 9: Run focused worker tests**

Run: `cargo test -p gzmo-evolver worker -- --nocapture`

Expected: PASS for full-vector exact OMP argv, exact environment names/values, strict real-key overlay/profile YAML, and source-shaped v18 JSONL; sealed bundle/receipt happy path; fake transient-unit launcher/worker; real-lstat rejection of coordinator-created output, request/all companion mutation including policy TOML, wrong UID/GID/owner/mode/root, lexical non-normalization, symlink swap, forbidden or extra env, untrusted executable/profile, OMP version mismatch, duplicate launch, deactivating then successful result, collected/failed unit, timeout stop+kill, nonzero exit, malformed/truncated/duplicate/unpaired JSON events, missing/inconsistent usage, raw-output/receipt mode/canonical/digest failures, output cap, ceiling-rounded wall time, over-budget use, dirty workspace, protected/special diff, and missing commit all fail closed. The committed fake-worker fixture must be executed or removed—no duplicate inline fake. Hidden worker rejects a supplied `--config`; public commands require it.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock gzmo-evolver
git commit -m "feat: run bounded OMP candidate workers"
```

---
### Task 6: Orchestrate Refresh → Prepare → Build → Evaluate Boundary

**Files:**
- Create: `gzmo-evolver/src/runner.rs`
- Modify: `gzmo-evolver/src/mission.rs`
- Modify: `gzmo-evolver/src/git.rs`
- Modify: `gzmo-evolver/src/state.rs`
- Modify: `gzmo-evolver/src/worker.rs`
- Modify: `gzmo-evolver/src/lib.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Modify: `gzmo-evolver/tests/repo_loop.rs`

**Interfaces:**
- Produces: `RepoEvolver::run_once()` / `resume()` ending at `CandidateState::Evaluating`, `abort()`, one structured status, `GitRepository::open_or_prepare_workspace`, and worker launch/status/wait/stop/provision seams.
- Consumes: coordinator lock/state, immutable mission generation, trusted baseline policy, Git repository/independent workspace, sealed worker runtime, launcher/request/receipt.

- [ ] **Step 1: Write the vertical state/audit test**

```rust
#[tokio::test]
async fn one_run_stops_at_evaluation_boundary() {
    let harness = RepoHarness::new().await;
    let outcome = harness.evolver.run_once().await.unwrap();
    assert_eq!(outcome.state, CandidateState::Evaluating);
    assert!(outcome.candidate_digest.as_deref().unwrap().starts_with("git-sha1:"));
    assert_eq!(harness.remote_main(), harness.initial_main());
    assert!(harness.remote_branches().iter().all(|b| b == "main"));
    assert!(harness.store.verify_audit_chain().is_ok());
    assert_eq!(harness.audit_states(), ["observed", "prepared", "building", "evaluating"]);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test repo_loop one_run_stops`

Expected: FAIL because coordinator does not exist.

- [ ] **Step 3: Implement the exact new-candidate sequence**

`run_once` holds `CoordinatorLock` for the complete mutation. It first validates state/audit and returns/resumes any active candidate. With none, it calls the existing active-first `prepare_candidate` flow rather than duplicating it; that flow refreshes the mirror, requires clean checkout HEAD/baseline policy match, then refreshes the mission and creates the Observed/Prepared candidate. Change `Mission::to_prepared_candidate` so `manifest.mission_id` is the immutable mission generation UUID (candidate ID still carries the bet slug), and add `MissionAdapter::load_generation(id)`; every later stage reloads exactly that generation, never `CURRENT`.

For a newly Prepared candidate:

1. revalidate audit, manifest, exact baseline policy bytes/digest, mission generation/content digest, and independent workspace;
2. resolve the fixed operations-installed worker UID/GID and invoke `WorkerRuntimeProvisioner` for `gzmo-evolver-worker-runtime@<candidate-id>.service`; this fixed unit may create only the six Task-5 output/HOME/runtime directories under the fixed output root as the fixed worker identity—no caller-supplied path/user/property—and must report successful terminal status; Task-5 seal then real-lstat revalidates every path;
3. probe/validate the installed profile and OMP v18 executable, render the trusted system prompt/separate user mission/strict overlay, and seal or fully validate the deterministic request bundle without overwriting it;
4. transition `Prepared → Building` before any worker launch;
5. launch/wait; independently load and validate receipt/raw output/workspace HEAD/branch/diff facts and budget;
6. normalize hostile worker history with an idempotent Git operation: use `receipt.completed_at` as the fixed UTC commit timestamp; if HEAD is already the exact one-parent normalized commit with expected tree/parent/identity/message/date, reuse it, otherwise validate and squash once via commit-tree/CAS;
7. re-run bounded diff/path/budget checks and require receipt changed-files/added-lines/head facts to equal coordinator observations;
8. transition `Building → Evaluating` atomically with immutable `git-sha1:` candidate digest plus canonical receipt JSON/`sha256:` digest, then stop. Never evaluate quality, push, open a PR, or advance past Evaluating.

Every transition is state+audit atomic. Before candidate creation, failure leaves no state. After creation, deterministic content/trust failure transitions to `Failed` with a bounded reason when legal; transient provisioner/mirror lease contention leaves the persisted resumable state unchanged. A failed database transition preserves artifacts and returns an explicit recovery-required error.


- [ ] **Step 4: Implement idempotent resume by persisted state**

- `Observed`: refresh current baseline and policy, require they still equal the manifest, reload the immutable mission generation, then `open_or_prepare_workspace`; an exact already-published baseline workspace is reused, any ambiguous/corrupt path fails closed.
- `Prepared`: open/revalidate the recorded workspace and immutable mission/baseline policy; provision runtime; seal or validate the deterministic request; transition Building; launch.
- `Building`: validate the sealed request against record/mission/policy/workspace. If a valid receipt exists, continue without relaunch. Otherwise inspect the exact worker unit: Running waits only to the persisted deadline; Succeeded without receipt or Failed/NotFound becomes `worker_lost_without_receipt`; never start a second unit from Building.
- Crash after squash is idempotent: recognize the exact normalized commit using receipt time and continue the single state transition rather than resquashing.
- `Evaluating`: return the existing outcome unchanged; the next plan owns evaluation.
- terminal latest candidate on explicit `resume`: return unchanged and create nothing. `ReviewReady|PromotionPending|Soaking` refuse as later-stage states. `run_once` may create a new candidate only when no active candidate exists.

Extend `WorkerLauncher` with typed `inspect`, `wait_existing`, and `stop`; unit states are `NotFound|Running|Succeeded|Failed` and never inferred from receipt presence. Extend StateStore with validated `latest_candidate(repository)` for status/resume. Never overwrite workspace/policy/mission/request/candidate/receipt digests. Revalidate the full audit chain before every resume action. Exclusive CoordinatorLock and Task-5 WorkerLease are the single-writer basis for path-based runtime cleanup; no worker child may exist while cleanup runs.

- [ ] **Step 5: Implement explicit abort without deletion**

`abort <candidate-id> --reason <text>` acquires the lock, validates audit/state and exact id, and for Building first requests stop/kill and verifies the unit inactive before atomically transitioning a legal pre-evaluation state to Failed. Observed/Prepared transition directly. It preserves workspace/request/receipt/raw output and never invokes cleanup. Abort cannot alter Evaluating or later states; future review lifecycle owns them. Empty/oversized reasons reject before side effects; a stop or database failure returns recovery-required with artifacts preserved.

- [ ] **Step 6: Implement one structured status model**

Replace the earlier partial status with one serializable `gzmo.repo_evolver.status/v1` model built read-only from `latest_candidate`, verified audit, parsed persisted receipt, and typed worker-unit inspection. It includes repository, mission generation/candidate ID, state, baseline/candidate/policy/manifest/receipt digests, budget max/used/remaining (unknown remains null, never zero), workspace, worker state/deadline, last audit sequence/hash, terminal reason, and one exact next allowed action. Human output derives only from this model. Do not expose raw logs, mission body, environment, or credentials.

Add public CLI commands `run`, `resume`, and `abort`; retain `config-check`, `status`, `refresh`, `prepare`, and hidden `worker`. Every command with output supports `--json`. No placeholder command remains. Production construction uses the fixed systemd provisioner/launcher and fixed worker account; hermetic tests define their own fakes outside the product library.

- [ ] **Step 7: Run all runner tests**

Run: `cargo test -p gzmo-evolver runner -- --nocapture && cargo test -p gzmo-evolver --test repo_loop one_run_stops -- --nocapture`

Expected: PASS for exact audit sequence, mission-generation binding, runtime provision-before-seal, every crash boundary/resume state, sealed-request reuse, worker Running/Succeeded/Failed/NotFound and receipt paths, no duplicate launch, post-squash resume, repeated run idempotence, terminal/later-state handling, abort stop-before-state and artifact preservation, status null-vs-zero semantics, coordinator/worker lock races, and no remote/trusted-checkout mutation.

- [ ] **Step 8: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: orchestrate one repository candidate at a time"
```

---
### Task 7: Prove Stage 1 Candidate Generation End to End

**Files:**
- Modify: `gzmo-evolver/tests/repo_loop.rs`
- Create: `gzmo-evolver/tests/fixtures/fixture-repo/README.md`
- Create: `gzmo-evolver/tests/fixtures/fixture-repo/config/repo-evolver.policy.toml`
- Create: `gzmo-evolver/tests/fixtures/fixture-repo/research/opportunities/fixture.md`
- Create: `gzmo-evolver/tests/fixtures/fixture-repo/scripts/opportunity-next-mission.sh`

**Interfaces:**
- Produces: hermetic acceptance proof for Stage 1 through `CandidateState::Evaluating` before any GitHub adapter or live system service exists.
- Consumes: all connected runner interfaces with real `SystemProcessRunner` mission/Git subprocesses, a GitHub-shaped raw origin mapped to the temporary bare origin only by the test-owned transport wrapper, and fake runtime-provisioner/worker-launcher seams. Product code never accepts a local/file remote.

- [ ] **Step 1: Add the complete failure matrix**

Build one matrix mapping every row below to a causal named test; reuse existing Task 1–6 unit/integration tests and add only missing scenarios—do not duplicate lower-level coverage or count assertions that never reach the named branch. Use real temporary bare Git origins/mirrors/independent clones. The vertical fixture executes its committed `scripts/opportunity-next-mission.sh` through the real `SystemProcessRunner`; only privileged runtime provisioning and OMP/systemd worker execution remain deterministic fakes.

- dirty trusted checkout and stale/changed remote base;
- malformed/hold/stale mission and two-active-mission refresh failure;
- one-active candidate lock and concurrent runner lock;
- worker timeout/lost unit/nonzero/malformed or missing receipt;
- forbidden environment name and wrong UID/request mode/digest/path;
- protected path, binary/submodule, changed-file/line cap, whitespace error;
- merge commit, no commit, dirty workspace, shared objects/alternates;
- audit/state/policy/manifest/receipt tamper;
- crashes after Observed, Prepared, Building, and receipt creation, then idempotent resume;
- repeated run at Evaluating creates no worker/workspace/remote change;
- abort preserves all evidence;
- trusted checkout main and bare remote main remain byte-identical; no remote candidate branch exists.

- [ ] **Step 2: Run the complete connected-runner quality gate**

```bash
cargo fmt --all -- --check
cargo clippy -p gzmo-evolver --all-targets -- -D warnings
cargo test -p gzmo-evolver --all-targets -- --nocapture
```

Expected: all pass at default parallelism. Tests use a GitHub-shaped raw origin and a test-only exact URL rewrite to local filesystem Git; product APIs still reject `file://` and filesystem origins. No GitHub/public network, provider credentials, live system services, or actual OMP/model process. Any LockBusy fixture retry remains bounded, exact-error-only, and absent from contention assertions.

- [ ] **Step 3: Run the hermetic vertical smoke**

Run the fixture integration test by exact name:

```bash
cargo test -p gzmo-evolver --test repo_loop fixture_run_reaches_evaluating_without_remote_mutation -- --nocapture
```

Expected: exactly one candidate reaches `evaluating`; its independent clone has one normalized one-parent candidate commit with a nonempty bounded diff; trusted checkout and remote main are byte-identical; remote refs contain only main; the fake worker sees the exact allowed environment and no forbidden name; canonical receipt facts equal coordinator diff facts; audit states are exactly Observed/Prepared/Building/Evaluating; a repeated run performs no extra provision, launch, workspace, state, or remote mutation.

The real OMP/systemd smoke is deliberately deferred to `2026-09-01-continuous-evolution-operations.md` after coordinator/worker/model system identities and the private model namespace exist. This subplan must not weaken the trust model to manufacture a live smoke.

- [ ] **Step 4: Commit**

```bash
git add gzmo-evolver
git commit -m "test: prove isolated repository candidate generation"
```
