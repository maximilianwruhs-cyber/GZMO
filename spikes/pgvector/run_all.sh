#!/usr/bin/env bash
# run_all.sh — orchestrate docker → import → recall on CT101 (read-only of prod vault).
set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
chmod +x docker-run.sh teardown.sh import_vault.py recall_bench.py 2>/dev/null || true

echo "==== 1. docker-run ===="
./docker-run.sh

echo "==== 2. import_vault ===="
python3 ./import_vault.py --schema "$DIR/schema.sql"

echo "==== 3. recall_bench ===="
python3 ./recall_bench.py

echo "==== done (run teardown.sh BEFORE commit) ===="
ls -la results.json import_counts.json
