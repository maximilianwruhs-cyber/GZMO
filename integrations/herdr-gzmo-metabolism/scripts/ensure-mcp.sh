#!/usr/bin/env bash
# Wire product gzmo-memory into Cursor/Pi/global mcp.json (herdr agents share those).
set -euo pipefail
# shellcheck disable=SC1091
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib.sh"
export_gzmo_env

INSTALL="${REPO_ROOT}/scripts/install-product-mcp.sh"
if [[ ! -x "$INSTALL" ]]; then
  echo "[!] missing $INSTALL" >&2
  exit 1
fi

echo "[*] Ensuring product MCP (agents in herdr panes use Cursor/Pi mcp.json)"
GZMO_BIN="$GZMO_BIN" GZMO_HOME="${GZMO_HOME:-${HOME}/.gzmo}" bash "$INSTALL"

# Prove product fragment points at this binary (do not invoke interactive `gzmo memory`).
FRAG="${GZMO_HOME:-${HOME}/.gzmo}/mcp.json"
if [[ -f "$FRAG" ]] && rg -q 'mcp-serve|"memory mcp"' "$FRAG" 2>/dev/null; then
  echo "[OK] product mcp.json wires mcp-serve"
elif [[ -f "${HOME}/.cursor/mcp.json" ]] && rg -q 'gzmo-memory' "${HOME}/.cursor/mcp.json"; then
  echo "[OK] Cursor mcp.json has gzmo-memory"
else
  echo "[!] gzmo-memory MCP fragment missing after install" >&2
  exit 1
fi

export STATUS_JSON="$STATE_DIR/mcp-ensure-latest.json"
export GZMO_HOME="${GZMO_HOME:-${HOME}/.gzmo}"
python3 - <<'PY'
import json, os, time
path = os.environ["STATUS_JSON"]
payload = {
    "ok": True,
    "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "gzmo_bin": os.environ.get("GZMO_BIN"),
    "product_home": os.environ.get("GZMO_HOME", os.path.expanduser("~/.gzmo")),
    "herdr_plugin": os.environ.get("HERDR_PLUGIN_ID", "gzmo.metabolism"),
}
open(path, "w", encoding="utf-8").write(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
