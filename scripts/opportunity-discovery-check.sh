#!/usr/bin/env bash
# Opportunity discovery gate.
#   bash scripts/opportunity-discovery-check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/opportunity-discovery"
OPP="$ROOT/research/opportunities"
mkdir -p "$OUT"
LOG="$OUT/gate.log"
: >"$LOG"

pass=0
fail=0
hold=0
declare -a ROWS=()

row() {
  local status="$1" name="$2" detail="$3"
  ROWS+=("$status|$name|$detail")
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
    HOLD) hold=$((hold + 1)) ;;
  esac
  echo "[$status] $name — $detail" | tee -a "$LOG"
}

echo "=== Opportunity discovery check ===" | tee -a "$LOG"

[[ -f "$ROOT/docs/OPPORTUNITY_DISCOVERY.md" ]] && row PASS "doctrine" "docs/OPPORTUNITY_DISCOVERY.md" || row FAIL "doctrine" "missing"
[[ -f "$ROOT/docs/templates/MISSION_CARD.md" ]] && row PASS "mission-template" "docs/templates/MISSION_CARD.md" || row FAIL "mission-template" "missing"
[[ -f "$OPP/README.md" ]] && row PASS "bet-log" "research/opportunities/" || row FAIL "bet-log" "missing README"

bash "$ROOT/scripts/opportunity-sense.sh" >>"$LOG" 2>&1 || true
bash "$ROOT/scripts/opportunity-rank.sh" >>"$LOG" 2>&1 || true

if [[ -f "$OUT/sense-latest.json" ]] && python3 -c "import json;d=json.load(open('$OUT/sense-latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "sense" "$(python3 -c "import json;d=json.load(open('$OUT/sense-latest.json')); print(d.get('advice',''))")"
  # Sense v2 depth fields (soft HOLD if old artifact)
  if python3 -c "import json;d=json.load(open('$OUT/sense-latest.json')); raise SystemExit(0 if 'felt_use_depth' in d and 'stack_gaps' in d else 1)"; then
    row PASS "sense-depth" "felt_use_depth + stack_gaps present (sense v2)"
  else
    row HOLD "sense-depth" "rerun opportunity-sense.sh for v2 depth scars"
  fi
else
  row FAIL "sense" "sense-latest.json missing/not ok"
fi

if [[ -f "$OUT/rank-latest.json" ]] && python3 -c "import json;d=json.load(open('$OUT/rank-latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "rank" "$(python3 -c "import json;d=json.load(open('$OUT/rank-latest.json')); print(d.get('advice',''))")"
else
  row FAIL "rank" "rank-latest.json missing/not ok"
fi

# Active bet invariant
active_n="$(ROOT="$ROOT" OPP="$OPP" python3 - <<'PY'
import os, sys
from pathlib import Path
sys.path.insert(0, str(Path(os.environ["ROOT"]) / "scripts"))
from opportunity_lib import load_bets
bets = load_bets(Path(os.environ["OPP"]))
print(sum(1 for b in bets if b.get("status") == "active"))
PY
)"

if [[ "$active_n" == "1" ]]; then
  row PASS "active-bet" "exactly one active bet"
elif [[ "$active_n" == "0" ]]; then
  row HOLD "active-bet" "no active bet — opportunity-bet.sh --from <id>"
else
  row FAIL "active-bet" "active_count=$active_n — need exactly one"
fi

# Horizon file present (local intel parked)
if [[ -f "$OPP/local-intel-32gb-128k.md" ]]; then
  row PASS "horizon-local-intel" "local intel parked as horizon (not active ship)"
else
  row HOLD "horizon-local-intel" "missing horizon bet file"
fi

# Scripts executable / present
for s in opportunity-sense.sh opportunity-rank.sh opportunity-bet.sh opportunity-next-mission.sh opportunity_lib.py; do
  if [[ -f "$ROOT/scripts/$s" ]]; then
    row PASS "script:$s" "present"
  else
    row FAIL "script:$s" "missing"
  fi
done

bash "$ROOT/scripts/opportunity-next-mission.sh" >>"$LOG" 2>&1 || true
if [[ -f "$OUT/next-mission.json" ]] \
  && python3 -c "import json;d=json.load(open('$OUT/next-mission.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "next-mission" "$(python3 -c "import json;print(json.load(open('$OUT/next-mission.json')).get('bet_id',''))")"
else
  row HOLD "next-mission" "no next-mission — need one active bet"
fi

export OUT pass fail hold
set +e
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["pass"])
fail_n = int(os.environ["fail"])
hold_n = int(os.environ["hold"])
verdict = "GREEN" if fail_n == 0 else "RED"
advice = (
    "opportunity_discovery_ready — Sense→Rank→Bet loop demable"
    if verdict == "GREEN"
    else "opportunity_discovery_hold — fix FAIL rows"
)
payload = {
    "schema": "gzmo.opportunity.discovery.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "doc": "docs/OPPORTUNITY_DISCOVERY.md",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

{
  echo "# Opportunity discovery check"
  echo
  echo "Verdict: **$(python3 -c "import json;print(json.load(open('$OUT/latest.json'))['verdict'])")**"
  echo
  echo "| Status | Check | Detail |"
  echo "|--------|-------|--------|"
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    detail="${detail//|/\\|}"
    echo "| $st | $name | $detail |"
  done
  echo
  echo "See: docs/OPPORTUNITY_DISCOVERY.md"
  echo
} >"$OUT/latest.md"

echo "=== opportunity-discovery done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
