#!/usr/bin/env bash
# Unpark Wave 1.1: demable herdr ↔ GZMO metabolism check (soft if herdr absent).
#   bash scripts/herdr-metabolism-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/herdr-metabolism"
PLUGIN="$ROOT/integrations/herdr-gzmo-metabolism"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== herdr metabolism check (Unpark W1.1) ==="
[[ -f "$PLUGIN/herdr-plugin.toml" ]] && row PASS "plugin-tree" "$PLUGIN" || row FAIL "plugin-tree" "missing plugin"
[[ -x "$ROOT/scripts/herdr-metabolism-link.sh" ]] && row PASS "link-script" "herdr-metabolism-link.sh" || row FAIL "link-script" "missing"
[[ -x "$PLUGIN/scripts/session-close.sh" ]] && row PASS "close-script" "session-close.sh" || row FAIL "close-script" "missing"

# Plugin contract: session-close action + pane.closed event (takeaway ritual surface)
if [[ -f "$PLUGIN/herdr-plugin.toml" ]] && python3 -c "
from pathlib import Path
t=Path('$PLUGIN/herdr-plugin.toml').read_text(encoding='utf-8')
ok = 'gzmo.metabolism' in t and 'session-close' in t and 'pane.closed' in t and 'ensure-mcp' in t
raise SystemExit(0 if ok else 1)
"; then
  row PASS "plugin-contract" "gzmo.metabolism + session-close + pane.closed"
else
  row FAIL "plugin-contract" "herdr-plugin.toml missing metabolism close-ritual actions"
fi

if command -v herdr >/dev/null 2>&1; then
  row PASS "herdr-bin" "$(command -v herdr)"
  if herdr plugin list 2>/dev/null | grep -qi 'gzmo.metabolism'; then
    row PASS "plugin-linked" "gzmo.metabolism present"
  else
    row HOLD "plugin-linked" "not linked — bash scripts/herdr-metabolism-link.sh"
  fi
  if [[ -x "$PLUGIN/scripts/status.sh" ]]; then
    if bash "$PLUGIN/scripts/status.sh" >/dev/null 2>&1; then
      row PASS "plugin-status" "status.sh ok"
    else
      row HOLD "plugin-status" "status.sh non-zero (plugin may need link)"
    fi
  fi
else
  row HOLD "herdr-bin" "herdr not on PATH — optional operator shell"
fi

# Close-ritual evidence from demo (lab enqueue, no --now)
if [[ -f "$OUT/close-ritual.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$OUT/close-ritual.json'))
ok=(
  d.get('schema')=='gzmo.unpark.herdr.close_ritual/v1'
  and d.get('ok') is True
  and d.get('now_flag') is False
  and d.get('session_has_takeaway') is True
)
raise SystemExit(0 if ok else 1)
"; then
    row PASS "close-ritual" "$(python3 -c "import json;print(json.load(open('$OUT/close-ritual.json')).get('advice',''))")"
  else
    row FAIL "close-ritual" "close-ritual.json incomplete — rerun herdr-metabolism-demo.sh"
  fi
else
  row HOLD "close-ritual" "no close-ritual.json yet — bash scripts/herdr-metabolism-demo.sh"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out = Path(os.environ["OUT"])
checks = {}
for line in os.environ.get("ROWS_TSV","").splitlines():
    if not line.strip(): continue
    st,n,d = line.split("|",2); checks[n]={"status":st,"detail":d}
fail_n=int(os.environ["fail"]); hold_n=int(os.environ["hold"]); pass_n=int(os.environ["pass"])
verdict="GREEN" if fail_n==0 else "RED"
contract_ok = checks.get("plugin-contract",{}).get("status")=="PASS"
ritual_ok = checks.get("close-ritual",{}).get("status")=="PASS"
advice = "herdr_metabolism_ok" if fail_n==0 and contract_ok and ritual_ok else (
    "herdr_metabolism_hold — link or run close-ritual demo" if fail_n==0 else "herdr_metabolism_fail")
payload={"schema":"gzmo.unpark.herdr/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"1.1","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
