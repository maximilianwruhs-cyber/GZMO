#!/usr/bin/env bash
# Wire OpenClaw → living GZMO memory MCP + takeaway helper (no dual-writer).
#
#   bash scripts/install-openclaw-living-attach.sh
#   bash scripts/install-openclaw-living-attach.sh --no-probe
#
# Docs: docs/EXTERNAL_LIVING_ATTACH.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WRAPPER="$ROOT/scripts/pi-gzmo-mcp-serve.sh"
TAKEAWAY="$ROOT/scripts/openclaw-takeaway.sh"
OC_WS="${OPENCLAW_WORKSPACE:-$HOME/.openclaw/workspace}"
PROBE=1
[[ "${1:-}" == "--no-probe" ]] && PROBE=0

if [[ "${GZMO_PRODUCT:-}" == "1" || "${GZMO_ALLOW_LAB_VAULT:-}" == "1" ]]; then
  echo "REFUSE: PRODUCT/LAB-ALLOW set — will not wire living OpenClaw attach" >&2
  exit 1
fi

command -v openclaw >/dev/null || {
  echo "REFUSE: openclaw CLI not on PATH" >&2
  exit 1
}

chmod +x "$WRAPPER" "$TAKEAWAY" \
  "$ROOT/scripts/herdr-living-enqueue.sh" \
  "$ROOT/scripts/living-attach-check.sh" 2>/dev/null || true

echo "=== living-attach-check ==="
if [[ "$PROBE" -eq 1 ]]; then
  bash "$ROOT/scripts/living-attach-check.sh"
else
  bash "$ROOT/scripts/living-attach-check.sh" || true
fi

echo "=== tools.deny cron (llama.cpp schema hygiene) ==="
# OpenClaw cron tool schemas use unanchored pattern "\\S"; llama.cpp rejects
# those with "Pattern must start with '^' and end with '$'" and Telegram gets
# the generic "Something went wrong" reply. Jobs still run via gateway cron CLI.
# Docs: config/openclaw-workspace/GZMO_ECOSYSTEM_CRON.md
openclaw config set tools.deny '["cron"]' --strict-json

echo "=== openclaw mcp set gzmo-living ==="
# Stdio SSH bridge — same contract as Cursor/Pi shared MCP
JSON=$(python3 - "$WRAPPER" <<'PY'
import json, sys
print(json.dumps({
  "command": sys.argv[1],
  "args": [],
  "env": {"GZMO_LIVING": "1"},
}))
PY
)
if [[ "$PROBE" -eq 1 ]]; then
  openclaw mcp set gzmo-living "$JSON"
  openclaw mcp configure gzmo-living --timeout 90 --connect-timeout 30 \
    --exclude 'gzmo_ops_health,gzmo_discovery_status' || true
  openclaw mcp probe gzmo-living --json 2>&1 | tail -60 || {
    echo "[!] probe soft-fail — server saved; check: openclaw mcp show gzmo-living" >&2
  }
else
  openclaw mcp add gzmo-living --command "$WRAPPER" --env GZMO_LIVING=1 --no-probe \
    --timeout 90 --connect-timeout 30 \
    --exclude 'gzmo_ops_health,gzmo_discovery_status'
fi

echo "=== workspace playbooks ==="
mkdir -p "$OC_WS/bin"
install -m 755 "$ROOT/scripts/openclaw-takeaway.sh" "$OC_WS/bin/openclaw-takeaway.sh"
# Stable symlink into repo for agent exec from GZMO cwd too
ln -sfn "$ROOT/scripts/openclaw-takeaway.sh" "$OC_WS/bin/openclaw-takeaway-repo.sh"

cat >"$OC_WS/LIVING_ATTACH.md" <<EOF
# OpenClaw ↔ GZMO living attach

**Server:** \`gzmo-living\` (MCP) via \`$WRAPPER\`  
**Search:** tools \`gzmo_memory_search\` / \`gzmo_memory_status\` / \`gzmo_wiki_search\`  
**Write nutrient:** \`bash bin/openclaw-takeaway.sh 'durable fact'\` → CT101 distill queue (**no --now**)

## NEVER

- curl upsert into Qdrant \`honeypot\`
- raw Neo4j writes for chat lore
- \`systemctl --user start gzmo-serve\` while CT101 lives
- \`GZMO_PRODUCT=1\` / \`GZMO_ALLOW_LAB_VAULT=1\` on this bridge

## Prove

\`\`\`bash
bash $ROOT/scripts/living-attach-check.sh
openclaw mcp show gzmo-living
openclaw mcp probe
\`\`\`
EOF

echo
echo "[OK] OpenClaw living attach installed"
echo "     syncing workspace contract…"
bash "$ROOT/scripts/sync-openclaw-workspace.sh" || true
echo "     openclaw mcp show gzmo-living"
echo "     takeaway: bash $TAKEAWAY 'fact text'"
openclaw mcp show gzmo-living 2>&1 | head -30 || true
