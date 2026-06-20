#!/usr/bin/env bash
# Smoke test: Dream merged pipeline + halved chunk retry (unit tests, no LLM).
set -euo pipefail

ROOT="${GZMO_ROOT:-$HOME/Projects/_foundation-audit/survey_GZMO}"
cd "$ROOT"

echo "dream-pipeline-smoke: kg_extract merged_pipeline halved retry"
cargo test -p gzmo-core merged_pipeline_halved_retry -- --nocapture

echo "dream-pipeline-smoke: cycle_guard"
cargo test -p gzmo-core cycle_guard -- --nocapture

echo "dream-pipeline-smoke: OK"
