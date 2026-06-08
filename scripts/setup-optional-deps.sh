#!/usr/bin/env bash
# Optional script deps (neo4j driver for graph-recall-stream.py).
# Uses uv when available; falls back to python3 -m venv + pip in scripts/.venv.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="${ROOT}/scripts/.venv"
REQ="${ROOT}/scripts/requirements-optional.txt"

if command -v uv >/dev/null 2>&1; then
  uv venv "${VENV}" --python python3
  uv pip install --python "${VENV}/bin/python" -r "${REQ}"
else
  python3 -m venv "${VENV}"
  "${VENV}/bin/pip" install -r "${REQ}"
fi

echo "[OK] optional deps → ${VENV} (neo4j for graph-recall-stream.py)"
