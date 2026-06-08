#!/usr/bin/env bash
# Start Qwen2.5-1.5B on VM200 :8083 (fast summaries / future session-distill offload).
set -euo pipefail

VM_HOST="${GZMO_VM200_HOST:-192.168.31.110}"
VM_USER="${GZMO_VM200_USER:-maximilian}"
SSH_KEY="${GZMO_VM200_SSH_KEY:-${HOME}/.ssh/id_sidecar_proxmox}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ssh -i "${SSH_KEY}" -o BatchMode=yes "${VM_USER}@${VM_HOST}" \
  "sudo tee /etc/systemd/system/llama-librarian.service" < "${SCRIPT_DIR}/llama-librarian.service" >/dev/null

ssh -i "${SSH_KEY}" -o BatchMode=yes "${VM_USER}@${VM_HOST}" \
  "sudo systemctl daemon-reload && sudo systemctl enable --now llama-librarian.service"

for i in $(seq 1 20); do
  if curl -sf "http://${VM_HOST}:8083/v1/models" >/dev/null; then
    echo "[OK] Librarian :8083 up (${i}s)"
    curl -sf "http://${VM_HOST}:8083/v1/models" | python3 -c "import sys,json; print('  model:', json.load(sys.stdin)['data'][0]['id'])" 2>/dev/null || true
    exit 0
  fi
  sleep 2
done
echo "[!] Librarian not reachable on :8083"
exit 1
