#!/usr/bin/env bash
# Finish vault→honeypot embedding backfill on CT101, then sync Qdrant.
# Run ON CT101: bash scripts/ct101-embed-backfill-loop.sh
set -euo pipefail

GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
GZMO_CONFIG="${GZMO_CONFIG:-/opt/gzmo/gzmo.toml}"
VAULT="${LIVING_VAULT_DB:-/opt/gzmo/data/vault.db}"
BATCH="${EMBED_BATCH:-4000}"
LOG="${EMBED_LOG:-/opt/gzmo/data/embed-backfill.log}"
SYNC_SCRIPT="${SYNC_SCRIPT:-/opt/gzmo/current/scripts/sync-vault-to-qdrant.py}"

cd /opt/gzmo

missing() {
  sqlite3 "$VAULT" "SELECT COUNT(*) FROM semantic_vault WHERE embedding IS NULL OR length(embedding) < 4;"
}

honey_gap() {
  sqlite3 "$VAULT" "SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND (embedding IS NULL OR length(embedding) < 4);"
}

mirror_honeypot() {
  sqlite3 "$VAULT" "
UPDATE honeypot
SET embedding = (
  SELECT sv.embedding FROM semantic_vault sv
  WHERE sv.id = honeypot.id AND length(sv.embedding) >= 4
)
WHERE is_latest = 1
  AND (embedding IS NULL OR length(embedding) < 4)
  AND EXISTS (
    SELECT 1 FROM semantic_vault sv
    WHERE sv.id = honeypot.id AND length(sv.embedding) >= 4
  );
SELECT changes();
"
}

sync_qdrant() {
  python3 "$SYNC_SCRIPT" \
    --db "$VAULT" \
    --url http://127.0.0.1:6333 \
    --collection honeypot \
    --source honeypot
}

echo "=== embed backfill loop (batch=$BATCH) ==="
while true; do
  m="$(missing)"
  h="$(honey_gap)"
  echo "$(date -u +%H:%M:%SZ) vault_missing=$m honeypot_no_emb=$h"
  # Stop when vault gap is gone (honeypot may still have orphan ids without vault rows).
  if [[ "$m" -eq 0 ]]; then
    break
  fi
  echo "Embedding up to $BATCH…"
  GZMO_CONFIG="$GZMO_CONFIG" "$GZMO_BIN" memory embed "$BATCH" | tee -a "$LOG"
  mirrored="$(mirror_honeypot)"
  echo "Mirrored vault→honeypot rows: $mirrored"
done

echo "Final mirror + Qdrant sync…"
mirror_honeypot >/dev/null || true
sync_qdrant | tee -a "$LOG" | tail -5
GZMO_CONFIG="$GZMO_CONFIG" "$GZMO_BIN" health 2>&1 | grep honeypot_qdrant || true
echo "=== embed backfill loop done ==="
