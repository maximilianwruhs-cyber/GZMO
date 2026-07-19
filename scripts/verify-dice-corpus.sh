#!/usr/bin/env bash
# Verify /dice event corpus (Slice A.0 — TOML + dice_corpus on main skill trait).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOML="$ROOT/data/dice_events.toml"
DICE_RS="$ROOT/gzmo-core/src/skills/dice.rs"
PASS=0
FAIL=0

pass() { echo "  OK: $1"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $1"; FAIL=$((FAIL + 1)); }

echo "== /dice corpus verification (Slice A.0) =="

if [[ -f "$TOML" ]]; then
  pass "data/dice_events.toml exists"
else
  fail "missing data/dice_events.toml"
fi

read -r D20_TIERS D6_TIERS TOTAL <<< "$(python3 - "$TOML" <<'PY'
import sys, tomllib
from pathlib import Path
p = Path(sys.argv[1])
data = tomllib.loads(p.read_text())
d20 = data.get("d20", {})
d6 = data.get("d6", {})
n20 = sum(len(v.get("variants", [])) for v in d20.values())
n6 = sum(len(v.get("variants", [])) for v in d6.values())
print(len(d20), len(d6), n20 + n6)
PY
)"

[[ "$D20_TIERS" -eq 20 ]] && pass "TOML d20 tiers = 20" || fail "TOML d20 tiers (got $D20_TIERS)"
[[ "$D6_TIERS" -eq 6 ]] && pass "TOML d6 tiers = 6" || fail "TOML d6 tiers (got $D6_TIERS)"
[[ "$TOTAL" -ge 118 ]] && pass "TOML event strings = $TOTAL" || fail "TOML events (got $TOTAL, want >= 118)"

if [[ -f "$ROOT/skills/skill_dice.sh" ]]; then
  pass "legacy skill_dice.sh present"
else
  fail "missing skills/skill_dice.sh"
fi

if rg -q 'pub mod dice_corpus' "$ROOT/gzmo-core/src/skills/mod.rs" \
  && rg -q 'include_str!' "$ROOT/gzmo-core/src/skills/dice_corpus.rs" \
  && rg -q 'dice_event\(' "$DICE_RS"; then
  pass "dice_corpus wired into /dice"
else
  fail "dice_corpus module not wired into /dice"
fi

if rg -q 'fn d20_event|fn d6_event|fn get_event' "$DICE_RS"; then
  fail "hardcoded event pools still in dice.rs"
else
  pass "hardcoded event pools removed from dice.rs"
fi

# Partial mechanical coverage on main (full 1–20 + D6 is feat Slice A)
MECH=$(rg -c 'ChaosEvent::Custom' "$DICE_RS" || true)
[[ "${MECH:-0}" -ge 5 ]] && pass "tier_mechanical_effect has Custom arms ($MECH)" \
  || fail "tier_mechanical_effect missing Custom arms"

echo ""
echo "Result: $PASS passed, $FAIL failed"
[[ $FAIL -eq 0 ]]
