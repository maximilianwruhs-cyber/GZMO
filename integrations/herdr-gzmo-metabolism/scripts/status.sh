#!/usr/bin/env bash
set -euo pipefail
# shellcheck disable=SC1091
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"
export_gzmo_env

echo "herdr plugin: ${HERDR_PLUGIN_ID:-gzmo.metabolism}"
echo "gzmo bin:     $GZMO_BIN"
echo "gzmo config:  $GZMO_CONFIG"
echo "instance:     ${GZMO_INSTANCE:-"(unset)"}"
echo "state dir:    $STATE_DIR"

if [[ -f "$STATE_DIR/mcp-ensure-latest.json" ]]; then
  echo "--- mcp ensure ---"
  cat "$STATE_DIR/mcp-ensure-latest.json"
fi
if [[ -f "$STATE_DIR/session-close-latest.json" ]]; then
  echo "--- last session close ---"
  cat "$STATE_DIR/session-close-latest.json"
fi
if [[ -f "$STATE_DIR/missed-close.jsonl" ]]; then
  echo "--- missed close (last 5) ---"
  tail -n 5 "$STATE_DIR/missed-close.jsonl"
fi

echo "--- memory status ---"
"$GZMO_BIN" memory status --json 2>/dev/null || "$GZMO_BIN" memory status || true
