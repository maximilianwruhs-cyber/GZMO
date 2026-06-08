#!/usr/bin/env bash
set -eo pipefail
export LC_ALL=C
export LC_NUMERIC=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$PROJECT_ROOT"

unset CARGO_TARGET_DIR

REPORT_PATH="scripts/ingest-quality/report.json"
CORPUS_DIR="/home/maximilian-wruhs/Schreibtisch/knowledge/archive/gzmo_obolus"
REPORTS_DIR="scripts/ingest-quality/reports"
ARCHIVE="${ARCHIVE_REPORT:-1}"

usage() {
  cat <<EOF
Usage: replay-wave.sh [options]

  Full dry-run eval (default):
    scripts/ingest-quality/replay-wave.sh

  YAML / contract only (no Prime, ~1s):
    RESCORE_ONLY=1 scripts/ingest-quality/replay-wave.sh
    scripts/ingest-quality/check-contract.sh [report.json]

  Gate table on existing report:
    scripts/ingest-quality/gate-report.sh [report.json]

Options via env:
  RESCORE_ONLY=1     Skip ingest-eval; gate current report.json
  GATE_MODE=strict   Override gate-config (strict | layered)
  ARCHIVE_REPORT=0   Do not copy report to reports/run-*.json
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ "${RESCORE_ONLY:-0}" == "1" ]]; then
  echo "=== Rescore-only mode (deterministic contract + gate on existing report) ==="
  bash "$DIR/check-contract.sh" "$REPORT_PATH"
  echo ""
  bash "$DIR/gate-report.sh" "$REPORT_PATH"
  exit $?
fi

echo "=== Building gzmo-cli ==="
cargo build --release -p gzmo-cli

echo "=== Running dry-run ingest evaluation on $CORPUS_DIR ==="
RUST_LOG=warn ./target/release/gzmo ingest-eval "$CORPUS_DIR" > "$REPORT_PATH" 2>>scripts/ingest-quality/replay-wave.stderr.log

echo "=== Evaluation Complete ==="
echo "Report written to $REPORT_PATH"

if [[ "$ARCHIVE" == "1" ]]; then
  mkdir -p "$REPORTS_DIR"
  STAMP="$(date +%Y%m%d-%H%M%S)"
  ARCHIVE_PATH="$REPORTS_DIR/run-${STAMP}.json"
  cp "$REPORT_PATH" "$ARCHIVE_PATH"
  echo "Archived: $ARCHIVE_PATH"
  ls -1t "$REPORTS_DIR"/run-*.json 2>/dev/null | tail -n +11 | xargs -r rm -f
fi

echo ""
python3 "$DIR/refresh-report-contract.py" --report "$REPORT_PATH" 2>/dev/null || true
python3 "$DIR/recalc-pipeline-summary.py" --report "$REPORT_PATH" --write 2>/dev/null || true
bash "$DIR/gate-report.sh" "$REPORT_PATH"

if [[ "${PROMOTE_BASELINE:-0}" == "1" ]]; then
  echo ""
  BASELINE_LABEL="${BASELINE_LABEL:-baseline-m4-current}" bash "$DIR/promote-baseline.sh" "$REPORT_PATH"
fi
