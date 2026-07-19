#!/usr/bin/env bash
# Verify headless mentor client — Wave 2b demable surface (no TUI/chat wire).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PASS=0
FAIL=0

ok() { echo "  OK  $1"; PASS=$((PASS + 1)); }
bad() { echo "  FAIL $1"; FAIL=$((FAIL + 1)); }

echo "=== mentor verify (headless pedagogy client) ==="

[[ -f gzmo-cli/src/mentor_cmd.rs ]] && ok "mentor_cmd.rs exists" || bad "missing mentor_cmd.rs"
[[ -f gzmo-cli/src/mentor_ipc.rs ]] && ok "mentor_ipc.rs exists" || bad "missing mentor_ipc.rs"
[[ -f gzmo-cli/src/pedagogy_bridge.rs ]] && ok "pedagogy_bridge.rs exists" || bad "missing pedagogy_bridge.rs"
rg -q 'mod mentor_cmd' gzmo-cli/src/main.rs && ok "main wires mentor_cmd" || bad "main missing mentor_cmd"
rg -q 'Command::Mentor' gzmo-cli/src/main.rs && ok "Command::Mentor present" || bad "Mentor command missing"
rg -q 'maybe_teach' gzmo-cli/src/pedagogy_bridge.rs && ok "PedagogyRuntime::maybe_teach present" || bad "maybe_teach missing"
rg -q 'maybe_teach' gzmo-cli/src/chat.rs && ok "chat.rs wires maybe_teach (Wave 2b)" \
  || bad "chat.rs missing maybe_teach"
rg -q 'should_delegate_exec' gzmo-cli/src/chat.rs && ok "chat.rs ops-delegates before mentor" \
  || bad "chat.rs missing should_delegate_exec"
rg -q 'maybe_teach' gzmo-cli/src/tui -g '*.rs' && ok "tui wires maybe_teach (Wave 2b.1)" \
  || bad "tui missing maybe_teach"
rg -q 'should_delegate_exec' gzmo-cli/src/tui/components/agent.rs \
  && ok "tui ops-delegates before mentor" \
  || bad "tui missing should_delegate_exec"

TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"
GZMO_BIN="${GZMO_BIN:-}"
if [[ -z "$GZMO_BIN" ]]; then
  for cand in \
    "$TARGET_DIR/release/gzmo" \
    "$TARGET_DIR/debug/gzmo" \
    "$ROOT/target/release/gzmo" \
    "$ROOT/target/debug/gzmo"; do
    if [[ -x "$cand" ]]; then GZMO_BIN="$cand"; break; fi
  done
fi

if [[ -z "${GZMO_BIN:-}" || ! -x "$GZMO_BIN" ]]; then
  echo "  … building gzmo-cli (debug) for smoke …"
  if cargo build -p gzmo-cli --quiet 2>/tmp/verify-mentor-build.log; then
    GZMO_BIN="$ROOT/target/debug/gzmo"
    [[ -x "$TARGET_DIR/debug/gzmo" ]] && GZMO_BIN="$TARGET_DIR/debug/gzmo"
    ok "cargo build gzmo-cli"
  else
    bad "cargo build gzmo-cli (see /tmp/verify-mentor-build.log)"
  fi
fi

if [[ -n "${GZMO_BIN:-}" && -x "$GZMO_BIN" ]]; then
  PING="$("$GZMO_BIN" mentor ping 2>/dev/null || true)"
  if echo "$PING" | rg -q 'pong'; then
    ok "gzmo mentor ping → pong"
  else
    bad "gzmo mentor ping (stale binary? rebuild)"
  fi
  STATUS="$("$GZMO_BIN" mentor status 2>/dev/null || true)"
  if echo "$STATUS" | rg -q 'learner=|ops_mode='; then
    ok "gzmo mentor status"
  else
    bad "gzmo mentor status"
  fi
else
  bad "no gzmo binary available for runtime smoke"
fi

echo ""
echo "Result: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
