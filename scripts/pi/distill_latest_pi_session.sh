#!/usr/bin/env bash
# Distill the newest Pi session JSONL into GZMO vault/episodic memory.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/gzmo.toml}"

GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/target/debug/gzmo"
fi

PI_SESSION="${1:-}"
if [[ -z "$PI_SESSION" ]]; then
  PI_SESSION="$(find "${HOME}/.pi/agent/sessions" -name '*.jsonl' -type f 2>/dev/null | sort -r | head -1 || true)"
fi

if [[ -z "$PI_SESSION" || ! -f "$PI_SESSION" ]]; then
  echo "ERROR: no Pi session .jsonl found under ~/.pi/agent/sessions" >&2
  exit 1
fi

echo "Distilling Pi session: $PI_SESSION"
exec "$GZMO_BIN" distill pi "$PI_SESSION"
