#!/usr/bin/env bash
# Unpark Wave 1.2 demable: report Pi glass hygiene; optional fix via MCP_ATTACH_FIX.
#   bash scripts/pi-glass-fix.sh
#   MCP_ATTACH_FIX=1 bash scripts/pi-glass-fix.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/pi-glass"
mkdir -p "$OUT"

bash "$ROOT/scripts/pi-glass-check.sh" | tee "$OUT/check.txt"
if [[ "${MCP_ATTACH_FIX:-0}" == "1" ]]; then
  echo "[*] MCP_ATTACH_FIX=1 → mcp-attach-check fix path"
  MCP_ATTACH_FIX=1 bash "$ROOT/scripts/mcp-attach-check.sh" || true
  bash "$ROOT/scripts/pi-glass-check.sh" | tee "$OUT/check-after-fix.txt"
fi
echo "[OK] Pi glass report → $OUT"
