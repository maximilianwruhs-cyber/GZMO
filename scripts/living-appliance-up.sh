#!/usr/bin/env bash
# Goal C: bring up living sidecars from the in-repo compose pin.
# Does not start gzmo-daemon (that stays on CT101 /opt/gzmo or operator systemd).
#
#   bash scripts/living-appliance-up.sh
#   LIVING_APPLIANCE_DIR=… bash scripts/living-appliance-up.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="${LIVING_APPLIANCE_DIR:-$ROOT/deploy/living-appliance}"
COMPOSE="$DIR/docker-compose.yml"

[[ -f "$COMPOSE" ]] || { echo "[!] missing $COMPOSE" >&2; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "[!] docker required" >&2; exit 1; }

if [[ ! -f "$DIR/.env" ]]; then
  if [[ -f "$DIR/.env.example" ]]; then
    cp "$DIR/.env.example" "$DIR/.env"
    echo "[*] Created $DIR/.env from .env.example — set NEO4J_AUTH for this pin only"
    echo "[*] Workstation ~/database-cluster Neo4j is throwaway; do not copy its password into the pin"
  else
    echo "[!] missing $DIR/.env and .env.example" >&2
    exit 1
  fi
fi

echo "[*] docker compose up -d ($DIR)"
(cd "$DIR" && docker compose up -d)

echo "[*] waiting for ports…"
ready=0
for _ in $(seq 1 30); do
  ok=0
  for port in 6379 6333 7687; do
    if timeout 1 bash -c "echo >/dev/tcp/127.0.0.1/${port}" 2>/dev/null; then
      ok=$((ok + 1))
    fi
  done
  if [[ "$ok" -eq 3 ]]; then
    ready=1
    break
  fi
  sleep 2
done

if [[ "$ready" -eq 1 ]]; then
  echo "[OK] redis :6379 · qdrant :6333 · neo4j :7687"
else
  echo "[!] ports not all open yet — check: docker compose -f $COMPOSE ps" >&2
fi

bash "$ROOT/scripts/living-appliance-gate.sh" || true
echo "[*] protocol smoke (Redis PING / Qdrant ready / Neo4j auth)…"
# Soft by default: HOLD if not fully live; FAIL only on protocol mismatch.
LIVING_APPLIANCE_REQUIRE_SMOKE="${LIVING_APPLIANCE_REQUIRE_SMOKE:-0}" \
  bash "$ROOT/scripts/living-appliance-smoke.sh" || true
echo "[*] daemon health smoke (lab GZMO_CONFIG → redis/qdrant/neo4j)…"
LIVING_APPLIANCE_REQUIRE_HEALTH="${LIVING_APPLIANCE_REQUIRE_HEALTH:-0}" \
  bash "$ROOT/scripts/living-appliance-health-smoke.sh" || true

echo ""
echo "Next (daemon is separate):"
echo "  # CT101 living brain"
echo "  ssh ct101 'systemctl status gzmo-daemon'"
echo "  # Labeled living MCP attach (does not overwrite product gzmo-memory)"
echo "  bash scripts/install-shared-mcp.sh"
echo "  # Docs: docs/LIVING_APPLIANCE.md"
