#!/usr/bin/env bash
# Pi / external ritual bridge → `gzmo chaos skill`.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <command> [args...]" >&2
  echo "Example: $0 dice d20 --json" >&2
  exit 2
fi

TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"
GZMO_BIN="${GZMO_BIN:-$TARGET_DIR/release/gzmo}"
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$TARGET_DIR/debug/gzmo"
fi
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/target/release/gzmo"
fi
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/target/debug/gzmo"
fi

exec "$GZMO_BIN" chaos skill "$@"
