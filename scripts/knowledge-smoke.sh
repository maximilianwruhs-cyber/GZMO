#!/usr/bin/env bash
# Compare honeypot is_latest count vs Qdrant collection points — drift alarm.
#
# Usage: knowledge-smoke.sh [--min-ratio R] [--warn-ratio R] [--gzmo-root PATH]
# Exit 0 = ok, 1 = critical drift, 2 = warn-only drift
set -euo pipefail

MIN_RATIO="${KNOWLEDGE_SMOKE_MIN_RATIO:-0.55}"
WARN_RATIO="${KNOWLEDGE_SMOKE_WARN_RATIO:-0.70}"
GZMO_ROOT="${GZMO_ROOT:-/opt/gzmo/survey_GZMO}"
QDRANT_URL="${QDRANT_URL:-http://127.0.0.1:6333}"
COLLECTION="${QDRANT_COLLECTION:-honeypot}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --min-ratio) MIN_RATIO="$2"; shift 2 ;;
    --warn-ratio) WARN_RATIO="$2"; shift 2 ;;
    --gzmo-root) GZMO_ROOT="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

VAULT="$GZMO_ROOT/data/vault.db"
if [[ ! -f "$VAULT" ]]; then
  echo "knowledge-smoke: vault missing at $VAULT" >&2
  exit 1
fi

read -r honeypot qdrant ratio_pct <<<"$(python3 - <<PY
import json, sqlite3, urllib.request, sys

vault = "$VAULT"
qurl = "${QDRANT_URL%/}/collections/$COLLECTION"

c = sqlite3.connect(vault)
honeypot = c.execute("SELECT COUNT(*) FROM honeypot WHERE is_latest=1").fetchone()[0]

try:
    d = json.load(urllib.request.urlopen(qurl, timeout=8))
    qdrant = int(d["result"]["points_count"])
except Exception as e:
    print(honeypot, -1, 0, file=sys.stderr)
    print(f"knowledge-smoke: qdrant unreachable: {e}", file=sys.stderr)
    sys.exit(1)

ratio = (qdrant / honeypot) if honeypot > 0 else 1.0
print(honeypot, qdrant, int(ratio * 100))
PY
)"

if [[ "$qdrant" -lt 0 ]]; then
  exit 1
fi

delta=$((honeypot - qdrant))
ratio="$(python3 -c "print($qdrant / $honeypot if $honeypot else 1.0)")"

echo "knowledge-smoke: honeypot=$honeypot qdrant=$qdrant delta=$delta ratio=${ratio_pct}%"

below_min="$(python3 -c "print(1 if float('$ratio') < float('$MIN_RATIO') else 0)")"
below_warn="$(python3 -c "print(1 if float('$ratio') < float('$WARN_RATIO') else 0)")"

if [[ "$below_min" -eq 1 ]]; then
  echo "knowledge-smoke: CRITICAL — qdrant/honeypot ratio ${ratio_pct}% below min ${MIN_RATIO}" >&2
  exit 1
fi

if [[ "$below_warn" -eq 1 ]]; then
  echo "knowledge-smoke: WARN — qdrant/honeypot ratio ${ratio_pct}% below warn ${WARN_RATIO}" >&2
  exit 2
fi

echo "knowledge-smoke: ok"
exit 0
