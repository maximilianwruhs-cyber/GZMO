#!/usr/bin/env bash
# Install product gzmo-memory MCP into Cursor / Pi / global mcp.json.
# Uses ~/.gzmo from `gzmo init` — no LAN hosts, no Neo4j, no living topology.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GZMO_HOME="${GZMO_HOME:-${HOME}/.gzmo}"
FRAG_SRC="${GZMO_HOME}/mcp.json"
CURSOR_MCP="${HOME}/.cursor/mcp.json"
PI_MCP="${HOME}/.pi/agent/mcp.json"
GLOBAL_MCP="${HOME}/.config/mcp/mcp.json"

resolve_bin() {
  if [[ -n "${GZMO_BIN:-}" ]]; then
    printf '%s' "$GZMO_BIN"
    return
  fi
  # Product attach prefers the installer path over a temp-bench / clone build.
  if [[ -x "${HOME}/.local/bin/gzmo" ]]; then
    printf '%s' "${HOME}/.local/bin/gzmo"
    return
  fi
  if command -v gzmo >/dev/null 2>&1; then
    command -v gzmo
    return
  fi
  if [[ -x "${ROOT}/target/release/gzmo" ]]; then
    printf '%s' "${ROOT}/target/release/gzmo"
    return
  fi
  if [[ -x "${CARGO_TARGET_DIR:-}/release/gzmo" ]]; then
    printf '%s' "${CARGO_TARGET_DIR}/release/gzmo"
    return
  fi
  if [[ -x "${ROOT}/target/debug/gzmo" ]]; then
    printf '%s' "${ROOT}/target/debug/gzmo"
    return
  fi
  echo "[!] No gzmo binary found. Build with: cargo build --release -p gzmo-cli" >&2
  echo "    Or set GZMO_BIN=/path/to/gzmo" >&2
  exit 1
}

GZMO_BIN="$(resolve_bin)"

if [[ ! -f "${GZMO_HOME}/gzmo.toml" ]]; then
  echo "[*] No ${GZMO_HOME}/gzmo.toml — running: ${GZMO_BIN} init --bin ${GZMO_BIN}"
  "${GZMO_BIN}" init --bin "${GZMO_BIN}" --dir "${GZMO_HOME}"
fi

if [[ ! -f "${FRAG_SRC}" ]]; then
  echo "[!] Missing ${FRAG_SRC} after init" >&2
  exit 1
fi

export FRAG_SRC CURSOR_MCP PI_MCP GLOBAL_MCP GZMO_BIN GZMO_HOME

python3 <<'PY'
import json, os, pathlib

frag = json.loads(pathlib.Path(os.environ["FRAG_SRC"]).read_text())
bin_path = os.environ["GZMO_BIN"]
home = pathlib.Path(os.environ["GZMO_HOME"])
cfg = str(home / "gzmo.toml")

# Normalize fragment to this machine's binary + config.
servers = frag.setdefault("mcpServers", {})
gm = servers.setdefault("gzmo-memory", {})
gm["command"] = bin_path
gm["args"] = ["mcp-serve"]
env = gm.setdefault("env", {})
env["GZMO_CONFIG"] = cfg
env["GZMO_ALLOW_LAB_VAULT"] = "1"
env["GZMO_PRODUCT"] = "1"
# Product path: never inject ops tools by default.
env.pop("GZMO_OPS_MCP", None)

pathlib.Path(os.environ["FRAG_SRC"]).write_text(json.dumps(frag, indent=2) + "\n")

def merge_servers(target: pathlib.Path, label: str) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists():
        cur = json.loads(target.read_text())
    else:
        cur = {"mcpServers": {}}
    cur.setdefault("mcpServers", {})
    cur["mcpServers"]["gzmo-memory"] = gm
    target.write_text(json.dumps(cur, indent=2) + "\n")
    print(f"[OK] {label} → {target}")

merge_servers(pathlib.Path(os.environ["CURSOR_MCP"]), "Cursor MCP")
merge_servers(pathlib.Path(os.environ["PI_MCP"]), "Pi MCP")
merge_servers(pathlib.Path(os.environ["GLOBAL_MCP"]), "Global shared MCP")

print("")
print("Product MCP installed (local SQLite vault):")
print(f"  binary: {bin_path}")
print(f"  config: {cfg}")
print("  tools:  gzmo_memory_status / search / recall / chain (ops gated)")
print("")
print("Verify in Cursor/Pi: gzmo_memory_status")
PY
