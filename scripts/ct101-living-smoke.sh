#!/usr/bin/env bash
# Product gate: is CT101 the living metabolism brain and healthy?
# Run from workstation: bash scripts/ct101-living-smoke.sh
# Exit 0 = pass, 1 = fail
set -euo pipefail

HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
MIN_FACTS="${CT101_MIN_VAULT_FACTS:-10000}"

ssh_ct() {
  ssh -o ConnectTimeout=10 -o BatchMode=yes "$HOST" "$@"
}

fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { echo "OK: $*"; }

echo "=== CT101 living smoke ($HOST) ==="

daemon="$(ssh_ct 'systemctl is-active gzmo-daemon' || true)"
[[ "$daemon" == "active" ]] || fail "gzmo-daemon is '$daemon' (want active)"
ok "gzmo-daemon active"

sidecars="$(ssh_ct 'docker ps --format "{{.Names}}:{{.Status}}"')"
echo "$sidecars" | grep -q 'sidecar-redis:Up' || fail "sidecar-redis not Up"
echo "$sidecars" | grep -q 'sidecar-qdrant:Up' || fail "sidecar-qdrant not Up"
echo "$sidecars" | grep -q 'sidecar-neo4j:Up' || fail "sidecar-neo4j not Up"
ok "sidecars Up (redis/qdrant/neo4j)"

facts="$(ssh_ct 'sqlite3 /opt/gzmo/data/vault.db "SELECT COUNT(*) FROM semantic_vault;"')"
[[ "$facts" =~ ^[0-9]+$ ]] || fail "vault fact count unreadable: $facts"
(( facts >= MIN_FACTS )) || fail "vault facts=$facts < min $MIN_FACTS"
ok "vault facts=$facts"

# Symlink hygiene
current="$(ssh_ct 'readlink -f /opt/gzmo/current')"
[[ -n "$current" ]] || fail "/opt/gzmo/current missing"
ok "current → $current"

health_out="$(ssh_ct "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN health'" 2>&1)" || {
  echo "$health_out" >&2
  fail "gzmo health exited non-zero"
}
echo "$health_out" | sed 's/^/  /'
ok "gzmo health"

mentor="$(ssh_ct "bash -lc 'test -S /opt/gzmo/data/gzmo_mentor.sock && cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN mentor ping'" 2>&1)" || {
  echo "$mentor" >&2
  fail "living mentor ping failed (want socket + pong)"
}
echo "$mentor" | grep -qi 'pong' || fail "mentor ping response missing pong: $mentor"
ok "mentor ping → pong"

journal="$(ssh_ct 'journalctl -u gzmo-daemon --since "2 hours ago" --no-pager' | tail -5 || true)"
if echo "$journal" | grep -qiE 'Heartbeat|job completed|Orchestrator|Mentor API'; then
  ok "recent daemon activity in journal"
else
  echo "WARN: no Heartbeat/job/Mentor lines in last 2h (daemon may be idle)" >&2
fi

echo "=== PASS: CT101 living ==="
exit 0
