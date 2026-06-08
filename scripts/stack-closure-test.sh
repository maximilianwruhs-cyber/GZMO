#!/usr/bin/env bash
# Full production stack closure — run before declaring GZMO stack "done".
# Run from a normal shell (not Cursor sandbox): uvx MCP needs ~/.local/share/uv writes.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
LOG="${ROOT}/logs/stack-closure-$(date +%Y%m%d_%H%M%S).log"
FAIL=0
FULL_SYNC="${STACK_FULL_QDRANT_SYNC:-0}"

log() { echo "$*" | tee -a "$LOG"; }
pass() { log "[PASS] $*"; }
fail() { log "[FAIL] $*"; FAIL=1; }

mkdir -p "${ROOT}/logs"
: >"$LOG"

log "=== GZMO stack closure $(date -Iseconds) ==="
log ""

log "--- Bootstrap ---"
"${ROOT}/scripts/start-production.sh" >>"$LOG" 2>&1 && pass "start-production" || fail "start-production"
"${ROOT}/scripts/start-production.sh" --daemon >>"$LOG" 2>&1 && pass "daemon" || fail "daemon"
log ""

log "--- Production E2E (verify-production) ---"
"${ROOT}/scripts/verify-production.sh" >>"$LOG" 2>&1 && pass "verify-production" || fail "verify-production"
log ""

log "--- P1 unit + gateway ---"
(cd "$ROOT" && cargo test -p gzmo-core --lib -- --skip test_web_search_live >>"$LOG" 2>&1) && pass "gzmo-core tests" || fail "gzmo-core tests"
log ""

log "--- P1 readiness (spark + chat) ---"
"${ROOT}/scripts/p1-readiness-test.sh" >>"$LOG" 2>&1 && pass "p1-readiness-test" || fail "p1-readiness-test"
log ""

if [[ "$FULL_SYNC" == "1" ]]; then
  log "--- Qdrant full sync (optional) ---"
  "${ROOT}/scripts/sync-vault-to-qdrant.sh" >>"$LOG" 2>&1 && pass "qdrant sync" || fail "qdrant sync"
  log ""
fi

log "--- Shared MCP fragment present ---"
[[ -f "${ROOT}/config/shared-mcp-memory.json" ]] && pass "shared-mcp-memory.json" || fail "shared-mcp-memory.json"
log ""

log "=== RESULT ==="
if [[ "$FAIL" -eq 0 ]]; then
  log "STACK CLOSURE: PASS"
  log "Log: $LOG"
  exit 0
else
  log "STACK CLOSURE: FAIL — see $LOG"
  exit 1
fi
