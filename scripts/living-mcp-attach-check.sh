#!/usr/bin/env bash
# Goal C: verify Cursor/Pi have labeled **gzmo-living** (not product gzmo-memory).
# Soft by default — HOLD when missing. FAIL only if living is mislabeled as gzmo-memory.
#
#   bash scripts/living-mcp-attach-check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/living-mcp-attach"
WRAPPER="$ROOT/scripts/pi-gzmo-mcp-serve.sh"
mkdir -p "$OUT"

export OUT WRAPPER
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

home = Path.home()
out = Path(os.environ["OUT"])
wrapper = str(Path(os.environ["WRAPPER"]).resolve())
targets = [
    ("Cursor", home / ".cursor" / "mcp.json"),
    ("Pi", home / ".pi" / "agent" / "mcp.json"),
    ("Global", home / ".config" / "mcp" / "mcp.json"),
]

checks = []
issues = []
fail = 0
hold = 0
found_living = 0

for label, path in targets:
    entry = {
        "label": label,
        "path": str(path),
        "present": path.is_file(),
        "gzmo_living": False,
        "gzmo_memory_is_living": False,
        "ok": True,
        "problems": [],
    }
    if not path.is_file():
        if label in ("Cursor", "Pi"):
            entry["ok"] = False
            entry["problems"].append("missing_mcp_json")
            hold += 1
            issues.append(f"{label}:missing_mcp_json")
        checks.append(entry)
        continue
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as e:
        entry["ok"] = False
        entry["problems"].append(f"parse:{e}")
        fail += 1
        issues.append(f"{label}:parse")
        checks.append(entry)
        continue
    servers = data.get("mcpServers") or {}
    living = servers.get("gzmo-living")
    gm = servers.get("gzmo-memory") or {}
    gm_cmd = str((gm or {}).get("command") or "").replace("\\", "/")
    if living:
        found_living += 1
        entry["gzmo_living"] = True
        cmd = str(living.get("command") or "").replace("\\", "/")
        if "pi-gzmo-mcp-serve" not in cmd and "/opt/gzmo" not in cmd:
            entry["problems"].append("gzmo-living_command_unexpected")
            entry["ok"] = False
            hold += 1
            issues.append(f"{label}:gzmo-living_command_unexpected")
        if (living.get("env") or {}).get("GZMO_PRODUCT") == "1":
            entry["problems"].append("gzmo-living_has_GZMO_PRODUCT")
            entry["ok"] = False
            fail += 1
            issues.append(f"{label}:gzmo-living_has_GZMO_PRODUCT")
    if "pi-gzmo-mcp-serve" in gm_cmd or (
        gm_cmd.endswith("/gzmo") and "mcp-serve" in " ".join(gm.get("args") or [])
    ):
        # Heuristic: product name wired to living bridge
        if "pi-gzmo-mcp-serve" in gm_cmd:
            entry["gzmo_memory_is_living"] = True
            entry["problems"].append("gzmo-memory_is_living_mislabel")
            entry["ok"] = False
            fail += 1
            issues.append(f"{label}:gzmo-memory_is_living_mislabel")
    checks.append(entry)

if found_living == 0:
    hold += 1
    issues.append("no_gzmo-living_in_any_mcp_json")

ok = fail == 0
if fail:
    advice = "living_mcp_fail — run bash scripts/install-shared-mcp.sh (gzmo-living)"
elif found_living == 0:
    advice = "living_mcp_hold — no gzmo-living yet; install-shared-mcp.sh when ready"
    ok = True  # soft: missing living attach is HOLD, not RED for gate consumers
else:
    advice = "living_mcp_ok — gzmo-living labeled; product gzmo-memory not hijacked"

payload = {
    "schema": "gzmo.living.mcp-attach/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "fail": fail,
    "hold": hold,
    "found_living": found_living,
    "advice": advice,
    "wrapper": wrapper,
    "checks": checks,
    "issues": issues,
    "goal": "C",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    f"# Living MCP attach\n\n**Advice:** {advice}\n\n"
    f"- found_living: {found_living}\n"
    f"- fail: {fail} hold: {hold}\n",
    encoding="utf-8",
)
print(json.dumps({"ok": ok, "advice": advice, "found_living": found_living, "issues": issues}, indent=2))
raise SystemExit(0 if ok else 1)
PY
