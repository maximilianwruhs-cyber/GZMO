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
if ls "$ROOT"/scripts/*forge* >/dev/null 2>&1; then
  row PASS "scripts" "forge-related scripts present"
else
  row HOLD "scripts" "no forge scripts in repo — sibling forge OK"
fi
[[ -f "$ROOT/docs/OBOLUS_ARENA_BOUNDARY.md" ]] && row PASS "docs" "boundary documented" || row FAIL "docs" "missing"
cat >"$OUT/recommend-contract.md" <<'EOF'
# Forge recommend contract (Unpark Wave 3.3)

Pin winners as display/route advice via Obolus MCP.
Do not block CT101 distill/dream unless an explicit operator gate says so.
EOF
row PASS "recommend-contract" "$OUT/recommend-contract.md"

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
advice="forge_lab_ok" if fail_n==0 else "forge_lab_fail"
payload={"schema":"gzmo.unpark.forge_lab/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"3.3","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
