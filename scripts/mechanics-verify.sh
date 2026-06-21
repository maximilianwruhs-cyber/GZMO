#!/usr/bin/env bash
# End-to-end mechanics verification: ARCH-DIR, Obolus, discovery hooks, fixer pipeline.
# Exit 0 = all checks pass, 1 = any hard fail.
#
# Optional:
#   MECHANICS_OBOLUS_SMOKE=1  — run obolus-gate-smoke.sh (temp gzmo.toml cap)
#   MECHANICS_LIVE_SPAWN=1    — spawn Epimetheus on synthetic FAIL/GAP report (~2–5 min)
#   MECHANICS_LIVE_PIPELINE=1 — run Section 11 on historical session-final reports (slow)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
SKILLS="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
export GZMO_ROOT="$ROOT"
export GZMO_SKILLS_ROOT="$SKILLS"
SYNAPSE="${ROOT}/data/Synapse/events.jsonl"
MARKER="${SKILLS}/data/.mechanics-verify-marker"
TEST_REPORT="${ROOT}/data/.mechanics-verify-report.md"
CHIMERA_MARKERS=(
  "${ROOT}/skills/data/.mechanics-verify-marker"
  "${ROOT}/gzmo_skills/data/.mechanics-verify-marker"
)
FAIL=0
PASS=0

pass() { echo "[PASS] $*"; PASS=$((PASS + 1)); }
fail() { echo "[FAIL] $*"; FAIL=$((FAIL + 1)); }
section() { echo; echo "═══ $* ═══"; }

is_chimera_path() {
  local p="$1"
  [[ "$p" == *survey_GZMO/gzmo_skills* ]] && return 0
  [[ "$p" == *gzmo_skills/survey_GZMO* ]] && return 0
  return 1
}

run_cargo_test() {
  local label="$1"
  shift
  set +e
  local out
  out="$(cargo test "$@" 2>&1)"
  local rc=$?
  set -e
  if [[ $rc -eq 0 ]]; then
    pass "$label"
  else
    echo "$out" | tail -20
    fail "$label"
  fi
}

section "0 — Canonical path guards"
if [[ ! -d "$SKILLS" ]]; then
  fail "GZMO_SKILLS_ROOT not a directory: $SKILLS"
else
  pass "GZMO_SKILLS_ROOT is directory: $SKILLS"
fi

CHIMERA_TREE="${ROOT}/gzmo_skills"
if [[ -d "$CHIMERA_TREE" ]] && [[ -n "$(find "$CHIMERA_TREE" -type f -print -quit 2>/dev/null)" ]]; then
  fail "chimera tree ${CHIMERA_TREE}/ still has files (run migrate-chimera-skills-artifacts.sh)"
else
  pass "no chimera files under ${CHIMERA_TREE}/"
fi

chimera_marker_found=0
for c in "${CHIMERA_MARKERS[@]}"; do
  if [[ -f "$c" ]]; then
    fail "chimera marker must not exist: $c (canonical: $MARKER)"
    chimera_marker_found=1
  fi
done
if [[ $chimera_marker_found -eq 0 ]]; then
  pass "no chimera markers under repo skills/ or gzmo_skills/"
fi

if [[ -f "${SKILLS}/scripts/pi-mentor-discovery-cycle.sh" ]]; then
  pass "skills discovery cycle script present"
else
  fail "missing ${SKILLS}/scripts/pi-mentor-discovery-cycle.sh"
fi

section "1 — Sovereignty baseline"
if ./scripts/sovereignty-verify.sh; then
  pass "sovereignty-verify.sh"
else
  fail "sovereignty-verify.sh"
fi

section "2 — Rust unit tests (discovery_fixer + obolus gate)"
run_cargo_test "discovery_fixer unit tests" -p gzmo-core --lib discovery_fixer
run_cargo_test "obolus gate unit tests" -p gzmo-core --lib obolus::gate

section "3 — Obolus preflight matrix"
for action in operator_chat discovery_cycle spawn_discovery_fix spawn_session_triage dice_loop dream_tick spark_tick; do
  set +e
  out="$("$BIN" obolus preflight "$action" --json 2>&1)"
  rc=$?
  set -e
  verdict="$(echo "$out" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('verdict','?'))" 2>/dev/null || echo "?")"
  if [[ $rc -eq 0 ]]; then
    pass "preflight $action → $verdict (exit 0)"
  elif [[ $rc -eq 1 && "$verdict" == "Deny" ]]; then
    pass "preflight $action → Deny (exit 1, expected for capped actions)"
  else
    fail "preflight $action rc=$rc verdict=$verdict"
  fi
done

section "4 — Discovery report parsing (FAIL/GAP vs OK-only)"
REPORT_ACTIONABLE="${SKILLS}/data/pi-mentor-discovery/reports/cycle-1-2026-06-16T07-18-52Z.md"
REPORT_OBSERVE="${SKILLS}/data/pi-mentor-discovery/reports/cycle-1-2026-06-16T09-07-36Z.md"
if [[ -f "$REPORT_ACTIONABLE" ]]; then
  if grep -qE '\*\*FAIL risk\*\*|\*\*GAP\*\*|\*\*Gap\*\*' "$REPORT_ACTIONABLE"; then
    pass "actionable report contains FAIL/GAP markers"
  else
    fail "actionable report missing FAIL/GAP markers"
  fi
else
  fail "missing $REPORT_ACTIONABLE"
fi
if [[ -f "$REPORT_OBSERVE" ]]; then
  out="$("$BIN" kurator fix-from-discovery --report "$REPORT_OBSERVE" --session-id "mechanics-parse-only" 2>&1 || true)"
  if echo "$out" | grep -q '0 actionable' && echo "$out" | grep -q 'fixer spawn skipped'; then
    pass "observe-only report → skip fixer (correct)"
  else
    fail "observe-only report: $out"
  fi
else
  fail "missing $REPORT_OBSERVE"
fi

section "5 — Discovery cycle script hooks (Obolus + fixer)"
for hook in \
  "obolus preflight discovery_cycle" \
  "obolus preflight spawn_discovery_fix" \
  "kurator fix-from-discovery" \
  "run_discovery_implementation_pipeline" \
  "enqueue-discovery-report.sh" \
  "run-discovery-implementation-pipeline.sh" \
  "DISCOVERY_INLINE_IMPLEMENT" \
  "build_session_log_extra" \
  "session-final criteria not met" \
  "DISCOVERY_SESSION_FINAL_ACTIONABILITY_RETRY" \
  "bind_session_duration_from_state"; do
  if grep -q "$hook" "${SKILLS}/scripts/pi-mentor-discovery-cycle.sh" 2>/dev/null; then
    pass "pi-mentor-discovery-cycle.sh contains: $hook"
  else
    fail "pi-mentor-discovery-cycle.sh missing: $hook"
  fi
done
# shellcheck disable=SC1091
source "${SKILLS}/scripts/lib/discovery-findings-lib.sh"
_norm_tmp="$(mktemp)"
echo '- L01: host —→ target | EVIDENCE: gzmo_health | WHY: test' >"$_norm_tmp"
discovery_normalize_report_links "$_norm_tmp"
if grep -qE '^-[[:space:]]*LINK: L01:' "$_norm_tmp"; then
  pass "discovery_normalize_report_links L## → LINK: L##"
else
  fail "discovery_normalize_report_links did not fix bare L## line"
fi
rm -f "$_norm_tmp"
if grep -q 'discovery_normalize_report_links' "${SKILLS}/scripts/eval-pi-mentor-discovery-report.sh" 2>/dev/null; then
  pass "eval script invokes link normalizer before gates"
else
  fail "eval script missing link normalizer hook"
fi
if grep -q 'obolus preflight discovery_cycle' "${SKILLS}/scripts/auto-socratic-discovery-cycle.sh" 2>/dev/null; then
  pass "auto-socratic-discovery-cycle.sh has discovery preflight"
else
  fail "auto-socratic-discovery-cycle.sh missing discovery preflight"
fi

section "6 — Remediation scripts (exist + dry-run)"
for script in \
  "${SKILLS}/scripts/cleanup_unbounded_sessions.sh" \
  "${SKILLS}/scripts/synapse_event_supplement.sh" \
  "${SKILLS}/scripts/synapse-health-check.sh" \
  "${SKILLS}/scripts/synapse-dashboard.sh"; do
  if [[ -x "$script" ]]; then
    pass "executable: $(basename "$script")"
  else
    fail "not executable or missing: $script"
  fi
done
if bash "${SKILLS}/scripts/cleanup_unbounded_sessions.sh" --dry-run >/dev/null 2>&1; then
  pass "cleanup_unbounded_sessions.sh --dry-run"
else
  fail "cleanup_unbounded_sessions.sh --dry-run"
fi
if bash "${SKILLS}/scripts/synapse-health-check.sh" >/dev/null 2>&1; then
  pass "synapse-health-check.sh runs"
else
  # non-zero exit = stalls detected (still runs)
  if [[ -x "${SKILLS}/scripts/synapse-health-check.sh" ]]; then
    pass "synapse-health-check.sh runs (exit non-zero = stalls found, OK)"
  else
    fail "synapse-health-check.sh"
  fi
fi

section "7 — Synapse spawn chain (today)"
if [[ -f "$SYNAPSE" ]]; then
  today="$(date -u +%Y-%m-%d)"
  for et in spawn.recommended agent.spawned agent.result spawn.executed obolus.denied obolus.budget_tick; do
    n="$(grep "$today" "$SYNAPSE" | grep -c "\"event_type\":\"$et\"" || true)"
    if [[ "$n" -gt 0 ]]; then
      pass "Synapse $et today: $n"
    else
      if [[ "$et" == "obolus.denied" ]]; then
        echo "[INFO] obolus.denied today: 0 (only if smoke ran)"
      else
        fail "Synapse $et today: 0"
      fi
    fi
  done
else
  fail "Synapse events.jsonl missing"
fi

section "8 — Stack health"
if systemctl --user is-active gzmo-daemon.service >/dev/null 2>&1; then
  pass "gzmo-daemon active"
else
  fail "gzmo-daemon not active"
fi
if curl -sf http://127.0.0.1:8000/v1/models >/dev/null 2>&1; then
  pass "Prime :8000 reachable"
else
  fail "Prime :8000 not reachable"
fi
if "$BIN" kurator status >/dev/null 2>&1; then
  pass "kurator status CLI"
else
  fail "kurator status CLI"
fi

if [[ "${MECHANICS_OBOLUS_SMOKE:-0}" == "1" ]]; then
  section "9 — Obolus gate live smoke (temp cap)"
  if ./scripts/obolus-gate-smoke.sh; then
    pass "obolus-gate-smoke.sh"
  else
    fail "obolus-gate-smoke.sh"
  fi
fi

if [[ "${MECHANICS_LIVE_SPAWN:-0}" == "1" ]]; then
  section "10 — Live fixer spawn (synthetic FAIL/GAP)"
  rm -f "$MARKER"
  cat > "$TEST_REPORT" <<'MD'
# Infrastructure Discovery Report — mechanics-verify

## Findings

### F1 — Mechanics verification marker
- Observation: E2E autospawn test needs a proof file on disk.
- Risk or opportunity: **GAP**: marker file missing at \`$GZMO_SKILLS_ROOT/data/.mechanics-verify-marker\`.

### F2 — Write proof
- Observation: Fixer must create the marker via shell or file write.
- Risk or opportunity: **FAIL risk**: pipeline cannot be verified without the marker file existing.
MD

  SESSION_ID="mechanics-verify-$(date -u +%Y%m%dT%H%M%SZ)"
  BEFORE_N="$(grep -c 'agent.result' "$SYNAPSE" 2>/dev/null || echo 0)"

  set +e
  spawn_out="$(
    "$BIN" kurator fix-from-discovery \
      --report "$TEST_REPORT" \
      --session-id "$SESSION_ID" \
      --spawn 2>&1
  )"
  spawn_rc=$?
  set -e
  echo "$spawn_out"

  if [[ $spawn_rc -ne 0 ]]; then
    fail "fix-from-discovery --spawn exit $spawn_rc"
  elif echo "$spawn_out" | grep -q 'Fixer sub-agent spawned'; then
    pass "fixer sub-agent spawned"
  else
    fail "no 'Fixer sub-agent spawned' in output"
  fi

  if [[ -f "$MARKER" ]]; then
    marker_canon="$(realpath "$MARKER")"
    skills_canon="$(realpath "$SKILLS")"
    if [[ "$marker_canon" == "$skills_canon"/* ]]; then
      pass "verify_gate: marker at canonical path $marker_canon"
    else
      fail "verify_gate: marker not under GZMO_SKILLS_ROOT ($marker_canon)"
    fi
    chimera_after_spawn=0
    for c in "${CHIMERA_MARKERS[@]}"; do
      if [[ -f "$c" ]]; then
        fail "verify_gate: chimera marker still exists at $c"
        chimera_after_spawn=1
      fi
    done
    if [[ $chimera_after_spawn -eq 0 ]]; then
      pass "verify_gate: no chimera markers after spawn"
    fi
  else
    fail "verify_gate: marker file missing at $MARKER"
  fi

  AFTER_N="$(grep -c 'agent.result' "$SYNAPSE" 2>/dev/null || echo 0)"
  if [[ "$AFTER_N" -gt "$BEFORE_N" ]]; then
    pass "new agent.result in Synapse"
  else
    fail "no new agent.result in Synapse"
  fi
fi

section "11 — Discovery implementation pipeline E2E"
if [[ -x "${SKILLS}/scripts/verify-discovery-action-pipeline.sh" ]]; then
  if [[ "${MECHANICS_LIVE_PIPELINE:-0}" == "1" ]]; then
    PIPELINE_REPORTS=(
      "${SKILLS}/data/pi-mentor-discovery/reports/session-final-2026-06-16T16-25-43Z.md"
      "${SKILLS}/data/pi-mentor-discovery/reports/session-final-2026-06-16T14-57-29Z.md"
    )
    PIPELINE_FIXER_AUTOSPAWN=1
    echo "[INFO] Section 11 live pipeline mode (slow, environment-dependent)"
  else
    PIPELINE_REPORTS=(
      "${SKILLS}/data/pi-mentor-discovery/reports/mechanics-e2e-fixture.md"
    )
    PIPELINE_FIXER_AUTOSPAWN=0
  fi
  for rpt in "${PIPELINE_REPORTS[@]}"; do
    if [[ -f "$rpt" ]]; then
      sess_id="mechanics-$(basename "$rpt" .md)-$(date -u +%H%M%S)"
      set +e
      pipe_out="$(
        GZMO_BIN="$BIN" DISCOVERY_FIXER_AUTOSPAWN="$PIPELINE_FIXER_AUTOSPAWN" \
          MECHANICS_LIVE_PIPELINE="${MECHANICS_LIVE_PIPELINE:-0}" \
          "${SKILLS}/scripts/verify-discovery-action-pipeline.sh" "$rpt" "$sess_id" 2>&1
      )"
      pipe_rc=$?
      set -e
      if [[ $pipe_rc -eq 0 ]]; then
        pass "pipeline E2E $(basename "$rpt")"
      else
        echo "$pipe_out" | tail -20
        fail "pipeline E2E $(basename "$rpt")"
      fi
    else
      fail "pipeline E2E report missing: $rpt"
    fi
  done
else
  fail "verify-discovery-action-pipeline.sh missing"
fi

if [[ -d "${SKILLS}/data/discovery-implementation/schemas" && -f "${SKILLS}/data/discovery-implementation/queue.jsonl" ]]; then
  pass "discovery-implementation Forum-2 scaffold"
else
  fail "discovery-implementation Forum-2 scaffold missing (run init-discovery-implementation.sh)"
fi

IFP="${SKILLS}/scripts/verify-implement-fixer-pipeline.sh"
if [[ -x "$IFP" ]]; then
  if bash "$IFP" >/dev/null 2>&1; then
    pass "verify-implement-fixer-pipeline.sh"
  else
    fail "verify-implement-fixer-pipeline.sh"
  fi
else
  fail "missing $IFP"
fi

section "12 — Spark/distill e2e verify bundle"
E2E="${SKILLS}/scripts/discovery-remediations/e2e-verify-17-35-00/run-all.sh"
if [[ -x "$E2E" ]]; then
  if bash "$E2E" >/dev/null 2>&1; then
    pass "e2e-verify-17-35-00/run-all.sh"
  else
    fail "e2e-verify-17-35-00/run-all.sh"
  fi
else
  fail "missing $E2E"
fi

section "Summary"
echo "Passed: $PASS | Failed: $FAIL"
if (( FAIL > 0 )); then
  echo "MECHANICS VERIFY: FAIL"
  exit 1
fi
echo "MECHANICS VERIFY: PASS"
exit 0
