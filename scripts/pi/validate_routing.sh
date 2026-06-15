#!/usr/bin/env bash
# Validate Obolus routing keys in gzmo.toml resolve (Stage 4).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
CONFIG="${GZMO_CONFIG:-$ROOT/gzmo.toml}"
[[ -f "$CONFIG" ]] || { echo "FAIL: missing $CONFIG"; exit 1; }
rg -q '^\[routing\]' "$CONFIG" && echo "OK  [routing] section" || { echo "FAIL: no [routing]"; exit 1; }
rg -q '^\[routing\.mappings\]' "$CONFIG" && echo "OK  [routing.mappings]" || { echo "FAIL: no mappings"; exit 1; }
for key in dream_extract dream_verify spark_hypothesis spark_verify ingest_extract distill_summary; do
  if rg -q "^${key}\s*=" "$CONFIG"; then
    echo "OK  mapping $key"
  else
    echo "WARN mapping $key absent (uses default_engine)"
  fi
done
echo "PASS: routing config present"
