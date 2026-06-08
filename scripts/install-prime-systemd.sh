#!/usr/bin/env bash
# Install user systemd unit for Prime (:8000). Does not enable by default.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UNIT_SRC="${SCRIPT_DIR}/systemd/gzmo-prime.service"
UNIT_DST="${HOME}/.config/systemd/user/gzmo-prime.service"

mkdir -p "${HOME}/.config/systemd/user"
sed "s/%h/${HOME//\//\\/}/g; s/%i/${USER}/g" "${UNIT_SRC}" >"${UNIT_DST}"

systemctl --user daemon-reload
echo "[OK] Installed ${UNIT_DST}"
echo ""
echo "Enable and start (optional — start-production.sh also works without systemd):"
echo "  systemctl --user enable --now gzmo-prime.service"
echo "  systemctl --user status gzmo-prime.service"
echo "  journalctl --user -u gzmo-prime -f"
