#!/usr/bin/env bash
# ARCH-DIR + Obolus sovereignty verification.
# Exit 0 = pass, 1 = fail, 2 = config/usage error.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
FAIL=0
WARN=0

pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; FAIL=1; }
warn() { echo "[WARN] $*"; WARN=$((WARN + 1)); }

toml_get() {
  python3 -c "
import tomllib, pathlib, sys
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
keys = sys.argv[1].split('.')
v = d
for k in keys:
    v = v.get(k) if isinstance(v, dict) else None
    if v is None:
        break
print(v if v is not None else '')
" "$1" 2>/dev/null || true
}

echo "GZMO sovereignty verify — $(date -Iseconds)"
echo "Root: $ROOT"
echo

# ─── ARCH-DIR checks 1–8 ─────────────────────────────────────────────

MODE="$(toml_get compliance.mode)"
[[ "$MODE" == "sovereign" ]] && pass "1 compliance.mode=sovereign" || fail "1 compliance.mode expected sovereign (got: $MODE)"

ACTIVE="$(toml_get engine.active_mode)"
ALLOW_CLOUD="$(toml_get compliance.allow_cloud_engine)"
if [[ "$ACTIVE" == "cloud" && "$ALLOW_CLOUD" != "True" && "$ALLOW_CLOUD" != "true" ]]; then
  fail "2 engine.active_mode=cloud but compliance.allow_cloud_engine=false"
elif [[ "$ACTIVE" == "cloud" ]]; then
  warn "2 engine.active_mode=cloud (allowed by compliance)"
else
  pass "2 engine not in cloud mode (active_mode=$ACTIVE)"
fi

TRUSTED="$(python3 -c "
import tomllib, pathlib
d = tomllib.loads(pathlib.Path('${ROOT}/gzmo.toml').read_text())
print(','.join(d.get('compliance', {}).get('trusted_cidrs', [])))
")"
if echo "$TRUSTED" | grep -q '192.168.31.0/24' && echo "$TRUSTED" | grep -q '127.0.0.0/8'; then
  pass "3 trusted_cidrs includes LAN + loopback"
else
  fail "3 trusted_cidrs missing expected CIDRs ($TRUSTED)"
fi

MAX_MB="$(toml_get compliance.max_binary_mb)"
MAX_MB="${MAX_MB:-80}"
if [[ -x "$BIN" ]]; then
  SIZE_MB=$(( $(stat -c%s "$BIN") / 1024 / 1024 ))
  if (( SIZE_MB <= MAX_MB )); then
    pass "4 binary size ${SIZE_MB}MB <= ${MAX_MB}MB"
  else
    fail "4 binary size ${SIZE_MB}MB exceeds max_binary_mb=${MAX_MB}"
  fi
else
  fail "4 gzmo binary not found — run cargo build --release"
fi

WS_DEPS="$(python3 -c "
import tomllib, pathlib, re
text = pathlib.Path('${ROOT}/Cargo.toml').read_text()
m = re.search(r'\[workspace\.dependencies\](.*?)(?=\n\[|\Z)', text, re.S)
print(len(re.findall(r'^\s*\w', m.group(1), re.M)) if m else 0)
")"
MAX_WS="$(toml_get compliance.max_workspace_deps)"
MAX_WS="${MAX_WS:-25}"
if (( WS_DEPS <= MAX_WS )); then
  pass "5 workspace.dependencies count=$WS_DEPS <= $MAX_WS"
else
  fail "5 workspace.dependencies count=$WS_DEPS > max_workspace_deps=$MAX_WS"
fi

if grep -rE '(sk-[a-zA-Z0-9]{20,}|api_key\s*=\s*\"[a-zA-Z0-9]{16,}\")' --include='*.toml' "$ROOT" 2>/dev/null | grep -v '.example' | grep -v '#'; then
  fail "6 possible inline secrets in committed toml"
else
  pass "6 no obvious inline secrets in toml"
fi

[[ -f "$ROOT/docs/ARCH-DIR-001-GZMO.md" ]] && pass "7 ARCH-DIR constitution present" || fail "7 missing docs/ARCH-DIR-001-GZMO.md"
[[ -f "$ROOT/docs/zero-bloat-reviews/BASELINE-2026-06.md" ]] && pass "8 zero-bloat baseline present" || fail "8 missing zero-bloat baseline"

# ─── Obolus checks 9–12 ────────────────────────────────────────────

OBOLUS_EN="$(toml_get obolus_analytics.enabled)"
[[ "$OBOLUS_EN" == "True" || "$OBOLUS_EN" == "true" ]] && pass "9 obolus_analytics.enabled" || fail "9 obolus_analytics disabled"

REQ_GOV="$(toml_get compliance.require_obolus_governance)"
GOV_EN="$(toml_get obolus_governance.enabled)"
if [[ "$REQ_GOV" == "True" || "$REQ_GOV" == "true" ]]; then
  [[ "$GOV_EN" == "True" || "$GOV_EN" == "true" ]] && pass "10 obolus_governance enabled (required)" || fail "10 obolus_governance required but disabled"
else
  pass "10 obolus_governance requirement not set"
fi

LEDGER="$(toml_get obolus_analytics.ledger_path)"
LEDGER_PATH="${ROOT}/${LEDGER:-data/Obolus/ledger.jsonl}"
mkdir -p "$(dirname "$LEDGER_PATH")"
touch "$LEDGER_PATH"
if [[ -w "$LEDGER_PATH" ]]; then
  pass "11 ledger path writable: $LEDGER_PATH"
else
  fail "11 ledger not writable: $LEDGER_PATH"
fi

if [[ -x "$BIN" ]]; then
  if "$BIN" obolus status >/dev/null 2>&1; then
    pass "12 obolus status CLI smoke"
  else
    fail "12 gzmo obolus status failed"
  fi
else
  fail "12 cannot run obolus status (no binary)"
fi

echo
if (( FAIL > 0 )); then
  echo "SOVEREIGNTY VERIFY: FAIL ($FAIL checks)"
  exit 1
fi
if (( WARN > 0 )); then
  echo "SOVEREIGNTY VERIFY: PASS with $WARN warning(s)"
  exit 0
fi
echo "SOVEREIGNTY VERIFY: PASS"
exit 0
