#!/usr/bin/env bash
# Install user systemd unit for Pi KB embed (:8002).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UNIT_SRC="${SCRIPT_DIR}/systemd/gzmo-embed.service"
UNIT_DST="${HOME}/.config/systemd/user/gzmo-embed.service"

mkdir -p "${HOME}/.config/systemd/user"
sed "s/%h/${HOME//\//\\/}/g; s/%i/${USER}/g" "${UNIT_SRC}" >"${UNIT_DST}"

systemctl --user daemon-reload
echo "[OK] Installed ${UNIT_DST}"
echo ""
echo "  systemctl --user enable --now gzmo-embed.service"
echo "  systemctl --user status gzmo-embed.service"
