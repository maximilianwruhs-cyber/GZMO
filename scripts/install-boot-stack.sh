#!/usr/bin/env bash
# Install and enable workstation user services for clean reboot startup.
# VM200 retrieval router is separate: scripts/vm200/deploy-retrieval-router.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "[*] Installing systemd units…"
"${SCRIPT_DIR}/install-prime-systemd.sh"
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
systemctl --user enable gzmo-prime.service
systemctl --user enable gzmo-daemon.service

echo ""
echo "[*] Starting stack now…"
systemctl --user start gzmo-prime.service || true
sleep 2
systemctl --user start gzmo-daemon.service || true

echo ""
echo "Status:"
systemctl --user is-active gzmo-prime.service gzmo-daemon.service 2>/dev/null || true
echo ""
echo "After reboot:"
echo "  1. VM200: ssh maximilian@192.168.31.110 — systemctl status llama-retrieval-router"
echo "  2. Workstation: ${ROOT}/scripts/after-boot-verify.sh"
echo "  3. Pi KB sync: ${ROOT}/scripts/pi-kb-reindex.sh (embed via VM200 :8081)"
echo ""
echo "Docs: ${ROOT}/docs/REBOOT_STARTUP.md"
