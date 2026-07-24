#!/usr/bin/env bash
# Daily evolve plane (workstation) — never starts gzmo-serve.
# Runs: ops-health → organ-watchdog (living) → research-scan → brain-feed-check (soft)
#
#   bash scripts/ecosystem-evolve-daily.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ecosystem-evolve"
mkdir -p "$OUT"
LOG="$OUT/daily.log"
: >"$LOG"

run_soft() {
  local name="$1"
  shift
  echo "=== $name ===" | tee -a "$LOG"
  set +e
  "$@" >>"$LOG" 2>&1
  local rc=$?
  set -e
  echo "[rc=$rc] $name" | tee -a "$LOG"
  return "$rc"
}

rc_ops=0; run_soft ops-health bash "$ROOT/scripts/ops-health.sh" || rc_ops=$?
rc_wd=0; run_soft organ-watchdog env ORGAN_LIVING=1 bash "$ROOT/scripts/organ-watchdog-check.sh" || rc_wd=$?
rc_rs=0; run_soft research-scan bash "$ROOT/scripts/research-scan.sh" || rc_rs=$?
rc_bf=0; run_soft brain-feed bash "$ROOT/scripts/brain-feed-check.sh" || rc_bf=$?
rc_sync=0
if [[ -x "$ROOT/scripts/sync-openclaw-workspace.sh" ]]; then
  run_soft openclaw-workspace-sync bash "$ROOT/scripts/sync-openclaw-workspace.sh" || rc_sync=$?
fi

python3 - "$OUT" "$rc_ops" "$rc_wd" "$rc_rs" "$rc_bf" "$rc_sync" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
out = Path(sys.argv[1])
ops, wd, rs, bf, sync = (int(x) for x in sys.argv[2:7])
# ops-health RED fails the day; others soft
ok = ops == 0
payload = {
    "schema": "gzmo.ecosystem_evolve.daily/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "rcs": {
        "ops_health": ops,
        "organ_watchdog": wd,
        "research_scan": rs,
        "brain_feed": bf,
        "openclaw_workspace_sync": sync,
    },
    "advice": (
        "daily_evolve_GREEN — review research inbox + living smoke"
        if ok
        else "daily_evolve_RED — ops-health failed; fix CT101/Prime before trusting overnight"
    ),
}
(out / "daily-latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
sys.exit(0 if ok else 1)
PY
