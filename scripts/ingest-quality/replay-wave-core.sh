#!/usr/bin/env bash
# Prime eval on 15 Batch-1 core golden files (~5–8 min), merge into report.json.
# Usage: replay-wave-core.sh
#   MERGE=0     — write only report-core-partial.json
#   SKIP_BUILD=1 — skip cargo build

set -eo pipefail
export LC_ALL=C
export LC_NUMERIC=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$PROJECT_ROOT"

CORPUS="${GZMO_WAVE1_CORPUS:-$HOME/Schreibtisch/knowledge/archive/gzmo_obolus}"
STAGING="$DIR/.core-corpus"
PARTIAL="$DIR/report-core-partial.json"
BASE="$DIR/report.json"
LIST="$DIR/core-golden-files.txt"
MERGE="${MERGE:-1}"
SKIP_BUILD="${SKIP_BUILD:-0}"

rm -rf "$STAGING"
mkdir -p "$STAGING"
linked=0
while IFS= read -r f || [[ -n "$f" ]]; do
  [[ -z "$f" || "$f" =~ ^# ]] && continue
  src="$CORPUS/$f"
  if [[ -f "$src" ]]; then
    ln -sf "$src" "$STAGING/$f"
    linked=$((linked + 1))
  else
    echo "[!] missing: $f" >&2
  fi
done <"$LIST"

echo "[*] core staging: $linked files"
if [[ "$linked" -eq 0 ]]; then
  echo "replay-wave-core: no files linked" >&2
  exit 2
fi

if [[ "$SKIP_BUILD" != "1" ]]; then
  unset CARGO_TARGET_DIR
  cargo build --release -p gzmo-cli -q
fi

echo "[*] ingest-eval core golden (Prime)..."
RUST_LOG=warn ./target/release/gzmo ingest-eval "$STAGING" >"$PARTIAL" 2>>"$DIR/replay-wave-core.stderr.log"

if [[ "$MERGE" == "1" ]]; then
  if [[ ! -f "$BASE" ]]; then
    echo "[!] No $BASE — copying partial as new report" >&2
    cp "$PARTIAL" "$BASE"
  else
    python3 "$DIR/merge-report-partial.py" --partial "$PARTIAL" --base "$BASE" --write
  fi
  echo ""
  bash "$DIR/gate-report.sh" "$BASE"
  bash "$DIR/check-contract.sh" "$BASE" || true
else
  echo "[*] partial only: $PARTIAL"
  bash "$DIR/gate-report.sh" "$PARTIAL" || true
fi
