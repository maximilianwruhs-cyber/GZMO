#!/usr/bin/env bash
# Align Cursor, global MCP, and Pi with GZMO shared MCP servers (Neo4j + gzmo-memory on CT101).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAG="${ROOT}/config/shared-mcp-memory.json"
CURSOR_MCP="${HOME}/.cursor/mcp.json"
PI_MCP="${HOME}/.pi/agent/mcp.json"
GLOBAL_MCP="${HOME}/.config/mcp/mcp.json"
WRAPPER="${ROOT}/scripts/pi-gzmo-mcp-serve.sh"

if [[ ! -f "${FRAG}" ]]; then
  echo "[!] Missing ${FRAG}" >&2
  exit 1
fi

chmod +x "${WRAPPER}" "${ROOT}/scripts/ct101-living-smoke.sh" 2>/dev/null || true

resolve_neo4j_password() {
  if [[ -n "${NEO4J_PASSWORD:-}" ]]; then
    printf '%s' "$NEO4J_PASSWORD"
    return
  fi
  local f v
  for f in "${ROOT}/.env" "${HOME}/.gzmo-vault/.env"; do
    if [[ -f "$f" ]]; then
      v="$(grep -E '^NEO4J_PASSWORD=' "$f" | head -1 | cut -d= -f2- | tr -d '"' || true)"
      if [[ -n "$v" ]]; then
        printf '%s' "$v"
        return
      fi
    fi
  done
  v="$(ssh -o ConnectTimeout=8 -o BatchMode=yes ct101 \
    'grep -E "^NEO4J_PASSWORD=" /opt/gzmo/.env 2>/dev/null | head -1 | cut -d= -f2-' 2>/dev/null | tr -d '"' || true)"
  if [[ -n "$v" ]]; then
    printf '%s' "$v"
    return
  fi
  echo "[!] NEO4J_PASSWORD not set. Export it or put it in /opt/gzmo/.env on CT101." >&2
  exit 1
}

export NEO4J_PASSWORD
NEO4J_PASSWORD="$(resolve_neo4j_password)"
export GZMO_ROOT="$ROOT"
export FRAG CURSOR_MCP PI_MCP GLOBAL_MCP

python3 <<'PY'
import json, os, pathlib

root = pathlib.Path(os.environ["GZMO_ROOT"])
frag = json.loads(pathlib.Path(os.environ["FRAG"]).read_text())
password = os.environ["NEO4J_PASSWORD"]
placeholder = "${NEO4J_PASSWORD}"

for name, cfg in frag.get("mcpServers", {}).items():
    env = cfg.get("env") or {}
    for k, v in list(env.items()):
        if isinstance(v, str) and placeholder in v:
            env[k] = v.replace(placeholder, password)
    cfg["env"] = env
    if name == "gzmo-memory":
        cfg["command"] = str(root / "scripts" / "pi-gzmo-mcp-serve.sh")
        cfg["args"] = []

def merge_servers(target: pathlib.Path, label: str) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        cur = json.loads(target.read_text())
    else:
        cur = {"mcpServers": {}}
    cur.setdefault("mcpServers", {})
    for name, cfg in frag.get("mcpServers", {}).items():
        cur["mcpServers"][name] = cfg
    target.write_text(json.dumps(cur, indent=2) + "\n")
    print(f"[OK] {label} → {target}")

merge_servers(pathlib.Path(os.environ["CURSOR_MCP"]), "Cursor MCP")
merge_servers(pathlib.Path(os.environ["PI_MCP"]), "Pi MCP")
merge_servers(pathlib.Path(os.environ["GLOBAL_MCP"]), "Global shared MCP")

print("")
print("Servers installed:")
for name in frag.get("mcpServers", {}):
    print(f"  - {name}")
print("")
print("gzmo-memory → ssh ct101 → /opt/gzmo/current/.../gzmo mcp-serve (living vault)")
print("Neo4j password resolved from env/.env (not stored in repo fragment).")
PY
