#!/usr/bin/env bash
# Kurator monitor unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo test -p gzmo-core kurator_monitor --quiet
echo "PASS: Kurator monitor tests"
