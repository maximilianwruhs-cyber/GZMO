#!/usr/bin/env bash
# Pi / external ritual bridge → `gzmo chaos skill`.
# Picks a binary that actually implements the one-shot skill path (not a stale
# release that falls through into interactive chat).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <command> [args...]" >&2
  echo "Example: $0 dice d20 --json" >&2
  exit 2
fi

supports_chaos_skill() {
  local bin="$1"
  [[ -x "$bin" ]] || return 1
  # Stale binaries ignore `chaos skill` and open chat — refuse those.
  local help
  help="$("$bin" chaos skill help 2>/dev/null || true)"
  echo "$help" | grep -q 'gzmo chaos skill' || return 1
  echo "$help" | grep -qiE 'PulseLoop|living daemon' || return 1
  return 0
}

pick_bin() {
  local cand
  if [[ -n "${GZMO_BIN:-}" ]]; then
    if supports_chaos_skill "$GZMO_BIN"; then
      echo "$GZMO_BIN"
      return 0
    fi
    echo "GZMO_BIN=$GZMO_BIN does not support 'chaos skill' (stale?). Unset or rebuild." >&2
  fi
  local target_dir="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"
  for cand in \
    "$target_dir/release/gzmo" \
    "$target_dir/debug/gzmo" \
    "$ROOT/target/release/gzmo" \
    "$ROOT/target/debug/gzmo"; do
    if supports_chaos_skill "$cand"; then
      echo "$cand"
      return 0
    fi
  done
  return 1
}

GZMO_BIN="$(pick_bin)" || {
  echo "No gzmo binary with 'chaos skill' found." >&2
  echo "Rebuild: cargo build -p gzmo-cli --release" >&2
  echo "Or: CARGO_TARGET_DIR=\$HOME/github-clone/temp-bench/target cargo build -p gzmo-cli --release" >&2
  exit 1
}

exec "$GZMO_BIN" chaos skill "$@"
