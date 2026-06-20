# THEMA_005 — Jules pattern mining (Abschluss)

**Date:** 2026-06-20  
**Scope:** Pattern adoption from Jules OSS into GZMO — **no Jules API integration**.

## Objective

Mine Google Labs Jules open-source repos for patterns that strengthen GZMO's sovereign discovery/remediation pipeline without cloud dependencies.

## Delivered

### Rust (gzmo-core)

| Module | Pattern |
|--------|---------|
| `discovery_git_context.rs` | Git show/log in briefs (`DISCOVERY_INCLUDE_GIT_CONTEXT=1`) |
| `remediation_snapshot.rs` | Session snapshot JSON + markdown with insights |
| `discovery_plan_agent.rs` | Ownership validation, plan approval helpers |
| `discovery_execute.rs` | `ensure_plan_executable` gate |
| `remediation_tracker.rs` | Snapshot write on spawn outcome |
| `kurator_monitor.rs` | Plan approval check before execute emit |

### CLI (gzmo-cli)

- `gzmo kurator approve-plan --plan <dir>`

### Scripts

- `scripts/act/install-act.sh`, `run-act.sh`
- `scripts/query-discovery-activities.sh`
- `scripts/run-discovery-goal-pipeline.sh`

### Discovery probe

- `~/gzmo_skills/scripts/discovery-probes/probe-jules-patterns.sh`

### Documentation

- `AGENTS.md` — Commands, Testing, Boundaries, Local CI Verification
- `docs/spec/verify-gate.md`
- `wiki/entities/jules-pattern-mining.md`
- Research: `~/Schreibtisch/research/thema_005/jules-{oss-inventory,pattern-matrix}.md`
- Curated KB: `~/Schreibtisch/knowledge/curated/thema_005-jules-patterns.md`

### Vendor clones

```
~/Schreibtisch/research/thema_005/vendor/
  jules-sdk/
  jules-action/
  jules-skills/
  jules-awesome-list/
```

## Explicitly out of scope

- `JULES_API_KEY`, `jules.googleapis.com`, `@google/jules-sdk`
- `skill_jules.sh`
- `compliance.network_exceptions` += jules

## Five-phase pipeline mapping

| Phase | Jules | GZMO |
|-------|-------|------|
| 1 Analyze | Issue fetch | Discovery report |
| 2 Plan | Planning session | `discovery_plan_agent` |
| 3 Validate | Ownership check | `verify_plan_agent_outcome` + ownership |
| 4 Dispatch | `jules.all()` | kurator spawns (spawn_gate) |
| 5 Verify | CI merge | `cargo test`, mechanics-verify, snapshots |

## Operator quickstart

```bash
export GZMO_SKILLS_ROOT=~/gzmo_skills
export GZMO_ROOT=~/Projects/_foundation-audit/survey_GZMO

cargo test -p gzmo-core
~/gzmo_skills/scripts/discovery-probes/probe-jules-patterns.sh

gzmo kurator plan-from-discovery --report <report.md>
gzmo kurator approve-plan --plan <plan_dir>
gzmo kurator execute-workstream --plan <plan_dir> --workstream WS1 --spawn
```

## Deferred (Tier B)

- ~~`scripts/reconcile-discovery-changes.sh` (jules-merge analogue)~~ **done**
- ~~Spawn polling retry (SDK polling.ts pattern)~~ **done** — `spawn_polling.rs`
- ~~TDD skill injection in code implementer brief~~ **done**

### Tier B commands

```bash
./scripts/reconcile-discovery-changes.sh scan --json
./scripts/reconcile-discovery-changes.sh status
# SPAWN_LOAD_POLLING=1 (default) — tracker load retries after spawn flush
# SPAWN_POLL_INTERVAL_MS / SPAWN_POLL_TIMEOUT_MS / SPAWN_LOAD_RETRY_ATTEMPTS
```

## Verification checklist

- [x] `cargo test -p gzmo-core` passes
- [x] `probe-jules-patterns.sh` all checks pass
- [x] `! rg jules.googleapis|JULES_API_KEY survey_GZMO/`
- [x] Ownership + approval unit tests in `discovery_plan_agent`
