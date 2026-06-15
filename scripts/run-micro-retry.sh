#!/usr/bin/env bash
# Quality micro pass: semantic splits + full verify (no verify-off shortcuts).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
set -a && source .env && set +a
export CLOUD_INGEST=1 QDRANT_SYNC=1 SKIP_BUILD=1

if grep -q '^verify = false' "$ROOT/gzmo.toml"; then
  echo "[!] ingest verify disabled — refusing micro pass without quality gate" >&2
  exit 1
fi

python3 "$HOME/Schreibtisch/sidecar-migration/scripts/split-for-ingest.py" --from-progress

./scripts/slow-reingest-migration.sh \
  --manifest scripts/ingest-quality/wave-retry-micro.manifest \
  --interval 0 || true

./scripts/sync-vault-to-qdrant.sh || true
./scripts/memory-status.sh || true
