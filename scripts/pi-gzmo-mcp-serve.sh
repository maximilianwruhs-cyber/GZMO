#!/usr/bin/env bash
# Stdio MCP bridge: run living gzmo mcp-serve on CT101 (correct vault).
# Used by Pi/Cursor via config/shared-mcp-memory.json.
set -euo pipefail

HOST="${CT101_SSH_HOST:-ct101}"
BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"

REMOTE_CMD="cd /opt/gzmo && export GZMO_CONFIG=/opt/gzmo/gzmo.toml"
if [[ -n "${GZMO_SESSION_ID:-}" ]]; then
  REMOTE_CMD+=" && export GZMO_SESSION_ID=$(printf '%q' "$GZMO_SESSION_ID")"
fi
REMOTE_CMD+=" && exec $(printf '%q' "$BIN") mcp-serve"

exec ssh -o ConnectTimeout=15 -o BatchMode=yes "$HOST" "bash -lc $(printf '%q' "$REMOTE_CMD")"
