#!/usr/bin/env bash
# One-shot nightburst operator bundle (no always-on serve required).
# Runs organ-trace → faithfulness → concept-gate → serendipity → scoreboard.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export GZMO_CLONE_ROOT="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"

cd "$ROOT"
fail=0

run() {
  local name="$1"
  shift
  echo "=== $name ==="
  if "$@"; then
    echo "[OK] $name"
  else
    echo "[HOLD/FAIL] $name (continuing)" >&2
    fail=1
  fi
  echo
}

run organ-trace bash scripts/organ-trace.sh
run faithfulness bash scripts/faithfulness-ci.sh
run concept-gate bash scripts/concept-review-gate.sh
run serendipity bash scripts/serendipity-digest.sh
run scoreboard bash scripts/nightburst-scoreboard.sh

echo "=== nightburst bundle done ==="
echo "Scoreboard: $ROOT/data-next/arena/scoreboard.html"
echo "Organ trace: $ROOT/data-next/organ-trace/latest.md"
echo "Faithfulness: $ROOT/data-next/faithfulness/latest.json"
echo "Concept gate: $ROOT/data-next/concept-gate/latest.md"
echo "Serendipity: $ROOT/data-next/serendipity/latest.md"
exit "$fail"
