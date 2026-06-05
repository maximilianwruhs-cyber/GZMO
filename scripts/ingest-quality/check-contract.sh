#!/usr/bin/env bash
# Fast deterministic contract check — no Prime, no full ingest-eval.
# Use after YAML-only changes against the latest report.json.

set -eo pipefail
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPORT="${1:-$DIR/report.json}"

echo "=== Contract check (deterministic) ==="
python3 "$DIR/rescore-golden.py" --report "$REPORT"
RC=$?

# MemScore one-liner (informational, does not affect exit code)
if [[ -f "$REPORT" ]] && [[ -f "$DIR/mem-score.py" ]]; then
  echo ""
  python3 "$DIR/mem-score.py" 2>/dev/null || true
fi

exit $RC
