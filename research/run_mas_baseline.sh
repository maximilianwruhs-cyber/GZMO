#!/usr/bin/env bash
# Phase 1 baseline: compare text-based MAS handoff costs via AttractorBench.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BENCH="$ROOT/attractorbench"
OUT_DIR="$ROOT/_foundation-audit/survey_GZMO/research/output"
RESEARCH_DIR="$ROOT/_foundation-audit/survey_GZMO/research"
mkdir -p "$OUT_DIR"

# Optional Python venv for RecursiveMAS bridge (FastAPI). Falls back to stdlib server.
BRIDGE_PY="$RESEARCH_DIR/recursivemas_bridge.py"

ENDPOINT="${MAS_ENDPOINT:-http://localhost:1234/v1}"
MODEL="${MAS_MODEL:-gemma-4-E4B-it-Q4_K_M.gguf}"
RUNS="${MAS_RUNS:-3}"

cd "$BENCH"
cargo build --release

echo "==> MAS baseline: single vs two_agent vs four_agent"
./target/release/attractorbench mas-compare \
  --modes single,two_agent,four_agent \
  --suite multi_agent \
  --runs "$RUNS" \
  --endpoint "$ENDPOINT" \
  --model "$MODEL" \
  --output "$OUT_DIR/mas_text_baseline.json"

if [[ -n "${RECURSIVEMAS_URL:-}" ]]; then
  echo "==> Latent bridge comparison (RecursiveMAS)"
  if ! pgrep -f "recursivemas_bridge" >/dev/null 2>&1; then
    RECURSIVEMAS_MOCK="${RECURSIVEMAS_MOCK:-1}" python3 "$BRIDGE_PY" --port "${RECURSIVEMAS_PORT:-8765}" &
    sleep 1
    RECURSIVEMAS_URL="${RECURSIVEMAS_URL:-http://127.0.0.1:8765}"
  fi
  ./target/release/attractorbench mas-compare \
    --modes single,two_agent,recursive_mas \
    --suite multi_agent \
    --runs "$RUNS" \
    --endpoint "$ENDPOINT" \
    --model "$MODEL" \
    --recursive-mas-url "$RECURSIVEMAS_URL" \
    --output "$OUT_DIR/mas_latent_compare.json"
fi

echo "Results in $OUT_DIR"
