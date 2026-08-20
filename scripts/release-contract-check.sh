#!/usr/bin/env bash
# Release contract gate — verifies the gzmo release binary and source still
# honor the public corpus-ingest / labeled-recall contract before packaging a
# release. See .superpowers/sdd/2026-08-20-gzmo-demo/task-2-brief.md Step 4.
#
#   bash scripts/release-contract-check.sh
#
# Builds the release binary into a temporary CARGO_TARGET_DIR (so this never
# clobbers a developer's own `target/`), then checks:
#   - `corpus ingest-dir --help` still documents the ingest-dir subcommand
#   - `memory search --help` still documents `--json`
#   - `MemoryHit` still labels corpus_passage / promoted_fact hits
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TARGET_DIR="$(mktemp -d)"
trap 'rm -rf "$TARGET_DIR"' EXIT

pass=0
fail=0

row() {
  local status="$1" desc="$2"
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
  esac
  echo "[$status] $desc"
}

# `2>&1 | grep -q ...` is run inside its own `bash -c` (no `set -o pipefail`
# there), so the pipeline's exit status is grep's alone — the CLI's own exit
# code (e.g. `--help` args triggering a usage error) is intentionally ignored.
check_pipe() {
  local desc="$1" cmd="$2"
  if bash -c "$cmd"; then
    row PASS "$desc"
  else
    row FAIL "$desc"
  fi
}

echo "=== Release contract check ==="
echo "Building release binary in temporary target dir: $TARGET_DIR"

if CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release -p gzmo-cli >"$TARGET_DIR/build.log" 2>&1; then
  row PASS "release build (cargo build --release -p gzmo-cli)"
else
  row FAIL "release build (cargo build --release -p gzmo-cli)"
  tail -n 40 "$TARGET_DIR/build.log" || true
fi

bin="$TARGET_DIR/release/gzmo"
if [[ -x "$bin" ]]; then
  row PASS "release binary present at $bin"

  check_pipe "corpus ingest-dir --help documents the ingest-dir subcommand" \
    "\"$bin\" corpus ingest-dir --help 2>&1 | grep -q 'corpus ingest-dir'"

  check_pipe "memory search --help documents --json" \
    "\"$bin\" memory search --help 2>&1 | grep -q -- '--json'"
else
  row FAIL "release binary present at $bin"
  row FAIL "corpus ingest-dir --help documents the ingest-dir subcommand (binary missing)"
  row FAIL "memory search --help documents --json (binary missing)"
fi

if grep -q 'corpus_passage' "$ROOT/gzmo-core/src/platform_memory.rs"; then
  row PASS "platform_memory.rs labels corpus_passage hits"
else
  row FAIL "platform_memory.rs labels corpus_passage hits"
fi

if grep -q 'promoted_fact' "$ROOT/gzmo-core/src/platform_memory.rs"; then
  row PASS "platform_memory.rs labels promoted_fact hits"
else
  row FAIL "platform_memory.rs labels promoted_fact hits"
fi

echo "=== $pass passed, $fail failed ==="
if (( fail > 0 )); then
  echo "=== release contract check FAILED ==="
  exit 1
fi

echo "=== release contract check PASSED ==="
