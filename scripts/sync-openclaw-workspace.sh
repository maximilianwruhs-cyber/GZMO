#!/usr/bin/env bash
# Sync GZMO ecosystem contract into ~/.openclaw/workspace (operator surface).
# Regenerates generated files; patches <!-- GZMO:ECOSYSTEM:BEGIN/END --> blocks.
# Never starts gzmo-serve. Never touches memory/*.md or MEMORY.md.
#
#   bash scripts/sync-openclaw-workspace.sh
#   OPENCLAW_WORKSPACE=~/.openclaw/workspace bash scripts/sync-openclaw-workspace.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/config/openclaw-workspace"
WS="${OPENCLAW_WORKSPACE:-$HOME/.openclaw/workspace}"
BEGIN="<!-- GZMO:ECOSYSTEM:BEGIN -->"
END="<!-- GZMO:ECOSYSTEM:END -->"

[[ -d "$SRC" ]] || { echo "REFUSE: missing $SRC" >&2; exit 1; }
mkdir -p "$WS/bin" "$WS/memory"

chmod +x \
  "$ROOT/scripts/openclaw-takeaway.sh" \
  "$ROOT/scripts/pi-gzmo-mcp-serve.sh" \
  "$WS/bin/list-gzmo-crons.sh" 2>/dev/null || true

# Helpers
install -m 755 "$ROOT/scripts/openclaw-takeaway.sh" "$WS/bin/openclaw-takeaway.sh"
ln -sfn "$ROOT/scripts/openclaw-takeaway.sh" "$WS/bin/openclaw-takeaway-repo.sh"
if [[ ! -x "$WS/bin/list-gzmo-crons.sh" ]]; then
  cat >"$WS/bin/list-gzmo-crons.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "=== OpenClaw cron (gateway) ==="
openclaw cron list 2>&1 || true
echo
echo "=== OpenClaw cron status ==="
openclaw cron status 2>&1 || true
echo
echo "=== Workstation systemd user timers (gzmo/okforge) ==="
systemctl --user list-timers --all 2>&1 | rg -i 'gzmo-|okforge' || true
echo
echo "=== Playbook ==="
echo "CRON_JOBS.md · GZMO_ECOSYSTEM_CRON.md · ECOSYSTEM.md"
EOF
  chmod +x "$WS/bin/list-gzmo-crons.sh"
fi

# Full generated copies
cp "$SRC/ECOSYSTEM.md" "$WS/ECOSYSTEM.md"
cp "$SRC/LIVING_ATTACH.md" "$WS/LIVING_ATTACH.md"
cp "$SRC/GZMO_ECOSYSTEM_CRON.md" "$WS/GZMO_ECOSYSTEM_CRON.md"

# Workspace skills (Telegram slash commands)
if [[ -d "$SRC/skills" ]]; then
  mkdir -p "$WS/skills"
  cp -a "$SRC/skills/." "$WS/skills/"
  # Ensure skill helpers are executable (cp -a may preserve mode; force for run.sh)
  find "$WS/skills" -type f -name 'run.sh' -exec chmod +x {} +
fi

# PATH-friendly alias for lean models / shell: gzmo-character <args>
mkdir -p "$HOME/.local/bin"
ln -sfn "$ROOT/scripts/openclaw-choose-character.sh" "$HOME/.local/bin/gzmo-character"
ln -sfn "$WS/skills/character/run.sh" "$WS/bin/character" 2>/dev/null || true
if [[ -f "$WS/bin/character" ]]; then
  chmod +x "$WS/bin/character"
fi

# Ensure hybrid files exist
[[ -f "$WS/AGENTS.md" ]] || cp "$HOME/.openclaw/workspace/AGENTS.md" "$WS/AGENTS.md" 2>/dev/null || touch "$WS/AGENTS.md"
for f in SOUL.md IDENTITY.md USER.md TOOLS.md; do
  [[ -f "$WS/$f" ]] || touch "$WS/$f"
done
[[ -f "$WS/TOOLS.local.md" ]] || cat >"$WS/TOOLS.local.md" <<'EOF'
# TOOLS.local.md — personal / non-ecosystem notes

Synced TOOLS.md ecosystem block is overwritten by `sync-openclaw-workspace.sh`.
Put cameras, TTS, nicknames, and one-off host aliases here.
EOF

# Patch marker regions
python3 - "$WS" "$SRC" "$BEGIN" "$END" <<'PY'
import re, sys
from pathlib import Path

ws, src, begin, end = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], sys.argv[4]

def load_block(name: str) -> str:
    raw = (src / name).read_text(encoding="utf-8")
    raw = raw.replace(begin, "").replace(end, "")
    return raw.strip() + "\n"

def patch(path: Path, block: str, insert_after: str | None = None) -> None:
    text = path.read_text(encoding="utf-8") if path.is_file() else ""
    chunk = f"{begin}\n{block.rstrip()}\n{end}\n"
    # Drop every existing ecosystem marker region (handles orphans / nesting)
    text = re.sub(
        re.escape(begin) + r".*?" + re.escape(end) + r"\s*",
        "",
        text,
        flags=re.S,
    )
    text = text.replace(begin, "").replace(end, "")
    if insert_after and insert_after in text:
        text = text.replace(insert_after, insert_after + "\n\n" + chunk, 1)
    else:
        text = text.rstrip() + "\n\n" + chunk
    path.write_text(text if text.endswith("\n") else text + "\n", encoding="utf-8")
    print(f"patched {path.name}")

patch(ws / "AGENTS.md", load_block("AGENTS.ecosystem.md"), insert_after="## Session Startup")
patch(ws / "TOOLS.md", load_block("TOOLS.ecosystem.md"))
patch(ws / "SOUL.md", load_block("SOUL.ecosystem.md"))
patch(ws / "IDENTITY.md", load_block("IDENTITY.ecosystem.md"))
patch(ws / "USER.md", load_block("USER.ecosystem.md"))

# Deduplicate old hand-pasted TOOLS sections outside markers (best-effort)
tools = (ws / "TOOLS.md").read_text(encoding="utf-8")
# If duplicate CT101 headers exist outside markers, leave them — operator can clean once.
(ws / "TOOLS.md").write_text(tools, encoding="utf-8")
PY

# CRON_JOBS.md live snapshot
if command -v openclaw >/dev/null 2>&1; then
  timeout 15 openclaw cron list --json >"$WS/.cron-jobs.json" 2>/dev/null || echo '{"jobs":[]}' >"$WS/.cron-jobs.json"
else
  echo '{"jobs":[]}' >"$WS/.cron-jobs.json"
fi

python3 - "$WS" <<'PY'
import json, subprocess
from datetime import datetime, timezone
from pathlib import Path

ws = Path(__import__("sys").argv[1])
jobs = []
try:
    jobs = json.loads((ws / ".cron-jobs.json").read_text()).get("jobs") or []
except Exception:
    jobs = []

timers = ""
try:
    timers = subprocess.check_output(
        ["bash", "-lc", "systemctl --user list-timers --all 2>/dev/null | rg -i 'gzmo-|okforge' || true"],
        text=True,
    )
except Exception as e:
    timers = f"(timers unavailable: {e})"

lines = [
    "# CRON_JOBS.md — live snapshot for OpenClaw agent",
    "",
    f"generated_at: {datetime.now(timezone.utc).isoformat()}",
    "",
    "**Important:** OpenClaw `cron` *tool* is DENIED. Jobs still exist — use exec:",
    "",
    "```bash",
    "bash bin/list-gzmo-crons.sh",
    "openclaw cron list",
    "```",
    "",
    "Also read `ECOSYSTEM.md` + `GZMO_ECOSYSTEM_CRON.md`.",
    "",
    f"## OpenClaw gateway jobs ({len(jobs)})",
    "",
]
for j in jobs:
    sched = j.get("schedule") or {}
    expr = sched.get("expr") or sched.get("kind")
    tz = sched.get("tz") or ""
    payload = j.get("payload") or {}
    lines += [
        f"### {j.get('name')}",
        f"- id: `{j.get('id')}`",
        f"- declaration: `{j.get('declarationKey')}`",
        f"- schedule: `{expr}` {tz}".rstrip(),
        f"- enabled: {j.get('enabled')}",
        f"- payload: {payload.get('kind')}",
        f"- delivery: {(j.get('delivery') or {}).get('mode')}",
        "",
    ]
lines += ["## Workstation systemd timers (gzmo/okforge)", "", "```", timers.rstrip() or "(none)", "```", ""]
(ws / "CRON_JOBS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
(ws / ".cron-jobs.json").unlink(missing_ok=True)
print(f"wrote CRON_JOBS.md ({len(jobs)} openclaw jobs)")
PY

# HEARTBEAT: keep comments-only unless HEARTBEAT_ENABLE=1
if [[ "${HEARTBEAT_ENABLE:-0}" != "1" ]]; then
  cat >"$WS/HEARTBEAT.md" <<'EOF'
<!-- Heartbeat template; comments-only content prevents scheduled heartbeat API calls. -->
<!-- Ecosystem digests use OpenClaw cron + systemd timers — see ECOSYSTEM.md / CRON_JOBS.md -->
# Keep this file empty (or with only comments) to skip heartbeat API calls.
EOF
fi

# Pointer file for agents
cat >"$WS/README.md" <<EOF
# OpenClaw workspace (GZMO-aligned)

| File | Owner |
|------|-------|
| \`ECOSYSTEM.md\` | **synced** — start here |
| \`LIVING_ATTACH.md\` / \`CRON_JOBS.md\` / \`GZMO_ECOSYSTEM_CRON.md\` | **synced** |
| \`AGENTS.md\` / \`TOOLS.md\` / \`SOUL.md\` / \`IDENTITY.md\` / \`USER.md\` | hybrid (markers synced) |
| \`TOOLS.local.md\` / \`memory/\` / \`MEMORY.md\` | local only |
| \`HEARTBEAT.md\` | comments-only by default |

Sync: \`bash $ROOT/scripts/sync-openclaw-workspace.sh\`  
Contract: \`$ROOT/docs/OPENCLAW_WORKSPACE_CONTRACT.md\`
EOF

echo "[OK] synced OpenClaw workspace → $WS"
ls -1 "$WS"/*.md | sed 's|.*/||' | sort
