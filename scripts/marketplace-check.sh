#!/usr/bin/env bash
# Unpark Wave 4.2 demable: OKCP marketplace notes + bundle stub.
#   bash scripts/marketplace-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/marketplace"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Marketplace check (Unpark W4.2) ==="
[[ -f "$ROOT/docs/OKCP_MARKETPLACE.md" ]] && row PASS "doc" "OKCP_MARKETPLACE.md" || row FAIL "doc" "missing"
cat >"$OUT/concept-bundle.stub.json" <<'EOF'
{
  "schema": "gzmo.okcp.concept_bundle/v0",
  "name": "unpark-stub",
  "concepts": [],
  "write_gated": true,
  "note": "Read-only browse spike; write path operator-gated"
}
EOF
row PASS "bundle-stub" "$OUT/concept-bundle.stub.json"
row PASS "boundary" "not product install requirement; not living SoT"

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
fail_n=int(os.environ["fail"]); pass_n=int(os.environ["pass"]); hold_n=int(os.environ["hold"])
verdict="GREEN" if fail_n==0 else "RED"
payload={"schema":"gzmo.unpark.marketplace/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":"marketplace_ok" if fail_n==0 else "marketplace_fail",
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"4.2","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":payload["advice"]},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
