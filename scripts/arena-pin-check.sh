#!/usr/bin/env bash
# Unpark Wave 3: Arena → Pin theater readiness (suggest-only; never living apply).
#   bash scripts/arena-pin-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/arena-pin"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Arena → Pin check (Unpark W3) ==="
[[ -f "$ROOT/docs/OBOLUS_ARENA_BOUNDARY.md" ]] && row PASS "boundary" "Arena outside living daemon" || row FAIL "boundary" "missing"
[[ -f "$ROOT/docs/ARENA_PIN_DEMO.md" ]] && row PASS "demo-door" "ARENA_PIN_DEMO.md" || row HOLD "demo-door" "missing theater front door"
[[ -f "$ROOT/docs/BRAIN_FEED.md" ]] && row PASS "brain-feed" "BRAIN_FEED.md P1 pin doctrine" || row FAIL "brain-feed" "missing"

[[ -x "$ROOT/scripts/arena-pin-demo.sh" ]] && row PASS "demo-script" "arena-pin-demo.sh" || row FAIL "demo-script" "missing"
[[ -x "$ROOT/scripts/brain-intel-promote.sh" ]] && row PASS "intel-promote" "brain-intel-promote.sh" || row FAIL "intel-promote" "missing"
[[ -x "$ROOT/scripts/brain-intel-pin-log.sh" ]] && row PASS "pin-log-script" "brain-intel-pin-log.sh" || row FAIL "pin-log-script" "missing"
[[ -x "$ROOT/scripts/forge-lab-demo.sh" ]] && row PASS "forge-demo" "forge-lab-demo.sh" || row FAIL "forge-demo" "missing"

CHAMP="$DATA/arena/champion-suggestion.toml"
if [[ -f "$CHAMP" ]]; then
  row PASS "champion" "champion-suggestion.toml (sibling)"
else
  row HOLD "champion" "missing — run arena-night / arena-pin-demo"
fi

if [[ -f "$DATA/arena/latest.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$DATA/arena/latest.json'))
raise SystemExit(1 if d.get('auto_apply') is True or d.get('daemon_jobs_touched') is True else 0)
"; then
    row PASS "suggest-only" "nightburst auto_apply/daemon_jobs_touched not true"
  else
    row FAIL "suggest-only" "arena latest claims auto apply / daemon touch — forbidden"
  fi
else
  row HOLD "suggest-only" "no arena/latest.json yet"
fi

if [[ -f "$DATA/forge-lab/recommend.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$DATA/forge-lab/recommend.json'))
ok=(d.get('blocks_distill') is False and isinstance(d.get('pins'), list))
raise SystemExit(0 if ok else 1)
"; then
    row PASS "forge-recommend" "blocks_distill=false + pins"
  else
    row FAIL "forge-recommend" "recommend.json unsafe or incomplete"
  fi
else
  row HOLD "forge-recommend" "run forge-lab-demo / arena-pin-demo"
fi

if [[ -f "$DATA/brain-intel/latest.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$DATA/brain-intel/latest.json'))
raise SystemExit(0 if d.get('ok') is True and d.get('auto_apply') is False else 1)
"; then
    row PASS "intel-latest" "brain-intel suggest ready; auto_apply=false"
  else
    row FAIL "intel-latest" "brain-intel latest not suggest-ready"
  fi
else
  row HOLD "intel-latest" "run brain-intel-promote / arena-pin-demo"
fi

PIN_ROLLUP=""
for cand in "$DATA/brain-intel/pin-log-latest.json" "$DATA/brain-intel/pin-log.json"; do
  [[ -f "$cand" ]] && PIN_ROLLUP="$cand" && break
done
if [[ -n "$PIN_ROLLUP" ]]; then
  if python3 -c "
import json
d=json.load(open('$PIN_ROLLUP'))
ok=(d.get('accepted',0)>=1 and d.get('rejected',0)>=1)
raise SystemExit(0 if ok else 1)
"; then
    row PASS "pin-log" "≥1 accept and ≥1 reject logged (toml unchanged)"
  else
    row HOLD "pin-log" "need accept+reject samples — arena-pin-demo records them"
  fi
else
  row HOLD "pin-log" "no pin-log yet — arena-pin-demo"
fi

if rg -n 'arena-pin|arena-night|obolus-arena' "$ROOT/scripts/living-readiness-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-living-required" "Arena/Pin wired into living-readiness — remove"
else
  row PASS "not-living-required" "living gate independent of Arena/Pin"
fi

if [[ -f "$OUT/demo.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$OUT/demo.json'))
ok=(
  d.get('schema')=='gzmo.unpark.arena_pin.demo/v1'
  and d.get('ok') is True
  and d.get('auto_apply') is False
  and d.get('daemon_jobs_touched') is False
  and d.get('blocks_distill') is False
)
raise SystemExit(0 if ok else 1)
"; then
    row PASS "demo-inventory" "demo.json — suggest→pin chain; living untouched"
  else
    row FAIL "demo-inventory" "demo.json incomplete — rerun arena-pin-demo.sh"
  fi
else
  row HOLD "demo-inventory" "no demo.json yet — bash scripts/arena-pin-demo.sh"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out=Path(os.environ["OUT"]); checks={}
for line in os.environ.get("ROWS_TSV","").splitlines():
    if not line.strip(): continue
    st,n,d=line.split("|",2); checks[n]={"status":st,"detail":d}
fail_n=int(os.environ["fail"]); hold_n=int(os.environ["hold"]); pass_n=int(os.environ["pass"])
verdict="GREEN" if fail_n==0 else "RED"
demo_ok = checks.get("demo-inventory",{}).get("status")=="PASS"
if fail_n==0 and demo_ok:
    advice="arena_pin_ok — suggest→recommend→pin-log theater; living toml untouched"
elif fail_n==0:
    advice="arena_pin_hold — run arena-pin-demo.sh"
else:
    advice="arena_pin_fail"
payload={"schema":"gzmo.unpark.arena_pin/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,"demo":demo_ok,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"3.pin","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
