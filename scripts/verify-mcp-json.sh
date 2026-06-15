#!/usr/bin/env bash
# Fail if repo .mcp.json is missing or empty (Pi cwd = survey_GZMO).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 - <<PY
import json, pathlib, sys
p = pathlib.Path("${ROOT}/.mcp.json")
if not p.is_file() or p.stat().st_size == 0:
    print(f"[FAIL] {p} missing or empty — run ./scripts/install-shared-mcp.sh", file=sys.stderr)
    sys.exit(1)
data = json.loads(p.read_text())
servers = data.get("mcpServers") or {}
if not servers:
    print(f"[FAIL] {p} has no mcpServers — run ./scripts/install-shared-mcp.sh", file=sys.stderr)
    sys.exit(1)
print(f"[OK] {p} ({len(servers)} servers: {', '.join(servers)})")
PY
