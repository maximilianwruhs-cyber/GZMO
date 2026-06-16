#!/usr/bin/env bash
# Restore production stack after emergency-quiesce.sh (Prime + daemon + discovery + HSP).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GZMO_SKILLS="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"

wait_url() {
  local url=$1 label=$2 max=${3:-180}
  for ((i = 1; i <= max; i++)); do
    if curl -sf "${url}" >/dev/null 2>&1; then
      echo "[OK] ${label} ready (${i}s)"
      return 0
    fi
    sleep 2
  done
  echo "[FAIL] ${label} not ready: ${url}" >&2
  return 1
}

echo "== restore production stack =="

echo "[*] Unmask + start Prime (gzmo-prime.service)…"
systemctl --user unmask gzmo-prime.service 2>/dev/null || true
systemctl --user start gzmo-prime.service 2>/dev/null || {
  echo "[*] systemd prime failed — falling back to start-production.sh"
  "${ROOT}/scripts/start-production.sh"
}
wait_url "http://127.0.0.1:8000/v1/models" "Prime" 120 || true

echo "[*] Start GZMO daemon…"
systemctl --user start gzmo-daemon.service 2>/dev/null || {
  "${ROOT}/scripts/start-production.sh" --daemon
}

echo "[*] Start HSP audio pipeline…"
systemctl --user start hsp-synth.service 2>/dev/null || true
systemctl --user start hsp-pipeline.service 2>/dev/null || true

echo "[*] Enable discovery timer…"
systemctl --user unmask pi-mentor-discovery.timer 2>/dev/null || true
systemctl --user enable --now pi-mentor-discovery.timer 2>/dev/null || true

echo ""
echo "--- status ---"
free -h | head -2
nvidia-smi --query-gpu=memory.used,utilization.gpu --format=csv,noheader 2>/dev/null || true
ss -ltnp 2>/dev/null | grep ':8000' || echo "port 8000: not listening"
systemctl --user is-active gzmo-prime gzmo-daemon pi-mentor-discovery.timer hsp-synth hsp-pipeline 2>/dev/null || true

if [[ -x "$BIN" ]]; then
  echo ""
  echo "--- Obolus (ARCH-DIR directive) ---"
  (cd "$ROOT" && "$BIN" obolus balance) || true
  echo ""
  (cd "$ROOT" && ./scripts/sovereignty-verify.sh) || true
fi

echo ""
echo "Done. Discovery: systemctl --user status pi-mentor-discovery.timer"
echo "Quiesce: ${GZMO_SKILLS}/scripts/emergency-quiesce.sh"
