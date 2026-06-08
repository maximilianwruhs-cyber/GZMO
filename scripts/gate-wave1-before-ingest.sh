#!/usr/bin/env bash
# Block B — pre-ingest gate on wave-1 corpus before live ingest.
set -eo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
cd "$REPO"

CORPUS="${GZMO_WAVE1_CORPUS:-$HOME/Schreibtisch/knowledge/archive/gzmo_obolus}"
LOG="logs/pre-ingest-wave1-$(date +%Y%m%d-%H%M%S).log"
MANIFEST="scripts/ingest-quality/wave1-ingest-ready.manifest"
SAMPLE="${PRE_INGEST_STAGE2_SAMPLE:-5}"

echo "Corpus: $CORPUS"
echo "Log: $LOG"

unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli -q

# Stage 1 — full corpus (no quarantine on production archive)
scripts/pre-ingest-gate.sh "$CORPUS" \
  --manifest "$MANIFEST" \
  2>&1 | tee "$LOG"
stage1=${PIPESTATUS[0]}

echo ""
echo "stage1_exit=$stage1 manifest_lines=$(wc -l <"$MANIFEST" 2>/dev/null || echo 0)"

if [[ "$SAMPLE" -gt 0 ]] && [[ -s "$MANIFEST" ]]; then
  echo ""
  echo "Stage 2 smoke ($SAMPLE files, Prime)..."
  i=0
  while IFS= read -r f && [[ "$i" -lt "$SAMPLE" ]]; do
    [[ -f "$f" ]] || continue
    echo "--- stage2: $(basename "$f")" | tee -a "$LOG"
    if RUST_LOG=error ./target/release/gzmo ingest --dry-run "$f" 2>&1 | tail -5 | tee -a "$LOG"; then
      echo "  stage2_ok" | tee -a "$LOG"
    else
      echo "  stage2_fail" | tee -a "$LOG"
    fi
    i=$((i + 1))
  done < <(head -n "$SAMPLE" "$MANIFEST")
fi

echo ""
echo "Next live ingest (human):"
echo "  # one file:"
echo "  ./target/release/gzmo ingest <path-from-manifest>"
echo "  # or directory (after reviewing manifest):"
echo "  ./target/release/gzmo ingest-dir $CORPUS"

exit "$stage1"
