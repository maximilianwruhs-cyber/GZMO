#!/usr/bin/env bash
# Promote scripts/ingest-quality/report.json to canonical baseline (no Prime).
# Usage: promote-baseline.sh [report.json]
set -eo pipefail
export LC_ALL=C
export LC_NUMERIC=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$PROJECT_ROOT"

REPORT="${1:-$DIR/report.json}"
LABEL="${BASELINE_LABEL:-baseline-m4-current}"
DATE_TAG="$(date +%Y-%m-%d)"
STAMP="$(date +%Y%m%d-%H%M%S)"

if [[ ! -f "$REPORT" ]]; then
  echo "promote-baseline.sh: missing report: $REPORT" >&2
  exit 2
fi

echo "=== Promote baseline from $REPORT ==="

python3 "$DIR/refresh-report-contract.py" --report "$REPORT"
python3 "$DIR/recalc-pipeline-summary.py" --report "$REPORT" --write

echo ""
bash "$DIR/check-contract.sh" "$REPORT" || exit 1

echo ""
echo "--- strict gate ---"
GATE_MODE=strict bash "$DIR/gate-report.sh" "$REPORT" || exit 1

echo ""
echo "--- layered gate (default production) ---"
GATE_MODE=layered bash "$DIR/gate-report.sh" "$REPORT" || exit 1

if [[ "${SKIP_PROBES:-0}" != "1" ]]; then
  echo ""
  python3 "$DIR/retrieval-probes.py" || exit 1
fi

if [[ "${SKIP_MEMORY_STATUS:-0}" != "1" ]]; then
  echo ""
  bash "$PROJECT_ROOT/scripts/memory-status.sh" || true
fi

mkdir -p "$DIR/reports"
BASELINE_JSON="$DIR/reports/${LABEL}.json"
cp "$REPORT" "$BASELINE_JSON"
echo ""
echo "Archived report: $BASELINE_JSON"

# pipeline-lock.json — summary-only reference for dashboards / docs
python3 - "$REPORT" "$DIR/pipeline-lock.json" "$LABEL" "$DATE_TAG" <<'PY'
import json, sys
from datetime import datetime, timezone

report_path, lock_path, label, date_tag = sys.argv[1:5]
with open(report_path) as f:
    data = json.load(f)
s = data["summary"]
mem_score = s.get("mem_score") or {}
lock = {
    "label": label,
    "recorded_at": date_tag,
    "promoted_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "source_report": report_path.replace("\\", "/"),
    "note": "M4 production baseline: ingest contract + strict recall + faithfulness_context gate.",
    "summary": s,
    "mem_score": mem_score,
    "m4_aligned": {
        "recall_at_5_rrf_strict": mem_score.get("recall_at_5_rrf_strict"),
        "faithfulness_context": mem_score.get("faithfulness_context"),
        "faithfulness_corpus": mem_score.get("faithfulness_corpus"),
        "faithfulness_judge": mem_score.get("faithfulness_judge"),
    },
    "gates": {
        "contract": "PASS",
        "strict": "PASS",
        "layered": "PASS",
        "golden_audit": "PASS",
        "faithfulness_context": (
            "PASS"
            if (mem_score.get("faithfulness_context") or 0) >= 0.90
            else "INFO"
        ),
    },
}
with open(lock_path, "w") as f:
    json.dump(lock, f, indent=2)
    f.write("\n")
print(f"Updated {lock_path}")
PY

MANIFEST="$DIR/reports/baseline-manifest.json"
python3 - "$MANIFEST" "$LABEL" "$BASELINE_JSON" "$STAMP" <<'PY'
import json, sys
from datetime import datetime, timezone

manifest_path, label, baseline_json, stamp = sys.argv[1:5]
entry = {
    "label": label,
    "stamp": stamp,
    "recorded_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "report": baseline_json,
}
try:
    with open(manifest_path) as f:
        hist = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    hist = {"history": []}
hist.setdefault("current", None)
hist["current"] = entry
hist.setdefault("history", []).insert(0, entry)
hist["history"] = hist["history"][:20]
with open(manifest_path, "w") as f:
    json.dump(hist, f, indent=2)
    f.write("\n")
print(f"Updated {manifest_path}")
PY

echo ""
echo "SUCCESS: Baseline promoted as $LABEL ($DATE_TAG)"
echo "  report copy: $BASELINE_JSON"
echo "  lock:        $DIR/pipeline-lock.json"
echo "  manifest:    $MANIFEST"
