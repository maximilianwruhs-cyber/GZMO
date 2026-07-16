#!/usr/bin/env bash
# Sample-verify honeypot IDs exist in Qdrant after nightly sync.
#
# Usage: qdrant-post-sync-verify.sh [--sample N] [--gzmo-root PATH]
# Exit 0 = all samples found, 1 = missing IDs or unreachable Qdrant
set -euo pipefail

SAMPLE="${QDRANT_VERIFY_SAMPLE:-10}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GZMO_ROOT="${GZMO_ROOT:-$ROOT}"
QDRANT_URL="${QDRANT_URL:-http://127.0.0.1:6333}"
COLLECTION="${QDRANT_COLLECTION:-honeypot}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --sample) SAMPLE="$2"; shift 2 ;;
    --gzmo-root) GZMO_ROOT="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

# Prefer explicit VAULT_PATH, then instance config, then GZMO-next data-next, then legacy.
VAULT="${VAULT_PATH:-}"
if [[ -z "$VAULT" && -n "${GZMO_CONFIG:-}" && -f "$GZMO_CONFIG" ]]; then
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
if [[ -z "$VAULT" || ! -f "$VAULT" ]]; then
  if [[ -f "$GZMO_ROOT/data-next/vault.db" ]]; then
    VAULT="$GZMO_ROOT/data-next/vault.db"
  else
    VAULT="$GZMO_ROOT/data/vault.db"
  fi
fi
if [[ ! -f "$VAULT" ]]; then
  echo "qdrant-post-sync-verify: vault missing at $VAULT" >&2
  exit 1
fi

python3 - <<PY
import json, random, sqlite3, sys, urllib.request

vault = "$VAULT"
qbase = "${QDRANT_URL%/}"
collection = "$COLLECTION"
sample_n = int("$SAMPLE")

conn = sqlite3.connect(vault)
rows = conn.execute(
    """SELECT id FROM honeypot
       WHERE is_latest=1 AND embedding IS NOT NULL AND length(embedding) >= 4
       ORDER BY RANDOM() LIMIT ?""",
    (sample_n,),
).fetchall()
if not rows:
    print("qdrant-post-sync-verify: no honeypot rows to sample")
    sys.exit(0)

missing = []
for (hid,) in rows:
    url = f"{qbase}/collections/{collection}/points/{hid}"
    try:
        with urllib.request.urlopen(url, timeout=8) as resp:
            body = json.load(resp)
        result = body.get("result")
        if not result or result.get("id") is None:
            missing.append(hid)
    except Exception:
        missing.append(hid)

checked = len(rows)
found = checked - len(missing)
print(f"qdrant-post-sync-verify: checked={checked} found={found} missing={len(missing)}")
if missing:
    print("qdrant-post-sync-verify: missing IDs:", ", ".join(missing[:5]), file=sys.stderr)
    sys.exit(1)
PY
