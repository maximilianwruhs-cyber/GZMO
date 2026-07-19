#!/usr/bin/env bash
# Unpark Wave 4.1 demable: AOS CE path smoke — living pin + product boundary.
# Never overwrites ~/.gzmo. Does not start a second overnight writer.
#
#   bash scripts/aos-ce-smoke.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/aos-ce"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== AOS CE smoke (Unpark W4.1) ==="
[[ -f "$ROOT/docs/AOS_CUSTOMER_EDITION.md" ]] && row PASS "ce-doc" "AOS_CUSTOMER_EDITION.md" || row FAIL "ce-doc" "missing"
[[ -f "$ROOT/deploy/living-appliance/docker-compose.yml" ]] && row PASS "living-pin" "compose pin present" || row FAIL "living-pin" "missing"

bash "$ROOT/scripts/living-appliance-gate.sh" >/dev/null 2>&1 || true
if [[ -f "$DATA/living-appliance/latest.json" ]] && python3 -c "import json;d=json.load(open('$DATA/living-appliance/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "living-pin-gate" "living appliance pin ok"
else
  row FAIL "living-pin-gate" "living-appliance-gate failed"
fi

# Product boundary: stranger path must not require sidecars
bash "$ROOT/scripts/mcp-attach-check.sh" >/dev/null 2>&1 || true
if [[ -f "$DATA/mcp-attach/latest.json" ]] && python3 -c "import json;d=json.load(open('$DATA/mcp-attach/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "product-boundary" "gzmo-memory still product ~/.gzmo"
else
  row FAIL "product-boundary" "product attach broken"
fi

# Dual-writer soft
if systemctl --user is-active gzmo-serve.service >/dev/null 2>&1; then
  row FAIL "dual-writer" "workstation gzmo-serve active — ADR-0003"
else
  row PASS "dual-writer" "workstation serve inactive"
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
advice="aos_ce_ok" if fail_n==0 else "aos_ce_fail"
payload={"schema":"gzmo.unpark.aos_ce/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"4.1","checks":checks,
  "note":"CE is on top of C; never stranger A default."}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
