# Implementation Queue — Sidecar Remediation Pipeline

**Source:** `gzmo_skills/scripts/run-discovery-implement.sh`, `write-sidecar-remediation.sh`  
**Parent:** [100-discovery-automation/SYSTEM.md](./SYSTEM.md)

---

## Capability

After a published discovery report, dequeues implementation work: run acceptance **probes**, register findings with `kurator`, spawn a **plan agent** (`plan-from-discovery`), evaluate the plan, and write **sidecar remediation** artifacts — all without touching `gzmo-core/` on frozen CT101.

---

## How it works

### Queue dequeue (`run-discovery-implement.sh`)

```50:58:github-clone/gzmo_skills/scripts/run-discovery-implement.sh
dequeue_entry() {
  [[ -f "$QUEUE" ]] || return 1
  if [[ -n "$REPORT" ]]; then
    jq -sc --arg rpt "$(realpath "$REPORT")" \
      'map(select(.source_report == $rpt and (.status == "pending_plan" or .status == "plan_failed" or .status == "plan_spawn_failed"))) | .[0] // empty' \
      "$QUEUE"
    return
  fi
  jq -sc 'map(select(.status == "pending_plan" or .status == "plan_failed" or .status == "plan_spawn_failed")) | sort_by(.priority, .ts) | .[0] // empty' "$QUEUE"
}
```

Queue file: `$DISCOVERY_IMPLEMENTATION_DATA/queue.jsonl`

### CT101 sidecar-only profile

```167:172:github-clone/gzmo_skills/scripts/run-discovery-implement.sh
if [[ "${DISCOVERY_PLAN_SIDECAR_ONLY:-0}" == "1" ]]; then
  export DISCOVERY_PLAN_VERSION="${DISCOVERY_PLAN_VERSION:-1.3-probes}"
  export DISCOVERY_PLAN_CORE_PASS=0
  impl_log "sidecar-only plan profile (DISCOVERY_PLAN_SIDECAR_ONLY=1)"
fi
```

### Pipeline steps

1. **cycle_guard_acquire** — prevents concurrent implement runs
2. **obolus preflight** `discovery_plan`
3. **implement-discovery-actions.sh** — acceptance probes
4. **kurator fix-from-discovery --register-only**
5. **kurator plan-from-discovery --spawn** → `plans/<session>/plan.json`
6. **eval-implementation-plan.sh** — gate before execution
7. **write-sidecar-remediation.sh** per workstream (sidecar targets only)

### Sidecar writer

```51:53:github-clone/gzmo_skills/scripts/write-sidecar-remediation.sh
mapfile -t TARGETS < <(jq -r --arg id "$WS_ID" \
  '.workstreams[] | select(.id == $id) | .target_paths[]?' "$PLAN_JSON" 2>/dev/null)
```

```61:65:github-clone/gzmo_skills/scripts/write-sidecar-remediation.sh
  if ! sidecar_is_remediation_target "$rel"; then
    echo "write-sidecar: skip non-remediation target: $rel" >&2
    continue
  fi
```

Writes under `GZMO_SKILLS_ROOT` only; preserves enriched sidecars unless `--force`:

```81:85:github-clone/gzmo_skills/scripts/write-sidecar-remediation.sh
  if [[ -f "$abs" ]] && ! sidecar_target_is_stub "$abs" && [[ "$FORCE" -eq 0 ]]; then
    echo "[SKIP] $abs (enriched sidecar preserved; use --force to overwrite)" >&2
    written=$((written + 1))
    continue
  fi
```

Stub detection: header `# Sidecar remediation for ` or ≤120 bytes.

---

## Interfaces

| Interface | CT101 path |
|-----------|------------|
| Queue | `/home/maximilian/gzmo_skills/data/discovery-implementation/queue.jsonl` |
| Plans | `.../plans/<report-id>/plan.json`, `plan.md` |
| Logs | `.../logs/implement.log` |
| GZMO_ROOT | `/opt/gzmo/survey_GZMO` (default in implement script) |
| GZMO_SKILLS_ROOT | `/home/maximilian/gzmo_skills` |
| Synapse | `$GZMO_ROOT/data/Synapse/events.jsonl` via `remediation-env.sh` |
| Drain entry | `discovery-drain-implementation-queue.sh` (called from auto-socratic) |

Remediation env defaults:

```4:9:github-clone/gzmo_skills/scripts/lib/remediation-env.sh
GZMO_ROOT="${GZMO_ROOT:-$HOME/Projects/_foundation-audit/survey_GZMO}"
GZMO_SKILLS_ROOT="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
DISCOVERY_IMPLEMENTATION_DATA="${DISCOVERY_IMPLEMENTATION_DATA:-$GZMO_SKILLS_ROOT/data/discovery-implementation}"
```

---

## THINKING nodes

> **THINKING — run-discovery-implement:obolus gate**
> - *Reviewed:* Plan spawn blocked when `obolus preflight discovery_plan` fails (exit 2).
> - *Insight:* Prevents runaway plan-agent LLM spend on CT101.
> - *Risk / limitation:* Silent queue stall if entries never retry after transient OBOLUS block.
> - *Enhancement:* Exponential backoff requeue with `plan_spawn_failed` status visibility. [CT101-safe]

> **THINKING — run-discovery-implement:sidecar-only**
> - *Reviewed:* `DISCOVERY_PLAN_SIDECAR_ONLY=1` forces probe plan version, disables core pass.
> - *Insight:* Enforces CT101 boundary — plans cannot target `gzmo-core/src/`.
> - *Risk / limitation:* Depends on plan-agent prompt discipline + eval script path checks.
> - *Enhancement:* Hard reject in `eval-implementation-plan.sh` if any target under `gzmo-core/`. [CT101-safe]

> **THINKING — write-sidecar:stub preservation**
> - *Reviewed:* Non-stub existing files skipped to protect human-enriched remediations.
> - *Insight:* Idempotent pipeline — re-runs don't clobber good sidecars.
> - *Risk / limitation:* Stale stubs block progress if eval thinks work is done (`written` incremented on SKIP).
> - *Enhancement:* Distinguish SKIP vs WRITE in exit code for queue status. [CT101-safe]

> **THINKING — write-sidecar:git add**
> - *Reviewed:* Auto `git add` in `GZMO_SKILLS_ROOT` when inside work tree.
> - *Insight:* Prepares sidecar fixes for operator commit on workstation mirror.
> - *Risk / limitation:* CT101 production tree may not be a git checkout.
> - *Enhancement:* Optional commit hook only when `DISCOVERY_AUTO_COMMIT=1`. [GZMO-next]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| Bash queue + kurator CLI | Lab `discovery-implement` recipe in scheduler |
| Sidecar scripts in gzmo_skills | Workstream targets in little-tools-lab pieces |
| Manual operator commit | CI pipeline for sidecar acceptance tests |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Hard block `gzmo-core/` targets in plan eval | [CT101-safe] |
| 2 | Queue retry policy for OBOLUS/plan_spawn failures | [CT101-safe] |
| 3 | Fix SKIP-stub counting as success in write-sidecar | [CT101-safe] |
| 4 | Unified implementation status API for Observatory | [GZMO-next] |
| 5 | Migrate queue to Redis on GZMO-next cutover | [GZMO-next] |
