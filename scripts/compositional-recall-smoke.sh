#!/usr/bin/env bash
# Compositional recall smoke — thema_009 / Verified Chain Recall.
# Mirrors discovery-kb-recall-smoke.sh; runs the three compositional probes
# (hop-1 fidelity, chain recall, hop-2 atomic difficulty) over mined Neo4j chains.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROBE="$ROOT/scripts/compositional-recall-probe.py"
OUT_DIR="$ROOT/data/discovery-kb-metrics"
CHAINS="${COMPOSITIONAL_CHAINS:-10}"
LIMIT="${COMPOSITIONAL_LIMIT:-5}"

mkdir -p "$OUT_DIR"

if [[ ! -f "$PROBE" ]]; then
  echo "[FAIL] probe missing: $PROBE" >&2
  exit 1
fi

PYTHON="${PYTHON:-python3}"
"$PYTHON" "$PROBE" --chains "$CHAINS" --limit "$LIMIT"
