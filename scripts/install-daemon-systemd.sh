#!/usr/bin/env bash
# Install user systemd unit for GZMO daemon.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ ! -x "${ROOT}/target/release/gzmo" ]]; then
  echo "[*] Building gzmo release binary…" >&2
  (cd "${ROOT}" && cargo build -p gzmo-cli --release)
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UNIT_SRC="${SCRIPT_DIR}/systemd/gzmo-daemon.service"
UNIT_DST="${HOME}/.config/systemd/user/gzmo-daemon.service"

mkdir -p "${HOME}/.config/systemd/user"
sed "s/%h/${HOME//\//\\/}/g; s/%i/${USER}/g" "${UNIT_SRC}" >"${UNIT_DST}"

systemctl --user daemon-reload
echo "[OK] Installed ${UNIT_DST}"
echo ""
echo "  systemctl --user enable --now gzmo-daemon.service"
echo "  journalctl --user -u gzmo-daemon -f"
