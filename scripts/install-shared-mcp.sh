#!/usr/bin/env bash
# Operator path: Neo4j + living MCP as **gzmo-living** (LAN / CT101 topology).
# Does not overwrite product **gzmo-memory** (~/.gzmo). Goal A vs C labels.
# Outsiders / laptop product MCP: use scripts/install-product-mcp.sh instead.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAG="${ROOT}/config/shared-mcp-memory.json"
CURSOR_MCP="${HOME}/.cursor/mcp.json"
PI_MCP="${HOME}/.pi/agent/mcp.json"
GLOBAL_MCP="${HOME}/.config/mcp/mcp.json"
WRAPPER="${ROOT}/scripts/pi-gzmo-mcp-serve.sh"
PRODUCT_FRAG="${HOME}/.gzmo/mcp.json"

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
  for f in "${ROOT}/.env" "${HOME}/.gzmo-vault/.env" "${ROOT}/deploy/living-appliance/.env"; do
    if [[ -f "$f" ]]; then
      v="$(grep -E '^NEO4J_PASSWORD=' "$f" | head -1 | cut -d= -f2- | tr -d '"' || true)"
      if [[ -n "$v" ]]; then
        printf '%s' "$v"
        return
      fi
      # Also accept NEO4J_AUTH=neo4j/password form from appliance .env
      v="$(grep -E '^NEO4J_AUTH=' "$f" | head -1 | cut -d= -f2- | tr -d '"' || true)"
      if [[ "$v" == neo4j/* ]]; then
        printf '%s' "${v#neo4j/}"
        return
      fi
    fi
  done
  v="$(ssh -o ConnectTimeout=8 -o BatchMode=yes ct101 \
    "grep -E '^NEO4J_PASSWORD=' /opt/gzmo/.env 2>/dev/null | head -1 | cut -d= -f2-" 2>/dev/null | tr -d '"' || true)"
  if [[ -n "$v" ]]; then
    printf '%s' "$v"
    return
  fi
  echo "[!] NEO4J_PASSWORD not set. Export it or put it in /opt/gzmo/.env on CT101." >&2
  exit 1
}

export NEO4J_PASSWORD
NEO4J_PASSWORD=$(resolve_neo4j_password)
export GZMO_ROOT="$ROOT"
export FRAG CURSOR_MCP PI_MCP GLOBAL_MCP PRODUCT_FRAG WRAPPER

python3 <<'PY'
import json, os, pathlib

root = pathlib.Path(os.environ["GZMO_ROOT"])
frag = json.loads(pathlib.Path(os.environ["FRAG"]).read_text())
password = os.environ["NEO4J_PASSWORD"]
placeholder = "${NEO4J_PASSWORD}"
wrapper = str(root / "scripts" / "pi-gzmo-mcp-serve.sh")

for name, cfg in frag.get("mcpServers", {}).items():
    env = cfg.get("env") or {}
    for k, v in list(env.items()):
        if isinstance(v, str) and placeholder in v:
            env[k] = v.replace(placeholder, password)
    cfg["env"] = env
    if name == "gzmo-living":
        cfg["command"] = wrapper
        cfg["args"] = []
        cfg.setdefault("env", {})["GZMO_LIVING"] = "1"

def is_living_command(cmd: str) -> bool:
    c = (cmd or "").replace("\\", "/")
    return "pi-gzmo-mcp-serve" in c or "/opt/gzmo" in c

def merge_servers(target: pathlib.Path, label: str) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        cur = json.loads(target.read_text())
    else:
        cur = {"mcpServers": {}}
    servers = cur.setdefault("mcpServers", {})

    # Migrate mislabeled living attach off product name
    old = servers.get("gzmo-memory")
    if isinstance(old, dict) and is_living_command(str(old.get("command") or "")):
        servers["gzmo-living"] = old
        del servers["gzmo-memory"]
        print(f"[*] {label}: renamed living gzmo-memory → gzmo-living")

    for name, cfg in frag.get("mcpServers", {}).items():
        servers[name] = cfg

    # Restore product gzmo-memory from ~/.gzmo if present and living took the name
    product_frag = pathlib.Path(os.environ["PRODUCT_FRAG"])
    if "gzmo-memory" not in servers and product_frag.is_file():
        try:
            pf = json.loads(product_frag.read_text())
            gm = (pf.get("mcpServers") or {}).get("gzmo-memory")
            if gm:
                servers["gzmo-memory"] = gm
                print(f"[*] {label}: restored product gzmo-memory from ~/.gzmo")
        except Exception as e:
            print(f"[!] {label}: could not restore product mcp: {e}")

    target.write_text(json.dumps(cur, indent=2) + "\n")
    print(f"[OK] {label} → {target}")

merge_servers(pathlib.Path(os.environ["CURSOR_MCP"]), "Cursor MCP")
merge_servers(pathlib.Path(os.environ["PI_MCP"]), "Pi MCP")
merge_servers(pathlib.Path(os.environ["GLOBAL_MCP"]), "Global shared MCP")

print("")
print("Servers (goal C living):")
print("  - gzmo-living → ssh ct101 → /opt/gzmo/.../gzmo mcp-serve")
print("  - memory      → Neo4j bolt on CT101")
print("Product goal A stays on server name: gzmo-memory (~/.gzmo)")
print("Neo4j password resolved from env/.env (not stored in repo fragment).")
PY
