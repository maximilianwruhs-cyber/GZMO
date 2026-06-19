#!/usr/bin/env bash
# Tiered eval: fast by default; optional core Prime replay (~5–8 min).
# Usage:
#   scripts/ingest-quality/eval-quick.sh           # Tier 0 (~30s)
#   CORE=1 scripts/ingest-quality/eval-quick.sh  # Tier 0 + core replay

set -eo pipefail
export LC_ALL=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$PROJECT_ROOT"

CORE="${CORE:-0}"
STRICT="${STRICT:-0}"
FAIL=0

echo "=== Tier 0: offline + infra (~30s) ==="
# Golden contract gate: reject artifact facts before they skew recall/judge.
REQUIRE_AUDIT="$(python3 -c "import yaml; print(yaml.safe_load(open('$DIR/gate-config.yaml')).get('golden',{}).get('require_audit_pass', False))" 2>/dev/null || echo False)"
if [[ "$REQUIRE_AUDIT" == "True" ]]; then
  if python3 "$DIR/validate-golden-facts.py" --fail-on-invalid --sample 5; then
    echo "[PASS] golden-fact audit"
  else
    FAIL=1
    echo "[FAIL] golden-fact audit (invalid must_recall_facts)"
  fi
fi
scripts/ingest-quality/check-contract.sh || FAIL=1
if [[ "$STRICT" == "1" ]]; then
  GATE_MODE=strict scripts/ingest-quality/gate-report.sh || FAIL=1
else
  scripts/ingest-quality/gate-report.sh || FAIL=1
fi
python3 scripts/ingest-quality/retrieval-probes.py || FAIL=1
scripts/check-fts-sanity.sh 2>/dev/null || true
scripts/memory-status.sh | head -6
python3 scripts/ingest-quality/mem-score.py 2>/dev/null || true

if [[ "${DISCOVERY_LOOP:-0}" == "1" ]]; then
  echo ""
  echo "=== Discovery ↔ KB loop gate ==="
  if DISCOVERY_LOOP_STRICT="$STRICT" bash scripts/ingest-quality/gate-discovery-loop.sh; then
    echo "[PASS] discovery loop gate"
  else
    FAIL=1
    echo "[FAIL] discovery loop gate"
  fi
fi

if [[ "${FAITHFULNESS_JUDGE:-0}" == "1" ]]; then
  echo ""
  echo "=== M4 faithfulness judge — context grounding (Prime LLM) ==="
  JUDGE_MAX="${JUDGE_MAX_FACTS:-0}"
  GATE_MIN="$(python3 -c "import yaml; m=yaml.safe_load(open('$DIR/gate-config.yaml'))['memscore']; print(m.get('faithfulness_context_min', m.get('faithfulness_judge_min', 0.90)))" 2>/dev/null || echo 0.90)"
  if python3 "$DIR/faithfulness-judge.py" \
    --mode llm \
    --grounding context \
    --max-facts "$JUDGE_MAX" \
    --write-report \
    --merge-mem-score \
    --gate-min "$GATE_MIN" \
    ${FAITHFULNESS_JUDGE_STRICT:+--gate}; then
    echo "[PASS] faithfulness_context >= $GATE_MIN"
  else
    if [[ "${FAITHFULNESS_JUDGE_STRICT:-0}" == "1" ]]; then
      FAIL=1
      echo "[FAIL] faithfulness_context below $GATE_MIN"
    else
      echo "[WARN] faithfulness_context below $GATE_MIN (non-blocking)"
    fi
  fi
  python3 scripts/ingest-quality/mem-score.py 2>/dev/null || true
fi

if [[ "$CORE" == "1" ]]; then
  echo ""
  echo "=== Tier 1: core golden replay (Prime, ~5–8 min) ==="
  SKIP_BUILD=1 scripts/ingest-quality/replay-wave-core.sh || FAIL=1
fi

echo ""
python3 scripts/ingest-quality/report-missing-facts.py --top 15 2>/dev/null || true

if [[ "$FAIL" -eq 0 ]]; then
  echo ""
  echo "SUCCESS: eval-quick complete (CORE=$CORE STRICT=$STRICT)"
  exit 0
fi
echo ""
echo "FAIL: one or more checks failed"
exit 1
