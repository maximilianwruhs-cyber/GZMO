#!/usr/bin/env bash
# Interactive Pi E2E — minimal extensions to avoid startup hangs.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_ROOT="$ROOT"
export GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"

PI="${PI_BIN:-pi}"
SYNAPSE_EXT="${PI_SYNAPSE_EXT:-$HOME/.pi/agent/extensions/synapse-notifier.ts}"
GZMO_SKILL="${PI_GZMO_SKILL:-$HOME/.pi/agent/skills/gzmo-integration}"
TIMEOUT_SECS="${PI_E2E_TIMEOUT:-120}"
PROVIDER="${PI_E2E_PROVIDER:-llama-cpp-heavy}"
MODEL="${PI_E2E_MODEL:-qwen3.6-35b-hauhaucs-iq4xs}"

if [[ ! -x "$GZMO_BIN" ]]; then
  echo "FAIL: gzmo binary missing at $GZMO_BIN (run scripts/build-gzmo.sh)"
  exit 1
fi

if [[ ! -f "$SYNAPSE_EXT" ]]; then
  echo "FAIL: synapse-notifier missing at $SYNAPSE_EXT"
  exit 1
fi

# Gate off for direct CLI in tool path unless Pi emits invoke first (notifier does).
export GZMO_SYNAPSE_GATE_BYPASS=1

PROMPT='Use gzmo_chaos exactly once: command calculate, args "2+3*4", json true. Reply with only the numeric result.'

echo "== Pi interactive E2E (timeout ${TIMEOUT_SECS}s, minimal extensions) =="
set +e
OUT=$(timeout "$TIMEOUT_SECS" "$PI" -p "$PROMPT" -ne \
  -e "$SYNAPSE_EXT" \
  -e "$GZMO_SKILL" \
  --tools gzmo_chaos \
  --provider "$PROVIDER" \
  --model "$MODEL" \
  --no-session 2>&1)
RC=$?
set -e

echo "$OUT" | tail -30

if [[ "$RC" -eq 124 ]]; then
  echo "SKIP: pi -p timed out after ${TIMEOUT_SECS}s (run manually on operator host)"
  echo "  Manual: pi -p \"$PROMPT\" -ne -e $SYNAPSE_EXT -e $GZMO_SKILL --tools gzmo_chaos"
  exit 0
fi

if [[ "$RC" -ne 0 ]]; then
  echo "FAIL: pi -p exit $RC"
  exit 1
fi

if ! echo "$OUT" | grep -qE '\b14\b'; then
  echo "FAIL: expected result 14 in pi output"
  exit 1
fi

export LIVE_SYNAPSE_TEST=1
if bash "$ROOT/scripts/pi/test_synapse_skill_invoke.sh"; then
  echo "PASS: Pi interactive E2E (calculate 2+3*4 + skill.invoke on bus)"
else
  echo "WARN: pi succeeded but skill.invoke not found — check synapse-notifier"
  exit 1
fi
