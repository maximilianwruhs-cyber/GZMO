#!/usr/bin/env bash
# Operator session preflight (~5–15s). Not the M4/platform baseline gate.
#
#   ./scripts/auto-health-check.sh         # quick — every pi session start
#   ./scripts/auto-health-check.sh --deep  # delegates to verify-baseline-green.sh
#
# Quick FAIL → stop operator work until Prime/embed/Qdrant/memory bridge recover.
# Deep baseline → after infra/ingest changes or before sign-off.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ "${1:-}" == "--deep" ]]; then
  echo "🏥 Deep layer → verify-baseline-green.sh"
  exec "$ROOT/scripts/verify-baseline-green.sh"
fi

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
PASS=0; FAIL=0; WARN=0
NOTES=()

ok()   { echo -e "${GREEN}[PASS]${NC} $1"; PASS=$((PASS+1)); }
fail() { echo -e "${RED}[FAIL]${NC} $1"; FAIL=$((FAIL+1)); }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; WARN=$((WARN+1)); NOTES+=("$1"); }
info() { echo -e "${CYAN}ℹ $1${NC}"; }

prime_models_ok() {
  local body
  body="$(curl -sf --max-time 5 http://127.0.0.1:8000/v1/models 2>/dev/null)" || return 1
  [[ -n "$body" ]] || return 1
  echo "$body" | grep -qE '"data"|"models"' || return 1
  echo "$body" | python3 -c "
import json, sys
j = json.load(sys.stdin)
items = j.get('data') or j.get('models') or []
if not items:
    sys.exit(1)
m = items[0]
print(m.get('id') or m.get('name') or m.get('model') or 'ok')
" 2>/dev/null
}

EMBED_URL="${EMBED_PROBE_URL:-$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('embeddings', {}).get('url', 'http://192.168.31.110:8081/v1').rstrip('/'))
" 2>/dev/null || echo 'http://192.168.31.110:8081/v1'
)}"
EMBED_MODEL="${EMBED_MODEL:-$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('embeddings', {}).get('model', 'Qwen3-Embedding-0.6B-Q8_0.gguf'))
" 2>/dev/null || echo 'Qwen3-Embedding-0.6B-Q8_0.gguf'
)}"
RERANK_URL="${RERANK_PROBE_URL:-$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('rerank', {}).get('url', 'http://192.168.31.110:8082/v1').rstrip('/'))
" 2>/dev/null || echo 'http://192.168.31.110:8082/v1'
)}"
RERANK_MODEL="${RERANK_MODEL:-$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('rerank', {}).get('model', 'bge-reranker-v2-m3-q8_0.gguf'))
" 2>/dev/null || echo 'bge-reranker-v2-m3-q8_0.gguf'
)}"
REDIS_URL="${REDIS_PROBE_URL:-$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('redis', {}).get('url', 'redis://192.168.31.202:6379'))
" 2>/dev/null || echo 'redis://192.168.31.202:6379'
)}"

embed_api_ok() {
  local url="$1"
  local dims
  dims="$(curl -sf --max-time 8 "${url%/}/embeddings" \
    -H 'Content-Type: application/json' \
    -d "{\"model\":\"${EMBED_MODEL}\",\"input\":\"preflight\"}" \
    | python3 -c "import sys,json; print(len(json.load(sys.stdin)['data'][0]['embedding']))" 2>/dev/null)" || return 1
  [[ "$dims" == "1024" ]] || return 1
  echo "$url (${dims}-dim)"
}

rerank_api_ok() {
  local url="$1"
  curl -sf --max-time 8 "${url%/}/rerank" \
    -H 'Content-Type: application/json' \
    -d "{\"model\":\"${RERANK_MODEL}\",\"query\":\"capital of France\",\"top_n\":1,\"documents\":[\"Paris is the capital of France.\",\"Berlin is in Germany.\"]}" \
    | python3 -c "import sys,json; r=json.load(sys.stdin); assert r.get('results')" 2>/dev/null
}

redis_ping_ok() {
  local host port reply
  host="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${REDIS_URL}'); print(u.hostname or '127.0.0.1')")"
  port="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${REDIS_URL}'); print(u.port or 6379)")"
  reply="$(timeout 3 bash -c "exec 3<>/dev/tcp/${host}/${port} && printf 'PING\r\n' >&3 && head -c 16 <&3" 2>/dev/null || true)"
  [[ "$reply" == *PONG* ]]
}

echo "🏥 Operator preflight (quick) — $(date +%H:%M:%S)"
echo "─────────────────────────────────────────"

# ── 1. Prime :8000 ──
if model="$(prime_models_ok)"; then
  ok "Prime :8000 ($model)"
else
  fail "Prime :8000 unreachable or invalid /v1/models JSON"
fi

# ── 2. Embeddings (functional /v1/embeddings, 1024-dim) ──
if embed_base="$(embed_api_ok "$EMBED_URL")"; then
  ok "Embeddings $embed_base"
else
  fail "Embeddings ${EMBED_URL} unreachable or not 1024-dim (needs LAN 192.168.31.0/24)"
fi

# ── 2b. Rerank (functional /rerank — /v1/models alone is insufficient) ──
if rerank_api_ok "$RERANK_URL"; then
  ok "Rerank ${RERANK_URL}"
else
  fail "Rerank ${RERANK_URL} unreachable or invalid /rerank response"
fi

# ── 2c. Redis scratch (PING — required for cross-process recall) ──
if redis_ping_ok; then
  ok "Redis scratch ${REDIS_URL}"
else
  fail "Redis ${REDIS_URL} unreachable (recall degrades to in-memory per process)"
fi

# ── 3. Qdrant :6333 ──
QDRANT_URL="${QDRANT_PROBE_URL:-http://192.168.31.202:6333}"
qdrant_json="$(curl -sf --max-time 5 "$QDRANT_URL/collections" 2>/dev/null || true)"
if [[ -n "$qdrant_json" ]]; then
  cols="$(echo "$qdrant_json" | python3 -c "import sys,json; print(len(json.load(sys.stdin)['result']['collections']))" 2>/dev/null || echo "?")"
  ok "Qdrant $QDRANT_URL ($cols collections)"
else
  fail "Qdrant $QDRANT_URL unreachable"
fi

# ── 4. Platform memory bridge (Redis scratch) ──
MEMORY_SCRIPT="$ROOT/scripts/pi-gzmo-memory.sh"
if [[ -f "$MEMORY_SCRIPT" ]]; then
  [[ -x "$MEMORY_SCRIPT" ]] || chmod +x "$MEMORY_SCRIPT" 2>/dev/null || true
  mem_status="$("$MEMORY_SCRIPT" status 2>/dev/null || true)"
  if echo "$mem_status" | grep -q 'scratch=redis'; then
    vault="$(echo "$mem_status" | grep -o 'vault_facts=[0-9]*' || true)"
    session="$(echo "$mem_status" | grep -o 'session=[a-f0-9-]*' || true)"
    ok "Platform memory scratch=redis ($vault, $session)"
  elif echo "$mem_status" | grep -q 'scratch=in-memory'; then
    warn "Platform memory scratch=in-memory (Redis/LAN down — recall degraded)"
  else
    fail "pi-gzmo-memory.sh status: ${mem_status:-empty}"
  fi
else
  fail "pi-gzmo-memory.sh missing"
fi

# ── 5. Git status (informational) ──
if git status --porcelain 2>/dev/null | grep -q .; then
  dirty_count="$(git status --porcelain 2>/dev/null | wc -l)"
  warn "Git dirty: $dirty_count changed files"
  info "Changed files (max 10):"
  git status --short 2>/dev/null | head -10 | sed 's/^/    /' || true
else
  ok "Git clean"
fi

# ── 6. Subagent extension (optional operator feature) ──
SETTINGS="$HOME/.pi/agent/settings.json"
if [[ -d "$HOME/.pi/agent/npm/node_modules/pi-subagents" ]]; then
  subagent_ver="$(node -e "console.log(require('$HOME/.pi/agent/npm/node_modules/pi-subagents/package.json').version)" 2>/dev/null || echo "?")"
  if [[ -f "$SETTINGS" ]] && grep -q 'pi-subagents' "$SETTINGS" 2>/dev/null; then
    ok "Subagent npm v$subagent_ver (listed in settings.json)"
  else
    warn "Subagent npm v$subagent_ver installed but not in settings.json packages — pi restart needed"
  fi
else
  warn "Subagent extension not installed (optional — pi install npm:pi-subagents)"
fi

# ── Summary ──
echo ""
echo "════════════════════════════════════════"
echo -e "  ${GREEN}PASS: $PASS${NC}  ${RED}FAIL: $FAIL${NC}  ${YELLOW}WARN: $WARN${NC}"
echo "════════════════════════════════════════"

if [[ "$FAIL" -gt 0 ]]; then
  echo -e "${RED}⚠ PREFLIGHT FAILED — restore Prime / embed / Qdrant / memory bridge before work${NC}"
  echo "  Deep check: ./scripts/auto-health-check.sh --deep"
  exit 1
fi

if [[ "$WARN" -gt 0 ]]; then
  echo -e "${YELLOW}⚠ Preflight OK with warnings${NC}"
  for note in "${NOTES[@]}"; do
    echo -e "   ${YELLOW}• $note${NC}"
  done
fi

echo "  Deep baseline: ./scripts/auto-health-check.sh --deep"
exit 0
