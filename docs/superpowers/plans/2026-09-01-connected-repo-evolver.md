# Connected Repository Evolver Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing Opportunity Discovery output into one isolated, budgeted repository candidate at a time, without allowing the worker to push, merge, modify `main`, or access GitHub credentials.

**Architecture:** A new `gzmo-evolver` binary is the trusted connected-host coordinator. It reads the existing `next-mission.json/md`, persists candidate state in a local SQLite database outside the repository, creates a detached Git worktree, invokes OMP as an uncredentialed worker, records the candidate commit and audit chain, and stops at `Evaluating`. Remote push/PR behavior belongs to the next plan.

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
- Worktree push URL is the literal disabled URL `no-push://candidate-worker`; only the later trusted GitHub adapter may push.
- Worker may edit only its worktree. Protected paths and resource budgets come from the signed policy artifact.
- No direct main update, PR creation, merge, release, repository setting, or visibility mutation in this plan.
- A crash is resumable from state; it must not create a second candidate.

## File Structure

| Path | Responsibility |
|---|---|
| `gzmo-evolver/Cargo.toml` | Connected runner crate |
| `gzmo-evolver/src/main.rs` | CLI only |
| `gzmo-evolver/src/config.rs` | Typed config and safe defaults |
| `gzmo-evolver/src/state.rs` | Local coordinator SQLite and audit persistence |
| `gzmo-evolver/src/mission.rs` | Existing Opportunity artifact adapter |
| `gzmo-evolver/src/git.rs` | Trusted Git/worktree operations |
| `gzmo-evolver/src/worker.rs` | OMP process invocation and environment isolation |
| `gzmo-evolver/src/runner.rs` | Candidate state orchestration |
| `gzmo-evolver/src/process.rs` | Internal `ProcessRunner` seam (real + test fake) |
| `gzmo-evolver/tests/repo_loop.rs` | Hermetic bare-remote/worktree vertical slice |
| `gzmo-evolver/tests/fixtures/fake-worker.sh` | Deterministic candidate edit/commit |
| `config/repo-evolver.toml.example` | Connected-host configuration |

---

### Task 1: Bootstrap `gzmo-evolver` with Fail-Closed Configuration

**Files:**
- Modify: `Cargo.toml`
- Create: `gzmo-evolver/Cargo.toml`
- Create: `gzmo-evolver/src/main.rs`
- Create: `gzmo-evolver/src/config.rs`
- Create: `config/repo-evolver.toml.example`
- Test: `gzmo-evolver/src/config.rs`

**Interfaces:**
- Consumes: `evolution_contracts::{ResourceBudget, PathPolicy}`.
- Produces: `RepoEvolverConfig::load(path)`, normalized repository/state/mission paths, worker argv.

- [ ] **Step 1: Write configuration rejection tests**

```rust
#[test]
fn rejects_repo_state_inside_worktree_and_shell_worker() {
    let root = tempdir().unwrap();
    let cfg = fixture_config(root.path());
    assert!(cfg.validate().is_ok());

    let mut bad_state = cfg.clone();
    bad_state.state_dir = root.path().join("repo/data-next/evolver");
    assert!(bad_state.validate().is_err());

    let mut bad_worker = cfg;
    bad_worker.worker.argv = vec!["bash".into(), "-c".into(), "omp ...".into()];
    assert!(bad_worker.validate().is_err());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver config`

Expected: FAIL: package not found.

- [ ] **Step 3: Add workspace member and dependencies**

Add `gzmo-evolver` to workspace members. Add workspace dependency:

```toml
clap = { version = "4", features = ["derive"] }
```

`gzmo-evolver` depends on `evolution-contracts`, `tokio`, `serde`, `serde_json`, `toml`, `chrono`, `uuid`, `rusqlite`, `fs2`, `sha2`, `anyhow`, `thiserror`, `tracing`, `tracing-subscriber`, and `clap`. Add `tempfile = "3"` as a dev-dependency.

- [ ] **Step 4: Define typed configuration**

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RepoEvolverConfig {
    pub repo: RepoConfig,
    pub state_dir: PathBuf,
    pub mission: MissionConfig,
    pub worker: WorkerConfig,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    pub path: PathBuf,
    pub remote: String,
    pub base_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub argv: Vec<String>,
    pub profile: String,
    pub max_time: String,
}
```

Validation requires absolute canonical repo/state/policy/worker paths, `.git` exists, state directory is outside repo, mission paths resolve under repo `data-next/opportunity-discovery`, worker argv starts with the pinned absolute OMP executable and contains no shell interpreter/`-c`, policy exists and matches its digest, and budget validates.

- [ ] **Step 5: Write the concrete example**

```toml
state_dir = "/var/lib/gzmo-evolver/coordinator"

[repo]
path = "/srv/gzmo/GZMO"
remote = "origin"
base_branch = "main"

[mission]
json = "data-next/opportunity-discovery/next-mission.json"
markdown = "data-next/opportunity-discovery/next-mission.md"
refresh_argv = ["bash", "scripts/opportunity-next-mission.sh"]

[worker]
argv = ["/usr/local/bin/omp", "-p", "--mode", "json", "--no-session", "--approval-mode", "yolo", "--tools", "read,bash,edit,write,grep,glob,lsp"]
profile = "gzmo-repo-evolver-worker"
max_time = "45m"

[policy]
path = "/etc/gzmo/repo-evolver.policy.toml"
```

The installer expands operator-provided paths once and writes absolute paths; runtime never relies on shell `~` expansion. Fixture configs use temporary absolute paths.

- [ ] **Step 6: Implement CLI skeleton**

Use Clap subcommands `refresh`, `prepare`, `run`, `resume`, `status`, and `abort`. Add a hidden `worker --request <absolute-path>` entry used only by the fixed worker system unit; it validates a sealed request before invoking OMP. Each public command prints JSON when `--json` is passed. `run` stops before evaluation in this plan.

- [ ] **Step 7: Run config tests**

Run: `cargo test -p gzmo-evolver config`

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock gzmo-evolver config/repo-evolver.toml.example
git commit -m "feat: bootstrap connected repository evolver"
```

---

### Task 2: Persist One Candidate and Its Audit Chain

**Files:**
- Create: `gzmo-evolver/src/state.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Test: `gzmo-evolver/src/state.rs`

**Interfaces:**
- Produces: `StateStore::{open,create_candidate,transition,active_candidate,append_audit}`.
- Consumes: `CandidateManifest`, `CandidateState`, `AuditEvent`.

- [ ] **Step 1: Write one-active-candidate tests**

```rust
#[test]
fn repository_allows_only_one_nonterminal_candidate() {
    let store = StateStore::open_in_memory().unwrap();
    store.create_candidate(&manifest("cand-20260901t070000z-one-aaaa1111")).unwrap();
    assert!(store.create_candidate(&manifest("cand-20260901t080000z-two-bbbb2222")).is_err());
    store.transition(&id("cand-20260901t070000z-one-aaaa1111"), CandidateState::Rejected, "gate failed").unwrap();
    assert!(store.create_candidate(&manifest("cand-20260901t080000z-two-bbbb2222")).is_ok());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver one_nonterminal`

Expected: FAIL: `StateStore` missing.

- [ ] **Step 3: Create SQLite schema**

```sql
CREATE TABLE candidates (
  id TEXT PRIMARY KEY,
  repository TEXT NOT NULL,
  manifest_json TEXT NOT NULL,
  state TEXT NOT NULL,
  worktree TEXT,
  candidate_commit TEXT,
  terminal_reason TEXT,
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

Enable WAL, foreign keys, `busy_timeout=5000`, and an exclusive process flock on `<state_dir>/runner.lock` before writes.

- [ ] **Step 4: Enforce transitions transactionally**

Load current state, call `can_transition_to`, update candidate, append audit event, and commit in one transaction. A crash cannot update state without audit or vice versa.

- [ ] **Step 5: Add `resume` semantics**

`Prepared` resumes worktree verification; `Building` checks worker PID/receipt then either continues or marks `Failed`; `Evaluating` is reserved for the next plan. Terminal states never resume.

- [ ] **Step 6: Run state tests**

Run: `cargo test -p gzmo-evolver state`

Expected: PASS including invalid transition and tampered-chain rejection.

- [ ] **Step 7: Commit**

```bash
git add gzmo-evolver/src
git commit -m "feat: persist one audited evolution candidate"
```

---

### Task 3: Adapt the Existing Opportunity Mission

**Files:**
- Create: `gzmo-evolver/src/mission.rs`
- Create: `gzmo-evolver/tests/fixtures/next-mission.json`
- Create: `gzmo-evolver/tests/fixtures/next-mission.md`
- Test: `gzmo-evolver/src/mission.rs`

**Interfaces:**
- Produces: `MissionAdapter::refresh_and_load() -> Mission` and `Mission::to_manifest`.
- Consumes: `gzmo.opportunity.next_mission/v1` and policy budget.

- [ ] **Step 1: Write malformed and multiple-mission tests**

```rust
#[test]
fn accepts_only_one_active_ship_bar_mission() {
    let mission = load_fixture("next-mission.json").unwrap();
    assert_eq!(mission.schema, "gzmo.opportunity.next_mission/v1");
    assert!(mission.ok && mission.ship_bar);
    assert_eq!(mission.bet_id, "felt-use-mass-growth");
}

#[test]
fn rejects_hold_or_missing_markdown() {
    assert!(load_hold_fixture().is_err());
    assert!(MissionAdapter::new(missing_md()).load().is_err());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver mission`

Expected: FAIL: mission adapter missing.

- [ ] **Step 3: Define the strict input struct**

```rust
#[derive(Deserialize)]
struct NextMissionV1 {
    schema: String,
    ok: bool,
    bet_id: String,
    title: String,
    score: i64,
    ship_bar: bool,
    mission_md: PathBuf,
}
```

Reject schema mismatch, `ok=false`, `ship_bar=false`, blank IDs/titles, path escape, or markdown without `## Mission`, `## Constraints`, and `## Verify`.

- [ ] **Step 4: Refresh via argv without a shell**

Invoke the exact configured argv (`bash`, `scripts/opportunity-next-mission.sh`) through `ProcessRunner`. Require exit zero and both artifacts newer than refresh start.

- [ ] **Step 5: Build the candidate manifest**

Read `git rev-parse <remote>/<base_branch>` as the commit; create lowercase ID `cand-20260901t070000z-<sanitized-bet-id>-<first-8-commit>`; set `baseline_digest = git-sha1:<commit>` and `CandidateTarget::Repository { owner, repository, base_branch, candidate_branch: evolve/<id> }`; copy gates/budget/protected paths from policy, not mission prose.

- [ ] **Step 6: Run mission tests**

Run: `cargo test -p gzmo-evolver mission`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: convert active opportunity into candidate manifest"
```

---

### Task 4: Create a Safe Git Worktree Without Push Authority

**Files:**
- Create: `gzmo-evolver/src/process.rs`
- Create: `gzmo-evolver/src/git.rs`
- Test: `gzmo-evolver/tests/repo_loop.rs`

**Interfaces:**
- Produces: `GitWorkspace::{prepare,candidate_commit,diff_stats,cleanup}`.
- Consumes: trusted repo path, exact base commit, candidate branch, external process runner.

- [ ] **Step 1: Write a bare-remote worktree test**

Create a temp bare remote and seed repo with `main`. Assert:

```rust
let ws = git.prepare(&manifest).unwrap();
let (branch, baseline) = manifest.repository_git_refs().unwrap();
assert_eq!(git.current_branch(&ws).unwrap(), branch);
assert_eq!(git.merge_base(&ws, "HEAD", baseline).unwrap(), baseline);
assert_eq!(git.push_url(&ws, "origin").unwrap(), "no-push://candidate-worker");
assert!(!git.main_head_changed().unwrap());
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test repo_loop prepare_worktree`

Expected: FAIL: Git workspace missing.

- [ ] **Step 3: Implement explicit Git argv**

Run only argument-vector commands:

```text
git fetch --prune origin main
git worktree add --detach <worktree> <baseline_commit>
git -C <worktree> switch -c evolve/<candidate-id>
git -C <worktree> remote set-url --push origin no-push://candidate-worker
```

Refuse dirty source repo, existing candidate branch/worktree, symbolic base commit, submodule changes, or base not reachable from `origin/main`.

- [ ] **Step 4: Validate post-worker candidate commit**

Require exactly one branch head descended from baseline, no merge commits, author recorded as `GZMO Evolver Candidate`, and clean worktree. Worker may make multiple local commits; squash them through trusted Git into one candidate commit before evaluation.

- [ ] **Step 5: Implement safe cleanup**

Only remove a worktree whose path is under `<state_dir>/worktrees` and whose HEAD equals the recorded candidate commit. Never delete the branch after `ReviewReady`; next plan owns remote lifecycle.

- [ ] **Step 6: Run Git tests**

Run: `cargo test -p gzmo-evolver --test repo_loop git`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: isolate candidate work in no-push worktrees"
```

---

### Task 5: Invoke OMP as an Uncredentialed Worker

**Files:**
- Create: `gzmo-evolver/src/worker.rs`
- Create: `gzmo-evolver/tests/fixtures/fake-worker.sh`
- Test: `gzmo-evolver/src/worker.rs`
- Test: `gzmo-evolver/tests/repo_loop.rs`

**Interfaces:**
- Produces: trusted `WorkerLauncher::run`, worker-side `run_worker_request`, and `WorkerReceipt` with output digest, exit, duration, and budget usage.
- Consumes: a sealed `WorkerRequest`, candidate worktree, rendered mission, manifest/policy digests, and pinned local OMP profile.

- [ ] **Step 1: Write environment-isolation tests**

The fake worker writes names (not values) of visible credential variables to a receipt. Assert none of these exist:

```rust
const FORBIDDEN_ENV: &[&str] = &[
    "GH_TOKEN", "GITHUB_TOKEN", "COPILOT_GITHUB_TOKEN",
    "SSH_AUTH_SOCK", "AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY",
    "OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GEMINI_API_KEY",
];
```

Also assert `HOME` is the dedicated worker home, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_TERMINAL_PROMPT=0`, and `GIT_ASKPASS` is unset.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver worker_environment`

Expected: FAIL: worker missing.

- [ ] **Step 3: Implement the sealed worker request**

The coordinator writes `/run/gzmo-evolver/<candidate-id>/request.json` before starting the unit. The directory is coordinator-owned, the JSON is read-only to the worker, and the separate output directory is worker-writable. `WorkerRequest` contains only candidate ID, manifest/policy digests, absolute worktree/mission/output paths, OMP argv, deadline, and budget. Worker-side validation recomputes digests, rejects path escape/symlinks, requires every writable path under the recorded candidate roots, and refuses any unknown field or environment override.

- [ ] **Step 4: Launch concrete OMP argv under the worker identity**

The coordinator starts only `gzmo-evolver-worker@<candidate-id>.service`; its fixed `ExecStart` calls `gzmo-evolver worker --request /run/gzmo-evolver/%i/request.json` as OS user `gzmo-evolver-worker`. That worker entry executes:

```text
omp -p --mode json --no-session
  --profile gzmo-repo-evolver-worker
  --cwd <candidate-worktree>
  --max-time <signed budget>
  --approval-mode yolo
  --tools read,bash,edit,write,grep,glob,lsp
  @<rendered-mission.md>
```

Use OMP v18 or newer. The dedicated worker profile points only to the qualified local code-model endpoint inside a private network namespace shared with a read-only local model service. No provider/API credential exists in the worker environment or profile. A root-owned policy permits the coordinator to start only the fixed worker unit with a validated candidate ID. Verify identity, zero egress, model locality, sealed-request permissions, and credential isolation in the install smoke before enabling cadence.

- [ ] **Step 5: Render a bounded worker prompt**

Append machine-generated instructions: exact baseline/branch, allowed roots, protected paths, required gates, budgets, no remote operations, no main switch, no credential search, no workflow/ADR/spec edits, commit required, and stop after one candidate. Include the approved mission text verbatim below the policy.

- [ ] **Step 6: Enforce process limits**

`WorkerLauncher` enforces wall/RSS/PID/output limits through the worker unit plus a Tokio deadline in the coordinator. On timeout it requests graceful unit stop, then hard-kills the unit cgroup. Sample output bytes, changed-file count, and process RSS; any hard budget breach fails the candidate.

- [ ] **Step 7: Require a candidate receipt**

OMP JSON must exit zero and the worktree must contain a commit. Store only sanitized summary, output SHA-256, duration, and budget use; do not persist model credentials or raw hidden reasoning. The coordinator verifies receipt and candidate HEAD before accepting it.

- [ ] **Step 8: Run worker tests**

Run: `cargo test -p gzmo-evolver worker -- --nocapture`

Expected: PASS for fake worker; modified request, path escape, wrong OS identity, timeout, nonzero, dirty tree, and missing commit all fail closed.

- [ ] **Step 9: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: run bounded OMP candidate workers"
```

---

### Task 6: Orchestrate Refresh → Prepare → Build → Evaluate Boundary

**Files:**
- Create: `gzmo-evolver/src/runner.rs`
- Modify: `gzmo-evolver/src/main.rs`
- Modify: `gzmo-evolver/tests/repo_loop.rs`

**Interfaces:**
- Produces: `RepoEvolver::run_once()` ending in `CandidateState::Evaluating` with a recorded commit.
- Consumes: state, mission, Git workspace, worker.

- [ ] **Step 1: Write the vertical state test**

```rust
#[tokio::test]
async fn one_run_stops_at_evaluation_boundary() {
    let harness = RepoHarness::new().await;
    let outcome = harness.evolver.run_once().await.unwrap();
    assert_eq!(outcome.state, CandidateState::Evaluating);
    assert!(outcome.candidate_commit.is_some());
    assert_eq!(harness.remote_main(), harness.initial_main());
    assert!(harness.remote_branches().iter().all(|b| b == "main"));
    assert!(harness.store.verify_audit_chain().is_ok());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test repo_loop one_run_stops`

Expected: FAIL: coordinator missing.

- [ ] **Step 3: Implement the state sequence**

`run_once` acquires runner lock, refuses an active candidate, refreshes mission, creates manifest/audit, prepares worktree, transitions to Building, invokes worker, normalizes candidate commit, records budget/diff receipt, and transitions to Evaluating. Every transition is transaction+audit.

- [ ] **Step 4: Implement idempotent `resume` and `abort`**

`resume` continues only from a persisted nonterminal state after verifying every recorded digest and worktree HEAD. `abort` marks Failed and preserves worktree/receipt for inspection; cleanup is a separate explicit command.

- [ ] **Step 5: Implement status output**

JSON includes schema, repository, active mission/candidate, state, baseline/candidate commits, budgets used/remaining, last audit hash, worktree, terminal reason, and next allowed action. Human output derives from the JSON struct.

- [ ] **Step 6: Run all runner tests**

Run: `cargo test -p gzmo-evolver --all-targets`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: orchestrate one repository candidate at a time"
```

---

### Task 7: Prove Stage 1 Candidate Generation End to End

**Files:**
- Modify: `gzmo-evolver/tests/repo_loop.rs`
- Create: `gzmo-evolver/tests/fixtures/fixture-repo/README.md`
- Create: `gzmo-evolver/tests/fixtures/fixture-repo/research/opportunities/fixture.md`

**Interfaces:**
- Produces: a hermetic acceptance test for Stage 1 before any GitHub adapter exists.
- Consumes: all runner interfaces.

- [ ] **Step 1: Add failure matrix tests**

Cover: dirty source repo, stale base, two active missions, active candidate lock, worker timeout, worker sees forbidden env name, protected path, changed-file/line cap, merge commit, no commit, audit tamper, crash after worktree creation, and resume.

- [ ] **Step 2: Run the acceptance suite**

```bash
cargo fmt --all -- --check
cargo clippy -p gzmo-evolver --all-targets -- -D warnings
cargo test -p gzmo-evolver --all-targets -- --nocapture
```

Expected: all pass.

- [ ] **Step 3: Run a real OMP stop-before-evaluation smoke**

Use a disposable fixture repository, not GZMO:

```bash
GZMO_EVOLVER_CONFIG=config/repo-evolver.fixture.toml \
  cargo run -p gzmo-evolver -- run --json
```

Expected: state `evaluating`, one local candidate commit, no remote candidate branch, unchanged remote main, no GitHub token in receipt.

- [ ] **Step 4: Commit**

```bash
git add gzmo-evolver
git commit -m "test: prove isolated repository candidate generation"
```
