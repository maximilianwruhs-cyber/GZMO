#!/usr/bin/env bash
# Production stack: Prime (:8000) + VM200 retrieval router (:8081) + gzmo health/daemon.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LLAMA="${GZMO_LLAMA_ROOT:-${HOME}/Projects/llama.cpp}"
PRIME_START="${LLAMA}/prime-bench/start-prime-gemma4-26b-a4b-256k.sh"
LOG_DIR="${ROOT}/logs"
mkdir -p "${LOG_DIR}"

gzmo_bin() {
  if [[ -x "${ROOT}/target/release/gzmo" ]]; then
    echo "${ROOT}/target/release/gzmo"
  elif [[ -x "${ROOT}/target/debug/gzmo" ]]; then
    echo "${ROOT}/target/debug/gzmo"
  else
    echo "[*] Building gzmo-cli (release)…" >&2
    (cd "${ROOT}" && cargo build -p gzmo-cli --release)
    echo "${ROOT}/target/release/gzmo"
  fi
}

wait_url() {
  local url=$1 label=$2 max=${3:-180}
  for ((i = 1; i <= max; i++)); do
    if curl -sf "${url}" >/dev/null 2>&1; then
      echo "[OK] ${label} ready (${i}s)"
      return 0
    fi
    sleep 2
  done
  echo "[FAIL] ${label} not ready: ${url}" >&2
  return 1
}

if ! curl -sf "http://127.0.0.1:8000/v1/models" >/dev/null 2>&1; then
  echo "[*] Starting Prime on :8000…"
  nohup "${PRIME_START}" >>"${LOG_DIR}/prime.log" 2>&1 &
  wait_url "http://127.0.0.1:8000/v1/models" "Prime"
else
  echo "[OK] Prime already listening on :8000"
fi

# GZMO daemon embeddings (gzmo.toml — VM200 :8081)
GZMO_EMBED_URL="${GZMO_EMBED_URL:-$(
  python3 -c "
import tomllib, pathlib
p = pathlib.Path('${ROOT}/gzmo.toml')
print(tomllib.loads(p.read_text()).get('embeddings', {}).get('url', 'http://192.168.31.110:8081/v1').rstrip('/'))
" 2>/dev/null || echo 'http://192.168.31.110:8081/v1'
)}"
GZMO_EMBED_HEALTH="${GZMO_EMBED_URL%/v1}/models"
if curl -sf "${GZMO_EMBED_HEALTH}" >/dev/null 2>&1; then
  echo "[OK] GZMO embed reachable (${GZMO_EMBED_URL})"
else
  echo "[!] GZMO embed not reachable: ${GZMO_EMBED_URL}" >&2
  echo "    VM200: ./scripts/vm200/deploy-retrieval-router.sh" >&2
fi

BIN="$(gzmo_bin)"
echo "[*] Running health probes…"
(cd "${ROOT}" && "${BIN}" health) || true

if [[ "${1:-}" == "--daemon" ]]; then
  PID_FILE="/tmp/gzmo_daemon.pid"
  if [[ -f "${PID_FILE}" ]] && kill -0 "$(cat "${PID_FILE}")" 2>/dev/null; then
    echo "[OK] GZMO daemon already running (PID $(cat "${PID_FILE}"))"
  else
    echo "[*] Starting GZMO daemon…"
    nohup "${BIN}" daemon >>"${LOG_DIR}/daemon.log" 2>&1 &
    disown
    for _ in $(seq 1 40); do
      if [[ -f "${PID_FILE}" ]] && kill -0 "$(cat "${PID_FILE}")" 2>/dev/null; then
        echo "[OK] Daemon PID $(cat "${PID_FILE}") — log: ${LOG_DIR}/daemon.log"
        break
      fi
      sleep 0.25
    done
    if [[ ! -f "${PID_FILE}" ]]; then
      echo "[!] Daemon started but ${PID_FILE} not written yet — check ${LOG_DIR}/daemon.log" >&2
    fi
  fi
fi

echo ""
echo "Production stack:"
echo "  Prime   http://127.0.0.1:8000/v1  (chat / dreams / spark / distill)"
echo "  GZMO    ${GZMO_EMBED_URL}  (daemon embed + rerank; VM200 router)"
echo "  Pi KB   ${GZMO_EMBED_URL%/v1}/embeddings  (knowledge_search / pi-kb-reindex.sh)"
echo "  Boot    ./scripts/install-boot-stack.sh  |  verify: ./scripts/after-boot-verify.sh"
echo "  Pi sync ./scripts/pi-kb-reindex.sh"
