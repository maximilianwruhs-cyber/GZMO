#!/usr/bin/env bash
# Deploy GZMO daemon to LXC101 (sidecar).
# Replaces workstation daemon, enabling cloud mode and local container services.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LXC_HOST="192.168.31.202"
LXC_USER="maximilian"
SSH_KEY="${HOME}/.ssh/id_sidecar_proxmox"

ssh_lxc() {
  ssh -i "${SSH_KEY}" -o BatchMode=yes "${LXC_USER}@${LXC_HOST}" "$@"
}

rsync_lxc() {
  rsync -avz -e "ssh -i ${SSH_KEY}" "$@"
}

echo "[*] Rsyncing codebase to LXC101 (state dirs excluded — canonical under /opt/gzmo)..."
rsync -avz --delete \
  --exclude '/target/' \
  --exclude '/.git/' \
  --exclude '/.venv/' \
  --exclude '/data/' \
  --exclude '/memory/' \
  --exclude '/skills/' \
  --exclude '/wiki/' \
  --exclude '/SOUL.md' \
  --exclude '/DREAMS.md' \
  --exclude '/logs/' \
  --exclude '/.Jules/' \
  --exclude '/research/' \
  --exclude '/experiments/' \
  -e "ssh -i ${SSH_KEY}" \
  "${ROOT}/" "${LXC_USER}@${LXC_HOST}:/opt/gzmo/survey_GZMO/"

echo "[*] Rsyncing mcp-neo4j-memory-gzmo..."
rsync -avz --delete \
  --exclude '.venv/' \
  --exclude 'node_modules/' \
  -e "ssh -i ${SSH_KEY}" \
  "${ROOT}/../../mcp-neo4j-memory-gzmo/" "${LXC_USER}@${LXC_HOST}:/opt/gzmo/mcp-neo4j-memory-gzmo/"

echo "[*] Copying configuration..."
rsync_lxc "${ROOT}/gzmo.toml" "${LXC_USER}@${LXC_HOST}:/opt/gzmo/gzmo.toml"

echo "[*] Installing sidecar helper scripts..."
rsync_lxc "${SCRIPT_DIR}/link-sidecar-data.sh" "${LXC_USER}@${LXC_HOST}:/opt/gzmo/link-sidecar-data.sh"
rsync_lxc "${SCRIPT_DIR}/verify-sidecar-layout.sh" "${LXC_USER}@${LXC_HOST}:/opt/gzmo/verify-sidecar-layout.sh"
ssh_lxc "chmod +x /opt/gzmo/link-sidecar-data.sh /opt/gzmo/verify-sidecar-layout.sh"

echo "[*] Linking canonical state dirs under /opt/gzmo..."
ssh_lxc "bash /opt/gzmo/link-sidecar-data.sh"

echo "[*] Building release binary on LXC101..."
ssh_lxc "cd /opt/gzmo/survey_GZMO && /home/${LXC_USER}/.cargo/bin/cargo build -p gzmo-cli --release"

echo "[*] Installing systemd unit..."
ssh_lxc "sudo tee /etc/systemd/system/gzmo-daemon.service" < "${ROOT}/scripts/systemd/gzmo-daemon-system.service" >/dev/null
ssh_lxc "sudo systemctl daemon-reload"

echo "[*] Verifying sidecar layout..."
ssh_lxc "bash /opt/gzmo/verify-sidecar-layout.sh"

echo "[OK] Deployment complete. Restart with: ssh ${LXC_USER}@${LXC_HOST} 'sudo systemctl restart gzmo-daemon'"
