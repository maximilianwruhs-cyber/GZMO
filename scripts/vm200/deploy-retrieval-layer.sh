#!/usr/bin/env bash
# Deploy consolidated embed+rerank on VM200 :8081 (single llama-server, both models).
# The server loads Qwen3-Reranker-0.6B.F16.gguf as primary model and also
# registers gzmo-embed as an alias so both /v1/embeddings and /v1/rerank work.
# Rerank on :8082 is OBSOLETE — consolidated to :8081.
set -euo pipefail

VM_HOST="${GZMO_VM200_HOST:-192.168.31.110}"
VM_USER="${GZMO_VM200_USER:-maximilian}"
SSH_KEY="${GZMO_VM200_SSH_KEY:-${HOME}/.ssh/id_sidecar_proxmox}"
EMBED_SRC="${GZMO_EMBED_MODEL:-${HOME}/.cache/huggingface/llamacpp-qwen36-mtp/Qwen3-Embedding-0.6B-Q8_0.gguf}"
RERANK_SRC="${GZMO_RERANK_MODEL:-${HOME}/.cache/huggingface/gzmo-rerank/Qwen3-Reranker-0.6B.F16.gguf}"
REMOTE_MODEL_DIR="/opt/models"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ssh_vm() {
  ssh -i "${SSH_KEY}" -o BatchMode=yes "${VM_USER}@${VM_HOST}" "$@"
}

echo "[*] VM200 retrieval deploy → ${VM_HOST}"

# --- Copy models ---
ssh_vm "sudo mkdir -p ${REMOTE_MODEL_DIR} && sudo chown ${VM_USER}:${VM_USER} ${REMOTE_MODEL_DIR}"

if [[ -f "${RERANK_SRC}" ]]; then
  echo "[*] Copy rerank GGUF (primary model)..."
  rsync -av --progress -e "ssh -i ${SSH_KEY}" "${RERANK_SRC}" "${VM_USER}@${VM_HOST}:${REMOTE_MODEL_DIR}/"
fi

if [[ -f "${EMBED_SRC}" ]]; then
  echo "[*] Copy embedding GGUF (secondary)..."
  rsync -av --progress -e "ssh -i ${SSH_KEY}" "${EMBED_SRC}" "${VM_USER}@${VM_HOST}:${REMOTE_MODEL_DIR}/"
fi

# --- Install systemd unit ---
echo "[*] Install consolidated systemd unit (llama-embed)..."
ssh_vm "sudo tee /etc/systemd/system/llama-embed.service" < "${SCRIPT_DIR}/llama-embed.service" >/dev/null

# --- Stop old separate services ---
ssh_vm "sudo systemctl disable --now llama-rerank.service 2>/dev/null || true"
ssh_vm "sudo systemctl disable --now llama-speculative.service 2>/dev/null || true"
ssh_vm "sudo systemctl daemon-reload"

# --- Start consolidated server ---
echo "[*] Start consolidated embed+rerank server (:8081)..."
ssh_vm "sudo systemctl enable --now llama-embed.service"

# --- Wait for :8081 ---
echo "[*] Waiting for :8081..."
for i in $(seq 1 30); do
  if curl -sf "http://${VM_HOST}:8081/v1/models" >/dev/null 2>&1; then
    echo "[OK] Server up (${i}s)"
    curl -sf "http://${VM_HOST}:8081/v1/models" | python3 -c "
import sys, json
for m in json.load(sys.stdin)['data']:
    print(f'  {m[\"id\"]} — status: {m[\"status\"][\"value\"]}')"
    break
  fi
  sleep 2
done

curl -sf "http://${VM_HOST}:8081/v1/models" >/dev/null || {
  echo "[!] Server not reachable — check: ssh ... journalctl -u llama-embed -f"
  exit 1
}

# --- Smoke tests ---
echo "[*] Smoke embed..."
curl -sf "http://${VM_HOST}:8081/v1/embeddings" \
  -H 'Content-Type: application/json' \
  -d '{"model":"gzmo-embed","input":"vm200 retrieval probe"}' \
  | python3 -c "import sys,json; d=json.load(sys.stdin)['data'][0]['embedding']; print(f'  dims={len(d)}')"

echo "[*] Smoke rerank..."
curl -sf "http://${VM_HOST}:8081/v1/rerank" \
  -H 'Content-Type: application/json' \
  -d '{"model":"gzmo-rerank","query":"test","documents":["doc a","doc b"]}' \
  | python3 -c "import sys,json; r=json.load(sys.stdin)['results']; print(f'  {len(r)} results')"

ssh_vm "nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader"

echo ""
echo "[OK] VM200 consolidated embed+rerank deployed."
echo "Config in gzmo.toml:"
echo "  [embeddings] url = http://${VM_HOST}:8081/v1  model = gzmo-embed"
echo "  [rerank]     url = http://${VM_HOST}:8081/v1  model = gzmo-rerank"
