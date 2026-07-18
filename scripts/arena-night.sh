#!/usr/bin/env bash
# Obolus Arena nightburst spike — overnight-shaped tasks on this workstation.
# Scores duration × recall hit rate; writes data-next/arena/latest.json.
# Champion suggestion is sibling-only (never overwrites live gzmo-next.toml).
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-$HOME/github-clone}/GZMO"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
export GZMO_BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
export ARENA_ENGINE_LABEL="${ARENA_ENGINE_LABEL:-prime-local}"
export ARENA_OUT_DIR="$ROOT/data-next/arena"

mkdir -p "$ARENA_OUT_DIR"
exec python3 "$ROOT/scripts/arena_night.py"
