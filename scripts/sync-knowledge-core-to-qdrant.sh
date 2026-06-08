#!/usr/bin/env bash
# Sync knowledge_core.db concept cards → Qdrant knowledge_core collection.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec python3 "${ROOT}/scripts/sync-knowledge-core-to-qdrant.py" "$@"
