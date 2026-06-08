#!/usr/bin/env bash
# Align Cursor, global MCP, and Pi with GZMO shared MCP servers (Neo4j + gzmo-memory).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRAG="${ROOT}/config/shared-mcp-memory.json"
CURSOR_MCP="${HOME}/.cursor/mcp.json"
PI_MCP="${HOME}/.pi/agent/mcp.json"
GLOBAL_MCP="${HOME}/.config/mcp/mcp.json"

if [[ ! -f "${FRAG}" ]]; then
  echo "[!] Missing ${FRAG}" >&2
  exit 1
fi

# Ensure release binary exists for gzmo-memory server
if [[ ! -x "${ROOT}/target/release/gzmo" ]]; then
  echo "[*] Building gzmo (release) for gzmo-memory MCP server…" >&2
  (cd "${ROOT}" && cargo build --release -p gzmo-cli -q)
fi

python3 - <<PY
import json, pathlib, shutil

root = pathlib.Path("${ROOT}")
frag = json.loads(pathlib.Path("${FRAG}").read_text())

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

merge_servers(pathlib.Path("${CURSOR_MCP}"), "Cursor MCP")
merge_servers(pathlib.Path("${PI_MCP}"), "Pi MCP")
merge_servers(pathlib.Path("${GLOBAL_MCP}"), "Global shared MCP")

print("")
print("Servers installed:")
for name in frag.get("mcpServers", {}):
    print(f"  - {name}")
print("")
print("GZMO daemon uses gzmo.toml [[mcp_servers]] for Neo4j (client mode).")
print("Pi/Cursor use merged mcp.json — gzmo-memory exposes gzmo_memory_* tools via stdio.")
PY
