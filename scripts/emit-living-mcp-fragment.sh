#!/usr/bin/env bash
# Emit host-agnostic living MCP fragment (JSON or Hermes YAML).
# Does NOT merge into ~/.hermes or start writers — stdout / optional --out only.
#
#   bash scripts/emit-living-mcp-fragment.sh --format hermes
#   bash scripts/emit-living-mcp-fragment.sh --format json --mode local
#   bash scripts/emit-living-mcp-fragment.sh --format both --out docs/examples/
#
# Docs: docs/EXTERNAL_LIVING_ATTACH.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORMAT="hermes" # json | hermes | both
MODE="${GZMO_ATTACH_MODE:-ssh}" # ssh | local
OUT=""
OPS_MCP="${GZMO_OPS_MCP:-}"

usage() {
  cat <<'EOF'
Usage: bash scripts/emit-living-mcp-fragment.sh [options]

Options:
  --format json|hermes|both   Output dialect (default: hermes)
  --mode ssh|local            Ops SSH wrapper vs on-box mcp-serve (default: ssh)
  --out DIR                   Write example files under DIR (does not touch ~/.hermes)
  --ops                       Include GZMO_OPS_MCP=1 on the living server env

Prints a single gzmo-living stanza. Never emits gzmo-memory. Never starts gzmo-serve.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --format) FORMAT="${2:-}"; shift 2 ;;
    --mode) MODE="${2:-}"; shift 2 ;;
    --out) OUT="${2:-}"; shift 2 ;;
    --ops) OPS_MCP=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "REFUSE: unknown arg: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

case "$FORMAT" in
  json|hermes|both) ;;
  *) echo "REFUSE: --format must be json|hermes|both" >&2; exit 1 ;;
esac
case "$MODE" in
  ssh|local) ;;
  *) echo "REFUSE: --mode must be ssh|local" >&2; exit 1 ;;
esac

# Fail closed on living/env conflict before emitting a tempting stanza
if [[ "${GZMO_PRODUCT:-}" == "1" ]]; then
  echo "REFUSE: GZMO_PRODUCT=1 set — will not emit living fragment (lite vs living conflict)" >&2
  exit 1
fi
if [[ "${GZMO_ALLOW_LAB_VAULT:-}" == "1" ]]; then
  echo "REFUSE: GZMO_ALLOW_LAB_VAULT=1 set — will not emit living fragment" >&2
  exit 1
fi

WRAPPER="$ROOT/scripts/pi-gzmo-mcp-serve.sh"
LOCAL_BIN="${GZMO_BIN:-/usr/local/bin/gzmo}"
LOCAL_CFG="${GZMO_CONFIG:-/opt/gzmo/gzmo.toml}"

export FORMAT MODE OUT OPS_MCP WRAPPER LOCAL_BIN LOCAL_CFG ROOT
python3 - <<'PY'
import json, os, sys
from pathlib import Path

fmt = os.environ["FORMAT"]
mode = os.environ["MODE"]
out = os.environ.get("OUT") or ""
ops = os.environ.get("OPS_MCP") == "1"
wrapper = os.environ["WRAPPER"]
local_bin = os.environ["LOCAL_BIN"]
local_cfg = os.environ["LOCAL_CFG"]

if mode == "ssh":
    server = {
        "command": wrapper,
        "args": [],
        "env": {"GZMO_LIVING": "1"},
    }
else:
    server = {
        "command": local_bin,
        "args": ["mcp-serve"],
        "env": {"GZMO_CONFIG": local_cfg},
    }

if ops:
    server["env"]["GZMO_OPS_MCP"] = "1"

# Explicitly omit forbidden keys even if present in parent env
for bad in ("GZMO_PRODUCT", "GZMO_ALLOW_LAB_VAULT"):
    server["env"].pop(bad, None)

payload = {"mcpServers": {"gzmo-living": server}}

def hermes_yaml(cfg: dict) -> str:
    env = cfg.get("env") or {}
    lines = [
        "# Hermes / generic YAML — paste under mcp_servers (label MUST be gzmo-living).",
        "# Do NOT put this under gzmo-memory. Do NOT set GZMO_PRODUCT or GZMO_ALLOW_LAB_VAULT.",
        "# Prove with: bash scripts/living-attach-check.sh",
        "# Repo example only — this emitter does not rewrite ~/.hermes.",
        "mcp_servers:",
        "  gzmo-living:",
        f"    command: {json.dumps(cfg['command'])}",
        "    args: []" if not cfg.get("args") else "    args:",
    ]
    for a in cfg.get("args") or []:
        lines.append(f"      - {json.dumps(a)}")
    if env:
        lines.append("    env:")
        for k, v in env.items():
            lines.append(f"      {k}: {json.dumps(v)}")
    lines.append("")
    return "\n".join(lines)

outputs = []
if fmt in ("json", "both"):
    outputs.append(("json", json.dumps(payload, indent=2) + "\n"))
if fmt in ("hermes", "both"):
    outputs.append(("hermes", hermes_yaml(server)))

if out:
    dest = Path(out)
    dest.mkdir(parents=True, exist_ok=True)
    for kind, text in outputs:
        if kind == "json":
            path = dest / "gzmo-living.mcp.json"
        else:
            path = dest / "hermes-gzmo-living.yaml"
        path.write_text(text, encoding="utf-8")
        print(f"[wrote] {path}", file=sys.stderr)
else:
    for _, text in outputs:
        sys.stdout.write(text)
        if not text.endswith("\n"):
            sys.stdout.write("\n")
PY
