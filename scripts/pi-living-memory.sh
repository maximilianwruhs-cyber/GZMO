#!/usr/bin/env bash
# Living memory bridge for Pi → CT101 (fresh stack).
# Prefer MCP server gzmo-living; this shell bridge matches for scripts/skills.
set -euo pipefail

HOST="${CT101_SSH_HOST:-ct101}"
BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
SESSION_FILE="${GZMO_SESSION_FILE:-$HOME/.pi/agent/gzmo-living-session.id}"

read_session() {
  if [[ -f "$SESSION_FILE" ]]; then
    tr -d '[:space:]' < "$SESSION_FILE"
    return 0
  fi
  mkdir -p "$(dirname "$SESSION_FILE")"
  if command -v uuidgen >/dev/null 2>&1; then
    uuidgen | tr '[:upper:]' '[:lower:]' | tee "$SESSION_FILE" >/dev/null
  else
    echo "pi-$(date +%s)-$$" | tee "$SESSION_FILE" >/dev/null
  fi
  tr -d '[:space:]' < "$SESSION_FILE"
}

remote() {
  local sid qargs=() a
  sid="$(read_session)"
  for a in "$@"; do
    qargs+=("$(printf '%q' "$a")")
  done
  ssh -o ConnectTimeout=15 -o BatchMode=yes "$HOST" \
    "bash -lc $(printf '%q' "cd /opt/gzmo && export GZMO_CONFIG=/opt/gzmo/gzmo.toml GZMO_ALLOW_LAB_VAULT=1 GZMO_PRODUCT=1 GZMO_SESSION_ID=$sid && exec $BIN ${qargs[*]}")"
}

cmd="${1:-}"
shift || true
case "$cmd" in
  ""|-h|--help|help)
    echo "Usage: $0 session|session-new|turn-start|search <q>|recall|status|prep <q>"
    ;;
  session) read_session ;;
  session-new) rm -f "$SESSION_FILE"; read_session ;;
  turn-start) remote memory turn-start ;;
  search)
    [[ $# -ge 1 ]] || { echo "missing query" >&2; exit 1; }
    remote memory search "$@"
    ;;
  recall) remote memory recall ;;
  status) remote memory status "$@" ;;
  prep)
    [[ $# -ge 1 ]] || { echo "missing query" >&2; exit 1; }
    remote memory turn-start
    remote memory search "$@"
    ;;
  *)
    echo "unknown: $cmd" >&2
    exit 1
    ;;
esac
