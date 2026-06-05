#!/usr/bin/env bash
# Certify and promote the M4-aligned production baseline (ingest + retrieval faithfulness).
#
# Prerequisites: Prime :8000, embed VM110, Qdrant, release gzmo binary, vault/honeypot populated.
#
# Usage:
#   scripts/ingest-quality/certify-production-baseline.sh
#   SKIP_RECALL=1 scripts/ingest-quality/certify-production-baseline.sh   # reuse gzmo_report.json
#   SKIP_JUDGE=1  ...                                                      # recall only
#
# Env:
#   BASELINE_LABEL   default baseline-m4-production
#   GATE_CONTEXT_MIN from gate-config.yaml faithfulness_context_min (default 0.90)
#   PROMOTE=0        run checks only, do not promote
#   LIVE_INGEST_SMOKE=1  run one-file live ingest (Neo4j MCP write path) after offline gates

set -eo pipefail
export LC_ALL=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$ROOT"
unset CARGO_TARGET_DIR

LABEL="${BASELINE_LABEL:-baseline-m4-production}"
LOG="${ROOT}/logs/m4-production-baseline.log"
mkdir -p "${ROOT}/logs"

GATE_MIN="$(python3 -c "
import yaml
m = yaml.safe_load(open('$DIR/gate-config.yaml'))['memscore']
print(m.get('faithfulness_context_min', m.get('faithfulness_judge_min', 0.90)))
" 2>/dev/null || echo 0.90)"

FAIL=0
section() { echo ""; echo "=== $* ==="; }

exec > >(tee -a "$LOG") 2>&1
echo "=== M4 production baseline certification ==="
echo "started: $(date -u +%Y-%m-%dT%H:%M:%SZ) label=$LABEL gate_context_min=$GATE_MIN"

section "Build"
if cargo build --release -p gzmo-cli -q 2>/dev/null; then
  echo "[PASS] cargo build --release -p gzmo-cli"
else
  echo "[FAIL] cargo build"
  FAIL=1
fi

section "Golden contract gate"
if python3 "$DIR/validate-golden-facts.py" --fail-on-invalid --sample 0; then
  echo "[PASS] golden-fact audit"
else
  echo "[FAIL] golden-fact audit"
  FAIL=1
fi

section "Ingest contract (EXTRACTION SNAPSHOT — not live store state)"
# NOTE (F8 audit fix): this gate validates a FROZEN extraction report. It proves the
# pipeline can extract the golden entities; it does NOT assert the live honeypot/evidence
# store is current. Live recall quality is gated separately by the strict-recall and
# faithfulness sections below. Re-run a live ingest + backfill to refresh the store.
REPORT_CANON="$DIR/reports/baseline-m4-post-sprint.json"
if [[ -f "$REPORT_CANON" ]]; then
  cp "$REPORT_CANON" "$DIR/report.json"
  # Freshness warning: flag if the frozen snapshot predates the live vault.
  VAULT_DB="${ROOT}/data/vault.db"
  if [[ -f "$VAULT_DB" && "$VAULT_DB" -nt "$REPORT_CANON" ]]; then
    echo "[WARN] $REPORT_CANON is OLDER than data/vault.db — extraction snapshot is stale vs live store"
  fi
  if bash "$DIR/check-contract.sh" "$DIR/report.json"; then
    echo "[PASS] ingest contract (extraction snapshot) on $REPORT_CANON"
  else
    echo "[FAIL] ingest contract"
    FAIL=1
  fi
else
  echo "[WARN] $REPORT_CANON missing — using existing report.json"
  bash "$DIR/check-contract.sh" "$DIR/report.json" || FAIL=1
fi

if [[ "${SKIP_RECALL:-0}" != "1" ]]; then
  section "Strict recall (rrf_strict)"
  if python3 "$DIR/run-recall-eval.py" --batch all --backend gzmo --match strict; then
    echo "[ok] recall eval ran"
  else
    echo "[FAIL] recall eval crashed"
    FAIL=1
  fi
  # N3 audit fix: enforce a real floor instead of treating "the script ran" as PASS.
  STRICT_MIN="$(python3 -c "
import yaml
m = yaml.safe_load(open('$DIR/gate-config.yaml')).get('memscore', {})
v = m.get('recall_rrf_strict_min')
print('' if v is None else v)
" 2>/dev/null || echo '')"
  if [[ -n "$STRICT_MIN" ]]; then
    STRICT_VAL="$(python3 -c "
import json
d = json.load(open('$DIR/reports/recall-metrics.json'))
e = d.get('latest', {}).get('rrf_strict', {})
print(e.get('recall_at_5', 0))
" 2>/dev/null || echo 0)"
    if python3 -c "exit(0 if float('$STRICT_VAL') >= float('$STRICT_MIN') else 1)"; then
      echo "[PASS] strict recall=$STRICT_VAL >= floor $STRICT_MIN"
    else
      echo "[FAIL] strict recall=$STRICT_VAL < floor $STRICT_MIN"
      FAIL=1
    fi
  else
    echo "[INFO] recall_rrf_strict_min is null — strict recall informational (not gating)"
  fi
else
  echo "[SKIP] recall eval (SKIP_RECALL=1)"
fi

if [[ "${SKIP_JUDGE:-0}" != "1" ]]; then
  section "Faithfulness judge (context gate)"
  if python3 "$DIR/faithfulness-judge.py" \
    --mode llm \
    --grounding context \
    --write-report \
    --merge-mem-score \
    --gate-min "$GATE_MIN" \
    --gate; then
    echo "[PASS] faithfulness_context >= $GATE_MIN"
  else
    echo "[FAIL] faithfulness_context below $GATE_MIN"
    FAIL=1
  fi
else
  echo "[SKIP] faithfulness judge (SKIP_JUDGE=1)"
fi

section "MemScore"
python3 "$DIR/mem-score.py" -v || true

CTX="$(python3 -c "
import json
p='$DIR/reports/faithfulness-judge-latest.json'
try:
  print(json.load(open(p))['summary'].get('faithfulness_context', 0))
except Exception:
  print(0)
")"
if python3 -c "exit(0 if float('$CTX') >= float('$GATE_MIN') else 1)"; then
  echo "[PASS] faithfulness_context=$CTX (threshold $GATE_MIN)"
else
  echo "[FAIL] faithfulness_context=$CTX < $GATE_MIN"
  FAIL=1
fi

section "eval-quick strict (offline gates)"
if STRICT=1 bash "$DIR/eval-quick.sh"; then
  echo "[PASS] eval-quick STRICT=1"
else
  echo "[FAIL] eval-quick STRICT=1"
  FAIL=1
fi

if [[ "${LIVE_INGEST_SMOKE:-0}" == "1" ]]; then
  section "Live ingest smoke (Neo4j MCP write path)"
  if bash "$DIR/live-ingest-smoke.sh"; then
    echo "[PASS] live-ingest-smoke"
  else
    echo "[FAIL] live-ingest-smoke"
    FAIL=1
  fi
else
  echo "[INFO] live ingest smoke skipped (set LIVE_INGEST_SMOKE=1 to exercise Neo4j MCP writes)"
fi

if [[ "$FAIL" -ne 0 ]]; then
  echo ""
  echo "RESULT: PRODUCTION BASELINE NOT CERTIFIED (see $LOG)"
  exit 1
fi

if [[ "${PROMOTE:-1}" == "1" ]]; then
  section "Promote baseline"
  BASELINE_LABEL="$LABEL" \
    SKIP_PROBES="${SKIP_PROBES:-0}" \
    bash "$DIR/promote-baseline.sh" "$DIR/report.json" || FAIL=1
  cp "$DIR/reports/${LABEL}.json" "$DIR/reports/baseline-m4-production.json" 2>/dev/null || true
fi

echo ""
echo "RESULT: PRODUCTION BASELINE CERTIFIED"
echo "  label:     $LABEL"
echo "  report:    $DIR/reports/${LABEL}.json"
echo "  lock:      $DIR/pipeline-lock.json"
echo "  log:       $LOG"
echo "  faithfulness_context: $CTX"
echo "finished: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
exit 0
