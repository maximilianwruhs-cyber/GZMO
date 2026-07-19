#!/usr/bin/env bash
# Verify /dice Wild Magic cascade — Slice A.3 nested dispatch.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PASS=0
FAIL=0

ok() { echo "  OK  $1"; PASS=$((PASS + 1)); }
bad() { echo "  FAIL $1"; FAIL=$((FAIL + 1)); }

echo "=== dice cascade verify (Slice A.3 nested execute) ==="

[[ -f data/dice_cascade.toml ]] && ok "data/dice_cascade.toml exists" || bad "missing dice_cascade.toml"
[[ -f gzmo-core/src/skills/dice_cascade.rs ]] && ok "dice_cascade.rs exists" || bad "missing dice_cascade.rs"

rg -q 'pub mod dice_cascade' gzmo-core/src/skills/mod.rs && ok "mod dice_cascade wired" || bad "mod not wired"
rg -q 'plan_cascade' gzmo-core/src/skills/dice.rs && ok "dice.rs plans cascade" || bad "dice.rs missing plan_cascade"
rg -q 'execute_cascade' gzmo-core/src/skills/dice.rs && ok "dice.rs executes nested cascade" || bad "dice.rs missing execute_cascade"
rg -q 'NestedDispatch' gzmo-core/src/skills/dice_cascade.rs && ok "cascade receives nested dispatch" || bad "cascade missing NestedDispatch"
rg -q 'dispatch::dispatch_skill' gzmo-core/src/skills/dice_cascade.rs && ok "cascade dispatches selected skill" || bad "cascade missing dispatch_skill"

if cargo test -p gzmo-core dice_cascade --quiet 2>/dev/null; then
  ok "cargo test dice_cascade"
else
  bad "cargo test dice_cascade"
fi

echo ""
echo "Result: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
