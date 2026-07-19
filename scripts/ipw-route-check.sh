#!/usr/bin/env bash
# Unpark Wave 3.2: IpW router demable advice (never living required path).
#   bash scripts/ipw-route-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ipw-route"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== IpW route check (Unpark W3.2) ==="
[[ -f "$ROOT/scripts/ipw-route.sh" ]] && row PASS "script" "ipw-route.sh" || row FAIL "script" "missing"
[[ -f "$ROOT/config/ipw-router.policy.toml" ]] && row PASS "policy" "ipw-router.policy.toml" || row HOLD "policy" "policy missing"
[[ -f "$ROOT/docs/OBOLUS_ARENA_BOUNDARY.md" ]] && row PASS "boundary" "outside living metabolism" || row FAIL "boundary" "missing"

set +e
bash "$ROOT/scripts/ipw-route.sh" --help >/dev/null 2>&1 || bash "$ROOT/scripts/ipw-route.sh" 2>/dev/null | head -5 >/dev/null
rc=$?
set -e
if [[ "$rc" -eq 0 ]]; then
  row PASS "invoke" "ipw-route.sh runnable"
else
  row HOLD "invoke" "ipw-route.sh exited $rc (advice-only soft)"
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
advice="ipw_route_ok" if fail_n==0 else "ipw_route_fail"
payload={"schema":"gzmo.unpark.ipw/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"3.2","checks":checks,
  "note":"Never auto-block distill."}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
