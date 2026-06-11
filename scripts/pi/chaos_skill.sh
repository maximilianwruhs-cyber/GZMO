#!/usr/bin/env bash
# Pi / external agent bridge → `gzmo chaos skill` (Rust pantheon + daemon IPC).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <command> [args...]" >&2
  echo "Example: $0 card creature" >&2
  exit 2
fi

CMD="$1"
shift || true
ARGS="$*"

GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/target/debug/gzmo"
fi

exec "$GZMO_BIN" chaos skill "$CMD" $ARGS
