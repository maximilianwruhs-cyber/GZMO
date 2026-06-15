#!/usr/bin/env bash
# Extensive infrastructure stages test harness (plan stages 0-6).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
GZMO="${GZMO_BIN:-$ROOT/target/release/gzmo}"
export GZMO_ROOT="$ROOT"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/gzmo.toml}"

PASS=0
FAIL=0
SKIP=0
declare -a FAILURES=()

ok()   { echo "  PASS  $1"; PASS=$((PASS + 1)); }
bad()  { echo "  FAIL  $1"; FAIL=$((FAIL + 1)); FAILURES+=("$1"); }
skip() { echo "  SKIP  $1"; SKIP=$((SKIP + 1)); }

run_step() {
  local name="$1"
  shift
  echo ""
  echo "── $name ──"
  if "$@"; then
    ok "$name"
  else
    bad "$name"
  fi
}

echo "=============================================="
echo " GZMO Infrastructure Stages — Extensive Tests"
echo " ROOT=$ROOT"
echo "=============================================="

echo ""
echo "── Rust: gzmo-core full suite ──"
if cargo test -p gzmo-core --quiet 2>&1; then
  ok "cargo test -p gzmo-core (all)"
else
  bad "cargo test -p gzmo-core (all)"
fi

echo ""
echo "── Rust: infrastructure modules ──"
for filter in synapse::tests kurator_monitor calculate::tests bibliothek; do
  if cargo test -p gzmo-core "$filter" --quiet 2>&1; then
    ok "cargo test $filter"
  else
    bad "cargo test $filter"
  fi
done

echo ""
echo "── Stage 1: /calculate v2 CLI ──"
if [[ -x "$GZMO" ]]; then
  OUT=$("$GZMO" chaos skill calculate "2+3*4" --json 2>/dev/null || true)
  if echo "$OUT" | python3 -c "
import json,sys
raw=sys.stdin.read()
# find JSON in output
start=raw.find('{')
if start<0: sys.exit(1)
d=json.loads(raw[start:raw.rfind('}')+1])
assert d.get('skill')=='calculate'
assert d.get('version')==2
assert d.get('result') in ('14', 14) or str(d.get('result'))=='14'
steps=d.get('steps',[])
assert isinstance(steps, list)
" 2>/dev/null; then
    ok "calculate v2 JSON (2+3*4, steps, version 2)"
  else
    bad "calculate v2 JSON output"
    echo "    output snippet: $(echo "$OUT" | head -c 200)"
  fi
  BC=$("$GZMO" chaos skill calculate "2^10" 2>/dev/null || true)
  if echo "$BC" | grep -q '1024'; then
    ok "calculate numeric (2^10=1024)"
  else
    bad "calculate numeric 2^10"
  fi
else
  skip "calculate CLI (gzmo binary missing)"
fi

echo ""
echo "── Stage 4/5: CLI kurator + routing ──"
if [[ -x "$GZMO" ]]; then
  if "$GZMO" kurator status 2>&1 | grep -q 'enabled:'; then
    ok "gzmo kurator status"
  else
    bad "gzmo kurator status"
  fi
  if "$GZMO" kurator approve test-id 2>&1 | grep -qi 'no pending recommendation'; then
    ok "gzmo kurator approve (phase 2 — expects pending id)"
  else
    bad "gzmo kurator approve unexpected response"
  fi
else
  skip "kurator CLI"
fi

# Shell scripts (stages 0-6)
SCRIPTS=(
  "scripts/pi/validate_routing.sh"
  "scripts/pi/test_forum_romanum_schema.sh"
  "scripts/pi/test_forum_romanum_emit.sh"
  "scripts/pi/test_kurator_monitor.sh"
  "scripts/pi/test_synapse_session_correlation.sh"
  "scripts/pi/test_synapse_skill_invoke.sh"
  "scripts/pi/test_synapse_writer_gate.sh"
  "scripts/pi/test_synapse_writer_forum_romanum.sh"
  "scripts/verify-dice-cascade.sh"
  "scripts/verify-mcp-json.sh"
  "scripts/pi/test_distill_pi.sh"
  "scripts/pi/test_session_end_distill.sh"
  "scripts/pi/test_topic_shift_distill.sh"
)

for s in "${SCRIPTS[@]}"; do
  run_step "$s" bash "$ROOT/$s"
done

echo ""
echo "── Stage 0: synapse-notifier + forum-romanum bridge sync ──"
REF="$ROOT/scripts/pi/synapse-notifier.reference.ts"
LIVE="${PI_SYNAPSE_NOTIFIER:-$HOME/.pi/agent/extensions/synapse-notifier.ts}"
FR_REF="$ROOT/scripts/pi/forum-romanum-bridge.reference.ts"
FR_LIVE="${PI_FORUM_BRIDGE:-$HOME/.pi/agent/extensions/forum-romanum-bridge.ts}"
if [[ -f "$REF" ]]; then
  ok "reference copy exists"
  for token in session_id skill.invoke correlation_id emitted_by; do
    if grep -q "$token" "$REF"; then
      ok "reference contains $token"
    else
      bad "reference missing $token"
    fi
  done
  if [[ -f "$LIVE" ]] && diff -q "$REF" "$LIVE" >/dev/null 2>&1; then
    ok "live Pi notifier matches reference"
  elif [[ -f "$LIVE" ]]; then
    skip "live Pi notifier differs from reference (copy if intended)"
  else
    skip "live Pi notifier not found at $LIVE"
  fi
else
  bad "synapse-notifier.reference.ts missing"
fi
if [[ -f "$FR_REF" ]]; then
  ok "forum-romanum reference exists"
  for token in agent.message proposal.created crewHooks; do
    if grep -q "$token" "$FR_REF"; then
      ok "forum reference contains $token"
    else
      bad "forum reference missing $token"
    fi
  done
  if [[ -f "$FR_LIVE" ]] && diff -q "$FR_REF" "$FR_LIVE" >/dev/null 2>&1; then
    ok "live Pi forum-romanum bridge matches reference"
  elif [[ -f "$FR_LIVE" ]]; then
    skip "live Pi forum bridge differs from reference (copy if intended)"
  else
    skip "live Pi forum bridge not found at $FR_LIVE"
  fi
else
  bad "forum-romanum-bridge.reference.ts missing"
fi

echo ""
echo "── Config: infrastructure sections ──"
for section in kurator bibliothek synapse_pull routing; do
  if grep -q "^\[$section\]" "$GZMO_CONFIG" 2>/dev/null || grep -q "^\[routing" "$GZMO_CONFIG" 2>/dev/null; then
    ok "gzmo.toml has $section"
  else
    bad "gzmo.toml missing $section"
  fi
done

echo ""
echo "── Docs contract files ──"
for doc in SYNAPSE_EVENT_OWNERSHIP.md FORUM_ROMANUM_SCHEMA.md CALCULATE_V2_FORMATTER_CONTRACT.md OBOLUS_ROUTING.md WUERFEL_DICE_LOOP.md; do
  if [[ -f "$ROOT/docs/$doc" ]]; then
    ok "docs/$doc"
  else
    bad "docs/$doc missing"
  fi
done

echo ""
echo "── Mentor socket (optional integration) ──"
if [[ -S "$ROOT/data/gzmo_mentor.sock" ]]; then
  run_step "scripts/pi/test_mentor_dialog.sh" bash "$ROOT/scripts/pi/test_mentor_dialog.sh"
  if [[ -f "$ROOT/scripts/pi/test_mcp_mentor.py" ]]; then
    run_step "test_mcp_mentor.py" python3 "$ROOT/scripts/pi/test_mcp_mentor.py"
  fi
else
  skip "mentor socket tests (start gzmo-daemon)"
fi

echo ""
echo "── Pedagogy graph validate ──"
if [[ -x "$GZMO" ]]; then
  run_step "pedagogy graph validate" "$GZMO" pedagogy graph validate "$ROOT/data/pedagogy/graphs/"
else
  skip "pedagogy graph validate"
fi

echo ""
echo "=============================================="
echo " SUMMARY: $PASS passed, $FAIL failed, $SKIP skipped"
echo "=============================================="
if [[ "$FAIL" -gt 0 ]]; then
  echo "Failures:"
  for f in "${FAILURES[@]}"; do echo "  - $f"; done
  exit 1
fi
exit 0
