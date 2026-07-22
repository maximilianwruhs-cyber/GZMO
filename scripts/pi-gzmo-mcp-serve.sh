#!/usr/bin/env bash
# Stdio MCP bridge: run living gzmo mcp-serve on CT101 (correct vault).
# Wired as MCP server name **gzmo-living** (goal C). Product uses **gzmo-memory**.
# Never starts local gzmo-serve. Never dual-writes. Docs: docs/EXTERNAL_LIVING_ATTACH.md
set -euo pipefail

# Fail closed: lite/lab markers must not ride the living bridge
if [[ "${GZMO_PRODUCT:-}" == "1" ]]; then
  echo "REFUSE: GZMO_PRODUCT=1 conflicts with living bridge (use gzmo-memory for lite)" >&2
  exit 1
fi
if [[ "${GZMO_ALLOW_LAB_VAULT:-}" == "1" ]]; then
  echo "REFUSE: GZMO_ALLOW_LAB_VAULT=1 forbidden on living attach" >&2
  exit 1
fi

HOST="${CT101_SSH_HOST:-ct101}"
BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"

# Remote living config only — do not inherit workstation lab paths
REMOTE_CMD="cd /opt/gzmo && export GZMO_CONFIG=/opt/gzmo/gzmo.toml"
if [[ -n "${GZMO_SESSION_ID:-}" ]]; then
  REMOTE_CMD+=" && export GZMO_SESSION_ID=$(printf '%q' "$GZMO_SESSION_ID")"
fi
# mcp-serve only (read/search attach). Never: gzmo serve / enable overnight writer.
REMOTE_CMD+=" && exec $(printf '%q' "$BIN") mcp-serve"

exec ssh -o ConnectTimeout=15 -o BatchMode=yes "$HOST" "bash -lc $(printf '%q' "$REMOTE_CMD")"
