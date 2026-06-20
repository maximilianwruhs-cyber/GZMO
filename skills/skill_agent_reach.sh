#!/usr/bin/env bash
# Agent-Reach shell bridge (Tier 2 network exception).
# Installs to ~/.agent-reach-venv on first `install` subcommand.
set -euo pipefail

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
VENV="${AGENT_REACH_VENV:-$HOME/.agent-reach-venv}"
BIN="$VENV/bin/agent-reach"
AR_HOME="${AGENT_REACH_HOME:-$HOME/.agent-reach}"

usage() {
  cat <<'EOF'
Agent-Reach skill (Tier 2 exception — see compliance.network_exceptions)

  install          Create venv + pip install agent-reach + safe configure
  doctor           Channel health check
  read --url URL   Read a URL via agent-reach
  status           Show venv + home paths
EOF
}

ensure_venv() {
  if [[ ! -x "$BIN" ]]; then
    echo "ERROR: agent-reach not installed — run: $0 install"
    exit 1
  fi
}

cmd="${1:-status}"
shift || true

case "$cmd" in
  install)
    if ! command -v uv >/dev/null 2>&1; then
      echo "ERROR: uv required for venv (PEP 668). Install uv or set AGENT_REACH_BIN."
      exit 1
    fi
    uv venv "$VENV"
    uv pip install --python "$VENV/bin/python" -q "https://github.com/Panniantong/agent-reach/archive/main.zip"
    "$BIN" install --env=auto --safe || true
    mkdir -p "$AR_HOME"
    jq -n \
      --arg venv "$VENV" \
      --arg home "$AR_HOME" \
      --arg bin "$BIN" \
      '{venv: $venv, home: $home, bin: $bin, network_tier: "exception"}'
    ;;

  doctor)
    ensure_venv
    "$BIN" doctor
    ;;

  read)
    url=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --url) url="${2:-}"; shift 2 ;;
        *) echo "Unknown arg: $1"; usage; exit 1 ;;
      esac
    done
    [[ -n "$url" ]] || { echo "ERROR: --url required"; exit 1; }
    ensure_venv
    "$BIN" read "$url"
    ;;

  status)
    jq -n \
      --arg venv "$VENV" \
      --arg home "$AR_HOME" \
      --arg bin "$BIN" \
      --argjson installed "$( [[ -x "$BIN" ]] && echo true || echo false )" \
      '{venv: $venv, home: $home, bin: $bin, installed: $installed, network_tier: "exception"}'
    ;;

  *)
    echo "Unknown command: $cmd"
    usage
    exit 1
    ;;
esac
