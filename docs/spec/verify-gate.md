# verify_gate specification

Canonical behavior for discovery plan, fixer, execute, and post-spawn hardening in GZMO.

## Plan agent (`verify_plan_agent_outcome`)

Required artifacts: `plan.md`, `plan.json`, `plan-provenance.json`.

Checks:

1. All actionable findings covered by workstreams or `deferred[]`.
2. Each workstream has ≥2 acceptance probes.
3. No chimera paths (`survey_GZMO/gzmo_skills`, `gzmo_skills/survey_GZMO`).
4. `complexity=complex` requires `gzmo-core/` or `gzmo.toml` in `target_paths`.
5. **Ownership** — no two workstreams share a `target_paths` entry or `spawn_command` (Jules fleet-dispatch).
6. Valid `spawn_command` format when present.

## Plan approval (execute gate)

When `DISCOVERY_PLAN_REQUIRE_APPROVAL` is unset or truthy (default):

- `plan.json` must contain non-empty `approved_at`.
- Operator approves: `gzmo kurator approve-plan --plan <dir>`.
- `load_workstream` and `process_discovery_execute` call `ensure_plan_executable`.

Disable: `DISCOVERY_PLAN_REQUIRE_APPROVAL=0`.

## Fixer / code implementer / execute (artifact gate)

`verify_discovery_fix_outcome` / `verify_code_implement_outcome` — after spawn:

- `written_paths` from tool telemetry must exist under canonical roots.
- Claimed summary paths must resolve (no chimera).
- Summary must not indicate max-iteration exhaustion or hallucinated `file_write` markup.

Returns `DiscoveryFixVerification` with fields:

| Field | Meaning |
|-------|---------|
| `passed` | Artifact gate result (before external acceptance) |
| `missing_paths` | Claimed paths not on disk |
| `hit_max_iterations` | Agent exhausted tool budget |
| `notes` | Human-readable failure/success summary |
| `acceptance_failed` | Populated by hypervisor when external acceptance fails |

## External acceptance gate (Pack A)

After artifact verify, [`kurator_spawn.rs`](../../gzmo-core/src/kurator_spawn.rs) runs hypervisor-side checks via [`discovery_acceptance_gate.rs`](../../gzmo-core/src/discovery_acceptance_gate.rs):

| Flow | Hypervisor action |
|------|-------------------|
| Execute | Re-run workstream `acceptance[]` via `bash -c` with `GZMO_ROOT` / `GZMO_SKILLS_ROOT` |
| Code implement | Re-run matching workstream `acceptance[]` for the finding |
| Fixer | Re-run ACTION probe scripts when mapped |

Non-zero exit codes append to `acceptance_failed` and force `passed = false` even when artifacts exist.

## StuckDetector (Pack C)

In [`agent_loop.rs`](../../gzmo-core/src/agent_loop.rs), before max-iteration exhaustion:

- **Exact tool repetition** — same `(tool_name, args)` hash ≥3 times in rolling window.
- **Ping-pong** — two tool signatures alternate across the window.
- **Text-only loop** — ≥3 consecutive text turns when `require_file_write_before_done` is set.

On trip: abort spawn with `hit_max_iterations = true`, emit Synapse `agent.stuck`.

## Worktree isolation (Pack D)

When `[kurator] fixer_worktree_isolation = true`:

- Fixer and code-implement spawns run in ephemeral git worktrees under `$GZMO_SKILLS_ROOT/.worktrees/fix-<uuid>`.
- Verify and acceptance gates evaluate against the worktree skills root.
- Worktree removed after pass/fail.

Default in `gzmo.toml.example`: `false` (opt-in trial). Enable locally when testing isolated fixer spawns.

## Rollback and escalation (Pack E)

When remediation retries are exhausted (`max_retries` in tracker):

- **Rollback** — `git checkout` skills repo to plan `git_baseline_tag` (skipped when worktree isolation is on; worktree removal handles cleanup).
- **Escalation** — emit Synapse `remediation.escalated` with `escalation_reason: max_retries_exhausted`.

## Remediation snapshot (post-spawn)

After `record_spawn_outcome`, writes:

- `$GZMO_SKILLS_ROOT/data/discovery-implementation/snapshots/<task_id>.json`
- `<task_id>.md` (timeline markdown)

Insights: `completion_attempts`, `plan_regenerations`, `failed_commands`.

## Path canon

- `GZMO_ROOT` — survey_GZMO repo root
- `GZMO_SKILLS_ROOT` — external skills tree (default `~/gzmo_skills`)
- Never use chimera `survey_GZMO/gzmo_skills/` paths in acceptance probes

## Spawn telemetry kinds

Synapse `spawn.executed` / `agent.spawned` `kind` / `spawn_kind` values:

| Value | Recommendation |
|-------|----------------|
| `discovery_fix` | Fixer |
| `discovery_plan` | Plan agent |
| `discovery_code_implement` | Code implementer |
| `discovery_execute` | Workstream executor |
| `session_triage` | Prometheus triage |

All discovery kinds share the same autospawn rate-limit bucket; only `session_triage` uses `auto_spawn_on_recommend`.

## Verification scripts

```bash
./scripts/mechanics-verify.sh
~/gzmo_skills/scripts/discovery-probes/probe-jules-patterns.sh
cargo test -p gzmo-core discovery_plan_agent spawn_gate
```
