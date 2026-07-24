#!/usr/bin/env bash
# Install GZMO-safe Telegram /character (skill + plugin tool + lean allowlist).
# Never starts gzmo-serve. Docs: docs/OPENCLAW_WORKSPACE_CONTRACT.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLUGIN="$ROOT/config/openclaw-plugins/gzmo-character"
CFG="${OPENCLAW_CONFIG:-$HOME/.openclaw/openclaw.json}"

[[ -d "$PLUGIN" ]] || { echo "REFUSE: missing $PLUGIN" >&2; exit 1; }
[[ -f "$CFG" ]] || { echo "REFUSE: missing $CFG" >&2; exit 1; }

bash "$ROOT/scripts/sync-openclaw-workspace.sh"

(
  cd "$PLUGIN"
  npm install --no-fund --no-audit
  openclaw plugins build --entry ./src/index.ts --root "$PLUGIN" >/dev/null
)

openclaw plugins install --link "$PLUGIN" || true
openclaw plugins enable gzmo-character

python3 - "$CFG" "$PLUGIN" <<'PY'
import json, sys
from pathlib import Path

cfg_path, plugin = Path(sys.argv[1]), str(Path(sys.argv[2]).resolve())
cfg = json.loads(cfg_path.read_text())

tg = cfg.setdefault("channels", {}).setdefault("telegram", {})
tg.setdefault("commands", {})["native"] = True
tg["commands"]["nativeSkills"] = True
cc = tg.get("customCommands") or []
tg["customCommands"] = [
    c for c in cc if c.get("command") not in {"character", "characters"}
]
if not tg["customCommands"]:
    tg.pop("customCommands", None)

cfg.setdefault("commands", {})["native"] = True
cfg["commands"]["nativeSkills"] = True

defaults = cfg.setdefault("agents", {}).setdefault("defaults", {})
skills = list(defaults.get("skills") or [])
if "character" not in skills:
    skills = ["character", *skills]
defaults["skills"] = skills

plugins = cfg.setdefault("plugins", {})
plugins.setdefault("entries", {}).setdefault("gzmo-character", {})["enabled"] = True
paths = plugins.setdefault("load", {}).setdefault("paths", [])
paths[:] = [p for p in paths if Path(p).resolve() != Path(plugin).resolve()]
paths.append(plugin)

cfg_path.write_text(json.dumps(cfg, indent=2) + "\n")
print("[OK] patched", cfg_path)
print("[OK] plugin path", plugin)
PY

systemctl --user restart openclaw-gateway.service
sleep 2
systemctl --user is-active openclaw-gateway.service
openclaw plugins list 2>&1 | rg -i 'gzmo-character|Character' | head -5 || true
openclaw skills info character 2>&1 | head -15 || true
echo "[OK] Try Telegram: /character who"
