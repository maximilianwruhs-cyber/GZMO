#!/usr/bin/env bash
# Post-reboot verification for workstation + VM200 retrieval router.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAIL=0
WARN=0

check() {
  local url=$1 label=$2
  if curl -sf --connect-timeout 3 --max-time 8 "${url}" >/dev/null 2>&1; then
    echo "  [OK] ${label}"
  else
    echo "  [FAIL] ${label} — ${url}" >&2
    FAIL=$((FAIL + 1))
  fi
}

warn_check() {
  local url=$1 label=$2
  if curl -sf --connect-timeout 3 --max-time 8 "${url}" >/dev/null 2>&1; then
    echo "  [OK] ${label}"
  else
    echo "  [WARN] ${label} — ${url}" >&2
    WARN=$((WARN + 1))
  fi
}

EMBED_MODEL="$(python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('embeddings', {}).get('model', 'gzmo-embed'))
" 2>/dev/null || echo 'gzmo-embed')"
RERANK_MODEL="$(python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
d = tomllib.loads(p.read_text())
print(d.get('rerank', {}).get('model', 'gzmo-rerank'))
" 2>/dev/null || echo 'gzmo-rerank')"

echo "=== Workstation services ==="
check "http://127.0.0.1:8000/v1/models" "Prime :8000"

check "http://192.168.31.110:8081/v1/models" "VM200 retrieval router :8081"

dims="$(curl -sf "http://192.168.31.110:8081/v1/embeddings" \
  -H 'Content-Type: application/json' \
  -d "{\"model\":\"${EMBED_MODEL}\",\"input\":\"boot probe\"}" \
  | python3 -c "import sys,json; print(len(json.load(sys.stdin)['data'][0]['embedding']))" 2>/dev/null || echo 0)"
if [[ "${dims}" == "1024" ]]; then
  echo "  [OK] VM200 embed ${dims}-dim (${EMBED_MODEL})"
else
  echo "  [FAIL] VM200 embed dims=${dims} (expected 1024)" >&2
  FAIL=$((FAIL + 1))
fi

if curl -sf "http://192.168.31.110:8081/v1/rerank" \
  -H 'Content-Type: application/json' \
  -d "{\"model\":\"${RERANK_MODEL}\",\"query\":\"capital of France\",\"top_n\":1,\"documents\":[\"Paris is the capital of France.\",\"Berlin is in Germany.\"]}" \
  | python3 -c "import sys,json; r=json.load(sys.stdin)['results'][0]; s=r.get('relevance_score', r.get('score',0)); assert abs(float(s))>1e-6" 2>/dev/null; then
  echo "  [OK] VM200 rerank (${RERANK_MODEL})"
else
  echo "  [FAIL] VM200 rerank — ${RERANK_MODEL}" >&2
  FAIL=$((FAIL + 1))
fi

check "http://192.168.31.202:6333/collections/honeypot" "Qdrant honeypot"

echo ""
echo "=== systemd (user) ==="
for u in gzmo-prime gzmo-daemon; do
  st=$(systemctl --user is-active "${u}.service" 2>/dev/null || echo inactive)
  en=$(systemctl --user is-enabled "${u}.service" 2>/dev/null || echo disabled)
  echo "  ${u}.service: ${st} (enabled=${en})"
done

echo ""
echo "=== GZMO health ==="
if [[ -x "${ROOT}/target/release/gzmo" ]]; then
  (cd "${ROOT}" && "${ROOT}/target/release/gzmo" health) || FAIL=$((FAIL + 1))
else
  echo "  [SKIP] gzmo binary not built"
fi

echo ""
if [[ "${FAIL}" -eq 0 ]]; then
  echo "Boot check passed (${WARN} warning(s))."
  echo "Pi KB incremental sync: ${ROOT}/scripts/pi-kb-reindex.sh"
  FUNC_HEALTH="${HOME}/gzmo_skills/scripts/functional-health-check.sh"
  SEED_OPS="${ROOT}/scripts/seed-ops-graph.py"
  NEO4J_PY="${NEO4J_PYTHON:-$HOME/Projects/mcp-neo4j-memory-gzmo/.venv/bin/python}"
  if [[ -f "$SEED_OPS" ]] && [[ -x "$NEO4J_PY" ]]; then
    echo ""
    echo "=== Ops graph seed (non-blocking) ==="
    set +e
    NEO4J_PASSWORD="${NEO4J_PASSWORD:-Easycheesy0815!}" "$NEO4J_PY" "$SEED_OPS" --gzmo-root "$ROOT" \
      || echo "  [WARN] seed-ops-graph failed (see above)"
    set -e
  fi
  if [[ -x "$FUNC_HEALTH" ]]; then
    echo ""
    echo "=== Functional health (non-blocking) ==="
    set +e
    "$FUNC_HEALTH" || echo "  [WARN] functional-health-check failed (see above)"
    set -e
  fi
else
  echo "${FAIL} check(s) failed, ${WARN} warning(s). See docs/REBOOT_STARTUP.md"
  exit 1
fi
