#!/usr/bin/env bash
# Product gate for CT101 — run ON the living host (no SSH).
# Exit 0 = pass, 1 = fail
set -euo pipefail

GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
MIN_FACTS="${CT101_MIN_VAULT_FACTS:-100}"
GZMO_CONFIG="${GZMO_CONFIG:-/opt/gzmo/gzmo.toml}"

fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { echo "OK: $*"; }

echo "=== CT101 living smoke (local) ==="

daemon="$(systemctl is-active gzmo-daemon || true)"
[[ "$daemon" == "active" ]] || fail "gzmo-daemon is '$daemon' (want active)"
ok "gzmo-daemon active"

sidecars="$(docker ps --format '{{.Names}}:{{.Status}}')"
echo "$sidecars" | grep -q 'sidecar-redis:Up' || fail "sidecar-redis not Up"
echo "$sidecars" | grep -q 'sidecar-qdrant:Up' || fail "sidecar-qdrant not Up"
echo "$sidecars" | grep -q 'sidecar-neo4j:Up' || fail "sidecar-neo4j not Up"
ok "sidecars Up (redis/qdrant/neo4j)"

facts="$(sqlite3 /opt/gzmo/data/vault.db 'SELECT COUNT(*) FROM semantic_vault;')"
[[ "$facts" =~ ^[0-9]+$ ]] || fail "vault fact count unreadable: $facts"
(( facts >= MIN_FACTS )) || fail "vault facts=$facts < min $MIN_FACTS"
ok "vault facts=$facts"

current="$(readlink -f /opt/gzmo/current)"
[[ -n "$current" ]] || fail "/opt/gzmo/current missing"
ok "current → $current"

health_out="$(cd /opt/gzmo && GZMO_CONFIG="$GZMO_CONFIG" "$GZMO_BIN" health 2>&1)" || {
  echo "$health_out" >&2
  fail "gzmo health exited non-zero"
}
echo "$health_out" | sed 's/^/  /'
ok "gzmo health"

[[ -S /opt/gzmo/data/gzmo_mentor.sock ]] || fail "mentor socket missing"
mentor="$(cd /opt/gzmo && GZMO_CONFIG="$GZMO_CONFIG" "$GZMO_BIN" mentor ping 2>&1)" || {
  echo "$mentor" >&2
  fail "living mentor ping failed"
}
echo "$mentor" | grep -qi 'pong' || fail "mentor ping response missing pong: $mentor"
ok "mentor ping → pong"

journal="$(journalctl -u gzmo-daemon --since '2 hours ago' --no-pager 2>/dev/null | tail -5 || true)"
if echo "$journal" | grep -qiE 'Heartbeat|job completed|Orchestrator|Mentor API'; then
  ok "recent daemon activity in journal"
else
  echo "WARN: no Heartbeat/job/Mentor lines in last 2h (daemon may be idle)" >&2
fi

echo "=== PASS: CT101 living ==="
exit 0
