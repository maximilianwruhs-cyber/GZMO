#!/usr/bin/env bash
# Unpark Wave 3.3: Forge recommend path — never auto-block distill.
#   bash scripts/forge-lab-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/forge-lab"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Forge lab check (Unpark W3.3) ==="
row PASS "boundary" "recommend only — never auto-block distill"

[[ -x "$ROOT/scripts/forge-lab-demo.sh" ]] \
  && row PASS "demo-script" "forge-lab-demo.sh executable" \
  || row FAIL "demo-script" "missing"

[[ -f "$ROOT/docs/OBOLUS_ARENA_BOUNDARY.md" ]] && row PASS "docs" "boundary documented" || row FAIL "docs" "missing"

# Ensure living gate does not require forge recommend
if rg -n 'forge-lab|recommend\.json|blocks_distill' "$ROOT/scripts/living-readiness-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-living-required" "Forge wired into living-readiness — remove"
else
  row PASS "not-living-required" "living gate independent of forge recommend"
fi

cat >"$OUT/recommend-contract.md" <<'EOF'
# Forge recommend contract (Unpark Wave 3.3)

Pin winners as display/route advice from `data-next/arena/forge/` (or champion TOML).
Do not block CT101 distill/dream unless an explicit operator gate says so.
`blocks_distill` in recommend.json must remain false.
Human promote only — never auto-overwrite live engine config.
EOF
row PASS "recommend-contract" "$OUT/recommend-contract.md"

REC="$OUT/recommend.json"
if [[ -f "$REC" ]]; then
  if python3 - <<PY
import json, sys
from pathlib import Path
rec = json.loads(Path("$REC").read_text())
if rec.get("schema") != "gzmo.unpark.forge.recommend/v1":
    sys.exit(1)
if rec.get("blocks_distill") is not False:
    sys.exit(2)
if not isinstance(rec.get("pins"), list) or not rec["pins"]:
    sys.exit(3)
if rec.get("action") != "recommend":
    sys.exit(4)
sys.exit(0)
PY
  then
    row PASS "recommend-json" "$REC (schema + blocks_distill=false + pins)"
  else
    row FAIL "recommend-json" "invalid recommend.json (must not block distill)"
  fi
else
  row HOLD "recommend-json" "run forge-lab-demo.sh"
fi

if [[ -f "$DATA/arena/forge/latest.json" ]] || [[ -f "$DATA/arena/champion-suggestion.toml" ]]; then
  row PASS "arena-source" "arena forge/champion artifacts present"
else
  row HOLD "arena-source" "no arena winners yet — demo may emit stub pins"
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
rec_ok = checks.get("recommend-json",{}).get("status")=="PASS"
if fail_n==0 and rec_ok:
    advice="forge_lab_ok — recommend pins; blocks_distill=false"
elif fail_n==0:
    advice="forge_lab_hold — run forge-lab-demo.sh"
else:
    advice="forge_lab_fail"
payload={"schema":"gzmo.unpark.forge_lab/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"3.3","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
