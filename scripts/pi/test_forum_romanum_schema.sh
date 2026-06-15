#!/usr/bin/env bash
# Forum Romanum MVP — serde round-trip via gzmo-core unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo test -p gzmo-core synapse::tests --quiet
echo "PASS: Forum Romanum schema serde tests"
