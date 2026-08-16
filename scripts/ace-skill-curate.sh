#!/usr/bin/env bash
# ace-skill-curate.sh — thin wrapper around `gzmo workflow-skill curate`.
#
# Single curator: gzmo-core::workflow_skills::ace (no Python clone).
# Writes require ACE_PIN_APPLY=1 (same shape as IMMUNE_APPLY=1).
# Literature: ACE (arXiv:2510.04618, ICLR 2026).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

resolve_gzmo() {
  if [[ -n "${GZMO_BIN:-}" ]]; then
    if [[ -x "${GZMO_BIN}" ]]; then
      printf '%s\n' "$GZMO_BIN"
      return 0
    fi
    if [[ -x "$ROOT/${GZMO_BIN}" ]]; then
      printf '%s\n' "$ROOT/${GZMO_BIN}"
      return 0
    fi
  fi
  if command -v gzmo >/dev/null 2>&1; then
    command -v gzmo
    return 0
  fi
  local c
  local targets=("$ROOT/target/debug/gzmo" "$ROOT/target/release/gzmo")
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    targets+=("$CARGO_TARGET_DIR/debug/gzmo" "$CARGO_TARGET_DIR/release/gzmo")
  fi
  for c in "${targets[@]}"; do
    if [[ -x "$c" ]]; then
      printf '%s\n' "$c"
      return 0
    fi
  done
  return 1
}

if ! GZMO="$(resolve_gzmo)"; then
  echo "ace-skill-curate: gzmo binary not found. Build it:" >&2
  echo "  cargo build -p gzmo-cli --bin gzmo" >&2
  echo "Or set GZMO_BIN=/path/to/gzmo" >&2
  exit 1
fi

exec "$GZMO" workflow-skill curate "$@"
