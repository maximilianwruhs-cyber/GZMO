#!/usr/bin/env bash
# Product gate: is the LIVING HOST the metabolism brain and healthy?
# CUTOVER A (2026-09-30): local-first — workstation daemon/vault when present,
# SSH fallback to CT101 (HERDR_LIVE_MODE=remote or LIVING_HOST=<host>).
#
#   bash scripts/living-smoke.sh                     # auto-detect (local preferred)
#   HERDR_LIVE_MODE=remote bash scripts/living-smoke.sh
#   LIVING_HOST=ct101 bash scripts/living-smoke.sh   # explicit host
# Exit 0 = pass, 1 = fail
set -euo pipefail

HOST="${CT101_SSH_HOST:-ct101}"
LIVE_ROOT="${GZMO_LIVING_ROOT:-$HOME/.gzmo}"
LOCAL_BIN="${GZMO_LIVING_BIN:-$HOME/github-clone/GZMO/target/release/gzmo}"
[[ -x "$LOCAL_BIN" ]] || LOCAL_BIN="$HOME/.local/bin/gzmo"

# Mode resolution: LIVING_HOST (explicit) > HERDR_LIVE_MODE > auto-detect.
LIVE_MODE="${HERDR_LIVE_MODE:-auto}"
if [[ -n "${LIVING_HOST:-}" && "${LIVING_HOST}" != "auto" ]]; then LIVE_MODE="$LIVING_HOST"; fi
if [[ "$LIVE_MODE" == "auto" ]]; then
  if [[ -f "$LIVE_ROOT/data/vault.db" ]] && systemctl --user is-active gzmo-daemon.service >/dev/null 2>&1; then
    LIVE_MODE="local"
  else
    LIVE_MODE="$HOST"
  fi
fi

MIN_FACTS="${CT101_MIN_VAULT_FACTS:-100}"

if [[ "$LIVE_MODE" == "local" ]]; then
  r() { bash -lc "$@"; }
else
  r() { ssh -o ConnectTimeout=10 -o BatchMode=yes "$HOST" "$@"; }
fi

fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { echo "OK: $*"; }

echo "=== Living smoke (mode=$LIVE_MODE) ==="

# 1) daemon
if [[ "$LIVE_MODE" == "local" ]]; then
  daemon="$(systemctl --user is-active gzmo-daemon.service 2>/dev/null || true)"
else
  daemon="$(r 'systemctl is-active gzmo-daemon' || true)"
fi
[[ "$daemon" == "active" ]] || fail "gzmo-daemon is '$daemon' (want active, mode=$LIVE_MODE)"
ok "gzmo-daemon active ($LIVE_MODE)"

# 2) sidecars (docker on the living host)
sidecars="$(r 'docker ps --format "{{.Names}}:{{.Status}}"' || true)"
echo "$sidecars" | grep -q 'sidecar-redis:Up'   || fail "sidecar-redis not Up"
echo "$sidecars" | grep -q 'sidecar-qdrant:Up'  || fail "sidecar-qdrant not Up"
echo "$sidecars" | grep -q 'sidecar-neo4j:Up'   || fail "sidecar-neo4j not Up"
ok "sidecars Up (redis/qdrant/neo4j)"

# 3) vault facts (semantic_vault table; python stdlib — sqlite3 CLI often absent locally)
if [[ "$LIVE_MODE" == "local" ]]; then
  facts="$(python3 - "$LIVE_ROOT/data/vault.db" <<'PYF' 2>/dev/null || echo 0
import sqlite3, sys
try:
    print(sqlite3.connect(sys.argv[1]).execute("SELECT COUNT(*) FROM semantic_vault").fetchone()[0])
except Exception:
    print(0)
PYF
)"
else
  facts="$(r 'sqlite3 /opt/gzmo/data/vault.db "SELECT COUNT(*) FROM semantic_vault;"' || echo 0)"
fi
facts="${facts:-0}"
[[ "$facts" =~ ^[0-9]+$ ]] || fail "vault fact count unreadable: $facts"
(( facts >= MIN_FACTS )) || fail "vault facts=$facts < min $MIN_FACTS"
ok "vault facts=$facts (min $MIN_FACTS)"

# 4) binary / current symlink hygiene
if [[ "$LIVE_MODE" == "local" ]]; then
  current="$(readlink -f "$LOCAL_BIN" 2>/dev/null || true)"
else
  current="$(r 'readlink -f /opt/gzmo/current' || true)"
fi
[[ -n "$current" ]] || fail "gzmo binary/current missing (mode=$LIVE_MODE)"
ok "binary → $current"

# 5) gzmo health
if [[ "$LIVE_MODE" == "local" ]]; then
  health_out="$(cd "$LIVE_ROOT" && GZMO_CONFIG="$LIVE_ROOT/gzmo.toml" "$LOCAL_BIN" health 2>&1)" || { echo "$health_out" >&2; fail "gzmo health exited non-zero"; }
else
  REMOTE_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
  health_out="$(r "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $REMOTE_BIN health'" 2>&1)" || { echo "$health_out" >&2; fail "gzmo health exited non-zero"; }
fi
echo "$health_out" | sed 's/^/  /'
ok "gzmo health"

# 6) mentor ping → pong (strongest single signal: daemon owns the control plane)
if [[ "$LIVE_MODE" == "local" ]]; then
  mentor="$(cd "$LIVE_ROOT" && GZMO_CONFIG="$LIVE_ROOT/gzmo.toml" "$LOCAL_BIN" mentor ping 2>&1)" || { echo "$mentor" >&2; fail "living mentor ping failed"; }
else
  REMOTE_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
  mentor="$(r "bash -lc 'test -S /opt/gzmo/data/gzmo_mentor.sock && cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $REMOTE_BIN mentor ping'" 2>&1)" || { echo "$mentor" >&2; fail "living mentor ping failed (want socket + pong)"; }
fi
echo "$mentor" | grep -qi 'pong' || fail "mentor ping response missing pong: ${mentor:0:120}"
ok "mentor ping → pong"

# 7) recent daemon activity (soft WARN, not a hard fail)
if [[ "$LIVE_MODE" == "local" ]]; then
  journal="$(journalctl --user -u gzmo-daemon.service --since '2 hours ago' --no-pager 2>/dev/null | tail -5 || true)"
else
  journal="$(r 'journalctl -u gzmo-daemon --since "2 hours ago" --no-pager' | tail -5 || true)"
fi
if echo "$journal" | grep -qiE 'Heartbeat|job completed|Orchestrator|Mentor API'; then
  ok "recent daemon activity in journal"
else
  echo "WARN: no Heartbeat/job/Mentor lines in last 2h (daemon may be idle)" >&2
fi

echo "=== PASS: living ($LIVE_MODE) ==="
exit 0
