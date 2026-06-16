#!/usr/bin/env bash
# Live ObolusGate smoke: temporary E_total cap → T2 deny + Synapse obolus.denied.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
BACKUP="${ROOT}/gzmo.toml.bak-obolus-smoke"
SYNAPSE="${ROOT}/data/Synapse/events.jsonl"
MARKER="obolus-gate-smoke-$(date -u +%Y%m%dT%H%M%SZ)"

pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; exit 1; }

cleanup() {
  if [[ -f "$BACKUP" ]]; then
    mv -f "$BACKUP" "${ROOT}/gzmo.toml"
    echo "[restore] gzmo.toml restored"
  fi
}
trap cleanup EXIT

cp "${ROOT}/gzmo.toml" "$BACKUP"

CURRENT_E="$(python3 - <<'PY'
import tomllib, pathlib, subprocess, json
from datetime import datetime, timezone, timedelta
root = pathlib.Path(".")
cfg = tomllib.loads((root / "gzmo.toml").read_text())
# read ledger 1h E_total
ledger = root / cfg.get("obolus_analytics", {}).get("ledger_path", "data/Obolus/ledger.jsonl")
since = datetime.now(timezone.utc) - timedelta(hours=1)
e = 0
if ledger.exists():
    import json as j
    for line in ledger.read_text().splitlines():
        if not line.strip():
            continue
        row = j.loads(line)
        ts = datetime.fromisoformat(row["ts"].replace("Z", "+00:00"))
        if ts >= since:
            e += int(row.get("total_tokens", 0))
print(e)
PY
)"

if [[ "$CURRENT_E" -lt 1000 ]]; then
  fail "ledger 1h E_total too low ($CURRENT_E) — need recent Prime activity for smoke test"
fi

CAP=$(( CURRENT_E * 80 / 100 ))
echo "Obolus gate smoke — 1h E_total=$CURRENT_E → temp cap=$CAP ($MARKER)"

CAP="$CAP" python3 - <<'PY'
import os, pathlib, re
cap = os.environ["CAP"]
p = pathlib.Path("gzmo.toml")
text = p.read_text()
text = re.sub(
    r"(?m)^max_e_total_per_hour = \d+",
    f"max_e_total_per_hour = {cap}",
    text,
    count=1,
)
p.write_text(text)
PY

echo "--- balance (temp cap) ---"
"$BIN" obolus balance

echo "--- preflight operator_chat (expect Allow or Warn, exit 0) ---"
set +e
"$BIN" obolus preflight operator_chat
OP_RC=$?
set -e
[[ "$OP_RC" -eq 0 ]] || fail "operator_chat preflight must not hard-fail (got $OP_RC)"

echo "--- preflight spawn_discovery_fix (expect Deny, exit 1) ---"
set +e
OUT=$("$BIN" obolus preflight spawn_discovery_fix 2>&1)
T2_RC=$?
set -e
echo "$OUT"
[[ "$T2_RC" -eq 1 ]] || fail "spawn_discovery_fix must deny (got exit $T2_RC)"
echo "$OUT" | grep -qi deny || fail "preflight output missing Deny"

# Emit Synapse event (same helper as production hooks)
OBOLUS_SYNAPSE_SMOKE=1 OBOLUS_SMOKE_REASON="smoke: E_total ${CURRENT_E} > cap ${CAP} (${MARKER})" \
  cargo test -p gzmo-core --lib synapse_emit_denied_smoke -- --nocapture 2>&1 | tail -5

mkdir -p "$(dirname "$SYNAPSE")"
if grep -q "$MARKER" "$SYNAPSE" 2>/dev/null && grep -q 'obolus.denied' "$SYNAPSE" 2>/dev/null; then
  pass "Synapse contains obolus.denied with marker $MARKER"
  grep "$MARKER" "$SYNAPSE" | tail -1
else
  fail "obolus.denied not found in $SYNAPSE"
fi

pass "Obolus gate smoke complete"
