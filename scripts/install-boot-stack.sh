#!/usr/bin/env bash
# Install and enable workstation user services for clean reboot startup.
# VM200 retrieval (embed :8081, rerank, librarian) is separate: scripts/vm200/deploy-retrieval-layer.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "[*] Installing systemd units…"
"${SCRIPT_DIR}/install-prime-systemd.sh"
"${SCRIPT_DIR}/install-embed-systemd.sh"
"${SCRIPT_DIR}/install-daemon-systemd.sh"

if command -v loginctl >/dev/null 2>&1; then
  if loginctl show-user "$(id -un)" -p Linger 2>/dev/null | grep -q 'Linger=no'; then
    echo "[*] Enabling linger (user systemd at boot without login)…"
    loginctl enable-linger "$(id -un)" || echo "[!] Could not enable linger (needs root?)" >&2
  else
    echo "[OK] Linger already enabled"
  fi
fi

echo "[*] Enabling services (start on boot)…"
systemctl --user enable gzmo-embed.service
systemctl --user enable gzmo-prime.service
systemctl --user enable gzmo-daemon.service

# HSP units if present
for u in hsp-synth.service hsp-pipeline.service; do
  if systemctl --user list-unit-files "${u}" &>/dev/null; then
    systemctl --user enable "${u}" 2>/dev/null && echo "[OK] enabled ${u}" || true
  fi
done

echo ""
echo "[*] Starting stack now…"
systemctl --user start gzmo-embed.service || true
systemctl --user start gzmo-prime.service || true
# Daemon after Prime begins loading (does not block on Prime ready)
sleep 2
systemctl --user start gzmo-daemon.service || true

echo ""
echo "Status:"
systemctl --user is-active gzmo-embed.service gzmo-prime.service gzmo-daemon.service 2>/dev/null || true
echo ""
echo "After reboot:"
echo "  1. VM200: ssh maximilian@192.168.31.110 — systemctl status llama-embed llama-rerank llama-librarian"
echo "  2. Workstation: ${ROOT}/scripts/after-boot-verify.sh"
echo "  3. Pi: new session → knowledge_reindex force=false (or ${ROOT}/scripts/pi-kb-reindex.sh)"
echo ""
echo "Docs: ${ROOT}/docs/REBOOT_STARTUP.md"
