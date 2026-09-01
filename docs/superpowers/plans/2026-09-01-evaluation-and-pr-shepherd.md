# Evolution Evaluation and Pull-Request Shepherd Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Evaluate repository candidates against immutable hard floors, then let a least-privileged trusted GitHub App push only approved `evolve/*` branches, open one transparent PR, and perform at most two evidence-bounded repairs without ever merging.

**Architecture:** Evaluation runs outside the candidate worktree using policy and holdouts from the trusted base commit. A passing `EvaluationReport` moves the candidate to `ReviewReady`. Only then does the trusted GitHub adapter mint a short-lived installation token, restore a push URL, push the exact evaluated commit, publish a check run, and create/update the PR. CI or explicit review feedback may trigger a new isolated worker attempt; each new commit is fully reevaluated.

**Tech Stack:** Rust, evolution-contracts, Git CLI, Reqwest, GitHub App JWT/installation tokens, wiremock, Serde/TOML, SHA-256, existing cargo and shell quality gates.

**Spec:** `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`

## Global Constraints

- Candidate workers never receive the GitHub App private key, installation token, credential helper, SSH agent, or trusted policy/holdout write access.
- Trusted policy is loaded from the baseline commit or an operator-owned path, never from candidate HEAD.
- A protected-path change is a hard failure, even if tests pass.
- Gate commands are argument arrays selected by trusted policy; mission text cannot inject commands or shell syntax.
- Candidate changes to tests do not replace external holdouts. Both candidate tests and immutable holdouts run.
- Evaluation hard floors use AND semantics. Metrics never compensate for failure/unavailability.
- The trusted adapter pushes only the recorded candidate commit to its exact `evolve/<candidate-id>` branch with lease; never `main`, tags, releases, repository settings, or other branches.
- The runner never merges or enables auto-merge. Human merge remains the Stage 1 promotion signature.
- Repairs are capped at two and require a failed CI check owned by the candidate PR or an explicit `evolve:revise` label plus human review text.
- Public-repository fork code never runs on the connected self-hosted runner. Cadence is local systemd, not `pull_request` on a self-hosted GitHub Actions runner.

## File Structure

| Path | Responsibility |
|---|---|
| `config/repo-evolver.policy.toml.example` | Gate, budget, path, branch, and repair policy |
| `gzmo-evolver/src/policy.rs` | Load trusted policy from base/operator path |
| `gzmo-evolver/src/diff_gate.rs` | Path, symlink, submodule, size, line, binary checks |
| `gzmo-evolver/src/evaluator.rs` | Run trusted gates and comparative holdouts |
| `gzmo-evolver/src/redact.rs` | Remove credentials and sensitive environment data from reports |
| `gzmo-evolver/src/github/auth.rs` | GitHub App JWT and installation-token lifecycle |
| `gzmo-evolver/src/github/client.rs` | Typed REST operations with strict owner/repo/branch checks |
| `gzmo-evolver/src/github/publish.rs` | Push exact commit, create check and PR |
| `gzmo-evolver/src/shepherd.rs` | CI/review status and capped repair attempts |
| `gzmo-evolver/tests/evaluator.rs` | Hard-floor and immutable-holdout tests |
| `gzmo-evolver/tests/github.rs` | Mock GitHub auth/API and branch safety tests |
| `gzmo-evolver/tests/shepherd.rs` | Repair state-machine tests |
| `tests/evolution-holdouts/holdout-gate.sh` | Protected behavior gate mounted from trusted base |
| `.github/ISSUE_TEMPLATE/autonomous-opportunity.yml` | Human-created mission input option |
| `.github/pull_request_template.md` | Candidate/evaluation/audit fields |

---

### Task 1: Freeze Trusted Evaluation Policy

**Files:**
- Create: `config/repo-evolver.policy.toml.example`
- Create: `gzmo-evolver/src/policy.rs`
- Create: `gzmo-evolver/src/diff_gate.rs`
- Test: `gzmo-evolver/tests/evaluator.rs`

**Interfaces:**
- Produces: `TrustedPolicy`, `DiffGate::evaluate`, policy digest.
- Consumes: `CapabilityEnvelope`, candidate manifest, baseline commit.

- [ ] **Step 1: Write protected-path and budget tests**

```rust
#[test]
fn protected_or_oversized_diff_fails_before_commands_run() {
    let policy = fixture_policy();
    assert!(DiffGate::evaluate(&policy, &[change("src/lib.rs", 20, 2)]).is_ok());
    assert!(DiffGate::evaluate(&policy, &[change(".github/workflows/ci.yml", 1, 1)]).is_err());
    assert!(DiffGate::evaluate(&policy, &[change("src/generated.bin", 1, 0).binary()]).is_err());
    assert!(DiffGate::evaluate(&policy, &[change("src/a.rs", 1601, 0)]).is_err());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test evaluator protected_or_oversized`

Expected: FAIL: policy/diff gate missing.

- [ ] **Step 3: Define concrete default policy**

```toml
schema = "gzmo.repo_evolver.policy/v1"
max_active_candidates = 1
max_repair_attempts = 2
allowed_branch_prefix = "evolve/"

[budget]
wall_seconds = 2700
max_attempts = 1
max_changed_files = 20
max_added_lines = 1500
max_tool_calls = 80
max_input_tokens = 250000
max_output_tokens = 50000

protected_paths = [
  ".github/workflows/",
  "docs/superpowers/specs/",
  "docs/ADR-",
  "AGENTS.md",
  "Cargo.toml",
  "Cargo.lock",
  "crates/evolution-contracts/",
  "gzmo-evolver/",
  "tests/evolution-holdouts/",
]

[[gates]]
name = "format"
class = "hard_floor"
argv = ["cargo", "fmt", "--all", "--", "--check"]
timeout_seconds = 300

[[gates]]
name = "clippy"
class = "hard_floor"
argv = ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"]
timeout_seconds = 1200

[[gates]]
name = "tests"
class = "hard_floor"
argv = ["cargo", "test", "--all"]
timeout_seconds = 1800

[[gates]]
name = "opportunity-contract"
class = "hard_floor"
argv = ["bash", "scripts/opportunity-discovery-check.sh"]
timeout_seconds = 300
```

- [ ] **Step 4: Load policy from a trusted source**

For a candidate based on commit `B`, read policy with `git show B:config/repo-evolver.policy.toml`; compare SHA-256 with the manifest policy digest. An operator path outside the repo may override only if its detached signature verifies in the later trusted signer module. Candidate HEAD is never the source.

- [ ] **Step 5: Implement diff inspection**

Use `git diff --raw -z`, `git diff --numstat -z`, and `git diff --check <base>..<candidate>`. Reject protected paths, absolute/parent paths, symlink targets escaping root, submodule/gitlink entries, binaries, files over 2 MiB, more than budgeted files/lines, whitespace errors, generated secrets, and changes outside the worktree.

- [ ] **Step 6: Run policy/diff tests**

Run: `cargo test -p gzmo-evolver --test evaluator diff_gate`

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add config/repo-evolver.policy.toml.example gzmo-evolver
git commit -m "feat: gate candidate diffs with trusted policy"
```

---

### Task 2: Run Hard Gates and Immutable Holdouts

**Files:**
- Create: `gzmo-evolver/src/evaluator.rs`
- Create: `tests/evolution-holdouts/holdout-gate.sh`
- Modify: `gzmo-evolver/tests/evaluator.rs`

**Interfaces:**
- Produces: `Evaluator::evaluate(manifest, commit) -> EvaluationReport`.
- Consumes: trusted policy, exact candidate commit, process runner, external holdout directory.

- [ ] **Step 1: Write non-compensation and holdout tests**

```rust
#[tokio::test]
async fn metric_gain_cannot_override_failed_holdout() {
    let eval = harness()
        .gate_pass("tests")
        .gate_fail("holdout.memory_lifecycle", "assertion failed")
        .metric("throughput_gain_pct", 200.0)
        .run().await;
    assert!(!eval.hard_floors_passed);
    assert_eq!(eval.gates.iter().find(|g| g.name == "holdout.memory_lifecycle").unwrap().status,
               GateStatus::Fail);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test evaluator metric_gain_cannot`

Expected: FAIL: evaluator missing.

- [ ] **Step 3: Build an immutable evaluation checkout**

Create two read-only checkouts under state: baseline and candidate. Copy `tests/evolution-holdouts` from baseline into a separate read-only directory. Execute candidate tests in candidate checkout, then run the baseline-owned `holdout-gate.sh` with explicit candidate binary/library paths. The candidate cannot edit the holdout copy or choose its arguments.

- [ ] **Step 4: Run commands without shell interpolation**

For each policy gate, invoke exact argv, clean sensitive env, cap stdout/stderr bytes, enforce timeout/process-tree kill, and record exit/duration/output digest. A missing binary, timeout, truncation before verdict, or infrastructure error is `Unavailable` and fails a hard floor.

- [ ] **Step 5: Add initial holdouts**

Protect observable contracts already present in GZMO:

- one-writer lock refuses a second owner;
- honeypot lifecycle never recalls superseded facts;
- evidence/faithfulness floor remains enforced;
- opportunity discovery permits exactly one active bet;
- workflow-skill and immune apply remain explicit-pin operations;
- context pruning keeps workflow contracts/tool-call integrity;
- no cloud requirement is introduced into core health.

`holdout-gate.sh` invokes exact existing test functions/commands where possible and adds fixture probes only for uncovered behavior. It emits structured `gzmo.evolution.holdout/v1` JSON; missing or malformed output is a hard failure.

- [ ] **Step 6: Add comparative metrics**

Run baseline and candidate for mission-declared benchmarks from an allowlist. Report delta plus raw values. Never gate on external folklore percentages. The policy owns exact floors.

- [ ] **Step 7: Transition atomically**

After report digest is stored: `Evaluating → Rejected` when any hard floor fails; otherwise `Evaluating → ReviewReady`. Append audit in the same state transaction.

- [ ] **Step 8: Run evaluator tests**

Run: `cargo test -p gzmo-evolver evaluator -- --nocapture`

Expected: PASS including timeout, unavailable, output cap, modified candidate tests, immutable holdout, and metric non-compensation.

- [ ] **Step 9: Commit**

```bash
git add gzmo-evolver tests/evolution-holdouts
git commit -m "feat: evaluate candidates against immutable hard floors"
```

---

### Task 3: Redact and Publish a Human-Readable Evaluation

**Files:**
- Create: `gzmo-evolver/src/redact.rs`
- Modify: `gzmo-evolver/src/evaluator.rs`
- Test: `gzmo-evolver/tests/evaluator.rs`

**Interfaces:**
- Produces: sanitized Markdown report and JSON digest; raw logs remain local with mode 0600.
- Consumes: process output and structured evaluation.

- [ ] **Step 1: Write secret-redaction tests**

```rust
#[test]
fn report_never_contains_known_or_assignment_secrets() {
    let raw = ["GITHUB_TOKEN=ghp", "_synthetic-test-value"].concat();
    let clean = redact(&raw);
    assert!(!clean.contains("ghp_"));
    assert!(clean.contains("GITHUB_TOKEN=<redacted>"));
}
```

Also cover bearer headers, common provider prefixes, private-key blocks, authorization URLs, and configured literal deny fingerprints without writing real tokens to fixtures.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver redaction`

Expected: FAIL: redactor missing.

- [ ] **Step 3: Implement structured-first reporting**

Publish only gate names/status/duration, output digest, bounded redacted excerpt, metrics, diff stats, artifact digests, candidate/base commits, and audit head. Never publish full environment, raw command line secrets, hidden reasoning, or private holdout content.

- [ ] **Step 4: Store raw logs safely**

Write under `<state_dir>/candidates/<id>/raw/` with directory mode 0700/files 0600. Raw logs are never Git-tracked or attached to PRs.

- [ ] **Step 5: Run reporting tests**

Run: `cargo test -p gzmo-evolver report redaction`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: produce sanitized evolution evaluation reports"
```

---

### Task 4: Authenticate as a Least-Privileged GitHub App

**Files:**
- Modify: `gzmo-evolver/Cargo.toml`
- Create: `gzmo-evolver/src/github/mod.rs`
- Create: `gzmo-evolver/src/github/auth.rs`
- Create: `gzmo-evolver/src/github/client.rs`
- Test: `gzmo-evolver/tests/github.rs`

**Interfaces:**
- Produces: `GitHubClient` using short-lived installation tokens kept only in memory.
- Consumes: App ID, installation ID, PEM path outside repo, fixed owner/repo.

- [ ] **Step 1: Write mocked token lifecycle tests**

Assert JWT issuer/audience/time bounds, installation-token request, expiry refresh, and token omission from `Debug`, errors, logs, and child environments.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test github auth`

Expected: FAIL: GitHub module missing.

- [ ] **Step 3: Add dependencies**

Use `jsonwebtoken = "9"`, `reqwest` with rustls/json, and `secrecy = "0.10"`. Private key bytes and installation token are `SecretString`/`SecretBox` and zeroized on drop where supported.

- [ ] **Step 4: Implement GitHub App auth**

Create RS256 JWT valid no more than 9 minutes; exchange at `POST /app/installations/{id}/access_tokens`; request only repository permissions configured on the App. Cache token until five minutes before expiry.

Required App permissions:

- Metadata: read
- Contents: read/write
- Pull requests: read/write
- Issues: read/write
- Checks: read/write

No Actions administration, administration, members, secrets, environments, deployments, or workflows permission.

- [ ] **Step 5: Pin repository identity**

Every REST path derives from configured owner/repo. Validate API response `full_name` equals it. Redirects to another owner/repo fail. The client exposes only typed methods required by Tasks 5–6, not a generic HTTP escape hatch.

- [ ] **Step 6: Run auth/client tests**

Run: `cargo test -p gzmo-evolver --test github`

Expected: PASS; captured mock requests contain the installation token only in Authorization headers.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock gzmo-evolver
git commit -m "feat: add least-privileged GitHub App client"
```

---

### Task 5: Push Only the Evaluated Commit and Open One PR

**Files:**
- Create: `gzmo-evolver/src/github/publish.rs`
- Modify: `gzmo-evolver/src/git.rs`
- Modify: `gzmo-evolver/src/runner.rs`
- Modify: `gzmo-evolver/tests/github.rs`
- Modify: `.github/pull_request_template.md`
- Create: `.github/ISSUE_TEMPLATE/autonomous-opportunity.yml`

**Interfaces:**
- Produces: remote candidate branch, GitHub check run, exactly one PR, persisted PR number/URL.
- Consumes: `ReviewReady` candidate, exact report/candidate/policy/audit digests, installation token.

- [ ] **Step 1: Write branch confinement tests**

```rust
#[tokio::test]
async fn publisher_refuses_main_tags_and_unevaluated_head() {
    let p = fixture_publisher();
    assert!(p.push("main", evaluated_commit()).await.is_err());
    assert!(p.push("refs/tags/v1", evaluated_commit()).await.is_err());
    assert!(p.push(candidate_branch(), other_commit()).await.is_err());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test github publisher_refuses`

Expected: FAIL: publisher missing.

- [ ] **Step 3: Push with an ephemeral askpass process**

Trusted parent creates a mode-0700 temporary directory and askpass executable that reads the in-memory installation token over a one-use inherited pipe, not argv or disk. Run:

```text
git -C <worktree> -c credential.helper= push
  --force-with-lease=refs/heads/evolve/<id>:<expected-old-or-empty>
  https://x-access-token@github.com/<owner>/<repo>.git
  <evaluated-commit>:refs/heads/evolve/<id>
```

The URL contains no token. Delete helper and close pipe before any worker can resume. Verify remote branch SHA equals evaluated commit.

- [ ] **Step 4: Create a trusted check run**

Create check `gzmo-evolver/evaluation` on the candidate SHA. Conclusion is `success` only when report hard floors pass. Include report digest, policy digest, audit head, bounded summary, and artifact links; no raw logs.

- [ ] **Step 5: Open or update exactly one PR**

Search open PR by exact head `<owner>:evolve/<id>`. Create only if absent. Title: `evolve(<mission-id>): <mission title>`. Base must equal configured base branch. Body includes mission, why, diff stats, tests, hard floors, metric deltas, report/audit digests, authority `human merge`, and explicit no-auto-merge statement.

- [ ] **Step 6: Update PR template and issue form**

PR template adds optional Evolver fields without breaking human PRs. Issue form creates a candidate opportunity with purpose, falsifiable done-when, required gate, protected-path exception request, and human priority; it does not activate itself.

- [ ] **Step 7: Persist and audit remote identity**

Store PR number/URL and remote SHA transactionally, transition `ReviewReady → PromotionPending`, append audit. If API succeeds but local commit fails, resume by querying exact branch/PR and reconciling idempotently.

- [ ] **Step 8: Run publisher tests**

Run: `cargo test -p gzmo-evolver --test github publish`

Expected: PASS including duplicate request replay, wrong repo/base, stale lease, and token redaction.

- [ ] **Step 9: Commit**

```bash
git add gzmo-evolver .github
git commit -m "feat: publish evaluated candidates as review-only PRs"
```

---

### Task 6: Shepherd CI and Explicit Review Repairs

**Files:**
- Create: `gzmo-evolver/src/shepherd.rs`
- Modify: `gzmo-evolver/src/github/client.rs`
- Modify: `gzmo-evolver/src/runner.rs`
- Test: `gzmo-evolver/tests/shepherd.rs`

**Interfaces:**
- Produces: `ShepherdDecision::{Wait,Repair,HumanReview,Accepted,Rejected,Failed}`.
- Consumes: PR/check/review state, repair count, evaluation report.

- [ ] **Step 1: Write state-decision tests**

```rust
#[test]
fn repair_requires_owned_failure_or_explicit_label_and_is_capped() {
    assert_eq!(decide(ci_failed_owned(), 0), ShepherdDecision::Repair);
    assert_eq!(decide(ci_failed_unrelated(), 0), ShepherdDecision::HumanReview);
    assert_eq!(decide(review_requested_without_label(), 0), ShepherdDecision::HumanReview);
    assert_eq!(decide(review_requested_with_label(), 0), ShepherdDecision::Repair);
    assert_eq!(decide(ci_failed_owned(), 2), ShepherdDecision::Failed);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p gzmo-evolver --test shepherd repair_requires`

Expected: FAIL: shepherd missing.

- [ ] **Step 3: Poll only the candidate PR**

Read check suites/check runs for its head SHA, reviews, requested changes, labels, merge/closed status. Ignore commands/comments from unaffiliated users unless a maintainer adds `evolve:revise`.

- [ ] **Step 4: Build a sanitized repair mission**

Include failing check names, bounded redacted log excerpts, review text from trusted maintainers, original manifest, and unchanged budgets/protected paths. Never include GitHub token, raw workflow logs, hidden holdouts, or new authority.

- [ ] **Step 5: Re-run the isolated worker**

Worker receives the candidate worktree with remote push disabled. After commit, run the entire evaluator—not only failed checks. Push with lease only if all hard floors pass. Increment attempt and append audit.

- [ ] **Step 6: Stop conditions**

Stop on two repair attempts, protected-path request, policy/evaluator change, ambiguous infrastructure failure, closed PR, human `evolve:stop` label, or any authority expansion. Human must reopen or create a fresh mission.

- [ ] **Step 7: Observe merge without causing it**

When GitHub reports merged, transition `PromotionPending → Soaking`, verify the merge commit contains the evaluated candidate commit and the required post-merge check succeeds, then transition `Soaking → Accepted`. Otherwise transition to `Failed` and require a human-owned revert/follow-up; never call merge, auto-merge, or automatic main-revert endpoints.

- [ ] **Step 8: Run shepherd tests**

Run: `cargo test -p gzmo-evolver --test shepherd`

Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add gzmo-evolver
git commit -m "feat: shepherd candidate PRs without auto-merge"
```

---

### Task 7: Run the Stage 1 Safety Matrix

**Files:**
- Modify: `gzmo-evolver/tests/github.rs`
- Modify: `gzmo-evolver/tests/shepherd.rs`
- Create: `tests/repo-evolver-live-smoke.sh`

**Interfaces:**
- Produces: complete connected-loop verification before cadence installation.
- Consumes: runner, evaluator, publisher, shepherd.

- [ ] **Step 1: Add adversarial tests**

Cover candidate attempts to change policy/holdouts/workflows/contracts; print environment; push main/tag; replace remote; create a second PR; spoof a check; exceed budget; inject shell argv through mission; mutate after evaluation; race another runner; replay stale report; repair a third time; and merge itself.

- [ ] **Step 2: Run all hermetic tests**

```bash
cargo fmt --all -- --check
cargo clippy -p gzmo-evolver --all-targets -- -D warnings
cargo test -p gzmo-evolver --all-targets -- --nocapture
```

Expected: all pass.

- [ ] **Step 3: Run live stop-before-push smoke**

`tests/repo-evolver-live-smoke.sh` requires `GZMO_EVOLVER_LIVE=1`, uses a disposable fixture repo, validates the authenticated target repository, runs a fixture candidate, and stops at `ReviewReady`. Expected: no remote branch/PR.

- [ ] **Step 4: Run live draft-PR smoke with explicit operator flag**

Run: `GZMO_EVOLVER_LIVE=1 GZMO_EVOLVER_ALLOW_PR=1 bash tests/repo-evolver-live-smoke.sh --open-draft`

Expected: one draft PR on an `evolve/fixture-*` branch, check report attached, no main change, no auto-merge. Close PR and delete fixture branch through trusted cleanup command; audit both.

- [ ] **Step 5: Commit**

```bash
git add gzmo-evolver tests/repo-evolver-live-smoke.sh
git commit -m "test: prove review-only autonomous pull request flow"
```
