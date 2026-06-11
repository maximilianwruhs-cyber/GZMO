#!/usr/bin/env bash
# Smoke test: Pi JSONL parser + optional live distill (set GZMO_DISTILL_SMOKE=1).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/gzmo.toml}"

echo "== pi_session unit tests =="
unset CARGO_TARGET_DIR
export CARGO_TARGET_DIR="$ROOT/target"
cargo test -p gzmo-core pi_session --quiet
cargo test -p gzmo-core session_end_targets --quiet

PI_SESSION="${PI_SESSION:-}"
if [[ -z "$PI_SESSION" ]]; then
  PI_SESSION="$(find "${HOME}/.pi/agent/sessions" -name '*.jsonl' -type f 2>/dev/null | sort -r | head -1 || true)"
fi

if [[ -z "$PI_SESSION" || ! -f "$PI_SESSION" ]]; then
  echo "WARN: no Pi session jsonl found; parser tests only" >&2
  echo "OK: distill pi smoke (parser only)"
  exit 0
fi

echo "== sample session: $PI_SESSION =="

GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/target/debug/gzmo"
fi

if [[ "${GZMO_DISTILL_SMOKE:-0}" != "1" ]]; then
  echo "SKIP live distill (set GZMO_DISTILL_SMOKE=1 to run LLM path)"
  echo "OK: distill pi smoke (parser only)"
  exit 0
fi

echo "== gzmo distill pi (live) =="
OUT="$("$GZMO_BIN" distill pi "$PI_SESSION" 2>&1)"
echo "$OUT" | tail -8
if echo "$OUT" | grep -qE 'distilled|skipped|vault truths'; then
  echo "OK: distill pi smoke passed"
else
  echo "FAIL: unexpected distill output" >&2
  exit 1
fi
