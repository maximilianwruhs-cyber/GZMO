#!/usr/bin/env bash
# Start the real RecursiveMAS HTTP bridge (not mock).
set -euo pipefail

RESEARCH_DIR="$(cd "$(dirname "$0")" && pwd)"
ENV_FILE="$RESEARCH_DIR/.env.recursivemas"
VENV_DIR="${RECURSIVEMAS_VENV:-$RESEARCH_DIR/.venv-rmas}"
PORT="${RECURSIVEMAS_PORT:-8765}"
HOST="${RECURSIVEMAS_HOST:-127.0.0.1}"

if [[ -f "$ENV_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
fi

if [[ ! -d "$VENV_DIR" ]]; then
  echo "RecursiveMAS venv missing. Run: $RESEARCH_DIR/setup_recursivemas.sh" >&2
  exit 1
fi

# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"
unset RECURSIVEMAS_MOCK

exec python "$RESEARCH_DIR/recursivemas_bridge.py" \
  --host "$HOST" \
  --port "$PORT"
