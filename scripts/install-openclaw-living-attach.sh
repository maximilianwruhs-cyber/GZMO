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
bash "$ROOT/scripts/living-attach-check.sh"

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

# Patch TOOLS.md marker block (idempotent)
python3 - "$OC_WS/TOOLS.md" <<'PY'
from pathlib import Path
import sys
p = Path(sys.argv[1])
text = p.read_text(encoding="utf-8") if p.is_file() else ""
marker = "### Living memory (gzmo-living MCP)"
block = """### Living memory (gzmo-living MCP)

- **Search:** MCP server `gzmo-living` → `gzmo_memory_search` / `gzmo_memory_status` (CT101 vault, ~40k+ honeypot)
- **Takeaway write path:** `bash bin/openclaw-takeaway.sh '…'` (enqueue only; never `--now`)
- **Playbook:** `LIVING_ATTACH.md`
- **Never:** Qdrant upsert / Neo4j auto-graph from chat / start `gzmo-serve`

"""
if marker in text:
    # replace from marker through next ### or EOF-ish section
    import re
    text2 = re.sub(
        r"### Living memory \(gzmo-living MCP\).*?(?=\n### |\n## |\Z)",
        block,
        text,
        count=1,
        flags=re.S,
    )
    p.write_text(text2, encoding="utf-8")
else:
    p.write_text(text.rstrip() + "\n\n" + block, encoding="utf-8")
print(f"updated {p}")
PY

python3 - "$OC_WS/AGENTS.md" <<'PY'
from pathlib import Path
import sys
p = Path(sys.argv[1])
text = p.read_text(encoding="utf-8") if p.is_file() else ""
needle = "### Living memory / takeaway"
block = """### Living memory / takeaway

For “was weiß ich über X?” use MCP **`gzmo-living`** (`gzmo_memory_search`) — not grep-only and not raw Qdrant upsert.

After a meaningful durable insight: `bash bin/openclaw-takeaway.sh '…'` (Brain Feed enqueue on CT101).

See `LIVING_ATTACH.md`.

"""
if needle in text:
    import re
    text = re.sub(
        r"### Living memory / takeaway.*?(?=\n### |\n## |\Z)",
        block,
        text,
        count=1,
        flags=re.S,
    )
else:
    # insert after Session Startup cron section if present, else append
    if "### Cron / schedules" in text:
        text = text.replace("### Cron / schedules", block + "### Cron / schedules", 1)
    else:
        text = text.rstrip() + "\n\n" + block
p.write_text(text, encoding="utf-8")
print(f"updated {p}")
PY

echo
echo "[OK] OpenClaw living attach installed"
echo "     openclaw mcp show gzmo-living"
echo "     takeaway: bash $TAKEAWAY 'fact text'"
openclaw mcp show gzmo-living 2>&1 | head -30 || true
