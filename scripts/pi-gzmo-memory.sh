#!/usr/bin/env bash
# Frontend bridge: pi-rust (or any operator client) → GZMO Platform hot memory.
# Wraps `gzmo memory *` with stable session id. See docs/ops/PI_GZMO_MEMORY_INTEGRATION.md
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/gzmo.toml}"
SESSION_FILE="${GZMO_SESSION_FILE:-$ROOT/data/pi-frontend-session.id}"
BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
CONTEXT_FILE="${PI_MEMORY_CONTEXT:-${HOME}/.pi/agent/MEMORY_CONTEXT.md}"
REFERENCE_FILE="${PI_MEMORY_REFERENCE:-${HOME}/.pi/agent/MEMORY_REFERENCE.md}"

usage() {
  cat <<EOF
Usage: $0 <command> [args]

Commands:
  session          Print stable GZMO_SESSION_ID (create if missing)
  session-new      Rotate session id (new operator conversation)
  turn-start       Clear scratch + start new turn (call before each user message)
  search <query>   gzmo_memory_search [--limit N]
  recall           gzmo_memory_recall_pull → [RECALL] block for LLM context
                   Options: --with-context, --with-reference
  status [--json]  Vault/scratch status
  prep <query>     turn-start + search (common pi workflow)
                   Options: --with-context, --with-reference

Tiered memory (Phase 3):
  --with-context   Append MEMORY_CONTEXT.md after recall/prep ([MEMORY_CONTEXT] block)
  --with-reference Append MEMORY_REFERENCE.md after recall/prep ([MEMORY_REFERENCE] block)

Env:
  GZMO_BIN, GZMO_CONFIG, GZMO_SESSION_FILE
  PI_MEMORY_CONTEXT, PI_MEMORY_REFERENCE
EOF
}

emit_tiered_layer() {
  local file="$1"
  local tag="$2"
  if [[ -f "$file" ]]; then
    echo ""
    echo "[${tag}]"
    cat "$file"
    echo "[/${tag}]"
  else
    echo "WARN: tiered layer not found: $file" >&2
  fi
}

parse_recall_flags() {
  WITH_CONTEXT=0
  WITH_REFERENCE=0
  local _args=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --with-context) WITH_CONTEXT=1; shift ;;
      --with-reference) WITH_REFERENCE=1; shift ;;
      *) _args+=("$1"); shift ;;
    esac
  done
  RECALL_ARGS=("${_args[@]}")
}

ensure_bin() {
  if [[ -x "$BIN" ]]; then
    return 0
  fi
  echo "Building gzmo (release)…" >&2
  (cd "$ROOT" && cargo build --release -p gzmo-cli -q)
  BIN="$ROOT/target/release/gzmo"
}

read_session() {
  if [[ -f "$SESSION_FILE" ]]; then
    tr -d '[:space:]' < "$SESSION_FILE"
    return 0
  fi
  mkdir -p "$(dirname "$SESSION_FILE")"
  if command -v uuidgen >/dev/null 2>&1; then
    uuidgen | tr '[:upper:]' '[:lower:]' | tee "$SESSION_FILE" >/dev/null
  else
    # fallback: timestamp-based id
    echo "pi-$(date +%s)-$$" | tee "$SESSION_FILE" >/dev/null
  fi
  tr -d '[:space:]' < "$SESSION_FILE"
}

export_session() {
  export GZMO_SESSION_ID
  GZMO_SESSION_ID="$(read_session)"
}

cmd="${1:-}"
shift || true

case "$cmd" in
  ""|-h|--help|help)
    usage
    ;;
  session)
    echo "$(read_session)"
    ;;
  session-new)
    rm -f "$SESSION_FILE"
    echo "$(read_session)"
    ;;
  turn-start)
    ensure_bin
    export_session
    "$BIN" memory turn-start
    ;;
  search)
    ensure_bin
    export_session
    if [[ $# -lt 1 ]]; then
      echo "missing query" >&2
      exit 1
    fi
    "$BIN" memory search "$@"
    ;;
  recall)
    ensure_bin
    export_session
    parse_recall_flags "$@"
    # recall ignores positional args; only forward extras when present (avoids passing "")
    if [[ ${#RECALL_ARGS[@]} -gt 0 ]]; then
      "$BIN" memory recall "${RECALL_ARGS[@]}"
    else
      "$BIN" memory recall
    fi
    if [[ "${WITH_CONTEXT:-0}" -eq 1 ]]; then
      emit_tiered_layer "$CONTEXT_FILE" "MEMORY_CONTEXT"
    fi
    if [[ "${WITH_REFERENCE:-0}" -eq 1 ]]; then
      emit_tiered_layer "$REFERENCE_FILE" "MEMORY_REFERENCE"
    fi
    ;;
  status)
    ensure_bin
    export_session
    "$BIN" memory status "$@"
    ;;
  prep)
    ensure_bin
    export_session
    parse_recall_flags "$@"
    if [[ ${#RECALL_ARGS[@]} -lt 1 ]]; then
      echo "missing query" >&2
      exit 1
    fi
    "$BIN" memory turn-start
    "$BIN" memory search "${RECALL_ARGS[@]}"
    if [[ "${WITH_CONTEXT:-0}" -eq 1 ]]; then
      emit_tiered_layer "$CONTEXT_FILE" "MEMORY_CONTEXT"
    fi
    if [[ "${WITH_REFERENCE:-0}" -eq 1 ]]; then
      emit_tiered_layer "$REFERENCE_FILE" "MEMORY_REFERENCE"
    fi
    ;;
  *)
    echo "unknown command: $cmd" >&2
    usage >&2
    exit 1
    ;;
esac
