#!/usr/bin/env bash
# /status — deterministic GZMO ecosystem snapshot (GZMO-next)
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-/home/gzmo/github-clone}"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/GZMO/config/gzmo-next.toml}"

GZMO_BIN="${CARGO_TARGET_DIR:-$ROOT/temp-bench/target}/release/gzmo"
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/GZMO/target/release/gzmo"
fi

exec "$GZMO_BIN" status
