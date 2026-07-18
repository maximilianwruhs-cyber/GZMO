#!/usr/bin/env bash
# Run concept-review-gate, then gzmo wiki push only on PASS.
# Soft operator path for OKForge (serve satellite also honors concept-gate HOLD).
#
#   bash scripts/wiki-push-gated.sh [--dry-run] [--origin NAME] …
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"

BIN="${GZMO_BIN:-}"
if [[ -z "$BIN" ]]; then
  if [[ -x "$CARGO_TARGET_DIR/release/gzmo" ]]; then
    BIN="$CARGO_TARGET_DIR/release/gzmo"
  elif command -v gzmo >/dev/null 2>&1; then
    BIN="$(command -v gzmo)"
  else
    echo "[!] gzmo binary not found" >&2
    exit 1
  fi
fi

echo "=== concept-gate ==="
if ! bash "$ROOT/scripts/concept-review-gate.sh"; then
  echo "[HOLD] refusing wiki push — fix vault evidence or GZMO_CONCEPT_GATE=0" >&2
  exit 1
fi

echo "=== wiki push ==="
export GZMO_CONCEPT_GATE="${GZMO_CONCEPT_GATE:-1}"
exec "$BIN" wiki push --require-gate "$@"
