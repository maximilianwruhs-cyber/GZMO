#!/usr/bin/env bash
# Build gzmo into ./target (not Cursor sandbox cache).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
unset CARGO_TARGET_DIR
export CARGO_TARGET_DIR="$ROOT/target"
cargo build -p gzmo-cli --release "$@"
echo "Built: $ROOT/target/release/gzmo"
