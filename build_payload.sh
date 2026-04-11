#!/usr/bin/env bash
set -euo pipefail

CYAN="\032[36m"
GREEN="\032[32m"
RESET="\032[0m"
BOLD="\032[1m"

log() { echo -e "  ${CYAN}▸${RESET} $1"; }
ok() { echo -e "  ${GREEN}✔${RESET} $1"; }

echo -e "\n${BOLD}╔══════════════════════════════════════════════╗"
echo -e "║          GZMO PAYLOAD PROVISIONER            ║"
echo -e "╚══════════════════════════════════════════════╝${RESET}\n"

# 1. Compile llama-server
log "Cloning llama.cpp repository into /tmp..."
rm -rf /tmp/llama.cpp
git clone --depth 1 https://github.com/ggerganov/llama.cpp.git /tmp/llama.cpp

log "Compiling llama-server (Optimized for Ubuntu/CUDA)..."
cd /tmp/llama.cpp
# Using modern CMake with GGML_CUDA=ON if nvidia-smi exists, else standard CPU
if command -v nvidia-smi >/dev/null 2>&1; then
    cmake -B build -S . -DGGML_CUDA=ON -DCMAKE_BUILD_TYPE=Release
else
    cmake -B build -S . -DCMAKE_BUILD_TYPE=Release
fi
cmake --build build --config Release -j$(nproc) --target llama-server

log "Installing llama-server into GZMO bin/..."
cd /home/maximilian-wruhs/Die\ \"Kuchl\"/GZMO
mkdir -p ./bin
cp /tmp/llama.cpp/build/bin/llama-server ./bin/llama-server
chmod +x ./bin/llama-server
ok "llama-server successfully provisioned to ./bin/"

# 2. Download Qwen 0.5B Model
mkdir -p ./models
MODEL_NAME="Qwen2.5-0.5B-Instruct-Q4_K_M.gguf"
MODEL_URL="https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf?download=true"

if [ ! -f "./models/$MODEL_NAME" ]; then
    log "Downloading light-weight testing model ($MODEL_NAME) from HuggingFace..."
    # Using L parameter to follow redirects
    curl -L "$MODEL_URL" -o "./models/$MODEL_NAME"
    ok "Model downloaded to ./models/"
else
    ok "Test model already exists in ./models/"
fi

# 3. Clean up the dummy model from earlier test if it exists
[ -f "./models/dummy.gguf" ] && rm -f ./models/dummy.gguf

ok "Payload provisioning complete. GZMO is ready for local ignition testing."
