#!/usr/bin/env bash
# Prime product: stranger path for Memory MCP appliance (install → verify → next call).
# Does not require CT101. Writes a demable checklist artifact.
#
#   bash scripts/product-stranger-path.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/product-stranger"
BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
mkdir -p "$OUT"

HOME_GZMO="${HOME}/.gzmo"
HAS_HOME_GZMO=0
[[ -f "$HOME_GZMO/gzmo.toml" ]] && HAS_HOME_GZMO=1

PATH_BIN="$(command -v gzmo 2>/dev/null || true)"
LOCAL_BIN=""
[[ -x "${HOME}/.local/bin/gzmo" ]] && LOCAL_BIN="${HOME}/.local/bin/gzmo"

VERIFY_OK=0
VERIFY_NOTE="skipped"
VERIFY_LOG="$OUT/verify-product-mcp.log"
if [[ -x "$BIN" ]]; then
  if KEEP_VERIFY_DIR=1 VERIFY_DIR="$OUT/product-verify" \
    GZMO_BIN="$BIN" bash "$ROOT/scripts/verify-product-mcp.sh" >"$VERIFY_LOG" 2>&1; then
    VERIFY_OK=1
    VERIFY_NOTE="verify-product-mcp PASS"
  else
    VERIFY_NOTE="verify-product-mcp FAIL (see verify-product-mcp.log)"
  fi
else
  VERIFY_NOTE="no release/dev gzmo binary for cold verify"
fi

# Optional: status against real ~/.gzmo if present (does not fail stranger path).
HOME_STATUS=""
if [[ "$HAS_HOME_GZMO" == "1" && -n "${PATH_BIN:-$LOCAL_BIN}" ]]; then
  GBIN="${PATH_BIN:-$LOCAL_BIN}"
  HOME_STATUS="$(
    GZMO_CONFIG="$HOME_GZMO/gzmo.toml" GZMO_ALLOW_LAB_VAULT=1 GZMO_PRODUCT=1 \
      "$GBIN" memory status --json 2>/dev/null || true
  )"
fi

export OUT BIN PATH_BIN LOCAL_BIN HAS_HOME_GZMO VERIFY_OK VERIFY_NOTE HOME_STATUS HOME_GZMO ROOT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()
verify_ok = os.environ.get("VERIFY_OK") == "1"
has_home = os.environ.get("HAS_HOME_GZMO") == "1"
home_status_raw = os.environ.get("HOME_STATUS") or ""
home_status = None
if home_status_raw.strip():
    try:
        home_status = json.loads(home_status_raw)
    except Exception:
        home_status = {"raw": home_status_raw[:200]}

checklist = [
    {
        "step": 1,
        "title": "Install binary + ~/.gzmo",
        "cmd": "curl -fsSL https://raw.githubusercontent.com/maximilianwruhs-cyber/GZMO/main/scripts/install-gzmo.sh | bash",
        "ok_hint": "gzmo on PATH or ~/.local/bin; ~/.gzmo/gzmo.toml exists",
        "observed_home_gzmo": has_home,
        "observed_path_bin": os.environ.get("PATH_BIN") or None,
        "observed_local_bin": os.environ.get("LOCAL_BIN") or None,
    },
    {
        "step": 2,
        "title": "Cold-path product verify",
        "cmd": "./scripts/verify-product-mcp.sh",
        "ok_hint": "init → memory status/search → mcp-serve tools",
        "observed_ok": verify_ok,
        "note": os.environ.get("VERIFY_NOTE"),
    },
    {
        "step": 3,
        "title": "Attach in Cursor / Pi",
        "cmd": "call gzmo_memory_status then gzmo_memory_search",
        "ok_hint": "agent sees vault_path under ~/.gzmo (product) not CT101",
        "docs": "docs/PRODUCT_MCP.md",
    },
]

stranger_ok = verify_ok
advice = (
    "stranger_ready — cold verify PASS; next attach MCP in Cursor/Pi"
    if stranger_ok
    else "hold — fix verify-product-mcp before claiming install UX"
)

payload = {
    "schema": "gzmo.product.stranger-path/v1",
    "generated_at": now,
    "ok": stranger_ok,
    "advice": advice,
    "checklist": checklist,
    "home_gzmo": {
        "present": has_home,
        "path": os.environ.get("HOME_GZMO"),
        "status": home_status,
    },
    "docs": [
        "README.md",
        "docs/PRODUCT_MCP.md",
        "docs/SPINE_FOCUS.md",
    ],
    "note": "Prime product UX — laptop Memory MCP in <10 minutes; CT101 is separate living brain.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
lines = [
    "# Product stranger path",
    "",
    f"Advice: **{advice}**",
    "",
    "## Checklist",
    "",
]
for c in checklist:
    mark = "PASS" if c.get("observed_ok") or c.get("observed_home_gzmo") else "…"
    if c["step"] == 3:
        mark = "manual"
    if c["step"] == 1:
        mark = "PASS" if has_home or os.environ.get("PATH_BIN") or os.environ.get("LOCAL_BIN") else "…"
    lines.append(f"{c['step']}. **{c['title']}** [{mark}]")
    lines.append(f"   `{c['cmd']}`")
    lines.append("")
lines += [payload["note"], ""]
(out / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({"ok": stranger_ok, "advice": advice, "verify_ok": verify_ok, "home_gzmo": has_home}, indent=2))
PY
exit 0
