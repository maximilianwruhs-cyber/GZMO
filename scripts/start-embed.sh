#!/usr/bin/env bash
# RETIRED (2026-06-13): Pi KB embed now uses VM200 :8081 only. Do not start.
# Kept for reference; use pi-kb-reindex.sh / knowledge-base.ts instead.
# Qwen3 embedding server on the workstation (:8002) — optional Pi KB offline fallback.
# Pi KB primary embed is VM200 :8081 (gzmo-embed); GZMO daemon uses the same (gzmo.toml).
# Default CPU inference (NGl=0) so Prime keeps VRAM on the 5070 Ti pair.
set -euo pipefail

EMBED_GGUF="${GZMO_EMBED_MODEL:-${HOME}/.cache/huggingface/llamacpp-qwen36-mtp/Qwen3-Embedding-0.6B-Q8_0.gguf}"
PORT="${GZMO_EMBED_PORT:-8002}"
NGl="${GZMO_EMBED_NGL:-0}"
LLAMA_ROOT="${GZMO_LLAMA_ROOT:-${HOME}/Projects/llama.cpp}"
SERVER="${LLAMA_ROOT}/build/bin/llama-server"

if [[ ! -f "${EMBED_GGUF}" ]]; then
  echo "[!] Embedding GGUF not found: ${EMBED_GGUF}" >&2
  exit 1
fi
if [[ ! -x "${SERVER}" ]]; then
  echo "[!] Build llama-server first: cmake -B build && cmake --build build -j" >&2
  exit 1
fi

echo "[*] embed server :${PORT} ngl=${NGl} — $(basename "${EMBED_GGUF}")"
exec "${SERVER}" \
  -m "${EMBED_GGUF}" \
  --alias gzmo-embed \
  --embedding \
  --pooling last \
  -ngl "${NGl}" \
  --port "${PORT}" \
  --host 127.0.0.1 \
  "$@"
