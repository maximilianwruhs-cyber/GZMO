#!/usr/bin/env bash
# Upsert vault.db embeddings into Qdrant (see sync-vault-to-qdrant.py).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "${ROOT}/scripts/sync-vault-to-qdrant.py" "$@"
