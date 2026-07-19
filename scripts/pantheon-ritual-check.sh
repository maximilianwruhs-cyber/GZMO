#!/usr/bin/env bash
# Unpark Wave 2.1: pantheon ritual front-door readiness (no ghost DICE_MASTER_*).
#   bash scripts/pantheon-ritual-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/pantheon-ritual"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== Pantheon ritual check (Unpark W2.1) ==="
[[ -f "$ROOT/docs/PANTHEON_SKILLS.md" ]] && row PASS "front-door" "PANTHEON_SKILLS.md" || row FAIL "front-door" "missing"
[[ -d "$ROOT/docs/research/pantheon" ]] && row PASS "archive" "docs/research/pantheon/" || row HOLD "archive" "archive dir missing"

for sk in dice.rs card.rs story.rs; do
  if [[ -f "$ROOT/gzmo-core/src/skills/$sk" ]]; then
    row PASS "skill:$sk" "Slice A skill present on main"
  else
    row HOLD "skill:$sk" "missing on main"
  fi
done

# Ghost masters must not exist as files; docs may warn against inventing them
if compgen -G "$ROOT/docs/DICE_MASTER*" >/dev/null \
  || compgen -G "$ROOT/skills/DICE_MASTER*" >/dev/null; then
  row FAIL "ghost-masters" "DICE_MASTER_* file invented — remove it"
elif rg -n 'DICE_MASTER_HANDOFF|DICE_MASTER_' "$ROOT/docs" "$ROOT/skills" 2>/dev/null \
  | grep -Eiv 'never existed|ghost|do not invent|don'\''t invent' >/dev/null; then
  row HOLD "ghost-masters" "DICE_MASTER_* mentions found — verify not invented as files"
else
  row PASS "ghost-masters" "no invented DICE_MASTER_* files"
fi

# Feat stack: A.0 corpus → A.1 plan → A.2/A.3 nested cascade → A.4 forge
if [[ -f "$ROOT/gzmo-core/src/skills/dispatch.rs" \
   && -f "$ROOT/gzmo-core/src/skills/card_forge.rs" \
   && -f "$ROOT/gzmo-core/src/skills/attractor_common.rs" ]]; then
  row PASS "feat-stack" "Slice A full — dispatch + nested cascade + card_forge"
elif [[ -f "$ROOT/data/dice_events.toml" && -f "$ROOT/gzmo-core/src/skills/dice_corpus.rs" \
   && -f "$ROOT/gzmo-core/src/skills/dice_cascade.rs" && -f "$ROOT/data/dice_cascade.toml" ]]; then
  row PASS "feat-stack" "Slice A.0+A.1 corpus + cascade (nested/forge pending)"
elif [[ -f "$ROOT/data/dice_events.toml" && -f "$ROOT/gzmo-core/src/skills/dice_corpus.rs" ]]; then
  row PASS "feat-stack" "Slice A.0 dice_corpus + dice_events.toml"
elif [[ -f "$ROOT/gzmo-core/src/skills/dice_loop.rs" ]] || [[ -f "$ROOT/data/dice_events.toml" ]]; then
  row PASS "feat-stack" "feat-adjacent files present"
else
  row HOLD "feat-stack" "feat attractor stack not on main — ritual PR pending"
fi

if [[ -x "$ROOT/scripts/verify-dice-cascade.sh" ]] \
  && bash "$ROOT/scripts/verify-dice-cascade.sh" >/tmp/pantheon-cascade-verify.log 2>&1; then
  if rg -q 'execute_cascade' "$ROOT/gzmo-core/src/skills/dice.rs" 2>/dev/null; then
    row PASS "cascade-verify" "verify-dice-cascade.sh ok (nested execute)"
  else
    row PASS "cascade-verify" "verify-dice-cascade.sh ok (plan-only)"
  fi
else
  row HOLD "cascade-verify" "verify-dice-cascade not green — see /tmp/pantheon-cascade-verify.log"
fi

[[ -f "$ROOT/docs/CHAOS_LIVING_VS_RITUAL.md" ]] && row PASS "chaos-boundary" "ritual ≠ living KPI" || row FAIL "chaos-boundary" "missing"

if [[ -x "$ROOT/scripts/verify-chaos-skill.sh" ]] \
  && bash "$ROOT/scripts/verify-chaos-skill.sh" >/tmp/pantheon-chaos-skill-verify.log 2>&1; then
  row PASS "chaos-skill-verify" "verify-chaos-skill.sh ok (C.0.1)"
else
  row HOLD "chaos-skill-verify" "verify-chaos-skill not green — see /tmp/pantheon-chaos-skill-verify.log"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out=Path(os.environ["OUT"]); checks={}
os.environ.setdefault("OUT", str(out))
for line in os.environ.get("ROWS_TSV","").splitlines():
    if not line.strip(): continue
    st,n,d=line.split("|",2); checks[n]={"status":st,"detail":d}
fail_n=int(os.environ["fail"]); hold_n=int(os.environ["hold"]); pass_n=int(os.environ["pass"])
verdict="GREEN" if fail_n==0 else "RED"
# Pantheon skills + front door = wave implemented; C.1 / daemon fire may still HOLD
skills_ok = all(checks.get(f"skill:{s}",{}).get("status")=="PASS" for s in ("dice.rs","card.rs","story.rs"))
demo = Path(os.environ.get("OUT","")).joinpath("demo.json")
demo_ok = demo.is_file()
if fail_n==0 and skills_ok:
    advice="pantheon_ritual_ok — Slice A skills on main; C.1 pedagogy still deferred"
elif fail_n==0:
    advice="pantheon_ritual_hold — front door incomplete"
else:
    advice="pantheon_ritual_fail"
payload={"schema":"gzmo.unpark.pantheon/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,"demo":demo_ok,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"2.1","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
