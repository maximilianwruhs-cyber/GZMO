#!/usr/bin/env bash
# DEPRECATED — use deploy-retrieval-router.sh (unified :8081 router).
# Legacy: standalone bge rerank :8082.
set -euo pipefail

VM_HOST="${GZMO_VM200_HOST:-192.168.31.110}"
VM_USER="${GZMO_VM200_USER:-maximilian}"
SSH_KEY="${GZMO_VM200_SSH_KEY:-${HOME}/.ssh/id_sidecar_proxmox}"
RERANK_SRC="${GZMO_RERANK_MODEL:-${HOME}/.cache/huggingface/bge-reranker-v2-m3-Felladrin/bge-reranker-v2-m3-q8_0.gguf}"
REMOTE_MODEL_DIR="/opt/models"
REMOTE_NAME="bge-reranker-v2-m3-q8_0.gguf"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Felladrin conversion loads on llama.cpp b9378+; lj027 Q8_0 file fails tensor bounds.
HF_REPO="Felladrin/gguf-Q8_0-bge-reranker-v2-m3"
HF_FILE="bge-reranker-v2-m3-q8_0.gguf"

ssh_vm() {
  ssh -i "${SSH_KEY}" -o BatchMode=yes "${VM_USER}@${VM_HOST}" "$@"
}

if [[ ! -f "${RERANK_SRC}" ]]; then
  echo "[*] Downloading ${HF_REPO} / ${HF_FILE} (~636 MB)..."
  mkdir -p "$(dirname "${RERANK_SRC}")"
  if command -v huggingface-cli >/dev/null 2>&1; then
    huggingface-cli download "${HF_REPO}" "${HF_FILE}" --local-dir "$(dirname "${RERANK_SRC}")"
    if [[ -f "$(dirname "${RERANK_SRC}")/${HF_FILE}" && ! -f "${RERANK_SRC}" ]]; then
      mv "$(dirname "${RERANK_SRC}")/${HF_FILE}" "${RERANK_SRC}"
    fi
  else
    python3 - <<'PY'
import os, urllib.request
repo = "lj027/bge-reranker-v2-m3-Q8_0-GGUF"
fname = "bge-reranker-v2-m3-q8_0.gguf"
dest = os.path.expanduser("~/.cache/huggingface/bge-reranker-v2-m3-Felladrin/bge-reranker-v2-m3-q8_0.gguf")
os.makedirs(os.path.dirname(dest), exist_ok=True)
url = f"https://huggingface.co/{repo}/resolve/main/{fname}"
print("GET", url)
urllib.request.urlretrieve(url, dest)
print("saved", dest, os.path.getsize(dest))
PY
    RERANK_SRC="${HOME}/.cache/huggingface/bge-reranker-v2-m3-Felladrin/bge-reranker-v2-m3-q8_0.gguf"
  fi
fi

if [[ ! -f "${RERANK_SRC}" ]]; then
  echo "[!] Missing rerank GGUF: ${RERANK_SRC}" >&2
  exit 1
fi

echo "[*] VM200 rerank deploy → ${VM_HOST}"
ssh_vm "sudo mkdir -p ${REMOTE_MODEL_DIR} && sudo chown ${VM_USER}:${VM_USER} ${REMOTE_MODEL_DIR}"
echo "[*] Copy rerank GGUF..."
rsync -av --progress -e "ssh -i ${SSH_KEY}" "${RERANK_SRC}" "${VM_USER}@${VM_HOST}:${REMOTE_MODEL_DIR}/${REMOTE_NAME}"

ssh_vm "sudo tee /etc/systemd/system/llama-rerank.service" < "${SCRIPT_DIR}/llama-rerank.service" >/dev/null
ssh_vm "sudo systemctl daemon-reload && sudo systemctl enable --now llama-rerank.service"

echo "[*] Waiting for :8082..."
for i in $(seq 1 30); do
  if curl -sf "http://${VM_HOST}:8082/v1/models" >/dev/null 2>&1; then
    echo "[OK] Rerank server up (${i}s)"
    break
  fi
  sleep 2
done

curl -sf "http://${VM_HOST}:8082/v1/models" >/dev/null || {
  echo "[!] Rerank not reachable — journalctl -u llama-rerank on ${VM_HOST}"
  exit 1
}

echo "[*] Smoke /v1/rerank..."
curl -sf "http://${VM_HOST}:8082/v1/rerank" \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "bge-reranker-v2-m3-q8_0.gguf",
    "query": "capital of France",
    "top_n": 2,
    "documents": [
      "Paris is the capital of France.",
      "Berlin is the capital of Germany."
    ]
  }' | python3 -c "
import sys, json
r = json.load(sys.stdin)
top = r['results'][0]
print(f\"  top index={top['index']} score={top.get('relevance_score', top.get('score'))}\")
"

ssh_vm "nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader"
echo "[OK] VM200 rerank :8082 deployed."
