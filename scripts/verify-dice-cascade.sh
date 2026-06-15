#!/usr/bin/env bash
# Verify /dice Wild Magic cascade tables and wiring.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PASS=0
FAIL=0

ok() { echo "  OK  $1"; PASS=$((PASS + 1)); }
bad() { echo "  FAIL $1"; FAIL=$((FAIL + 1)); }

echo "=== dice cascade verify ==="

[[ -f data/dice_cascade.toml ]] && ok "data/dice_cascade.toml exists" || bad "missing dice_cascade.toml"
[[ -f gzmo-core/src/skills/dice_cascade.rs ]] && ok "dice_cascade.rs exists" || bad "missing dice_cascade.rs"

rg -q 'wild_magic' gzmo-core/src/skills/dice.rs && ok "dice evidence includes wild_magic" || bad "wild_magic missing in dice.rs"
rg -q 'dice_cascade' gzmo-chaos/src/thoughts.rs && ok "Thought Cabinet dice_cascade arm" || bad "no dice_cascade crystallization"
rg -q 'NestedDispatch' gzmo-core/src/skills/mod.rs && ok "NestedDispatch in SkillContext" || bad "NestedDispatch missing"
rg -q 'build_chaos_skill_registry' gzmo-cli/src/chat.rs && ok "chat uses full pantheon registry" || bad "chat partial registry"

if cargo test -p gzmo-core dice_cascade --quiet 2>/dev/null; then
  ok "cargo test dice_cascade"
else
  bad "cargo test dice_cascade"
fi

rg -q 'WUERFEL_CRON_SOURCE' gzmo-cli/src/daemon_cmd.rs && ok "chaos.dice_loop wuerfel-cron tag" || bad "missing wuerfel-cron synapse tag"

echo ""
echo "Result: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
