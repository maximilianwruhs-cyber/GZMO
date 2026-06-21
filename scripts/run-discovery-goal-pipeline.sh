#!/usr/bin/env bash
# Five-phase discovery goal pipeline (Jules automate-github-issues analogue).
# Analyze → Plan → Validate → Dispatch → Verify
#
# Usage:
#   run-discovery-goal-pipeline.sh --report <path> [--session-id <id>] [--spawn]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
SKILLS="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
export GZMO_ROOT="$ROOT"
export GZMO_SKILLS_ROOT="$SKILLS"

REPORT=""
SESSION_ID=""
SPAWN=0
FORCE_REPLAN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report) REPORT="$2"; shift 2 ;;
    --session-id) SESSION_ID="$2"; shift 2 ;;
    --spawn) SPAWN=1; shift ;;
    --force-replan) FORCE_REPLAN=1; shift ;;
    *) echo "Unknown arg: $1"; exit 1 ;;
  esac
done

[[ -n "$REPORT" ]] || { echo "--report required"; exit 1; }

echo "═══ Phase 1 — Analyze (report present) ═══"
echo "  report: $REPORT"
echo "  Enforcing Analyze -> Plan Stage Gate..."
"$BIN" kurator verify-gate analyze --report "$REPORT"

echo "═══ Phase 2 — Plan ═══"
PLAN_ARGS=(kurator plan-from-discovery --report "$REPORT")
[[ -n "$SESSION_ID" ]] && PLAN_ARGS+=(--session-id "$SESSION_ID")
[[ $FORCE_REPLAN -eq 1 ]] && PLAN_ARGS+=(--force-replan)
[[ $SPAWN -eq 1 ]] && PLAN_ARGS+=(--spawn)
"$BIN" "${PLAN_ARGS[@]}"

PLAN_DIR="$(ls -td "$SKILLS/data/discovery-implementation/plans"/*/ 2>/dev/null | head -1 || true)"
[[ -n "$PLAN_DIR" ]] || { echo "No plan dir found"; exit 1; }
echo "  plan_dir: $PLAN_DIR"

echo "═══ Phase 3 — Validate (ownership + verify gate) ═══"
echo "  Enforcing Plan -> Approve Stage Gate..."
"$BIN" kurator verify-gate plan --plan-dir "$PLAN_DIR" --report "$REPORT"

if ! "$BIN" kurator plan-from-discovery --report "$REPORT" 2>&1 | grep -q "verify gate"; then
  echo "  plan verify: check plan.json manually"
fi
if jq -e '.workstreams' "$PLAN_DIR/plan.json" >/dev/null 2>&1; then
  echo "  workstreams: $(jq '.workstreams | length' "$PLAN_DIR/plan.json")"
fi

echo "═══ Phase 4 — Approve + Dispatch ═══"
"$BIN" kurator approve-plan --plan "$PLAN_DIR"

echo "  Enforcing Approve -> Execute Stage Gate..."
"$BIN" kurator verify-gate approve --plan-dir "$PLAN_DIR"

WS_ID="$(jq -r '.workstreams[0].id // empty' "$PLAN_DIR/plan.json")"
[[ -n "$WS_ID" ]] || { echo "No workstream in plan"; exit 1; }

EXEC_ARGS=(kurator execute-workstream --plan "$PLAN_DIR" --workstream "$WS_ID")
[[ $SPAWN -eq 1 ]] && EXEC_ARGS+=(--spawn)
"$BIN" "${EXEC_ARGS[@]}"

echo "  Enforcing Execute -> Distill Stage Gate..."
"$BIN" kurator verify-gate execute --plan-dir "$PLAN_DIR" --workstream "$WS_ID"

echo "═══ Phase 5 — Verify ═══"
"$ROOT/scripts/query-discovery-activities.sh" summary
echo "Pipeline complete."
