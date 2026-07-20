#!/usr/bin/env bash
# Post-reboot verification for workstation + pointers for VM200 / Pi KB.
# Matches current boot reality: llama-prime.service owns :8000; gzmo-* units are optional.
#
#   bash scripts/after-boot-verify.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAIL=0
HOLD=0

ok() { echo "  [OK] $1"; }
fail() { echo "  [FAIL] $1 — $2" >&2; FAIL=$((FAIL + 1)); }
hold() { echo "  [HOLD] $1 — $2"; HOLD=$((HOLD + 1)); }

check_url() {
  local url=$1 label=$2 mode=${3:-require} # require|soft
  if curl -sf --connect-timeout 3 --max-time 8 "${url}" >/dev/null 2>&1; then
    ok "${label}"
  elif [[ "$mode" == soft ]]; then
    hold "${label}" "${url}"
  else
    fail "${label}" "${url}"
  fi
}

echo "=== Workstation services ==="
check_url "http://127.0.0.1:8000/v1/models" "Prime :8000"
check_url "http://127.0.0.1:8002/v1/models" "Pi KB embed :8002" soft
check_url "http://192.168.31.110:8081/v1/models" "GZMO embed VM200 :8081"
check_url "http://192.168.31.110:8082/v1/health" "Rerank VM200 :8082" soft
check_url "http://192.168.31.110:8083/v1/models" "Librarian VM200 :8083" soft
check_url "http://192.168.31.202:6333/collections/knowledge" "Qdrant knowledge"
check_url "http://192.168.31.202:6333/collections/honeypot" "Qdrant honeypot"

echo ""
echo "=== systemd (user) ==="
# Canonical Prime unit on this workstation
if systemctl --user cat llama-prime.service >/dev/null 2>&1; then
  st=$(systemctl --user is-active llama-prime.service 2>/dev/null || echo inactive)
  en=$(systemctl --user is-enabled llama-prime.service 2>/dev/null || echo disabled)
  if [[ "$st" == active ]]; then
    ok "llama-prime.service active (enabled=${en})"
  elif curl -sf --connect-timeout 2 --max-time 4 http://127.0.0.1:8000/v1/models >/dev/null 2>&1; then
    hold "llama-prime.service" "inactive but :8000 responds (manual llama-server?)"
  else
    fail "llama-prime.service" "inactive and :8000 down — systemctl --user start llama-prime"
  fi
else
  hold "llama-prime.service" "unit not installed — :8000 URL check is source of truth"
fi

# Optional install-boot-stack units (may be absent)
for u in gzmo-embed gzmo-prime gzmo-daemon; do
  if systemctl --user cat "${u}.service" >/dev/null 2>&1; then
    st=$(systemctl --user is-active "${u}.service" 2>/dev/null || echo inactive)
    echo "  [info] ${u}.service: ${st}"
  else
    echo "  [info] ${u}.service: not installed (optional — see scripts/install-boot-stack.sh)"
  fi
done

echo ""
echo "=== GZMO binary / health ==="
GZMO_BIN=""
for cand in \
  "${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo" \
  "$HOME/.local/bin/gzmo" \
  "$ROOT/target/release/gzmo" \
  "$ROOT/target/debug/gzmo"; do
  if [[ -x "$cand" ]]; then GZMO_BIN="$cand"; break; fi
done

if [[ -n "$GZMO_BIN" ]]; then
  ok "gzmo binary ${GZMO_BIN}"
  # Soft: lab MCP memory may be unset after reboot; HTTP stack is the boot gate.
  if (cd "$ROOT" && "$GZMO_BIN" health) >/tmp/after-boot-gzmo-health.log 2>&1; then
    ok "gzmo health"
  else
    hold "gzmo health" "see /tmp/after-boot-gzmo-health.log (often mcp_memory after cold boot)"
  fi
else
  hold "gzmo binary" "not found — build or install-product-mcp / cargo release"
fi

echo ""
echo "=== Living pointer (CT101) ==="
if ssh -o ConnectTimeout=3 -o BatchMode=yes ct101 'systemctl is-active gzmo-daemon' >/tmp/after-boot-ct101.txt 2>&1; then
  if grep -qx active /tmp/after-boot-ct101.txt; then
    ok "CT101 gzmo-daemon active"
  else
    hold "CT101 gzmo-daemon" "$(tr '\n' ' ' </tmp/after-boot-ct101.txt)"
  fi
else
  hold "CT101 ssh" "unreachable from this host — check living separately"
fi

echo ""
if [[ "${FAIL}" -eq 0 ]]; then
  echo "Boot check passed (FAIL=0 HOLD=${HOLD})."
  echo "Pi KB incremental sync: ${ROOT}/scripts/pi-kb-reindex.sh"
  echo "Docs: ${ROOT}/docs/REBOOT_STARTUP.md"
  exit 0
else
  echo "${FAIL} check(s) failed (HOLD=${HOLD}). See docs/REBOOT_STARTUP.md"
  exit 1
fi
