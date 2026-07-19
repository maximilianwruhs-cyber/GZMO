#!/usr/bin/env bash
# Unpark Wave 3.1: Arena / RAPL / € observability lab (sibling-first; not gzmo-daemon).
#   bash scripts/arena-lab-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/arena-lab"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Arena lab check (Unpark W3.1) ==="
[[ -f "$ROOT/docs/OBOLUS_ARENA_BOUNDARY.md" ]] && row PASS "boundary" "Arena outside living daemon" || row FAIL "boundary" "missing"
[[ -f "$ROOT/docs/OBOLUS_ENERGY.md" ]] && row PASS "energy-docs" "OBOLUS_ENERGY.md" || row HOLD "energy-docs" "missing"

# Observability only
if [[ -f "$ROOT/data/power.jsonl" ]] || [[ -f "$DATA/power.jsonl" ]]; then
  row PASS "rapl-log" "power.jsonl present (observability)"
else
  row HOLD "rapl-log" "no power.jsonl yet — RAPL optional"
fi

ARENA="${OBOLUS_ARENA_ROOT:-$HOME/github-clone/obolus-arena}"
if [[ -d "$ARENA" ]]; then
  row PASS "arena-sibling" "$ARENA"
else
  row HOLD "arena-sibling" "obolus-arena not cloned — contract only"
fi

# Ensure living gate does not require Arena
if rg -n 'arena-lab|obolus-arena|arena-night' "$ROOT/scripts/living-readiness-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-living-required" "Arena wired into living-readiness — remove"
else
  row PASS "not-living-required" "living gate independent of Arena"
fi

cat >"$OUT/lab-contract.md" <<'EOF'
# Arena lab contract (Unpark Wave 3)

- Run overnight z-loops in sibling `obolus-arena/`.
- GZMO RAPL / € artifacts are observability only.
- Never add Arena jobs to `gzmo-daemon` by default.
EOF
row PASS "lab-contract" "$OUT/lab-contract.md"

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
advice="arena_lab_ok" if fail_n==0 and hold_n==0 else ("arena_lab_hold" if fail_n==0 else "arena_lab_fail")
payload={"schema":"gzmo.unpark.arena_lab/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"3.1","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
