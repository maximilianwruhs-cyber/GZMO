#!/usr/bin/env bash
# Host-side LAN forwards for Pi containers that only have loopback.
# Run on the workstation host (not inside the container).
#
#   ./scripts/container-lan-forward.sh start
#   ./scripts/container-lan-forward.sh stop
#   ./scripts/container-lan-forward.sh status
#
# Container overlay (gzmo.container.toml or env):
#   embeddings.url = "http://172.17.0.1:18081/v1"
#   redis.url      = "redis://172.17.0.1:16379"
#   qdrant.url     = "http://172.17.0.1:16333"
set -euo pipefail

VM200_EMBED="${VM200_EMBED:-192.168.31.110:8081}"
LXC_REDIS="${LXC_REDIS:-192.168.31.202:6379}"
LXC_QDRANT="${LXC_QDRANT:-192.168.31.202:6333}"

LOCAL_EMBED=18081
LOCAL_REDIS=16379
LOCAL_QDRANT=16333

PID_DIR="${XDG_RUNTIME_DIR:-/tmp}/gzmo-lan-forward"
mkdir -p "$PID_DIR"

start_one() {
  local name="$1" listen="$2" target="$3"
  local pidfile="$PID_DIR/${name}.pid"
  if [[ -f "$pidfile" ]] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
    echo "[OK] $name already running (PID $(cat "$pidfile"))"
    return 0
  fi
  socat "TCP-LISTEN:${listen},fork,reuseaddr" "TCP:${target}" &
  echo $! >"$pidfile"
  echo "[OK] $name localhost:${listen} -> ${target} (PID $(cat "$pidfile"))"
}

stop_one() {
  local name="$1"
  local pidfile="$PID_DIR/${name}.pid"
  if [[ -f "$pidfile" ]]; then
    kill "$(cat "$pidfile")" 2>/dev/null || true
    rm -f "$pidfile"
    echo "[OK] stopped $name"
  fi
}

status_one() {
  local name="$1" listen="$2" target="$3"
  local pidfile="$PID_DIR/${name}.pid"
  if [[ -f "$pidfile" ]] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
    echo "[UP] $name :${listen} -> ${target} (PID $(cat "$pidfile"))"
  else
    echo "[DOWN] $name :${listen} -> ${target}"
  fi
}

cmd="${1:-status}"
case "$cmd" in
  start)
    command -v socat >/dev/null || { echo "Install socat first" >&2; exit 1; }
    start_one embed "$LOCAL_EMBED" "$VM200_EMBED"
    start_one redis "$LOCAL_REDIS" "$LXC_REDIS"
    start_one qdrant "$LOCAL_QDRANT" "$LXC_QDRANT"
    echo ""
    echo "Container gateway (docker0): $(ip -4 route show default 2>/dev/null | awk '{print $3}' || echo 172.17.0.1)"
    ;;
  stop)
    stop_one embed
    stop_one redis
    stop_one qdrant
    ;;
  status)
    status_one embed "$LOCAL_EMBED" "$VM200_EMBED"
    status_one redis "$LOCAL_REDIS" "$LXC_REDIS"
    status_one qdrant "$LOCAL_QDRANT" "$LXC_QDRANT"
    ;;
  *)
    echo "Usage: $0 {start|stop|status}" >&2
    exit 1
    ;;
esac
