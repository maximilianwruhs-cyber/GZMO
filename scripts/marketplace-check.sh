#!/usr/bin/env bash
# Unpark Wave 4.2 demable: OKCP marketplace notes + read-only concept fixture browse.
#   bash scripts/marketplace-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/marketplace"
FIXTURE="$ROOT/data/okcp/concept-bundle.fixture.json"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Marketplace check (Unpark W4.2) ==="
[[ -f "$ROOT/docs/OKCP_MARKETPLACE.md" ]] && row PASS "doc" "OKCP_MARKETPLACE.md" || row FAIL "doc" "missing"

if rg -n 'marketplace-check|okcp.concept' "$ROOT/scripts/living-readiness-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-living-required" "marketplace wired into living-readiness — remove"
else
  row PASS "not-living-required" "living gate independent of marketplace"
fi

if rg -n 'marketplace-check|OKCP_MARKETPLACE' "$ROOT/scripts/product-readiness-gate.sh" >/dev/null 2>&1; then
  row FAIL "not-product-required" "marketplace wired into product-readiness — remove"
else
  row PASS "not-product-required" "product A install independent of marketplace"
fi

[[ -f "$FIXTURE" ]] && row PASS "fixture-file" "$FIXTURE" || row FAIL "fixture-file" "missing data/okcp/concept-bundle.fixture.json"

BROWSE="$OUT/concept-bundle.browse.json"
if [[ -f "$FIXTURE" ]]; then
  cp "$FIXTURE" "$BROWSE"
  # Keep a legacy stub name that points at the browse copy for older consumers
  cp "$BROWSE" "$OUT/concept-bundle.stub.json"
  if python3 - <<PY
import json, sys
from pathlib import Path
b = json.loads(Path("$BROWSE").read_text())
concepts = b.get("concepts") or []
if not isinstance(concepts, list) or len(concepts) < 1:
    sys.exit(2)
if b.get("write_gated") is not True:
    sys.exit(3)
for c in concepts:
    if not c.get("id") or not c.get("title"):
        sys.exit(4)
sys.exit(0)
PY
  then
    row PASS "fixture-browse" "non-empty concepts + write_gated=true"
  else
    row FAIL "fixture-browse" "fixture must have concepts and write_gated"
  fi
else
  row FAIL "fixture-browse" "no fixture to browse"
fi

row PASS "boundary" "not product install requirement; not living SoT; writes operator-gated"

cat >"$OUT/boundary.md" <<'EOF'
# Marketplace boundary (Unpark Wave 4.2)

- Read-only browse of `data/okcp/concept-bundle.fixture.json` → `data-next/marketplace/`.
- `write_gated: true` — no auto-publish into product MCP or CT101 living vault.
- Not required for stranger `install-gzmo.sh` (A) or living GREEN overnight (C).
EOF

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
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"4.2","checks":checks,
  "fixture":"data/okcp/concept-bundle.fixture.json"}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":payload["advice"],"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
