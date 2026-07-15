#!/usr/bin/env bash
# Sync data-next vault honeypot embeddings → local Qdrant (GZMO-next).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(cd "$ROOT/.." && pwd)}"

VAULT="${VAULT_PATH:-$ROOT/data-next/vault.db}"
if [[ -n "${GZMO_CONFIG:-}" && -f "$GZMO_CONFIG" ]]; then
  # Prefer vault path from instance config when VAULT_PATH unset
  VAULT="$(python3 - <<'PY' "$GZMO_CONFIG"
import sys, tomllib
from pathlib import Path
cfg = Path(sys.argv[1])
base = cfg.parent
data = tomllib.loads(cfg.read_text())
p = Path(data.get("memory", {}).get("vault_db", "../data-next/vault.db"))
print((base / p).resolve())
PY
)"
fi

QDRANT_URL="${QDRANT_URL:-http://127.0.0.1:6333}"
COLLECTION="${QDRANT_COLLECTION:-honeypot}"

if [[ ! -f "$VAULT" ]]; then
  echo "qdrant-vault-sync: vault missing at $VAULT" >&2
  exit 1
fi

exec python3 "$ROOT/scripts/sync-vault-to-qdrant.py" \
  --db "$VAULT" \
  --url "$QDRANT_URL" \
  --collection "$COLLECTION" \
  --source honeypot
