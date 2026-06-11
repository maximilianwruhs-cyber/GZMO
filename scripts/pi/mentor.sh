#!/usr/bin/env bash
# Pi / external agent bridge → `gzmo mentor` (daemon Unix socket or local fallback).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

export GZMO_LEARNER_ID="${GZMO_LEARNER_ID:-operator}"

pick_gzmo_bin() {
  local release="$ROOT/target/release/gzmo"
  local debug="$ROOT/target/debug/gzmo"
  if [[ -n "${GZMO_BIN:-}" && -x "$GZMO_BIN" ]]; then
    printf '%s' "$GZMO_BIN"
    return
  fi
  if [[ -x "$release" && -x "$debug" ]]; then
    if [[ "$release" -nt "$debug" ]]; then
      printf '%s' "$release"
    else
      printf '%s' "$debug"
    fi
    return
  fi
  if [[ -x "$release" ]]; then
    printf '%s' "$release"
  elif [[ -x "$debug" ]]; then
    printf '%s' "$debug"
  else
    echo "gzmo binary not found (build with: cargo build -p gzmo-cli --release)" >&2
    exit 1
  fi
}

GZMO_BIN="$(pick_gzmo_bin)"
export GZMO_BIN

if [[ "${1:-}" == "teach" && -n "${MENTOR_JSON:-}" ]]; then
  exec "$GZMO_BIN" mentor teach --json-file "$MENTOR_JSON"
fi

exec "$GZMO_BIN" mentor "$@"
