#!/usr/bin/env bash
# Unpark Wave 1.1 demable: link herdr plugin + status (no auto-distill).
#   bash scripts/herdr-metabolism-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/herdr-metabolism"
mkdir -p "$OUT"

if ! command -v herdr >/dev/null 2>&1; then
  echo "[!] herdr not on PATH" >&2
  exit 1
fi

bash "$ROOT/scripts/herdr-metabolism-link.sh"
# Non-interactive status via plugin script (avoids REPL)
bash "$ROOT/integrations/herdr-gzmo-metabolism/scripts/status.sh" >"$OUT/status.txt" 2>&1 || true
bash "$ROOT/scripts/herdr-metabolism-check.sh"
echo "[OK] herdr metabolism demo → $OUT"
