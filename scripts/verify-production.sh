#!/usr/bin/env bash
# End-to-end production verification (Prime + embed + vault + Neo4j).
# Exit 0 if all required checks pass. Sovereign :8010 is optional (expected down).
#
# NOT an M4 eval gate — this script does not run ingest-quality contract/recall/faithfulness.
# See docs/INFRASTRUCTURE_OVERVIEW.md §6.4 (verify-production vs eval-quick vs certify).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
FAIL=0

pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; FAIL=1; }

EMBED_URL="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('embeddings', {}).get('url', 'http://192.168.31.110:8081/v1').rstrip('/'))
" 2>/dev/null || echo "http://192.168.31.110:8081/v1"
)"
EMBED_MODEL="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('embeddings', {}).get('model', 'gzmo-embed'))
" 2>/dev/null || echo "gzmo-embed"
)"
QDRANT_ENABLED="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print('1' if d.get('qdrant', {}).get('enabled') else '0')
" 2>/dev/null || echo "0"
)"
LIBRARIAN_URL="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
lib = d.get('librarian', {})
if lib.get('enabled'):
    print(lib.get('url', '').rstrip('/'))
" 2>/dev/null || true
)"
RERANK_URL="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
rr = d.get('rerank', {})
if rr.get('enabled'):
    print(rr.get('url', '').rstrip('/'))
" 2>/dev/null || true
)"
RERANK_MODEL="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('rerank', {}).get('model', 'bge-reranker-v2-m3-q8_0.gguf'))
" 2>/dev/null || echo "bge-reranker-v2-m3-q8_0.gguf"
)"
REDIS_URL="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
r = d.get('redis', {})
if r.get('enabled'):
    print(r.get('url', 'redis://192.168.31.202:6379'))
" 2>/dev/null || true
)"

echo "GZMO production E2E — $(date -Iseconds)"
echo "Embed endpoint: ${EMBED_URL}"
echo ""

SKIP_PRIME_CHECK="${GZMO_SKIP_PRIME:-}"
if [[ -z "${SKIP_PRIME_CHECK}" ]]; then
  SKIP_PRIME_CHECK="$(python3 -c "
import tomllib, pathlib
d = tomllib.loads(pathlib.Path('${ROOT}/gzmo.toml').read_text())
mode = d.get('engine', {}).get('active_mode', 'local')
cloud_bg = d.get('routing', {}).get('cloud_first_background', False)
print('1' if mode == 'cloud' or cloud_bg else '0')
" 2>/dev/null || echo '0')"
fi

if [[ "${SKIP_PRIME_CHECK}" == "1" ]]; then
  pass "Prime :8000 (skipped — cloud mode)"
else
  curl -sf http://127.0.0.1:8000/v1/models >/dev/null && pass "Prime :8000" || fail "Prime :8000"
fi
if [[ -n "${REDIS_URL:-}" ]]; then
  redis_host="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${REDIS_URL}'); print(u.hostname or '127.0.0.1')")"
  redis_port="$(python3 -c "from urllib.parse import urlparse; u=urlparse('${REDIS_URL}'); print(u.port or 6379)")"
  if python3 -c "
import socket
s = socket.create_connection(('${redis_host}', int('${redis_port}')), timeout=3)
s.sendall(b'PING\r\n')
reply = s.recv(16).decode(errors='ignore')
s.close()
print(reply)
" 2>/dev/null | grep -q PONG; then
    pass "Redis scratch ${REDIS_URL}"
  else
    fail "Redis scratch ${REDIS_URL}"
  fi
fi
if systemctl --user is-active gzmo-daemon.service >/dev/null 2>&1; then
  pass "Daemon running (gzmo-daemon.service)"
elif [[ -f /tmp/gzmo_daemon.pid ]] && kill -0 "$(cat /tmp/gzmo_daemon.pid)" 2>/dev/null; then
  pass "Daemon running (PID $(cat /tmp/gzmo_daemon.pid))"
elif pgrep -f '[^/]gzmo daemon$' >/dev/null 2>&1 || pgrep -f '/gzmo daemon$' >/dev/null; then
  pass "Daemon running"
else
  fail "Daemon running"
fi

if [[ -x "$BIN" ]]; then
  # One-shot health — avoid pipefail false negatives when gzmo health exits 1 (e.g. sovereign).
  HEALTH_OUT="$(cd "$ROOT" && "$BIN" health 2>/dev/null || true)"
  health_probe() {
    echo "$HEALTH_OUT" | grep -q "\\[OK\\] $1" && pass "gzmo health ($1)" || fail "gzmo health ($1)"
  }
  health_probe llm
  health_probe embeddings
  health_probe neo4j
  health_probe mcp_memory
  if [[ "$QDRANT_ENABLED" == "1" ]]; then
    health_probe qdrant
  fi
  if [[ -n "${RERANK_URL:-}" ]]; then
    health_probe rerank
    RERANK_SCORE="$(
      curl -sf "${RERANK_URL}/rerank" \
        -H 'Content-Type: application/json' \
        -d "{\"model\":\"${RERANK_MODEL}\",\"query\":\"capital of France\",\"top_n\":1,\"documents\":[\"Paris is the capital of France.\",\"Berlin is in Germany.\"]}" \
        | python3 -c "import sys,json; r=json.load(sys.stdin)['results'][0]; print(r.get('relevance_score', r.get('score', 0)))" 2>/dev/null || echo 0
    )"
    if python3 -c "s=float('${RERANK_SCORE}'); assert abs(s) > 1e-6" 2>/dev/null; then
      pass "Rerank ${RERANK_URL} (top ${RERANK_SCORE})"
    else
      fail "Rerank ${RERANK_URL} (near-zero score ${RERANK_SCORE})"
    fi
  fi
  if [[ -n "${LIBRARIAN_URL:-}" ]]; then
    health_probe librarian
    curl -sf "${LIBRARIAN_URL}/models" >/dev/null && pass "Librarian ${LIBRARIAN_URL}" || fail "Librarian ${LIBRARIAN_URL}"
  fi
else
  fail "gzmo binary missing (cargo build -p gzmo-cli)"
fi

DIMS=$(curl -sf "${EMBED_URL}/embeddings" \
  -H 'Content-Type: application/json' \
  -d "{\"model\":\"${EMBED_MODEL}\",\"input\":\"e2e\"}" \
  | python3 -c "import sys,json; print(len(json.load(sys.stdin)['data'][0]['embedding']))" 2>/dev/null || echo 0)
[[ "$DIMS" == "1024" ]] && pass "Embed API 1024-dim" || fail "Embed API (got dims=$DIMS)"

[[ -f "${ROOT}/data/vault.db" ]] && pass "vault.db exists" || fail "vault.db missing"

if [[ -x "${ROOT}/scripts/check-fts-sanity.sh" ]]; then
  "${ROOT}/scripts/check-fts-sanity.sh" >/dev/null 2>&1 && pass "honeypot FTS parity (no trg_honeypot_*)" || fail "honeypot FTS sanity"
fi

if [[ -x "$BIN" ]]; then
  (cd "$ROOT" && "$BIN" memory dump >/dev/null 2>&1) && pass "Vault export (memory dump)" || fail "Vault export"
fi

# Platform MCP server (gzmo mcp-serve) — tools/list smoke
if [[ -x "$BIN" ]]; then
  MCP_TOOLS="$(
    python3 - <<PY 2>/dev/null || true
import json, os, subprocess, sys

root = "${ROOT}"
bin_path = "${BIN}"
proc = subprocess.Popen(
    [bin_path, "mcp-serve"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True,
    env={**os.environ, "GZMO_CONFIG": f"{root}/gzmo.toml"},
)

def send(msg):
    proc.stdin.write(json.dumps(msg) + "\n")
    proc.stdin.flush()

send({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "verify", "version": "1.0"},
    },
})
proc.stdout.readline()
send({"jsonrpc": "2.0", "method": "notifications/initialized"})
send({"jsonrpc": "2.0", "id": 3, "method": "tools/list", "params": {}})

tools = []
for _ in range(10):
    line = proc.stdout.readline()
    if not line:
        break
    line = line.strip()
    if not line.startswith("{"):
        continue
    try:
        msg = json.loads(line)
    except json.JSONDecodeError:
        continue
    if msg.get("id") == 3 and "result" in msg:
        tools = [t.get("name", "") for t in msg["result"].get("tools", [])]
        break

proc.terminate()
print(",".join(tools))
PY
  )"
  if echo "$MCP_TOOLS" | grep -q 'gzmo_memory_search' \
     && echo "$MCP_TOOLS" | grep -q 'gzmo_memory_status' \
     && echo "$MCP_TOOLS" | grep -q 'gzmo_memory_recall_pull' \
     && echo "$MCP_TOOLS" | grep -q 'gzmo_retrieve_context'; then
    pass "gzmo mcp-serve tools/list (gzmo_memory_* + retrieve)"
  else
    fail "gzmo mcp-serve tools/list (got: ${MCP_TOOLS:-none})"
  fi
  if echo "$MCP_TOOLS" | grep -q 'gzmo_wiki_search'; then
    pass "gzmo mcp-serve tools/list (gzmo_wiki_search)"
  else
    fail "gzmo mcp-serve tools/list missing gzmo_wiki_search (got: ${MCP_TOOLS:-none})"
  fi
fi

# Cross-collection platform search (honeypot + knowledge when enabled)
PLATFORM_KB="$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
ps = d.get('platform_search', {})
print('1' if ps.get('include_knowledge_collection') else '0')
" 2>/dev/null || echo "0"
)"
if [[ -x "$BIN" && "$PLATFORM_KB" == "1" ]]; then
  SEARCH_JSON="$(cd "$ROOT" && "$BIN" memory search "GZMO infrastructure" --limit 3 --json --no-scratch 2>/dev/null || true)"
  if echo "$SEARCH_JSON" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    assert d.get('hits', 0) >= 0
    print('ok')
except Exception:
    sys.exit(1)
" 2>/dev/null; then
    pass "gzmo_memory_search JSON (platform cross-search path)"
  else
    fail "gzmo_memory_search JSON (platform cross-search)"
  fi
fi

if [[ "$QDRANT_ENABLED" == "1" ]]; then
  if python3 "${ROOT}/scripts/sync-vault-to-qdrant.py" --dry-run 2>/dev/null | grep -q 'facts.*with'; then
    pass "Qdrant sync dry-run (vault embeddings)"
  else
    fail "Qdrant sync dry-run"
  fi
fi

echo ""
if [[ "$FAIL" -eq 0 ]]; then
  echo "RESULT: PRODUCTION E2E OK (sovereign optional)"
  exit 0
else
  echo "RESULT: ONE OR MORE CHECKS FAILED"
  exit 1
fi
