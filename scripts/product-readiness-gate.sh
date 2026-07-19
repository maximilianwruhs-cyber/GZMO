#!/usr/bin/env bash
# Product production-readiness gate (laptop Memory MCP + living owner separation).
# Exit 0 = PRODUCT GREEN. Soft living CT101 lane is reported but does not fail the gate
# unless PRODUCT_GATE_REQUIRE_CT101=1.
#
#   bash scripts/product-readiness-gate.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/product-readiness"
BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
mkdir -p "$OUT"
LOG="$OUT/gate.log"
: >"$LOG"

pass=0
fail=0
hold=0
declare -a ROWS=()

row() {
  local status="$1" name="$2" detail="$3"
  ROWS+=("$status|$name|$detail")
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
    HOLD) hold=$((hold + 1)) ;;
  esac
  echo "[$status] $name — $detail" | tee -a "$LOG"
}

echo "=== Product readiness gate ===" | tee -a "$LOG"

# 1) Binary
if [[ -x "$BIN" ]]; then
  row PASS "gzmo-binary" "$BIN"
else
  row FAIL "gzmo-binary" "missing — build release gzmo-cli"
fi

# 2) Cold product MCP
if [[ -x "$BIN" ]] && KEEP_VERIFY_DIR=1 VERIFY_DIR="$OUT/product-verify" \
  GZMO_BIN="$BIN" bash "$ROOT/scripts/verify-product-mcp.sh" >>"$LOG" 2>&1; then
  row PASS "verify-product-mcp" "cold init/status/search/mcp tools"
else
  row FAIL "verify-product-mcp" "see gate.log"
fi

# 3) MCP attach
bash "$ROOT/scripts/mcp-attach-check.sh" >>"$LOG" 2>&1 || true
if python3 -c "import json;d=json.load(open('$DATA/mcp-attach/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "mcp-attach" "$(python3 -c "import json;print(json.load(open('$DATA/mcp-attach/latest.json')).get('advice',''))")"
else
  row FAIL "mcp-attach" "Cursor/Pi not on ~/.gzmo — MCP_ATTACH_FIX=1 bash scripts/mcp-attach-check.sh"
fi

# 4) Product home hygiene (no LAN / CT101 in product config)
HOME_CFG="${HOME}/.gzmo/gzmo.toml"
if [[ -f "$HOME_CFG" ]]; then
  if rg -n '192\.168|/opt/gzmo|CT101|neo4j\.|qdrant\.|:6333|:7474' "$HOME_CFG" >/dev/null 2>&1; then
    row FAIL "product-config-hygiene" "LAN/CT101/sidecar hosts found in ~/.gzmo/gzmo.toml"
  else
    row PASS "product-config-hygiene" "no LAN/CT101 hosts in product toml"
  fi
  # Engine reachable?
  ENG="$(python3 - <<'PY'
import re
from pathlib import Path
t=Path.home().joinpath(".gzmo/gzmo.toml").read_text()
m=re.search(r'(?m)^\s*url\s*=\s*"([^"]+)"', t)
print(m.group(1) if m else "")
PY
)"
  if [[ -n "$ENG" ]]; then
    if curl -fsS --max-time 2 "${ENG%/}/models" >/dev/null 2>&1; then
      row PASS "product-engine" "$ENG reachable"
    else
      row HOLD "product-engine" "$ENG not reachable — first-fact/metabolize will need Prime"
    fi
  else
    row HOLD "product-engine" "no [engine].url in ~/.gzmo/gzmo.toml"
  fi
else
  row HOLD "product-config-hygiene" "~/.gzmo missing — run install-gzmo.sh / gzmo init"
  row HOLD "product-engine" "no product home"
fi

# 5) refresh-engine (non-destructive) when binary + engine up
if [[ -x "$BIN" && -f "$HOME_CFG" ]]; then
  if "$BIN" init --refresh-engine >>"$LOG" 2>&1; then
    row PASS "refresh-engine" "gzmo init --refresh-engine ok"
  else
    row HOLD "refresh-engine" "could not refresh (engine down or init error)"
  fi
else
  row HOLD "refresh-engine" "skipped"
fi

# 6) Hello / first fact
bash "$ROOT/scripts/product-hello-memory.sh" >>"$LOG" 2>&1 || true
if python3 -c "import json;d=json.load(open('$DATA/product-hello/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "product-hello" "attach + first fact remembered"
else
  row HOLD "product-hello" "needs live engine for metabolize (Prime :8000)"
fi

# 7) Scanner unit preference
if (cd "$ROOT" && CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}" \
  cargo test -p gzmo-core prefer_product_engine -q >>"$LOG" 2>&1); then
  row PASS "prefer-prime-tests" "scanner prefers :8000"
else
  row FAIL "prefer-prime-tests" "cargo test prefer_product_engine failed"
fi

# 8) Living owner separation (operator lane)
bash "$ROOT/scripts/ct101-living-probe.sh" >>"$LOG" 2>&1 || true
CT_OK=0
if python3 -c "import json;d=json.load(open('$DATA/ct101-living/latest.json')); raise SystemExit(0 if d.get('living_proof') else 1)"; then
  CT_OK=1
  row PASS "ct101-living-owner" "CT101 smoke PASS; workstation serve inactive"
else
  if [[ "${PRODUCT_GATE_REQUIRE_CT101:-0}" == "1" ]]; then
    row FAIL "ct101-living-owner" "required by PRODUCT_GATE_REQUIRE_CT101=1"
  else
    row HOLD "ct101-living-owner" "optional operator lane — not required for laptop product GREEN"
  fi
fi

# Verdict: product GREEN if no FAIL. HOLD is ok for optional engine/CT101.
export OUT pass fail hold CT_OK
set +e
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["pass"])
fail_n = int(os.environ["fail"])
hold_n = int(os.environ["hold"])
verdict = "GREEN" if fail_n == 0 else "RED"
advice = (
    "product_ready — laptop Memory MCP production gate GREEN"
    if verdict == "GREEN"
    else "product_hold — fix FAIL rows before claiming production readiness"
)
payload = {
    "schema": "gzmo.product.readiness/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "ct101_living_proof": os.environ.get("CT_OK") == "1",
    "note": "Product GREEN = stranger laptop MCP path. Living CT101 is separate owner lane.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

{
  echo "# Product readiness gate"
  echo
  echo "Verdict: **$(python3 -c "import json;print(json.load(open('$OUT/latest.json'))['verdict'])")**"
  echo
  echo "| Status | Check | Detail |"
  echo "|--------|-------|--------|"
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    echo "| $st | $name | $detail |"
  done
  echo
  echo "See also: docs/PRODUCT_PRODUCTION_READINESS.md"
  echo
} >"$OUT/latest.md"

echo "=== gate done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
