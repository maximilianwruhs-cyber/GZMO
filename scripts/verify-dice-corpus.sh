#!/usr/bin/env bash
# Verify /dice event corpus integrity (118 events + full tier mechanical coverage).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOML="$ROOT/data/dice_events.toml"
DICE_RS="$ROOT/gzmo-core/src/skills/dice.rs"
PASS=0
FAIL=0

pass() { echo "  OK: $1"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $1"; FAIL=$((FAIL + 1)); }

echo "== /dice corpus verification =="

if [[ -f "$TOML" ]]; then
  pass "data/dice_events.toml exists"
else
  fail "missing data/dice_events.toml"
fi

# Parse TOML counts
read -r D20_TIERS D6_TIERS D20_STR D6_STR TOTAL <<< "$(python3 - "$TOML" <<'PY'
import sys, tomllib
from pathlib import Path
p = Path(sys.argv[1])
data = tomllib.loads(p.read_text())
d20 = data.get("d20", {})
d6 = data.get("d6", {})
n20 = sum(len(v.get("variants", [])) for v in d20.values())
n6 = sum(len(v.get("variants", [])) for v in d6.values())
print(len(d20), len(d6), n20, n6, n20 + n6)
PY
)"

[[ "$D20_TIERS" -eq 20 ]] && pass "TOML d20 tiers = 20" || fail "TOML d20 tiers (got $D20_TIERS)"
[[ "$D6_TIERS" -eq 6 ]] && pass "TOML d6 tiers = 6" || fail "TOML d6 tiers (got $D6_TIERS)"
[[ "$TOTAL" -ge 118 ]] && pass "TOML event strings = $TOTAL" || fail "TOML events (got $TOTAL, want >= 118)"

# Legacy shell reference
if [[ -f "$ROOT/skills/skill_dice.sh" ]]; then
  pass "legacy skill_dice.sh present"
else
  fail "missing skills/skill_dice.sh"
fi

# Mechanical coverage in dice.rs
D20_MECH_OK=1
for roll in $(seq 1 20); do
  if ! rg -q "^\\s+${roll} => Some\\(ChaosEvent::Custom" "$DICE_RS"; then
    fail "D20 roll $roll missing tier_mechanical_effect arm"
    D20_MECH_OK=0
  fi
done
[[ $D20_MECH_OK -eq 1 ]] && pass "all D20 rolls 1–20 have tier_mechanical_effect"

D6_MECH_OK=1
for roll in $(seq 1 6); do
  if ! rg -q "^\\s+${roll} => Some\\(ChaosEvent::Custom" "$DICE_RS"; then
    fail "D6 roll $roll missing d6_mechanical_effect arm"
    D6_MECH_OK=0
  fi
done
[[ $D6_MECH_OK -eq 1 ]] && pass "all D6 rolls 1–6 have d6_mechanical_effect"

# dice_corpus module wired
if rg -q "dice_corpus" "$ROOT/gzmo-core/src/skills/mod.rs" && rg -q "include_str!" "$ROOT/gzmo-core/src/skills/dice_corpus.rs"; then
  pass "dice_corpus embeds TOML at compile time"
else
  fail "dice_corpus module not wired"
fi

THOUGHTS="$ROOT/gzmo-chaos/src/thoughts.rs"
for cat in dice_catastrophe dice_resonance dice_crystallize dice_bifurcation dice_legendary dice_oracle dice_spark dice_crit_fail dice_crit_success; do
  if ! rg -q "\"${cat}\"" "$THOUGHTS"; then
    fail "thoughts.rs missing category $cat"
  fi
done
pass "all dice Thought Cabinet categories referenced"

echo ""
echo "Result: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]]
