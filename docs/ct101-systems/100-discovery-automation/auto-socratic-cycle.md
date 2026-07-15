# Auto-Socratic Discovery Cycle

**Source:** `gzmo_skills/scripts/auto-socratic-discovery-cycle.sh`  
**Parent:** [100-discovery-automation/SYSTEM.md](./SYSTEM.md)

---

## Capability

Daemon-invoked entry point when chaos tension drops: starts (or continues) a Pi mentor discovery session without arming the systemd timer directly. Handles **flock locking**, **pending queue** when busy, OBOLUS budget gate, and post-cycle implementation drain.

---

## How it works

### Invocation

```1:7:github-clone/gzmo_skills/scripts/auto-socratic-discovery-cycle.sh
#!/usr/bin/env bash
# AUTO Socratic trigger → one pi-mentor-discovery cycle (pillar probe + cycle report).
# Invoked by gzmo-daemon low-tension watcher. Does NOT arm systemd timer.
#
# Usage: auto-socratic-discovery-cycle.sh [trigger] [tension] [tick]
#   trigger: low_tension (default)
```

### Path resolution (Jul 10 fix context)

```14:15:github-clone/gzmo_skills/scripts/auto-socratic-discovery-cycle.sh
GZMO_ROOT="${GZMO_ROOT:-$HOME/Projects/_foundation-audit/survey_GZMO}"
GZMO_BIN="${GZMO_BIN:-$GZMO_ROOT/target/release/gzmo}"
```

On CT101 production, environment/cron must set `GZMO_ROOT=/opt/gzmo/survey_GZMO`. Wrong root → OBOLUS skip or mentor ping hang → zero published cycles (documented in infrastructure report).

`run-discovery-implement.sh` hardcodes production default:

```118:119:github-clone/gzmo_skills/scripts/run-discovery-implement.sh
export GZMO_ROOT="${GZMO_ROOT:-/opt/gzmo/survey_GZMO}"
export GZMO_DATA_DIR="${GZMO_DATA_DIR:-$GZMO_ROOT/data}"
```

### Queue and lock

```167:183:github-clone/gzmo_skills/scripts/auto-socratic-discovery-cycle.sh
if ! ( flock -n 9 ) 9>"$LOCK_FILE"; then
  QUEUE_FILE="$DATA_DIR/.discovery-queue"
  MAX_PENDING="${DISCOVERY_QUEUE_MAX_PENDING}"
  // ...
  echo "$(date -u +"%Y-%m-%dT%H:%M:%SZ") $TRIGGER $TENSION $TICK" >> "$QUEUE_FILE"
  log "ENQUEUED (queue: $(wc -l < "$QUEUE_FILE" 2>/dev/null || echo 0) pending)"
  exit 0
fi
```

`DISCOVERY_QUEUE_MAX_PENDING` read from `gzmo.toml` `[pedagogy.low_tension_dialogue.discovery_queue]`.

### OBOLUS + cycle dispatch

```222:240:github-clone/gzmo_skills/scripts/auto-socratic-discovery-cycle.sh
if [[ -x "$GZMO_BIN" ]]; then
  if ! (cd "$GZMO_ROOT" && "$GZMO_BIN" obolus preflight discovery_cycle); then
    obolus_rc=$?
    log "OBOLUS: auto-socratic discovery_cycle blocked (exit $obolus_rc)"
    exit "$obolus_rc"
  fi
fi
// ...
"$ROOT/scripts/pi-mentor-discovery-cycle.sh"
cycle_status=$?
```

### Publish outcome logging

```270:274:github-clone/gzmo_skills/scripts/auto-socratic-discovery-cycle.sh
if [[ "$is_published" == "true" ]]; then
  log "OK report=${last_pub}"
else
  log "SKIP: cycle ran successfully but did not publish (unpublished)"
fi
```

Live probe: unpublished cycles occur when session-final eval detects **template placeholder text** (see `discovery-findings-lib.sh` placeholder pattern).

---

## Interfaces

| Interface | CT101 path / value |
|-----------|-------------------|
| Data dir | `/home/maximilian/gzmo_skills/data/pi-mentor-discovery/` |
| Lock | `$DATA_DIR/.cycle.lock` |
| Queue | `$DATA_DIR/.discovery-queue` |
| Logs | `$DATA_DIR/logs/auto-socratic.log`, `auto-triggers.jsonl` |
| Trigger source | Daemon chaos low-tension watcher (~30 min cadence) |
| Config | `$DATA_DIR/config.env`, `gzmo.toml` `[pedagogy]` |
| Env overrides | `DISCOVERY_AUTO_MULTI_CYCLE`, `DISCOVERY_ISOLATION_*`, `GZMO_OSCILLATION_ID` |

---

## THINKING nodes

> **THINKING — auto-socratic:multi-cycle branch**
> - *Reviewed:* `DISCOVERY_AUTO_MULTI_CYCLE=1` re-enters foreground cycle on active non-stale sessions.
> - *Insight:* Allows arc sessions to continue without timer re-arm noise.
> - *Risk / limitation:* Stale session detection depends on duration helpers — clock skew can strand locks.
> - *Enhancement:* Heartbeat file touched each Pi dialogue round for staleness. [CT101-safe]

> **THINKING — auto-socratic:queue drain**
> - *Reviewed:* On lock acquire, drains `.discovery-queue` synchronously with recursive `$0` calls.
> - *Insight:* Prevents trigger loss during busy periods while bounding pending depth.
> - *Risk / limitation:* Recursive re-entry can stack tension/tick args incorrectly if queue format drifts.
> - *Enhancement:* Structured JSON queue entries instead of space-delimited fields. [CT101-safe]

> **THINKING — auto-socratic:GZMO_ROOT default**
> - *Reviewed:* Default still points at workstation audit path in script header.
> - *Insight:* CT101 ops must export `GZMO_ROOT=/opt/gzmo/survey_GZMO` in systemd/timer env.
> - *Risk / limitation:* Jul 10 fix documented but default in clone still workstation-centric.
> - *Enhancement:* Fail fast if `GZMO_ROOT` path lacks `/opt/gzmo` on CT101 hostname. [CT101-safe]

> **THINKING — auto-socratic:post-cycle drain**
> - *Reviewed:* Calls `discovery-drain-implementation-queue.sh` after every cycle.
> - *Insight:* Ties discovery publish to remediation pipeline without separate cron.
> - *Risk / limitation:* Drain failure does not fail cycle exit code (logged only).
> - *Enhancement:* Surface drain failures in Synapse `discovery_cycle_complete` event. [GZMO-next]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| Bash trigger from inline chaos watcher | Lab `pedagogy-probe` recipe with beat-gate baseline |
| File-based queue + flock | Redis stream queue on shared infra |
| Manual OBOLUS preflight CLI | Centralized budget service |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | CT101 hostname guard for `GZMO_ROOT` | [CT101-safe] |
| 2 | JSONL structured discovery queue | [CT101-safe] |
| 3 | Alert on `completed_unpublished` streak > 2 | [CT101-safe] |
| 4 | Integrate auto-socratic metrics into Observatory | [GZMO-next] |
| 5 | Replace recursive queue drain with iterative worker | [CT101-safe] |
