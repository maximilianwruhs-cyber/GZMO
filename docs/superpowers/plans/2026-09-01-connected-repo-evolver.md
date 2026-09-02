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

Create a unique coordinator-owned 0700 staging root under `<state_dir>/mission-staging/`, set only a fixed safe `PATH`, a coordinator-owned 0700 `HOME`, and `GZMO_DATA_NEXT=<staging-root>`, and invoke the validated `bash scripts/opportunity-next-mission.sh` argv through `ProcessRunner`. Read `json_rel`/`markdown_rel` beneath staging. Require exit zero, bounded stdout/stderr, both staged artifacts present, no symlink at any path component, canonical containment, modification times at or after refresh start, and JSON `generated_at` inside the actual refresh interval. After full validation, create a 0700 immutable generation under `<state_dir>/missions/generations/<uuid>/` containing 0600 `mission.md` plus canonical sanitized `mission.json`, with `mission_md` rebound to that generation's absolute Markdown path. Fsync the files and generation directory before publishing. Atomically replace a 0600 `<state_dir>/missions/CURRENT` pointer containing only the validated generation basename; readers resolve and revalidate the immutable pair through that pointer, so the pair—not two independent files—is the publication unit. On any handled failure remove staging and the unpublished generation and leave the prior `CURRENT` target untouched; a later refresh removes abandoned temporary directories. Then remove staging.

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

Expected: PASS for valid refresh/conversion and rejection of stale timestamps/files, output overflow, timeout with process reaping, nonzero exit, actual producer payload mismatch, payload/config path mismatch, symlink ancestors, unsafe IDs/titles, oversized artifacts, missing sections, shell-string attempts, failed publication preserving prior `CURRENT`, and overlong candidate IDs.

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
- Test: `gzmo-evolver/tests/repo_loop.rs`

**Interfaces:**
- Produces: `GitRepository::{refresh,resolve_baseline,read_file_at}`, `GitWorkspace::{prepare,candidate_commit,diff_stats,squash_candidate,cleanup}`.
- Consumes: trusted repo config, coordinator state root, exact repository CandidateManifest, and synchronous `ProcessRunner`.

- [ ] **Step 1: Write a bare-remote independent-clone test**

Create a temporary bare origin and trusted checkout with `main`. Assert:

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

Read the configured remote URL from the trusted checkout and verify it resolves to the configured owner/repository. Under `<state_dir>/mirror.git`, create or update a bare mirror as the coordinator using explicit argv and no credential forwarding to workers:

```text
git clone --mirror <trusted-remote-url> <state_dir>/mirror.git
git --git-dir <mirror> remote update --prune
git --git-dir <mirror> rev-parse refs/heads/<base_branch>
git --git-dir <mirror> show <baseline>:<policy_repo_path>
```

Refuse a dirty trusted checkout, non-40-lowercase-hex baseline, missing base branch/policy, repository identity mismatch, submodule/gitlink in the baseline tree, or baseline not equal to the freshly fetched remote base. Mirror and coordinator state are unreadable to the worker OS identity.

- [ ] **Step 4: Clone an independent candidate repository**

Do **not** use `git worktree`; linked worktrees share the common Git directory and would let the worker mutate trusted refs/config. Create `<state_dir>/workspaces/<candidate-id>` with:

```text
git clone --no-local --single-branch --no-tags --branch <base_branch> <mirror> <workspace>
git -C <workspace> rev-parse HEAD
git -C <workspace> switch -c evolve/<candidate-id>
git -C <workspace> remote set-url origin no-fetch://candidate-worker
git -C <workspace> remote set-url --push origin no-push://candidate-worker
git -C <workspace> config user.name "GZMO Evolver Candidate"
git -C <workspace> config user.email "candidate@gzmo.invalid"
```

Verify objects are copied independently: no `.git/objects/info/alternates`, no `objects` symlink, and workspace git-dir differs from mirror/trusted git-dir. Only after preparation succeeds does the coordinator transfer workspace ownership to the worker identity in the later system-service adapter.

- [ ] **Step 5: Validate and normalize the post-worker candidate commit**

Require recorded branch, clean workspace, HEAD descended from exact baseline, no merge commits in `baseline..HEAD`, no submodule/gitlink changes, and at least one change. The trusted coordinator then squashes `baseline..HEAD` into one commit with author/committer `GZMO Evolver Candidate <candidate@gzmo.invalid>`, message `evolve(<mission-id>): candidate`, and injected timestamp; verify the resulting `git-sha1:` digest and diff are stable. Worker-local commits are untrusted inputs and are never pushed directly.

- [ ] **Step 6: Inspect bounded diff facts without evaluating quality**

`diff_stats` uses `git diff --raw -z`, `--numstat -z`, and `--check` through argument vectors. It reports paths, modes, binary markers, added/deleted lines, and whitespace status. It rejects malformed output and paths that fail the manifest `PathPolicy`; it enforces manifest file/added-line ceilings before returning. The deeper trusted gate/holdout evaluator remains in the next plan.

- [ ] **Step 7: Implement safe cleanup**

Cleanup accepts a validated terminal `CandidateRecord`. It removes only an independent workspace whose canonical path is an immediate descendant of `<state_dir>/workspaces`, whose repository candidate ID/HEAD matches the record, and whose `.git` directory is inside that workspace. It never removes the trusted checkout, mirror, coordinator state, a symlink, a nonterminal candidate, or a workspace after `ReviewReady` until the later PR plan owns remote lifecycle.

- [ ] **Step 8: Add a real `prepare` CLI**

`prepare [--json]` acquires `CoordinatorLock`, refreshes/validates mission, refreshes the mirror, loads and validates policy bytes from the baseline commit, creates `PreparedCandidate`, inserts `Observed`, transitions to `Prepared` while recording the independent workspace, then stops. Repeated prepare with an active candidate returns that candidate and creates nothing else.

- [ ] **Step 9: Run focused Git/workspace tests**

Run: `cargo test -p gzmo-evolver --test repo_loop`

Expected: PASS for independent object storage, no trusted-ref mutation, disabled remotes, exact baseline/policy read, squash, diff limits, and cleanup; reject dirty checkout, stale/unknown base, submodule, shared objects/alternates, existing candidate, symlink/path escape, merge commit, empty/dirty workspace, and cleanup mismatch.

- [ ] **Step 10: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: isolate candidates in independent no-push clones"
```

---
### Task 5: Invoke OMP as an Uncredentialed Worker

**Files:**
- Modify: `gzmo-evolver/Cargo.toml`
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

Assert the worker sees only a dedicated HOME, fixed safe PATH, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_TERMINAL_PROMPT=0`, and loopback model configuration. Reject unknown request/receipt fields, digest mismatch, wrong candidate/UID, mutable or worker-owned request, symlink/path escape, output outside the candidate output root, missing usage, over-budget usage, and malformed OMP JSON.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver worker`

Expected: FAIL because worker module/command does not exist.

- [ ] **Step 3: Define sealed request and bounded receipt**

```rust
#[derive(Serialize)]
pub struct WorkerRequest {
    pub schema: String,
    pub candidate_id: CandidateId,
    pub manifest_digest: String,
    pub policy_digest: String,
    pub workspace: PathBuf,
    pub mission_markdown: PathBuf,
    pub output_dir: PathBuf,
    pub omp_executable: PathBuf,
    pub omp_profile: String,
    pub expected_uid: u32,
    pub budget: ResourceBudget,
    pub issued_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WorkerReceipt {
    pub schema: String,
    pub candidate_id: CandidateId,
    pub manifest_digest: String,
    pub policy_digest: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub exit_code: i32,
    pub output_digest: String,
    pub worker_head_digest: Option<String>,
    pub usage: ResourceUsage,
}
```

Use schemas `gzmo.repo_evolver.worker_request/v1` and `gzmo.repo_evolver.worker_receipt/v1`. Each type uses a private `#[serde(deny_unknown_fields)]` Raw form plus custom validated `Deserialize` for intrinsic fields: exact schema, `sha256:` manifest/policy/output digests, optional `git-sha1:` untrusted worker-head digest, safe profile, absolute lexically normalized paths, valid budget, `deadline == issued_at + budget.wall_seconds`, monotonic receipt timestamps, and nonnegative exit representation. Context-dependent checks are explicit: `load_sealed_request(path, expected_roots, current_uid)` verifies canonical path ownership, non-symlink request, Unix mode 0440, effective UID, companion file digests, and immediate-child workspace/output placement; `WorkerReceipt::validate_against(request)` verifies candidate/digest binding, exit zero, completion in `[issued_at, deadline]`, `usage.fits(request.budget)`, and the reported untrusted worker HEAD. The coordinator's later squash produces the distinct authoritative `candidate_digest`. Deserialization alone must not claim those external checks.

- [ ] **Step 4: Implement the hidden worker command under the worker identity**

Add dependency `nix` with `user` feature on Unix. Hidden CLI:

```text
gzmo-evolver worker --request /run/gzmo-evolver/<candidate-id>/request.json
```

It verifies effective UID equals `expected_uid`, request owner/mode/path, manifest/policy companion digests, workspace branch/baseline, and output directory before spawning OMP. It refuses to run as root or coordinator UID.

- [ ] **Step 5: Construct fixed OMP argv in code**

The request may choose executable/profile but no arbitrary args. Build exactly:

```text
<omp> -p --mode json --no-session
  --profile <profile>
  --cwd <independent-workspace>
  --max-time <budget.wall_seconds>s
  --approval-mode yolo
  --no-extensions --no-skills --no-rules
  --tools read,bash,edit,write,grep,glob,lsp
  @<rendered-mission.md>
```

Clear the environment; set only fixed safe PATH, dedicated HOME, locale, Git identity/config guards, and loopback local-model profile variables required by the installed profile. No provider/API/Git/SSH/proxy credential survives.

- [ ] **Step 6: Render the bounded mission outside the worker**

The coordinator writes a read-only rendered mission that includes exact candidate/baseline, independent workspace, protected paths, required gates, signed budgets, no remote/main/credential/policy/evaluator changes, commit required, and stop-after-one-candidate. Append the untrusted approved mission Markdown under an explicit data delimiter; it cannot override the preceding policy.

- [ ] **Step 7: Enforce limits and parse OMP JSON fail-closed**

`SystemdWorkerLauncher` starts only `gzmo-evolver-worker@<candidate-id>.service`, waits for its terminal state/receipt, and on deadline requests stop then kill of that unit cgroup. Unit-level cgroups enforce RSS/PIDs/wall/disk namespace; the launcher caps stdout/stderr bytes. Parse OMP JSON for tool-call/input/output-token usage; if the installed OMP format lacks required counters, mark usage unavailable and fail the candidate rather than treating zero as measured. After completion compute changed-file/line counts and candidate HEAD, fill `ResourceUsage`, and require `fits`.

Unit files and live OS identities are installed by the operations plan. Here, `FakeWorkerLauncher` and a fake worker fixture prove the contract; no live systemd/OMP service is required.

- [ ] **Step 8: Require a verified receipt before accepting the candidate**

Store raw bounded OMP output under worker-only output; receipt contains only sanitized summary hashes and usage. The coordinator independently validates receipt, candidate workspace branch/HEAD, manifest/policy digests, and budget before allowing Task 6 to record the candidate digest.

- [ ] **Step 9: Run focused worker tests**

Run: `cargo test -p gzmo-evolver worker -- --nocapture`

Expected: PASS for fake launcher/worker; request mutation, wrong UID/mode/path, forbidden env, timeout/kill, nonzero, malformed or missing usage, output cap, over-budget use, dirty workspace, and missing commit fail closed.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock gzmo-evolver
git commit -m "feat: run bounded OMP candidate workers"
```

---
### Task 6: Orchestrate Refresh → Prepare → Build → Evaluate Boundary

**Files:**
- Create: `gzmo-evolver/src/runner.rs`
- Modify: `gzmo-evolver/src/lib.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Modify: `gzmo-evolver/tests/repo_loop.rs`

**Interfaces:**
- Produces: `RepoEvolver::run_once()` / `resume()` ending at `CandidateState::Evaluating`, plus `abort()` and structured status.
- Consumes: coordinator lock/state, mission, trusted policy, Git repository/independent workspace, worker launcher/request/receipt.

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

`run_once` holds `CoordinatorLock` for the complete mutation. With no active candidate:

1. refresh and validate mission;
2. refresh mirror and resolve exact remote baseline;
3. read `config/repo-evolver.policy.toml` from that baseline, parse/validate it, and verify target config;
4. create `PreparedCandidate` and atomically insert `Observed` + audit;
5. create independent workspace and transition `Observed → Prepared` with immutable workspace metadata;
6. render/seal WorkerRequest and transition `Prepared → Building` before launch;
7. launch/wait for worker; independently validate receipt/workspace, squash candidate, check bounded diff;
8. transition `Building → Evaluating` with immutable `git-sha1:` candidate digest and receipt digest;
9. stop. This plan never evaluates quality, pushes, opens a PR, or advances past Evaluating.

Every transition is state+audit atomic. A failure before candidate creation returns an error without state. A failure after creation transitions to `Failed` with a bounded reason when legal; if the database transition itself fails, preserve artifacts and return a recovery-required error.

- [ ] **Step 4: Implement idempotent resume by persisted state**

- `Observed`: recreate/verify baseline and policy, then prepare one workspace.
- `Prepared`: verify workspace/baseline/branch/digests, seal request, transition to Building, launch.
- `Building`: if a valid receipt exists, continue verification/squash; if the worker unit is active, wait within remaining deadline; otherwise transition Failed with `worker_lost_without_receipt`.
- `Evaluating`: return the existing outcome unchanged; the next plan owns evaluation.
- terminal states: return unchanged and create nothing.
- `ReviewReady|PromotionPending|Soaking`: refuse because those states belong to a later installed stage, not this runner version.

Never rerun a completed worker or overwrite workspace/policy/candidate digests. Revalidate audit chain before any resume action.

- [ ] **Step 5: Implement explicit abort without deletion**

`abort <candidate-id> --reason <text>` acquires the lock, validates audit/state, transitions any legal nonterminal pre-evaluation state to Failed, requests worker-unit stop if Building, and preserves workspace/request/receipt/raw output. `abort` cannot alter Evaluating or later states in this subplan; future plan supplies review lifecycle controls. Cleanup remains a separate trusted method with terminal checks.

- [ ] **Step 6: Implement one structured status model**

`gzmo.repo_evolver.status/v1` includes repository, mission/candidate ID, state, baseline/candidate/policy/manifest/receipt digests, budget max/used/remaining, workspace, worker state/deadline, last audit sequence/hash, terminal reason, and exact next allowed action. Human output derives from this struct. Use algorithm-qualified digest field names; do not expose raw logs, mission body, environment, or credentials.

Add public CLI commands `run`, `resume`, and `abort`; retain `config-check`, `status`, `refresh`, `prepare`, and hidden `worker`. Every command supports `--json` where output exists. No placeholder command remains.

- [ ] **Step 7: Run all runner tests**

Run: `cargo test -p gzmo-evolver --all-targets`

Expected: PASS for exact audit sequence, each crash boundary/resume path, worker-lost/receipt paths, repeated run idempotence, terminal handling, abort preservation, lock race, and no remote/trusted-checkout mutation.

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
- Consumes: all connected runner interfaces with fake process/worker launchers and real local Git repositories.

- [ ] **Step 1: Add the complete failure matrix**

Use real temporary bare Git origins/mirrors/independent clones and deterministic fake mission/worker processes. Cover:

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

Expected: all pass. Tests may use local `file://`/filesystem Git only; no GitHub, public network, provider credentials, system services, or actual OMP process.

- [ ] **Step 3: Run the hermetic vertical smoke**

Run the fixture integration test by exact name:

```bash
cargo test -p gzmo-evolver --test repo_loop fixture_run_reaches_evaluating_without_remote_mutation -- --nocapture
```

Expected: exactly one candidate reaches `evaluating`; its independent clone has one normalized candidate commit; trusted and remote main are unchanged; remote refs contain only main; receipt exposes no forbidden environment name; audit chain has Observed/Prepared/Building/Evaluating.

The real OMP/systemd smoke is deliberately deferred to `2026-09-01-continuous-evolution-operations.md` after coordinator/worker/model system identities and the private model namespace exist. This subplan must not weaken the trust model to manufacture a live smoke.

- [ ] **Step 4: Commit**

```bash
git add gzmo-evolver
git commit -m "test: prove isolated repository candidate generation"
```
