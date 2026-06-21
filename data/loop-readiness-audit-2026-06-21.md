# GZMO Loop Readiness Audit — 2026-06-21

This document provides a quantified, read-only audit of the live GZMO loop-readiness state, combining industry rubrics, stack health checks, native E2E mechanics, and KB feedback loop metrics.

---

## 1. Audit Summary & Scores

| Dimension | Evidence Source | Heuristic / Pass Condition | Score / Status |
| :--- | :--- | :--- | :--- |
| **External Hypervisor** | `kurator_monitor` + `spawn_gate` | FAIL/GAP report triggers recommendation outside LLM | **PASS (10/10)** |
| **Canonical Artifact Paths** | `verify_gate` + mechanics §10 | Marker under `$GZMO_SKILLS_ROOT` only; chimera paths rejected | **FAIL (4/10)** (Leftover files in `gzmo_skills/` repo tree) |
| **Worktree Isolation** | `gzmo.toml` + `kurator_spawn` | `fixer_worktree_isolation = true` | **PASS (10/10)** |
| **Stuck / Circuit Break** | `agent_loop` + `spawn_gate` | `StuckDetector` active + hourly limits respected | **PASS (10/10)** |
| **Stage Gates** | `verify-gate.md` + `acceptance_gate` | Gates at every phase transition (not just post-spawn) | **PARTIAL (5/10)** (Requires Phase 2 gates) |
| **KB Loop Closure** | `discovery-kb-recall-smoke` | recall-smoke query pass rate $\ge 66\%$ | **PASS (10/10)** (5/5 hits, rate 1.0) |
| **Discovery Signal Quality** | `discovery-kb-metrics` | Novel LINKs/cycle, dedup skips active | **PARTIAL (6/10)** (No-progress / duplicates observed) |

---

## 2. Infrastructure Health & Stack Verification

All core GZMO system services are verified active and reachable:

- **gzmo-daemon.service**: **ACTIVE** (Active status verified via `systemctl`)
- **Prime :8000 (LLM Model Server)**: **REACHABLE** (V1 models API returned 200 OK)
- **kurator status CLI**: **PASS** (CLI returns active recommendation list)
- **Obolus governance**: **PASS** (preflight checks return Allow for core actions)

---

## 3. Quantified Mechanics Baseline

Running `./scripts/mechanics-verify.sh` returned:
- **Passed checks**: 45
- **Failed checks**: 4
- **Overall status**: **FAIL** (due to specific non-canonical paths and E2E pipeline timeouts)

### Identified Failures in Mechanics verification
1. **Chimera files**: `gzmo_skills/` directory under `survey_GZMO` still contains lingering files. These must be clean-migrated via `migrate-chimera-skills-artifacts.sh`.
2. **E2E Pipeline (session-final-2026-06-16T16-25-43Z.md)**: **FAIL** (tracker shows 3 still pending, only 2 of 5 fixed).
3. **E2E Pipeline (session-final-2026-06-16T14-57-29Z.md)**: **FAIL** (tracker shows 4 still pending, only 2 of 6 fixed).
4. **Fixer pipeline script**: `verify-implement-fixer-pipeline.sh` failed.

---

## 4. KB Feedback Loop Metrics

Executing `./scripts/discovery-kb-metrics.sh` compiled the following baseline metrics:

```json
{
  "counts": {
    "semantic_vault": 58196,
    "honeypot_latest": 36150,
    "discovery_sourced_vault": 47,
    "distill_dedup_rows": 65,
    "cycle_reports": 263
  },
  "distill": {
    "log_attempts": 0,
    "log_dedup_skips": 7,
    "dedup_skip_rate_estimate": 0.0
  },
  "discovery_links": {
    "registry_total": 164,
    "registry_entries_30d": 164
  }
}
```

- **Vault-to-Honeypot ratio**: $\sim 62\%$ of vault entries promoted.
- **Discovery Sourced Vault entries**: 47 total.
- **Link Registry total**: 164 active link fingerprints tracked.

---

## 5. Epistemological Recall Smoke Test

Executing `./scripts/discovery-kb-recall-smoke.sh` verified recall accuracy for recent `link-registry.jsonl` queries:

- **Queries tested**: 5
- **Recall hits**: 5 (pass rate = 1.0)
- **Verified recall queries**:
  - `` `distillation_phase_transition` `` (Score 0.13, Vault hit)
  - `intent-based authorization` (Score 0.21, Vault hit)
  - `data contract validation` (Score 0.08, Vault hit)
  - `endpoint consistency` (Score 0.30, Vault hit)
  - `semantic availability` (Score 0.11, Vault hit)

The 100% recall pass rate confirms that semantic search accurately retrieves context for newly promoted concepts, satisfying the feedback loop quality target ($\ge 66\%$).

---

## 6. Remediation Activity Snapshot

The current state of GZMO's remediation database queried via `query-discovery-activities.sh summary`:

- **open**: 35
- **in_flight**: 1
- **probed**: 80
- **fixed**: 35
- **failed**: 5
- **total**: 156
