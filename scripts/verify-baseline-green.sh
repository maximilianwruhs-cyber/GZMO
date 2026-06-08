#!/usr/bin/env bash
# Unified baseline gate: M4 eval (offline) + production E2E + GZMO Platform (P0–P3).
# Exit 0 only when all required tiers pass. ~1–2 min (no replay-wave).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
FAIL=0
BIN="${ROOT}/target/release/gzmo"

pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; FAIL=1; }
section() { echo ""; echo "=== $* ==="; }

retry() {
  local n="${1:?}"; shift
  local i=1
  while [[ "$i" -le "$n" ]]; do
    if "$@"; then
      return 0
    fi
    [[ "$i" -lt "$n" ]] && sleep 5
    i=$((i + 1))
  done
  return 1
}

wait_for_live_stack() {
  local ok=0
  for _ in 1 2 3 4 5 6; do
    if curl -sf --max-time 8 http://127.0.0.1:8000/v1/models >/dev/null \
      && curl -sf --max-time 8 "${EMBED_PROBE_URL:-http://192.168.31.110:8081/v1}/models" >/dev/null \
      && curl -sf --max-time 8 "${QDRANT_PROBE_URL:-http://192.168.31.202:6333}/collections" >/dev/null; then
      ok=1
      break
    fi
    sleep 5
  done
  [[ "$ok" -eq 1 ]]
}

section "Build"
if cargo build --release -p gzmo-cli -q 2>/dev/null; then
  pass "cargo build --release -p gzmo-cli"
else
  fail "cargo build --release -p gzmo-cli"
fi

section "Unit tests (gzmo-core)"
if cargo test -p gzmo-core -q 2>/dev/null; then
  pass "cargo test -p gzmo-core (all)"
else
  fail "cargo test -p gzmo-core"
fi

section "Live stack preflight"
if wait_for_live_stack; then
  pass "Prime + embed + Qdrant reachable"
else
  fail "Prime/embed/Qdrant not ready (retry later)"
fi

section "M4 ingest (eval-quick strict)"
if retry 5 env STRICT=1 scripts/ingest-quality/eval-quick.sh >/tmp/gzmo-eval-quick.log 2>&1; then
  pass "eval-quick.sh STRICT=1"
else
  fail "eval-quick.sh STRICT=1 (see /tmp/gzmo-eval-quick.log)"
fi

section "M4 contract (canonical report)"
REPORT="${ROOT}/scripts/ingest-quality/reports/baseline-m4-production.json"
if [[ ! -f "$REPORT" ]]; then
  REPORT="${ROOT}/scripts/ingest-quality/reports/baseline-m4-post-sprint.json"
fi
if scripts/ingest-quality/check-contract.sh "$REPORT" >/tmp/gzmo-contract.log 2>&1; then
  pass "check-contract.sh $(basename "$REPORT")"
else
  fail "check-contract.sh on canonical report"
fi

if [[ "${FAITHFULNESS_JUDGE:-0}" == "1" || "${FAITHFULNESS_JUDGE_STRICT:-0}" == "1" ]]; then
  section "M4 faithfulness (context gate)"
  mkdir -p "${ROOT}/logs"
  GATE_MIN="$(python3 -c "import yaml; m=yaml.safe_load(open('${ROOT}/scripts/ingest-quality/gate-config.yaml'))['memscore']; print(m.get('faithfulness_context_min', 0.90))" 2>/dev/null || echo 0.90)"
  unset CARGO_TARGET_DIR
  if python3 scripts/ingest-quality/validate-golden-facts.py --fail-on-invalid \
       >>logs/m4-faithfulness-green.log 2>&1 \
     && python3 scripts/ingest-quality/run-recall-eval.py --batch all --backend gzmo --match strict \
       >>logs/m4-faithfulness-green.log 2>&1 \
     && python3 scripts/ingest-quality/faithfulness-judge.py --mode llm --grounding context \
       --write-report --merge-mem-score --gate-min "$GATE_MIN" \
       ${FAITHFULNESS_JUDGE_STRICT:+--gate} >>logs/m4-faithfulness-green.log 2>&1; then
    pass "faithfulness_context >= $GATE_MIN (logs/m4-faithfulness-green.log)"
  else
    if [[ "${FAITHFULNESS_JUDGE_STRICT:-0}" == "1" ]]; then
      fail "faithfulness_context below $GATE_MIN (logs/m4-faithfulness-green.log)"
    else
      echo "[WARN] faithfulness_context below $GATE_MIN (non-blocking)"
    fi
  fi
fi

section "Production E2E"
if retry 5 scripts/verify-production.sh >/tmp/gzmo-prod-e2e.log 2>&1; then
  pass "verify-production.sh"
else
  fail "verify-production.sh (see /tmp/gzmo-prod-e2e.log)"
fi

section "Platform hot memory (P1 + P3)"
if [[ ! -x "${ROOT}/scripts/pi-gzmo-memory.sh" ]]; then
  chmod +x "${ROOT}/scripts/pi-gzmo-memory.sh"
fi
if timeout 30 ./scripts/pi-gzmo-memory.sh turn-start >/tmp/gzmo-pi-turn.log 2>&1 \
  && timeout 60 ./scripts/pi-gzmo-memory.sh search "GZMO honeypot" --limit 1 >/tmp/gzmo-pi-search.log 2>&1 \
  && timeout 15 ./scripts/pi-gzmo-memory.sh recall 2>/tmp/gzmo-pi-recall.log | grep -q '^\[RECALL\]'; then
  pass "pi-gzmo-memory.sh turn-start + search + recall"
else
  fail "pi-gzmo-memory bridge"
fi

if [[ -x "$BIN" ]]; then
  export GZMO_SESSION_ID="baseline-green-$(date +%Y%m%d)"
  if "$BIN" memory status 2>/tmp/gzmo-mem-status.log | grep -q 'scratch=redis'; then
    pass "gzmo memory status (redis scratch)"
  else
    fail "gzmo memory status"
  fi
else
  fail "gzmo binary missing after build"
fi

section "Redis scratch"
# Authoritative: gzmo connects to [redis] in gzmo.toml (nc/redis-cli often unavailable).
if [[ -x "$BIN" ]] && "$BIN" memory turn-start >/dev/null 2>&1 \
  && "$BIN" memory status 2>/dev/null | grep -q 'scratch=redis'; then
  pass "Redis scratch backend (gzmo memory status)"
else
  fail "Redis scratch backend"
fi

echo ""
if [[ "$FAIL" -eq 0 ]]; then
  echo "RESULT: BASELINE GREEN (M4 + platform P0–P3)"
  echo "Label: baseline-m4-platform-20260604"
  echo "Doc: docs/PLATFORM_BASELINE_STATUS.md"
  exit 0
else
  echo "RESULT: ONE OR MORE BASELINE CHECKS FAILED"
  exit 1
fi
