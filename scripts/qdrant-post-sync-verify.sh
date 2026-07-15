#!/usr/bin/env bash
# Sample-verify honeypot IDs exist in Qdrant after nightly sync.
#
# Usage: qdrant-post-sync-verify.sh [--sample N] [--gzmo-root PATH]
# Exit 0 = all samples found, 1 = missing IDs or unreachable Qdrant
set -euo pipefail

SAMPLE="${QDRANT_VERIFY_SAMPLE:-10}"
GZMO_ROOT="${GZMO_ROOT:-/opt/gzmo/survey_GZMO}"
QDRANT_URL="${QDRANT_URL:-http://127.0.0.1:6333}"
COLLECTION="${QDRANT_COLLECTION:-honeypot}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --sample) SAMPLE="$2"; shift 2 ;;
    --gzmo-root) GZMO_ROOT="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

VAULT="$GZMO_ROOT/data/vault.db"
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
