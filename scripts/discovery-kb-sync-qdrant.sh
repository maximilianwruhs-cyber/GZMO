#!/usr/bin/env bash
# Incremental honeypot Qdrant sync after discovery distill (closes vector lag).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ "${DISCOVERY_KB_SKIP_QDRANT_SYNC:-0}" == "1" ]]; then
  echo "SKIP: DISCOVERY_KB_SKIP_QDRANT_SYNC=1"
  exit 0
fi

if [[ -x "$ROOT/scripts/sync-vault-to-qdrant.sh" ]]; then
  echo "Discovery KB: honeypot Qdrant sync"
  "$ROOT/scripts/sync-vault-to-qdrant.sh" --source honeypot
else
  echo "WARN: sync-vault-to-qdrant.sh missing" >&2
  exit 1
fi
