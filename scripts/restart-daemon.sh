#!/usr/bin/env bash
# Rebuild (optional) and restart GZMO daemon after Rust changes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ "${1:-}" == "--build" ]]; then
  "$ROOT/scripts/build-gzmo.sh"
  shift
fi

stop_pid() {
  local pid="$1"
  if kill -0 "$pid" 2>/dev/null; then
    echo "[*] Stopping daemon PID $pid"
    kill "$pid" 2>/dev/null || true
    for _ in $(seq 1 20); do
      kill -0 "$pid" 2>/dev/null || return 0
      sleep 0.25
    done
    kill -9 "$pid" 2>/dev/null || true
  fi
}

# Canonical: /tmp/gzmo_daemon.pid (legacy /tmp/gzmo_rust.pid still checked)
for f in /tmp/gzmo_daemon.pid /tmp/gzmo_rust.pid; do
  if [[ -f "$f" ]]; then
    stop_pid "$(cat "$f")"
    rm -f "$f"
  fi
done

if pgrep -f '/target/(release|debug)/gzmo daemon' >/dev/null 2>&1; then
  echo "[*] Stopping stray gzmo daemon processes"
  pkill -f '/target/(release|debug)/gzmo daemon' 2>/dev/null || true
  sleep 1
fi

rm -f "$ROOT/data/gzmo_mentor.sock"

exec "$ROOT/scripts/start-production.sh" --daemon
