#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo test -p gzmo-core context_compress::tests::test_run_benchmarks -- --nocapture
python3 scripts/compression-bench/compare_rust_vs_fixtures.py
