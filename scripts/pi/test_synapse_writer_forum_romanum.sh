#!/usr/bin/env bash
# Forum Romanum emitters in synapse_writer.rs
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo test -p gzmo-core synapse_writer::tests::forum_romanum --quiet
echo "PASS: synapse_writer Forum Romanum emitters"
