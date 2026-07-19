#!/usr/bin/env bash
# Unpark Wave 1.2: Pi optional-glass attach hygiene (CLI remains canonical).
#   bash scripts/pi-glass-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/pi-glass"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Pi glass check (Unpark W1.2) ==="
PI_MCP="${HOME}/.pi/agent/mcp.json"
CURSOR_MCP="${HOME}/.cursor/mcp.json"
[[ -f "$ROOT/docs/OPERATOR_FRONTEND_DECISION.md" ]] && row PASS "doctrine" "CLI canonical; Pi optional glass" || row FAIL "doctrine" "missing"
[[ -f "$ROOT/docs/PI_LIVING_STACK.md" ]] && row PASS "pi-living-docs" "PI_LIVING_STACK.md" || row HOLD "pi-living-docs" "missing"

# Product must not be hijacked; living may use gzmo-living
bash "$ROOT/scripts/mcp-attach-check.sh" >/dev/null 2>&1 || true
if [[ -f "$DATA/mcp-attach/latest.json" ]] && python3 -c "import json;d=json.load(open('$DATA/mcp-attach/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "product-attach" "$(python3 -c "import json;print(json.load(open('$DATA/mcp-attach/latest.json')).get('advice',''))")"
else
  row FAIL "product-attach" "gzmo-memory not pointing at ~/.gzmo"
fi

if [[ -f "$PI_MCP" ]]; then
  row PASS "pi-mcp-file" "$PI_MCP"
  if python3 -c "import json;d=json.load(open('$PI_MCP')); s=d.get('mcpServers') or {}; raise SystemExit(0 if 'gzmo-memory' in s or 'gzmo-living' in s else 1)"; then
    row PASS "pi-gzmo-servers" "gzmo-memory and/or gzmo-living present"
  else
    row HOLD "pi-gzmo-servers" "no gzmo-* servers in Pi mcp.json"
  fi
  # Flag dual-config collision: product env pointing at /opt/gzmo
  if python3 -c "
import json
d=json.load(open('$PI_MCP'))
for name,cfg in (d.get('mcpServers') or {}).items():
  if name!='gzmo-memory': continue
  env=cfg.get('env') or {}
  cfg_path=str(env.get('GZMO_CONFIG',''))
  if '/opt/gzmo' in cfg_path or 'data-next' in cfg_path:
    raise SystemExit(1)
raise SystemExit(0)
"; then
    row PASS "pi-product-boundary" "gzmo-memory not aimed at living/lab vault"
  else
    row FAIL "pi-product-boundary" "gzmo-memory GZMO_CONFIG points at living/lab — fix attach"
  fi
else
  row HOLD "pi-mcp-file" "no ~/.pi/agent/mcp.json — Pi glass not installed"
fi

[[ -f "$CURSOR_MCP" ]] && row PASS "cursor-mcp" "$CURSOR_MCP" || row HOLD "cursor-mcp" "no Cursor mcp.json"

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out = Path(os.environ["OUT"])
checks={}
for line in os.environ.get("ROWS_TSV","").splitlines():
    if not line.strip(): continue
    st,n,d=line.split("|",2); checks[n]={"status":st,"detail":d}
fail_n=int(os.environ["fail"]); hold_n=int(os.environ["hold"]); pass_n=int(os.environ["pass"])
verdict="GREEN" if fail_n==0 else "RED"
advice="pi_glass_ok" if fail_n==0 and hold_n==0 else ("pi_glass_hold — Pi optional" if fail_n==0 else "pi_glass_fail")
payload={"schema":"gzmo.unpark.pi_glass/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"1.2","checks":checks,
  "note":"CLI remains canonical operator UI."}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
