#!/usr/bin/env bash
# Extended infrastructure tests — Kurator thresholds, Bibliothek, bus contracts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
GZMO="${GZMO_BIN:-$ROOT/target/release/gzmo}"
BUS="${GZMO_SYNAPSE_BUS:-$ROOT/data/Synapse/events.jsonl}"
export GZMO_ROOT="$ROOT"

PASS=0
FAIL=0
SKIP=0
failures=()

ok()  { echo "  PASS  $1"; PASS=$((PASS + 1)); }
bad() { echo "  FAIL  $1"; FAIL=$((FAIL + 1)); failures+=("$1"); }
skip(){ echo "  SKIP  $1"; SKIP=$((SKIP + 1)); }

echo "=============================================="
echo " GZMO Infrastructure — Extended Tests"
echo "=============================================="

echo ""
echo "── Kurator spawn.recommended (integration unit) ──"
if cargo test -p gzmo-core spawn_recommended_emitted --quiet 2>&1; then
  ok "kurator spawn.recommended integration (unit)"
else
  bad "kurator spawn.recommended integration"
fi

echo ""
echo "── Bibliothek state + promotion gate ──"
BIB="$ROOT/data/bibliothek_state.json"
if [[ -f "$BIB" ]]; then
  CYCLES=$(python3 -c "import json; print(json.load(open('$BIB')).get('dream_cycles_completed',0))")
  ok "bibliothek_state.json exists (cycles=$CYCLES)"
else
  skip "bibliothek_state.json (no dream run yet)"
fi
if cargo test -p gzmo-core promotion_gate --quiet 2>&1; then
  ok "bibliothek promotion_gate unit test"
else
  bad "bibliothek promotion_gate"
fi

echo ""
echo "── wuerfel-cron on live bus ──"
if grep -q 'wuerfel-cron' "$BUS" 2>/dev/null || tail -2000 "$BUS" | grep -q '"source":"wuerfel-cron"'; then
  ok "wuerfel-cron present on bus (historical or recent)"
else
  skip "wuerfel-cron not in bus tail (dice loop may not have fired recently)"
fi

echo ""
echo "── Forum Romanum legacy + envelope deserialize ──"
if cargo test -p gzmo-core test_legacy_event_without_envelope --quiet 2>&1; then
  ok "legacy synapse events deserialize"
else
  bad "legacy deserialize"
fi

echo ""
echo "── Calculate edge cases ──"
for expr in "sqrt(144)" "2^10" "(2+3)*4"; do
  if OUT=$("$GZMO" chaos skill calculate "$expr" --json 2>/dev/null); then
    RES=$(echo "$OUT" | python3 -c "import json,sys; r=sys.stdin.read(); i=r.find('{'); d=json.loads(r[i:r.rfind('}')+1]); print(d.get('result',''))" 2>/dev/null || echo "")
    if [[ -n "$RES" ]]; then
      ok "calculate $expr -> $RES"
    else
      bad "calculate $expr (no result)"
    fi
  else
    bad "calculate $expr (command failed)"
  fi
done

echo ""
echo "── remediation-verify.sh ──"
if bash "$ROOT/scripts/remediation-verify.sh" 2>&1 | tail -8; then
  ok "remediation-verify.sh"
else
  bad "remediation-verify.sh"
fi

echo ""
echo "── verify-dice-cascade + production checks ──"
bash "$ROOT/scripts/verify-dice-cascade.sh" && ok "verify-dice-cascade" || bad "verify-dice-cascade"
if [[ -x "$ROOT/scripts/verify-production.sh" ]]; then
  if bash "$ROOT/scripts/verify-production.sh" 2>&1 | tail -5; then
    ok "verify-production.sh"
  else
    bad "verify-production.sh"
  fi
else
  skip "verify-production.sh"
fi

echo ""
echo "── Live fixture round 2 ──"
export LIVE_SYNAPSE_TEST=1
FIXTURE=$(python3 "$ROOT/scripts/pi/emit_synapse_live_fixture.py")
echo "$FIXTURE" | sed -n 's/^SESSION_ID=//p' | read -r SYNAPSE_TEST_SESSION_ID || true
export SYNAPSE_TEST_SESSION_ID=$(echo "$FIXTURE" | sed -n 's/^SESSION_ID=//p')
bash "$ROOT/scripts/pi/test_synapse_session_correlation.sh"
bash "$ROOT/scripts/pi/test_synapse_skill_invoke.sh"
ok "live fixture round 2 + strict scripts"

echo ""
echo "── smoke.sh ──"
if bash "$ROOT/scripts/pi/smoke.sh" 2>&1 | tail -3; then
  ok "smoke.sh"
else
  bad "smoke.sh"
fi

echo ""
echo "=============================================="
echo " SUMMARY: $PASS passed, $FAIL failed, $SKIP skipped"
echo "=============================================="
[[ "$FAIL" -eq 0 ]] || { printf ' - %s\n' "${failures[@]}"; exit 1; }
