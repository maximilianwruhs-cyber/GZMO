#!/usr/bin/env bash
# Phase 1 extension: does Lorenz modulation + multi-agent handoff beat static single-agent?
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BENCH="$ROOT/attractorbench"
OUT_DIR="$ROOT/_foundation-audit/survey_GZMO/research/output"
mkdir -p "$OUT_DIR"

ENDPOINT="${MAS_ENDPOINT:-http://localhost:1234/v1}"
MODEL="${MAS_MODEL:-gemma-4-E4B-it-Q4_K_M.gguf}"
RUNS="${MAS_RUNS:-5}"

cd "$BENCH"
cargo build --release

echo "==> Chaos modulation on multi_agent suite (lorenz vs static)"
./target/release/attractorbench compare \
  --strategies lorenz,static \
  --suite multi_agent \
  --runs "$RUNS" \
  --endpoint "$ENDPOINT" \
  --model "$MODEL" \
  --output "$OUT_DIR/chaos_mas_synergy.html"

echo "Saved $OUT_DIR/chaos_mas_synergy.html"
