---
type: entity
title: "Jules Pattern Mining (thema_005)"
created: "2026-06-20"
updated: "2026-06-20"
status: active
tags:
  - research
  - jules
  - discovery
  - pattern-mining
---

# Jules Pattern Mining (thema_005)

Offline archaeology of Google Labs Jules OSS for **sovereign, local-first** patterns in GZMO. No Jules cloud API.

## Adopted in survey_GZMO

| Pattern | Module / script |
|---------|-----------------|
| File ownership gate | `discovery_plan_agent::validate_workstream_ownership` |
| Plan approval | `approve_plan`, `ensure_plan_executable`, `gzmo kurator approve-plan` |
| Session snapshot | `remediation_snapshot.rs`, snapshots under discovery-implementation |
| Git brief enrichment | `discovery_git_context.rs` |
| Five-phase pipeline | `scripts/run-discovery-goal-pipeline.sh` |
| Local CI (act) | `scripts/act/` |
| Activity query | `scripts/query-discovery-activities.sh` |
| Change reconciliation | `scripts/reconcile-discovery-changes.sh` |
| Spawn polling retry | `spawn_polling.rs` |

## Research artifacts

- Vendor clones: `~/Schreibtisch/research/thema_005/vendor/`
- Matrix: `jules-pattern-matrix.md`
- Curated KB: `~/Schreibtisch/knowledge/curated/thema_005-jules-patterns.md`

## Probe

```bash
~/gzmo_skills/scripts/discovery-probes/probe-jules-patterns.sh
```

## Related

- [Open Knowledge Format](/entities/open-knowledge-format.md)
- Spec: `docs/spec/verify-gate.md`
