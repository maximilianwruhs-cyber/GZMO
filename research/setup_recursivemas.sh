#!/usr/bin/env bash
# Install RecursiveMAS + bridge deps for real latent-space inference.
set -euo pipefail

RESEARCH_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECTS_ROOT="$(cd "$RESEARCH_DIR/../../.." && pwd)"
RECURSIVEMAS_ROOT="${RECURSIVEMAS_ROOT:-$PROJECTS_ROOT/RecursiveMAS}"
VENV_DIR="${RECURSIVEMAS_VENV:-$RESEARCH_DIR/.venv-rmas}"
PYTHON="${PYTHON:-python3.11}"
HF_HOME="${HF_HOME:-$RESEARCH_DIR/.cache/huggingface}"
export HF_HOME

echo "==> RecursiveMAS root: $RECURSIVEMAS_ROOT"
echo "==> Python venv:      $VENV_DIR"
echo "==> HF cache:         $HF_HOME"

if [[ ! -d "$RECURSIVEMAS_ROOT/.git" ]]; then
  echo "==> Cloning RecursiveMAS..."
  git clone --depth 1 https://github.com/RecursiveMAS/RecursiveMAS.git "$RECURSIVEMAS_ROOT"
fi

if ! command -v "$PYTHON" >/dev/null 2>&1; then
  echo "error: $PYTHON not found. Install Python 3.10+ (3.11 recommended)." >&2
  exit 1
fi

echo "==> Creating venv with $PYTHON"
"$PYTHON" -m venv "$VENV_DIR"
# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"
python -m pip install --upgrade pip wheel

echo "==> Installing PyTorch 2.9.0 (CUDA 12.8 wheels for Blackwell / RTX 50-series)"
python -m pip install torch==2.9.0 --index-url https://download.pytorch.org/whl/cu128

echo "==> Installing RecursiveMAS requirements"
python -m pip install -r "$RECURSIVEMAS_ROOT/requirements.txt"
python -m pip install -r "$RESEARCH_DIR/requirements-bridge.txt"

mkdir -p "$HF_HOME"

cat >"$RESEARCH_DIR/.env.recursivemas" <<EOF
# Source before running the real RecursiveMAS bridge:
#   source $RESEARCH_DIR/.env.recursivemas
export RECURSIVEMAS_ROOT="$RECURSIVEMAS_ROOT"
export RECURSIVEMAS_VENV="$VENV_DIR"
export HF_HOME="$HF_HOME"
export RECURSIVEMAS_STYLE="${RECURSIVEMAS_STYLE:-sequential_light}"
export RECURSIVEMAS_DATASET="${RECURSIVEMAS_DATASET:-math500}"
export RECURSIVEMAS_DEVICE="${RECURSIVEMAS_DEVICE:-cuda:0}"
unset RECURSIVEMAS_MOCK
# Optional: export HF_TOKEN=hf_... for faster Hugging Face downloads
EOF

echo
echo "==> Setup complete."
echo "Next:"
echo "  source $RESEARCH_DIR/.env.recursivemas"
echo "  $RESEARCH_DIR/run_recursivemas_bridge.sh"
echo
echo "Optional: prefetch HF checkpoints (downloads ~5–8 GB for sequential_light):"
echo "  source $VENV_DIR/bin/activate && source $RESEARCH_DIR/.env.recursivemas"
echo "  python $RECURSIVEMAS_ROOT/inference/run.py --style sequential_light --dataset math500 --device cuda:0 --num_samples 1"
