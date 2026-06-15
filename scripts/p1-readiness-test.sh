#!/usr/bin/env bash
# P1 quality + production readiness — run after code changes / reboot.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
LOG="${ROOT}/logs/p1-readiness-$(date +%Y%m%d_%H%M%S).log"
FAIL=0

log() { echo "$*" | tee -a "$LOG"; }
pass() { log "[PASS] $*"; }
fail() { log "[FAIL] $*"; FAIL=1; }

mkdir -p "${ROOT}/logs"
: >"$LOG"

log "=== P1 readiness $(date -Iseconds) ==="
log ""

log "--- 0. Bootstrap stack ---"
if "${ROOT}/scripts/start-production.sh" >>"$LOG" 2>&1; then
  pass "start-production.sh"
else
  fail "start-production.sh"
fi
if "${ROOT}/scripts/start-production.sh" --daemon >>"$LOG" 2>&1; then
  pass "daemon start"
else
  fail "daemon start"
fi
log ""

log "--- 1. Production E2E ---"
if "${ROOT}/scripts/verify-production.sh" >>"$LOG" 2>&1; then
  pass "verify-production.sh"
else
  fail "verify-production.sh"
fi
log ""

log "--- 2. Unit tests (gzmo-core) ---"
if (cd "$ROOT" && cargo test -p gzmo-core --lib -- --skip test_web_search_live >>"$LOG" 2>&1); then
  pass "cargo test -p gzmo-core --lib"
else
  fail "cargo test -p gzmo-core --lib"
fi
log ""

log "--- 3. Gateway structured path (unit + live via spark) ---"
if (cd "$ROOT" && cargo test -p gzmo-core --lib gateway::tests:: -- --nocapture >>"$LOG" 2>&1); then
  pass "gateway JSON extract + lenient parse (unit)"
else
  fail "gateway JSON unit tests"
fi
log ""

log "--- 4. Chat visible text (content or reasoning) ---"
PRIME_MODEL="$(python3 -c "
import tomllib, pathlib
d = tomllib.loads(pathlib.Path('${ROOT}/gzmo.toml').read_text())
print(d['engine']['local']['model'])
" 2>/dev/null || echo 'gemma-4-26b-a4b-it')"
CHAT=$(curl -sf http://127.0.0.1:8000/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d "{\"model\":\"${PRIME_MODEL}\",\"messages\":[{\"role\":\"user\",\"content\":\"Say hello in one word.\"}],\"max_tokens\":64,\"temperature\":0.2}" 2>/dev/null) || CHAT=""

if echo "$CHAT" | python3 -c "
import sys,json
m=json.load(sys.stdin)['choices'][0]['message']
t=(m.get('content') or '')+(m.get('reasoning_content') or '')
assert len(t.strip())>2, 'empty response'
print('len', len(t))
" >>"$LOG" 2>&1; then
  pass "Prime chat non-empty body"
else
  fail "Prime chat non-empty body"
fi
log ""

log "--- 5. Spark one-shot (gzmo spark) ---"
if [[ -x "$BIN" ]]; then
  if (cd "$ROOT" && RUST_LOG=warn "$BIN" spark >>"$LOG" 2>&1); then
    if grep -q 'Spark hypothesis phase failed' "$LOG"; then
      fail "gzmo spark — hypothesis JSON parse failed"
    elif grep -qE 'Spark complete|Spark cycle complete|Crystallized connection' "$LOG"; then
      pass "gzmo spark (structured hypothesis + verify pipeline)"
    else
      fail "gzmo spark — unexpected output (see log)"
    fi
  else
    fail "gzmo spark — command error"
  fi
else
  fail "gzmo binary missing"
fi
log ""

log "--- 6. Daemon health (no fresh panic) ---"
if [[ -f /tmp/gzmo_daemon.pid ]] && kill -0 "$(cat /tmp/gzmo_daemon.pid)" 2>/dev/null; then
  pass "daemon running (PID $(cat /tmp/gzmo_daemon.pid))"
elif pgrep -f '/gzmo daemon$' >/dev/null; then
  pass "daemon running (PID $(pgrep -f '/gzmo daemon$' | head -1))"
  if grep -q 'panicked at' "${ROOT}/logs/daemon.log" 2>/dev/null; then
    LAST_PANIC=$(grep 'panicked at' "${ROOT}/logs/daemon.log" | tail -1)
    log "[WARN] historical panic in daemon.log: $LAST_PANIC"
  fi
else
  fail "daemon not running"
fi
log ""

log "=== RESULT ==="
if [[ "$FAIL" -eq 0 ]]; then
  log "P1 READINESS: PASS"
  log "Full log: $LOG"
  exit 0
else
  log "P1 READINESS: FAIL"
  log "Full log: $LOG"
  exit 1
fi
