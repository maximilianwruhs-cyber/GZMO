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
DOCTRINE="$ROOT/docs/OPERATOR_FRONTEND_DECISION.md"
if [[ -f "$DOCTRINE" ]] && python3 -c "
from pathlib import Path
t=Path('$DOCTRINE').read_text(encoding='utf-8').lower()
need=['gzmo_cli','canonical','optional auxiliary']
raise SystemExit(0 if all(x in t for x in need) else 1)
"; then
  row PASS "doctrine" "CLI canonical; Pi optional glass"
else
  row FAIL "doctrine" "OPERATOR_FRONTEND_DECISION.md missing CLI-canonical phrases"
fi
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
# Missing boundary row = Pi not installed (ok). Only FAIL is a hard boundary break.
bnd = checks.get("pi-product-boundary",{}).get("status")
boundary_ok = bnd != "FAIL"
cli_ok = checks.get("doctrine",{}).get("status")=="PASS"
attach_ok = checks.get("product-attach",{}).get("status")=="PASS"
advice="pi_glass_ok" if fail_n==0 and hold_n==0 else ("pi_glass_hold — Pi optional" if fail_n==0 else "pi_glass_fail")
payload={"schema":"gzmo.unpark.pi_glass/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"1.2","checks":checks,
  "note":"CLI remains canonical operator UI.",
  "cli_canonical": cli_ok,
  "product_attach_ok": attach_ok,
  "product_boundary_ok": boundary_ok}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
surface={
  "schema":"gzmo.unpark.pi_glass.surface/v1",
  "generated_at":payload["generated_at"],
  "wave":"1.2",
  "cli_canonical": cli_ok,
  "product_attach_ok": attach_ok,
  "product_boundary_ok": boundary_ok,
  "pi_optional": True,
  "ok": fail_n==0 and cli_ok and attach_ok and boundary_ok,
  "advice": "pi_surface_ok — CLI canonical; Pi optional glass" if fail_n==0 and cli_ok and attach_ok else advice,
}
(out/"surface.json").write_text(json.dumps(surface,indent=2)+"\n")
if not surface["ok"] and fail_n==0:
    # surface incomplete without FAIL rows → HOLD advice already set
    pass
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n,
                  "surface":str(out/"surface.json")},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
