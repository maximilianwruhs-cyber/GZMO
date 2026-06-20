#!/usr/bin/env bash
# Rebuild (optional) and restart GZMO daemon after Rust changes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

FORCE=0
if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
  shift
fi

GUARD="$ROOT/data/cycle-guard.json"
if [[ -f "$GUARD" && "$FORCE" -ne 1 ]]; then
  kind="$(jq -r '.kind // "unknown"' "$GUARD" 2>/dev/null || echo unknown)"
  echo "[!] Critical cycle in progress ($kind) — waiting up to 120s (use --force to override)"
  for _ in $(seq 1 120); do
    [[ -f "$GUARD" ]] || break
    sleep 1
  done
  if [[ -f "$GUARD" ]]; then
    echo "[!] Cycle still active — aborting restart (pass --force to override)" >&2
    exit 1
  fi
fi

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
