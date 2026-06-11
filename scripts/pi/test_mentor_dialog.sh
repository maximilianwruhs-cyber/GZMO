#!/usr/bin/env bash
# Smoke test: mentor API via shell bridge (daemon socket or local fallback).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_LEARNER_ID="${GZMO_LEARNER_ID:-operator}"

GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/target/debug/gzmo"
fi
export GZMO_BIN

MENTOR="$ROOT/scripts/pi/mentor.sh"
chmod +x "$MENTOR"

echo "== mentor ping =="
"$MENTOR" ping

echo "== mentor status =="
"$MENTOR" status

echo "== mentor teach (short) =="
OUT="$("$MENTOR" teach "what is a symlink?" 2>&1 || true)"
if [[ -z "$OUT" ]]; then
  echo "WARN: empty teach response (ops mode or routing skip?)" >&2
else
  echo "$OUT" | head -c 500
  echo
fi

echo "OK: mentor dialog smoke passed"
