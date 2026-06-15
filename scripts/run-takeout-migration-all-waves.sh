#!/usr/bin/env bash
# Cloud-only takeout ingest: waves 01–04, full speed, Qdrant sync after each wave.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
set -a && source .env && set +a
export CLOUD_INGEST=1
export QDRANT_SYNC=1
export SKIP_BUILD=1
MASTER_LOG="$ROOT/logs/takeout-migration-all-$(date +%Y%m%d-%H%M%S).log"
exec > >(tee -a "$MASTER_LOG") 2>&1
echo "=== takeout migration all waves $(date -Is) ==="
echo "master_log=$MASTER_LOG"
START_WAVE="${START_WAVE:-01}"
for wave in 01 02 03 04; do
  [[ "$wave" < "$START_WAVE" ]] && continue
  echo ""
  echo "========== WAVE $wave $(date -Is) =========="
  ./scripts/slow-reingest-migration.sh \
    --manifest "scripts/ingest-quality/wave${wave}-takeout-curated.manifest" \
    --interval 0 || echo "[!] wave $wave had failures (continuing)"
  if [[ -x ./scripts/sync-vault-to-qdrant.sh ]]; then
    echo "[*] post-wave Qdrant sync"
    ./scripts/sync-vault-to-qdrant.sh
  fi
done
echo ""
echo "=== ALL WAVES COMPLETE $(date -Is) ==="
./scripts/memory-status.sh || true
