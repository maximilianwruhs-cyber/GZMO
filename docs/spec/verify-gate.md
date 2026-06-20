# verify_gate specification

Canonical behavior for discovery plan, fixer, and execute verify gates in GZMO.

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

## Fixer / code implementer

`verify_code_implement_outcome` / `verify_fixer_outcome` — written paths must exist under allowed roots; summary must not indicate max-iteration exhaustion.

## Remediation snapshot (post-spawn)

After `record_spawn_outcome`, writes:

- `$GZMO_SKILLS_ROOT/data/discovery-implementation/snapshots/<task_id>.json`
- `<task_id>.md` (timeline markdown)

Insights: `completion_attempts`, `plan_regenerations`, `failed_commands`.

## Path canon

- `GZMO_ROOT` — survey_GZMO repo root
- `GZMO_SKILLS_ROOT` — external skills tree (default `~/gzmo_skills`)
- Never use chimera `survey_GZMO/gzmo_skills/` paths in acceptance probes

## Verification scripts

```bash
./scripts/mechanics-verify.sh
~/gzmo_skills/scripts/discovery-probes/probe-jules-patterns.sh
cargo test -p gzmo-core discovery_plan_agent
```
