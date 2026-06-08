#!/usr/bin/env bash
# Deploy VM200 (:110) as GZMO retrieval GPU: embed :8081, retire legacy 7B :8080.
set -euo pipefail

VM_HOST="${GZMO_VM200_HOST:-192.168.31.110}"
VM_USER="${GZMO_VM200_USER:-maximilian}"
SSH_KEY="${GZMO_VM200_SSH_KEY:-${HOME}/.ssh/id_sidecar_proxmox}"
EMBED_SRC="${GZMO_EMBED_MODEL:-${HOME}/.cache/huggingface/llamacpp-qwen36-mtp/Qwen3-Embedding-0.6B-Q8_0.gguf}"
REMOTE_MODEL_DIR="/opt/models"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ssh_vm() {
  ssh -i "${SSH_KEY}" -o BatchMode=yes "${VM_USER}@${VM_HOST}" "$@"
}

if [[ ! -f "${EMBED_SRC}" ]]; then
  echo "[!] Missing embed GGUF: ${EMBED_SRC}" >&2
  exit 1
fi

echo "[*] VM200 retrieval deploy → ${VM_HOST}"
echo "[*] Copy embedding GGUF (~610 MB)..."
ssh_vm "sudo mkdir -p ${REMOTE_MODEL_DIR} && sudo chown ${VM_USER}:${VM_USER} ${REMOTE_MODEL_DIR}"
rsync -av --progress -e "ssh -i ${SSH_KEY}" "${EMBED_SRC}" "${VM_USER}@${VM_HOST}:${REMOTE_MODEL_DIR}/"

echo "[*] Install systemd unit (llama-embed)..."
ssh_vm "sudo tee /etc/systemd/system/llama-embed.service" < "${SCRIPT_DIR}/llama-embed.service" >/dev/null
ssh_vm "sudo systemctl daemon-reload"

echo "[*] Stop legacy 7B speculative server (:8080) — frees 1070 VRAM..."
ssh_vm "sudo systemctl disable --now llama-speculative.service 2>/dev/null || true"

echo "[*] Start embedding server (:8081)..."
ssh_vm "sudo systemctl enable --now llama-embed.service"

echo "[*] Waiting for :8081..."
for i in $(seq 1 30); do
  if curl -sf "http://${VM_HOST}:8081/v1/models" >/dev/null 2>&1; then
    echo "[OK] Embed server up (${i}s)"
    curl -sf "http://${VM_HOST}:8081/v1/models" | python3 -c "import sys,json; print('  model:', json.load(sys.stdin)['data'][0]['id'])" 2>/dev/null || true
    break
  fi
  sleep 2
done

curl -sf "http://${VM_HOST}:8081/v1/models" >/dev/null || {
  echo "[!] Embed server not reachable — check: ssh ... journalctl -u llama-embed -f"
  exit 1
}

echo "[*] Smoke embed..."
curl -sf "http://${VM_HOST}:8081/v1/embeddings" \
  -H 'Content-Type: application/json' \
  -d '{"model":"Qwen3-Embedding-0.6B-Q8_0.gguf","input":"vm200 retrieval probe"}' \
  | python3 -c "import sys,json; d=json.load(sys.stdin)['data'][0]['embedding']; print(f'  dims={len(d)}')"

ssh_vm "nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader"

echo ""
echo "[OK] VM200 retrieval layer deployed."
echo "Next on workstation:"
echo "  1. pkill -f 'llama-server.*--port 8002'   # stop local embed"
echo "  2. Set gzmo.toml [embeddings] url = http://${VM_HOST}:8081/v1"
echo "  3. ./scripts/verify-production.sh"
