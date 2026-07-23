#!/usr/bin/env bash
# Weekly evolve plane — opportunity + serendipity honesty (no auto-apply, no ship kickoff).
#
#   bash scripts/ecosystem-evolve-weekly.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ecosystem-evolve"
mkdir -p "$OUT"
LOG="$OUT/weekly.log"
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

rc_opp=0; run_soft opportunity-discovery bash "$ROOT/scripts/opportunity-discovery-check.sh" || rc_opp=$?
rc_ser=0; run_soft serendipity-weekly bash "$ROOT/scripts/serendipity-weekly-check.sh" || rc_ser=$?
rc_kg=0; run_soft kg-reconcile-dry bash "$ROOT/scripts/kg-reconcile-dry.sh" || rc_kg=$?
rc_mission=0
if [[ -x "$ROOT/scripts/opportunity-next-mission.sh" ]]; then
  run_soft next-mission bash "$ROOT/scripts/opportunity-next-mission.sh" || rc_mission=$?
fi

python3 - "$OUT" "$rc_opp" "$rc_ser" "$rc_kg" "$rc_mission" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
out = Path(sys.argv[1])
opp, ser, kg, mission = (int(x) for x in sys.argv[2:6])
ok = opp == 0
payload = {
    "schema": "gzmo.ecosystem_evolve.weekly/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "rcs": {
        "opportunity_discovery": opp,
        "serendipity_weekly": ser,
        "kg_reconcile_dry": kg,
        "next_mission": mission,
    },
    "artifacts": {
        "next_mission": str(Path(out).parent / "opportunity-discovery" / "next-mission.md"),
        "serendipity": str(Path(out).parent / "serendipity" / "cadence-latest.md"),
    },
    "advice": (
        "weekly_evolve_ok — review next-mission.md; human kickoff only; never auto-apply serendipity"
        if ok
        else "weekly_evolve_HOLD — opportunity gate not green; see data-next/opportunity-discovery/"
    ),
}
(out / "weekly-latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
sys.exit(0 if ok else 1)
PY
