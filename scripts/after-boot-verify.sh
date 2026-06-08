#!/usr/bin/env bash
# Post-reboot verification for workstation + pointers for VM200 / Pi KB.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAIL=0

check() {
  local url=$1 label=$2
  if curl -sf --connect-timeout 3 --max-time 8 "${url}" >/dev/null 2>&1; then
    echo "  [OK] ${label}"
  else
    echo "  [FAIL] ${label} — ${url}" >&2
    FAIL=$((FAIL + 1))
  fi
}

echo "=== Workstation services ==="
check "http://127.0.0.1:8000/v1/models" "Prime :8000"
check "http://127.0.0.1:8002/v1/models" "Pi KB embed :8002"
check "http://192.168.31.110:8081/v1/models" "GZMO embed VM200 :8081"
check "http://192.168.31.110:8082/v1/health" "Rerank VM200 :8082"
check "http://192.168.31.110:8083/v1/models" "Librarian VM200 :8083"
check "http://192.168.31.202:6333/collections/knowledge" "Qdrant knowledge"
check "http://192.168.31.202:6333/collections/honeypot" "Qdrant honeypot"

echo ""
echo "=== systemd (user) ==="
for u in gzmo-embed gzmo-prime gzmo-daemon; do
  st=$(systemctl --user is-active "${u}.service" 2>/dev/null || echo inactive)
  echo "  ${u}.service: ${st}"
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
  echo "Boot check passed."
  echo "Pi KB incremental sync: ${ROOT}/scripts/pi-kb-reindex.sh"
else
  echo "${FAIL} check(s) failed. See docs/REBOOT_STARTUP.md"
  exit 1
fi
