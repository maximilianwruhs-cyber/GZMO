#!/usr/bin/env bash
# One-shot nightburst operator bundle (no always-on serve required).
# Runs organ-trace → faithfulness → concept-gate → serendipity → hsp-sonify → scoreboard.
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
run hsp-sonify bash scripts/hsp-metabolism-sonify.sh
run euro-night bash scripts/euro-night-aggregate.sh
run price-window bash scripts/price-window-suggest.sh
run price-shift bash scripts/price-shift-soft.sh
run concept-webhook bash scripts/concept-gate-webhook.sh
run forge-mutate bash scripts/obolus-forge-mutate.sh
run ipw-route bash scripts/ipw-route.sh --task overnight
run cognition-pack bash scripts/cognition-pack.sh
run tinyfolder bash scripts/tinyfolder-drop.sh --demo
run beat-gate bash scripts/beat-gate-kit.sh
run zpd-lab bash scripts/zpd-tutor-lab.sh
run scoreboard bash scripts/nightburst-scoreboard.sh
run aos-poll bash scripts/aos-gzmo-poll.sh

echo "=== nightburst bundle done ==="
echo "Scoreboard: $ROOT/data-next/arena/scoreboard.html"
echo "Organ trace: $ROOT/data-next/organ-trace/latest.md"
echo "Faithfulness: $ROOT/data-next/faithfulness/latest.json"
echo "Concept gate: $ROOT/data-next/concept-gate/latest.md"
echo "Serendipity: $ROOT/data-next/serendipity/latest.md"
echo "HSP motif: $ROOT/data-next/hsp-metabolism/latest.md"
echo "€/night: $ROOT/data-next/arena/euro-night.json"
echo "Price window: $ROOT/data-next/price-window/latest.md"
echo "Price shift: $ROOT/data-next/scheduler-runs/latest-price-shift.json"
echo "Concept webhook: $ROOT/data-next/concept-gate/webhook-latest.json"
echo "Forge: $ROOT/data-next/arena/forge/latest.json"
echo "IpW route: $ROOT/data-next/ipw-router/latest.json"
echo "Cognition pack: $ROOT/data-next/cognition-pack/latest.json"
echo "tinyFolder: $ROOT/data-next/tinyfolder/latest.json"
echo "Beat-gate: $ROOT/data-next/beat-gate/latest.json"
echo "ZPD lab: $ROOT/data-next/zpd-tutor/latest.json"
echo "AOS feed: $ROOT/data-next/aos-status/latest.json"
exit "$fail"
