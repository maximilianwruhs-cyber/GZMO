#!/usr/bin/env bash
# Sync in-repo living appliance pin onto CT101 (staging under /opt/gzmo/current).
# Does not touch /opt/database-cluster or any remote .env — operator promotes when ready.
#
#   bash scripts/ct101-sync-living-appliance.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
SRC="$ROOT/deploy/living-appliance/"
DEST="${CT101_LIVING_APPLIANCE_DEST:-/opt/gzmo/current/deploy/living-appliance/}"

[[ -f "$SRC/docker-compose.yml" ]] || { echo "[!] missing $SRC/docker-compose.yml" >&2; exit 1; }

echo "[*] rsync $SRC → ${HOST}:${DEST}"
ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "mkdir -p $(printf '%q' "$DEST")"
rsync -az --delete \
  --exclude '.env' \
  --exclude '*.local' \
  "$SRC" "${HOST}:${DEST}"

echo "[OK] pin synced (remote .env excluded)"
echo "[*] pin-vs-live shape check (no .env reads)…"
bash "$ROOT/scripts/ct101-living-appliance-pin-check.sh" || true
echo ""
echo "On CT101 (when you choose to activate this pin):"
echo "  ssh $HOST"
echo "  cd $DEST"
echo "  # secrets: set NEO4J_AUTH in pin .env from /opt/gzmo/.env — do not copy workstation throwaway Neo4j"
echo "  docker compose config && docker compose up -d"
echo "  # daemon still uses /opt/gzmo/gzmo.toml — see config/living-appliance.gzmo.toml.example"
echo ""
echo "Verify from workstation:"
echo "  bash scripts/ct101-living-appliance-pin-check.sh"
echo "  bash scripts/living-appliance-gate.sh"
echo "  bash scripts/ct101-living-smoke.sh"
