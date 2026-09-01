# Hybrid Continuous Self-Development — Session Handoff

- **Date:** 2026-09-01
- **Status:** User-requested stop at a clean subplan boundary
- **Branch:** `feat/hybrid-continuous-self-development`
- **Implementation checkpoint:** `f8322caf2764ab1af1a16c5512dfd4d53b1cf24c`
- **Worktree:** `C:/Users/z005a5ff/AppData/Local/Temp/GZMO/.worktrees/hybrid-continuous-self-development`
- **Remote base at stop:** `origin/main` = `fc5025d59cda4211bbb6ac392f12783668146cde`

## Resume condition

Resume this handoff when continuing the hybrid self-development implementation. Work in the existing linked worktree and branch. Do not restart completed contracts/governance tasks, re-author the approved North Star, or implement on `main`.

## Read first

1. `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`
2. `docs/superpowers/plans/2026-09-01-hybrid-continuous-self-development-program.md`
3. This handoff
4. `.superpowers/sdd/2026-09-01-hybrid-continuous-self-development-program/progress.md`
5. `docs/superpowers/plans/2026-09-01-connected-repo-evolver.md`

Required workflow: `using-superpowers` → `subagent-driven-development`. The current checkout is already an isolated worktree; verify it instead of creating another one.

## Completed

### Planning

Six additive plans are committed on the history leading to this branch:

- hybrid program sequencing;
- evolution contracts/governance;
- connected repository evolver;
- evaluation and PR shepherd;
- air-gapped evolution controller;
- continuous evolution operations/cutover.

### Contracts and governance foundation

The entire `2026-09-01-evolution-contracts-and-governance.md` subplan is complete and reviewed.

Delivered:

- `docs/ADR-INDEX.md` with decision/implementation states and authority order;
- ADR-0011 through ADR-0014;
- historical ADR-0003 through ADR-0010 status/lineage metadata;
- active authority pointers in `AGENTS.md`, `README.md`, `MACHINE.md`, and `docs/SPINE_FOCUS.md`;
- `scripts/adr-check.sh` and `tests/adr-index-test.sh`;
- pure `crates/evolution-contracts` crate;
- validated Candidate ID/Target/Manifest/state graph;
- bounded CapabilityEnvelope/ResourceBudget/PathPolicy/TunableRule;
- non-compensable EvaluationReport and structural PromotionRequest/UnverifiedAuthorityGrant;
- canonical SHA-256 AuditEvent chain with replayable timestamps;
- deterministic frozen JSON Schemas for candidate, envelope, evaluation, promotion, and audit.

Foundation task/review loop completed with task-scoped reviews, fix rounds, broad final review, one final fix wave, and a localized schema/runtime consistency re-review.

## Fresh verification at checkpoint

Run from WSL Ubuntu at the worktree path:

```bash
cargo fmt --all -- --check
cargo clippy -p evolution-contracts --all-targets -- -D warnings
cargo test -p evolution-contracts
bash tests/adr-index-test.sh
bash scripts/adr-check.sh
```

Observed at `f8322ca`:

- format: PASS;
- clippy with `-D warnings`: PASS;
- evolution-contracts: 17 library + 1 exporter + 68 contract + 10 schema tests PASS, 0 failed;
- ADR index test: PASS;
- ADR checker: `ok=true`, 71/71 checks.

The only output noise is the pre-existing workspace warning that `crates/eml-core/Cargo.toml` has non-root package profiles ignored.

## Not started

`docs/superpowers/plans/2026-09-01-connected-repo-evolver.md` has **zero implementation tasks completed**. Its SDD workspace directory exists, but no task was dispatched and no connected-runner source files exist.

Also not started:

- `gzmo-evolver` crate;
- GitHub App setup/client;
- candidate worktree/OMP worker;
- evaluator/PR shepherd;
- system services/timers;
- internal PostgreSQL evolution controller;
- offline bundle bridge;
- legacy evolve-loop retirement;
- PR, merge, deployment, or production mutation.

No GitHub credential, provider credential, or signing key was written into the repository or handoff.

## Exact next action

1. Verify branch/worktree:

```bash
git status -sb
git branch --show-current
git rev-parse HEAD
```

Expected branch: `feat/hybrid-continuous-self-development`. The first implementation base for the connected-runner plan is the branch tip containing this handoff; foundation checkpoint remains `f8322ca`.

2. Initialize/read the connected-plan ledger at:

```text
.superpowers/sdd/2026-09-01-connected-repo-evolver/progress.md
```

Its first line must be:

```markdown
# SDD ledger — plan: docs/superpowers/plans/2026-09-01-connected-repo-evolver.md
```

3. Perform the subagent-driven preflight table for all seven tasks and shared interfaces. Important pairs:

- Task 1 config/CLI → every later task;
- Task 2 StateStore/audit → Tasks 3, 5, 6;
- Task 3 mission manifest → Task 4 Git workspace and Task 6 runner;
- Task 4 Git workspace → Task 5 worker and Task 6 runner;
- Task 5 sealed WorkerRequest/Receipt → Task 6 state orchestration;
- Task 6 runner → Task 7 vertical acceptance.

4. Initialize todos for all seven tasks plus final connected-plan review.

5. Generate Task 1 brief with:

```bash
bash skill://subagent-driven-development/scripts/task-brief \
  docs/superpowers/plans/2026-09-01-connected-repo-evolver.md 1
```

6. Dispatch one implementer only. Task 1 builds the `gzmo-evolver` crate/config/CLI skeleton. Follow each task with independent spec+quality review and fix loop before moving on.

## Connected-runner trust rulings

These were locked during planning and must carry into implementation:

- Stage 1 is a connected development service, never the Living writer.
- Exactly one active mission and one nonterminal candidate per repository.
- Trusted coordinator and OMP worker are separate OS users.
- Only the coordinator can read the GitHub App key/state; only the worker can write its candidate worktree/output.
- Worker receives a sealed read-only WorkerRequest and writes a bounded receipt.
- Worker push URL is `no-push://candidate-worker`; no GitHub/PAT/SSH/provider credential enters the worker.
- OMP uses a qualified local code model in a worker/model private network namespace with loopback only and no host/default route.
- Candidate worker cannot edit trusted policy, evaluator, ADR/spec, workflows, contracts crate, or evolver implementation.
- Stage 1 may push only an evaluated `evolve/<candidate-id>` branch through the later trusted GitHub App adapter. It never pushes or merges `main`, tags, releases, settings, or visibility.
- Human merge remains Stage 1 promotion authority.

## Rulings carried from completed foundation

- Foundation task order was `1 → 2 → 4 → 3 → 5 → 6 → 7` because CandidateManifest consumes the real ResourceBudget.
- Contract modules are exported only after their implementations compile.
- Use WSL Ubuntu Rust 1.97.1; Windows has no Rust toolchain.
- Resource ceilings are fixed v1 authority; changing them later requires a versioned Authority-tier decision.
- `allow_missing_energy_meter` lives only under ResourceBudget.
- CandidateManifest includes CandidateKind and cross-stage CandidateTarget.
- Promotion is a distinct v1 wire schema; no VerifiedAuthorityGrant exists in the pure contract crate.
- Audit uses canonical structured preimages, explicit genesis, stored event hashes, caller-supplied replay timestamps, and bare 64-hex internal chain hashes.
- JSON Schemas encode expressible constraints and name runtime-only invariants explicitly.

## Residual non-blocking findings

Do not expand the next task to fix these unless a touched path makes the fix local and independently testable:

- PathPolicy normalizes/folds protected patterns per check rather than memoizing.
- Digest/path validation helpers remain duplicated across contract modules.
- CandidateId custom deserialization allocates one String on a cold path.
- Envelope `allowed_candidate_kinds` is a BTreeSet: duplicate JSON values collapse while JSON Schema `uniqueItems:true` rejects them. No authority expansion results.

## Program sequence after connected runner

1. Complete connected repository candidate generation.
2. Complete deterministic evaluation and review-only GitHub PR shepherd.
3. Soak four weekly Stage 1 outcomes.
4. Implement the air-gapped EvolutionController only after PostgreSQL authority, BootTrust, candidate storage, local role qualification, and rollback targets exist.
5. Add signed offline bundle bridge, unified status, cadence, chaos/rollback drills.
6. Retire legacy `idle_evolve` and ecosystem wrappers only after shadow parity and soak evidence.

## Stop-state

- No implementation subagent is running.
- No long-running service or watcher was started.
- No PR or merge was created.
- Preserve this linked worktree; it is the resume surface.
