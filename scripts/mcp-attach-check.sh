#!/usr/bin/env bash
# Prime product: verify Cursor/Pi MCP attach points at ~/.gzmo product (not CT101/lab).
#
#   bash scripts/mcp-attach-check.sh
#   MCP_ATTACH_FIX=1 bash scripts/mcp-attach-check.sh  # re-merge via install-product-mcp
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/mcp-attach"
mkdir -p "$OUT"

export ROOT DATA OUT
python3 - <<'PY'
import json, os, shutil, subprocess
from datetime import datetime, timezone
from pathlib import Path

home = Path.home()
out = Path(os.environ["OUT"])
root = Path(os.environ["ROOT"])
now = datetime.now(timezone.utc).isoformat()
product_cfg = home / ".gzmo" / "gzmo.toml"
local_bin = home / ".local" / "bin" / "gzmo"

targets = [
    ("Cursor", home / ".cursor" / "mcp.json"),
    ("Pi", home / ".pi" / "agent" / "mcp.json"),
    ("Global", home / ".config" / "mcp" / "mcp.json"),
    ("Fragment", home / ".gzmo" / "mcp.json"),
]

def load_gm(path: Path):
    if not path.is_file():
        return None, "missing"
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as e:
        return None, f"parse_error:{e}"
    servers = data.get("mcpServers") or {}
    gm = servers.get("gzmo-memory") or servers.get("gzmo")
    if not gm:
        return None, "no_gzmo-memory"
    return gm, "ok"

checks = []
issues = []
for label, path in targets:
    gm, status = load_gm(path)
    entry = {
        "label": label,
        "path": str(path),
        "present": path.is_file(),
        "status": status,
        "command": (gm or {}).get("command") if gm else None,
        "config": ((gm or {}).get("env") or {}).get("GZMO_CONFIG") if gm else None,
        "product": ((gm or {}).get("env") or {}).get("GZMO_PRODUCT") if gm else None,
        "allow_lab": ((gm or {}).get("env") or {}).get("GZMO_ALLOW_LAB_VAULT") if gm else None,
        "ok": False,
        "problems": [],
    }
    if gm:
        cmd = Path(gm.get("command") or "")
        cfg = (gm.get("env") or {}).get("GZMO_CONFIG") or ""
        problems = []
        if not cmd.is_file() and not shutil.which(str(cmd)):
            problems.append("command_missing")
        if "/opt/gzmo" in cfg or "ct101" in cfg.lower():
            problems.append("points_at_ct101")
        if "data-next" in cfg:
            problems.append("points_at_lab_data-next")
        cfg_ok = False
        if cfg:
            try:
                cfg_ok = Path(cfg).expanduser().resolve() == product_cfg.resolve()
            except Exception:
                cfg_ok = ".gzmo" in cfg.replace("\\", "/") and cfg.endswith("gzmo.toml")
        if not cfg_ok:
            problems.append("config_not_product_home")
        if (gm.get("env") or {}).get("GZMO_PRODUCT") != "1":
            problems.append("GZMO_PRODUCT_not_1")
        # Prefer ~/.local/bin when temp-bench binary is wired but local install exists
        if local_bin.is_file() and "temp-bench" in str(cmd):
            problems.append("prefer_local_bin_over_temp-bench")
        entry["problems"] = problems
        hard = (
            "command_missing",
            "points_at_ct101",
            "points_at_lab_data-next",
            "config_not_product_home",
        )
        entry["ok"] = status == "ok" and not any(p in problems for p in hard)
        if problems:
            issues.extend(f"{label}:{p}" for p in problems)
    elif label in ("Cursor", "Pi", "Fragment"):
        issues.append(f"{label}:{status}")
    checks.append(entry)

# Prefer Cursor+Pi+Fragment green for attach_ok
core = [c for c in checks if c["label"] in ("Cursor", "Pi", "Fragment")]
attach_ok = all(c.get("ok") for c in core if c["label"] != "Pi" or c["present"])
# Pi optional if missing entirely
if not any(c["label"] == "Pi" and c["present"] for c in checks):
    attach_ok = all(c.get("ok") for c in checks if c["label"] in ("Cursor", "Fragment"))

advice = (
    "attach_ok — Cursor/Pi product MCP points at ~/.gzmo"
    if attach_ok and not any("prefer_local_bin" in i for i in issues)
    else (
        "attach_ok_with_hints — wired; consider ~/.local/bin/gzmo instead of temp-bench"
        if attach_ok
        else "hold — fix MCP wiring (see problems); run scripts/install-product-mcp.sh"
    )
)

payload = {
    "schema": "gzmo.mcp.attach-check/v1",
    "generated_at": now,
    "ok": attach_ok,
    "advice": advice,
    "product_config": str(product_cfg),
    "product_config_present": product_cfg.is_file(),
    "local_bin": str(local_bin) if local_bin.is_file() else None,
    "checks": checks,
    "issues": issues,
    "fix": "MCP_ATTACH_FIX=1 bash scripts/mcp-attach-check.sh",
    "note": "Prime product attach gate — living CT101 MCP is a separate operator surface.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
lines = [
    "# MCP attach check",
    "",
    f"Advice: **{advice}**",
    "",
]
for c in checks:
    mark = "PASS" if c.get("ok") else ("…" if not c["present"] else "HOLD")
    lines.append(f"- {c['label']}: **{mark}** `{c['path']}`")
    if c.get("config"):
        lines.append(f"  - config: `{c['config']}`")
    if c.get("problems"):
        lines.append(f"  - problems: {', '.join(c['problems'])}")
lines += ["", payload["note"], ""]
(out / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({"ok": attach_ok, "advice": advice, "issues": issues[:12]}, indent=2))
PY

if [[ "${MCP_ATTACH_FIX:-0}" == "1" ]]; then
  echo "=== MCP_ATTACH_FIX: re-merge product MCP ==="
  if [[ -x "$ROOT/scripts/install-product-mcp.sh" ]]; then
    # Prefer product installer binary; ignore a leftover GZMO_BIN=temp-bench in the shell.
    if [[ -x "${HOME}/.local/bin/gzmo" ]]; then
      GZMO_BIN="${HOME}/.local/bin/gzmo" bash "$ROOT/scripts/install-product-mcp.sh" || true
    else
      env -u GZMO_BIN bash "$ROOT/scripts/install-product-mcp.sh" || true
    fi
  else
    echo "install-product-mcp.sh missing" >&2
  fi
  # Re-run check after fix
  MCP_ATTACH_FIX=0 bash "$ROOT/scripts/mcp-attach-check.sh"
  exit 0
fi
exit 0
