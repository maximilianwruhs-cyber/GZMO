#!/usr/bin/env bash
# verify-skill-standard.sh — Golden standard checks (docs/SKILL_GOLDEN_STANDARD.md §10)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PASS=0
WARN=0
FAIL=0

ok()   { echo "  ✓ $*"; PASS=$((PASS + 1)); }
warn() { echo "  ⚠ $*"; WARN=$((WARN + 1)); }
bad()  { echo "  ✗ $*"; FAIL=$((FAIL + 1)); }

SKILL_FILTER="${1:-}"

echo "=== GZMO Skill Golden Standard Verification ==="
echo "Root: $ROOT"
echo ""

# ── Tier 1: Unit tests ─────────────────────────────────────────────
echo "Tier 1 — Unit tests"
if unset CARGO_TARGET_DIR && cargo test -p gzmo-core skill_ccl attractor poem_brief joke_brief story help 2>/dev/null | tail -3 | grep -q "0 failed"; then
  ok "gzmo-core skill module tests"
else
  if unset CARGO_TARGET_DIR && cargo test -p gzmo-core 2>&1 | tail -5 | grep -q "test result: ok"; then
    ok "gzmo-core full test suite"
  else
    bad "gzmo-core tests failed"
  fi
fi

if unset CARGO_TARGET_DIR && cargo test -p gzmo-chaos 2>&1 | tail -3 | grep -q "0 failed"; then
  ok "gzmo-chaos tests"
else
  bad "gzmo-chaos tests failed"
fi
echo ""

# ── Tier 2: Registry / CCL static checks ─────────────────────────
echo "Tier 2 — Registry & CCL"
GOLD_STANDARD="docs/SKILL_GOLDEN_STANDARD.md"
for f in \
  gzmo-core/src/skills/skill_ccl.rs \
  gzmo-core/src/skills/attractor_common.rs \
  gzmo-core/src/skills/registry.rs \
  "$GOLD_STANDARD"; do
  if [[ -f "$f" ]]; then ok "present: $f"; else bad "missing: $f"; fi
done

CCL4_SKILLS=(story poem joke card pkm word define)
for s in "${CCL4_SKILLS[@]}"; do
  if grep -q "\"$s\" |" gzmo-core/src/skills/skill_ccl.rs 2>/dev/null || \
     grep -q "$s.*Ccl4" gzmo-core/src/skills/skill_ccl.rs; then
    ok "CCL-4 registered: $s"
  else
    bad "CCL-4 not in skill_ccl.rs: $s"
  fi
  if [[ -f "gzmo-core/src/skills/${s}.rs" ]]; then
    if grep -q "attractor_common\|Attractor" "gzmo-core/src/skills/${s}.rs"; then
      ok "attractor wiring: $s.rs"
    else
      bad "no attractor_common in $s.rs"
    fi
  fi
done
echo ""

# ── Tier 3: Binary & help output ─────────────────────────────────
echo "Tier 3 — CLI smoke"
GZMO="${GZMO_BIN:-$ROOT/target/release/gzmo}"
if [[ ! -x "$GZMO" ]]; then
  echo "  Building release binary..."
  unset CARGO_TARGET_DIR
  cargo build --release -p gzmo-cli -q
fi
if [[ -x "$GZMO" ]]; then ok "gzmo binary"; else bad "gzmo binary missing"; fi

HELP_OUT="$("$GZMO" chaos skill help 2>/dev/null || true)"
if echo "$HELP_OUT" | grep -q "CCL-4"; then
  ok "/help shows CCL-4 badges"
else
  bad "/help missing CCL-4 badges"
fi
if echo "$HELP_OUT" | grep -q "★"; then
  ok "/help shows gold-star for CCL-4"
else
  warn "/help missing ★ marker"
fi
echo ""

# ── Tier 4: Generative live checks (optional — needs Prime) ────────
echo "Tier 4 — Generative uniqueness (optional, needs Prime)"
if curl -sf "http://localhost:8000/v1/models" >/dev/null 2>&1; then
  ok "Prime reachable"
  GEN_SKILLS=(story poem joke card pkm word define)
  if [[ -n "$SKILL_FILTER" ]]; then GEN_SKILLS=("$SKILL_FILTER"); fi
  for s in "${GEN_SKILLS[@]}"; do
    extra=""
    if [[ "$s" == "define" ]]; then extra="chaos"; fi
    OUT1="$("$GZMO" chaos skill "$s" $extra 2>/dev/null || true)"
    OUT2="$("$GZMO" chaos skill "$s" $extra 2>/dev/null || true)"
    if [[ -z "$OUT1" || -z "$OUT2" ]]; then
      warn "$s: empty output (LLM error?)"
      continue
    fi
    if echo "$OUT1" | grep -q "inv #"; then ok "$s: inv # in header"; else bad "$s: missing inv #"; fi
    if echo "$OUT1" | grep -qiE "ATTRACTOR|inv #"; then ok "$s: attractor header"; else warn "$s: no ATTRACTOR title"; fi
    H1=$(echo "$OUT1" | sha256sum | awk '{print $1}')
    H2=$(echo "$OUT2" | sha256sum | awk '{print $1}')
    if [[ "$H1" != "$H2" ]]; then ok "$s: two calls differ"; else bad "$s: duplicate output"; fi
  done
else
  warn "Prime not up — skipping live generative checks (curl localhost:8000)"
fi
echo ""

# ── Tier 5: Shell delegate stubs ─────────────────────────────────
echo "Tier 5 — Shell delegates"
for s in story poem joke card pkm word define; do
  SH="skills/skill_${s}.sh"
  if [[ -f "$SH" ]] && grep -q "deprecated" "$SH" && grep -q "chaos skill $s" "$SH"; then
    ok "delegate stub: $SH"
  else
    warn "shell stub incomplete: $SH"
  fi
done
echo ""

echo "=== Summary: $PASS passed, $WARN warnings, $FAIL failed ==="
if [[ "$FAIL" -gt 0 ]]; then exit 2; fi
if [[ "$WARN" -gt 0 ]]; then exit 1; fi
exit 0
