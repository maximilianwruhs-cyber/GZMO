#!/usr/bin/env bash
# Deploy VM200 (:110) unified retrieval router :8081 (Qwen3 embed + Qwen3-Reranker).
# Replaces llama-embed (:8081), llama-rerank (:8082), llama-librarian (:8083).
set -euo pipefail

VM_HOST="${GZMO_VM200_HOST:-192.168.31.110}"
VM_USER="${GZMO_VM200_USER:-maximilian}"
SSH_KEY="${GZMO_VM200_SSH_KEY:-${HOME}/.ssh/id_sidecar_proxmox}"
EMBED_SRC="${GZMO_EMBED_MODEL:-${HOME}/.cache/huggingface/llamacpp-qwen36-mtp/Qwen3-Embedding-0.6B-Q8_0.gguf}"
RERANK_REPO="${GZMO_RERANK_REPO:-Voodisss/Qwen3-Reranker-0.6B-GGUF-llama_cpp}"
RERANK_FILE="${GZMO_RERANK_GGUF:-Qwen3-Reranker-0.6B.F16.gguf}"
REMOTE_MODEL_DIR="/opt/models"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ssh_vm() {
  ssh -i "${SSH_KEY}" -o BatchMode=yes "${VM_USER}@${VM_HOST}" "$@"
}

if [[ ! -f "${EMBED_SRC}" ]]; then
  echo "[!] Missing embed GGUF: ${EMBED_SRC}" >&2
  exit 1
fi

echo "[*] VM200 retrieval router deploy → ${VM_HOST}"

RERANK_SRC="${GZMO_RERANK_GGUF_PATH:-${HOME}/.cache/gzmo-models/qwen3-rerank/${RERANK_FILE}}"
download_rerank() {
  mkdir -p "${HOME}/.cache/gzmo-models/qwen3-rerank"
  local dest="${HOME}/.cache/gzmo-models/qwen3-rerank/${RERANK_FILE}"
  local url="https://huggingface.co/${RERANK_REPO}/resolve/main/${RERANK_FILE}"
  echo "[*] Downloading ${RERANK_FILE}…"
  if command -v huggingface-cli >/dev/null 2>&1; then
    huggingface-cli download "${RERANK_REPO}" "${RERANK_FILE}" \
      --local-dir "${HOME}/.cache/gzmo-models/qwen3-rerank"
  elif command -v hf >/dev/null 2>&1; then
    hf download "${RERANK_REPO}" "${RERANK_FILE}" \
      --local-dir "${HOME}/.cache/gzmo-models/qwen3-rerank"
  else
    curl -fL --progress-bar -o "${dest}" "${url}"
  fi
  RERANK_SRC="${dest}"
}
if [[ ! -f "${RERANK_SRC}" ]]; then
  download_rerank
fi
if [[ ! -f "${RERANK_SRC}" ]]; then
  echo "[!] Missing rerank GGUF: ${RERANK_SRC}" >&2
  echo "    Set GZMO_RERANK_GGUF_PATH=/path/to/${RERANK_FILE}" >&2
  exit 1
fi

echo "[*] Copy GGUFs to ${REMOTE_MODEL_DIR}…"
ssh_vm "sudo mkdir -p ${REMOTE_MODEL_DIR} && sudo chown ${VM_USER}:${VM_USER} ${REMOTE_MODEL_DIR}"
rsync -av --progress -e "ssh -i ${SSH_KEY}" "${EMBED_SRC}" "${VM_USER}@${VM_HOST}:${REMOTE_MODEL_DIR}/"
rsync -av --progress -e "ssh -i ${SSH_KEY}" "${RERANK_SRC}" "${VM_USER}@${VM_HOST}:${REMOTE_MODEL_DIR}/"

echo "[*] Install router preset + systemd…"
ssh_vm "sudo tee ${REMOTE_MODEL_DIR}/gzmo-retrieval.ini" < "${SCRIPT_DIR}/gzmo-retrieval.ini" >/dev/null
ssh_vm "sudo tee /etc/systemd/system/llama-retrieval-router.service" \
  < "${SCRIPT_DIR}/llama-retrieval-router.service" >/dev/null
ssh_vm "sudo systemctl daemon-reload"

echo "[*] Retire legacy retrieval units…"
ssh_vm "sudo systemctl disable --now llama-embed.service 2>/dev/null || true"
ssh_vm "sudo systemctl disable --now llama-rerank.service 2>/dev/null || true"
ssh_vm "sudo systemctl disable --now llama-librarian.service 2>/dev/null || true"
ssh_vm "sudo systemctl disable --now llama-speculative.service 2>/dev/null || true"

echo "[*] Start retrieval router (:8081)…"
ssh_vm "sudo systemctl enable --now llama-retrieval-router.service"

echo "[*] Waiting for :8081…"
for i in $(seq 1 30); do
  if curl -sf "http://${VM_HOST}:8081/v1/models" >/dev/null 2>&1; then
    echo "[OK] Router up (${i}s)"
    break
  fi
  sleep 2
done
curl -sf "http://${VM_HOST}:8081/v1/models" >/dev/null || {
  echo "[!] Router not reachable — journalctl -u llama-retrieval-router on VM" >&2
  exit 1
}

echo "[*] Smoke embed (gzmo-embed)…"
dims="$(curl -sf "http://${VM_HOST}:8081/v1/embeddings" \
  -H 'Content-Type: application/json' \
  -d '{"model":"gzmo-embed","input":"vm200 router probe"}' \
  | python3 -c "import sys,json; print(len(json.load(sys.stdin)['data'][0]['embedding']))")"
[[ "${dims}" == "1024" ]] || { echo "[!] embed dims=${dims}, expected 1024" >&2; exit 1; }
echo "  embed dims=${dims}"

echo "[*] Smoke rerank (gzmo-rerank)…"
top_score="$(curl -sf "http://${VM_HOST}:8081/v1/rerank" \
  -H 'Content-Type: application/json' \
  -d '{"model":"gzmo-rerank","query":"GZMO Prime cognition","documents":["VM200 retrieval router","unrelated weather"],"top_n":1}' \
  | python3 -c "import sys,json; r=json.load(sys.stdin)['results'][0]; print(r.get('relevance_score', r.get('score', 0)))")"
python3 -c "s=float('${top_score}'); assert abs(s) > 1e-6, f'near-zero score {s}'"
echo "  rerank top_score=${top_score}"

ssh_vm "nvidia-smi --query-gpu=memory.used,memory.total --format=csv,noheader"

echo ""
echo "[OK] VM200 retrieval router deployed."
echo "Next: ./scripts/verify-production.sh"
echo "Bench: ./scripts/vm200/retrieval-bench/runner.py --profile profiles/post-router-qwen3.json"
