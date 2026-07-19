#!/usr/bin/env bash
# Unpark Wave 2.2: mutual-discovery theater ≠ living scout KPI.
#   bash scripts/discovery-theater-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/discovery-theater"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Discovery theater check (Unpark W2.2) ==="
[[ -f "$ROOT/docs/MUTUAL_DISCOVERY_THEATER.md" ]] && row PASS "theater-doc" "MUTUAL_DISCOVERY_THEATER.md" || row FAIL "theater-doc" "missing"
[[ -f "$ROOT/docs/DISCOVERY_LIFECYCLE.md" ]] && row PASS "lifecycle" "DISCOVERY_LIFECYCLE.md" || row FAIL "lifecycle" "missing"
# Ensure theater is not wired as living readiness KPI
if rg -n 'mutual-discovery|MUTUAL_DISCOVERY' "$ROOT/scripts/living-readiness-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-living-kpi" "theater referenced in living-readiness-gate — remove"
else
  row PASS "not-living-kpi" "theater not a living-readiness row"
fi
[[ -d "$ROOT/docs/research/mutual-discovery" ]] && row PASS "archive" "research/mutual-discovery/" || row HOLD "archive" "archive optional"

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
advice="discovery_theater_ok" if fail_n==0 else "discovery_theater_fail"
payload={"schema":"gzmo.unpark.discovery_theater/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"2.2","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
