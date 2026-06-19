#!/usr/bin/env bash
# Discovery ↔ KB loop health gate (optional block in eval-quick).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_ROOT="$(cd "$ROOT/.." && pwd)"
STRICT="${STRICT:-0}"
FAIL=0

METRICS="$PROJECT_ROOT/scripts/discovery-kb-metrics.sh"
if [[ ! -x "$METRICS" ]]; then
  echo "[SKIP] discovery-kb-metrics.sh not found"
  exit 0
fi

"$METRICS" >/dev/null
LATEST="$PROJECT_ROOT/data/discovery-kb-metrics/latest.json"
if [[ ! -f "$LATEST" ]]; then
  echo "[FAIL] metrics file missing: $LATEST"
  exit 1
fi

DEDUP_MAX="$(jq -r '.targets.dedup_skip_rate_max // 0.5' "$LATEST")"
LINK_TARGET="$(jq -r '.targets.novel_links_per_cycle // 2' "$LATEST")"
DEDUP_RATE="$(jq -r '.distill.dedup_skip_rate_estimate // 0' "$LATEST")"
REGISTRY_TOTAL="$(jq -r '.discovery_links.registry_total // 0' "$LATEST")"

echo "=== Discovery loop gate ==="
echo "dedup_skip_rate=${DEDUP_RATE} (max=${DEDUP_MAX})"
echo "link_registry_total=${REGISTRY_TOTAL} (target novel/cycle=${LINK_TARGET})"

if python3 -c "import sys; sys.exit(0 if float('${DEDUP_RATE}') <= float('${DEDUP_MAX}') else 1)"; then
  echo "[PASS] dedup skip rate"
else
  echo "[FAIL] dedup skip rate above max"
  if [[ "$STRICT" == "1" ]]; then
    FAIL=1
  else
    echo "[WARN] dedup skip rate high (non-blocking unless STRICT=1)"
  fi
fi

# Require registry activity once discovery cycles have run (reports exist)
REPORT_COUNT="$(jq -r '.counts.cycle_reports // 0' "$LATEST")"
if [[ "$REPORT_COUNT" -gt 0 && "$REGISTRY_TOTAL" -eq 0 ]]; then
  echo "[WARN] cycle reports exist but link registry empty (closure not wired yet?)"
  if [[ "$STRICT" == "1" ]]; then
    FAIL=1
    echo "[FAIL] strict: link registry required when reports exist"
  fi
else
  echo "[PASS] link registry presence"
fi

RECALL_LATEST="$PROJECT_ROOT/data/discovery-kb-metrics/recall-smoke-latest.json"
if [[ -f "$RECALL_LATEST" ]]; then
  PASS_RATE="$(jq -r '.pass_rate // 0' "$RECALL_LATEST")"
  RECALL_MIN="$(jq -r '.targets.recall_smoke_pass_min // 0.33' "$LATEST")"
  echo "recall_smoke_pass_rate=${PASS_RATE} (min=${RECALL_MIN})"
  if python3 -c "import sys; sys.exit(0 if float('${PASS_RATE}') >= float('${RECALL_MIN}') else 1)"; then
    echo "[PASS] recall smoke"
  elif [[ "$STRICT" == "1" ]]; then
    echo "[FAIL] recall smoke below min"
    FAIL=1
  else
    echo "[WARN] recall smoke below min (non-blocking)"
  fi
fi

if [[ "$FAIL" -eq 0 ]]; then
  echo "[PASS] discovery loop gate"
  exit 0
fi
echo "[FAIL] discovery loop gate"
exit 1
