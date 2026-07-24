#!/usr/bin/env bash
# Choose an OpenClaw persona from will-assistant/openclaw-agents WITHOUT wiping GZMO contract.
#
# Upstream install.sh copies SOUL.md + IDENTITY.md + AGENTS.md (destructive to AGENTS).
# This wrapper:
#   1) installs SOUL.md + IDENTITY.md only
#   2) saves pack AGENTS.md as CHARACTER.md (persona overlay, optional read)
#   3) re-runs sync-openclaw-workspace.sh (restores GZMO:ECOSYSTEM markers)
#   4) openclaw agents set-identity --from-identity
#
#   bash scripts/openclaw-choose-character.sh --list
#   bash scripts/openclaw-choose-character.sh --search pirate
#   bash scripts/openclaw-choose-character.sh glados
#   bash scripts/openclaw-choose-character.sh bob-ross --force
#
# Docs: docs/OPENCLAW_WORKSPACE_CONTRACT.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WS="${OPENCLAW_WORKSPACE:-$HOME/.openclaw/workspace}"
PACK_DIR="${OPENCLAW_AGENTS_DIR:-$HOME/github-clone/openclaw-agents}"
PACK_REPO="${OPENCLAW_AGENTS_REPO:-https://github.com/will-assistant/openclaw-agents.git}"
FORCE=0
QUERY=""
SEARCH_Q=""

usage() {
  cat <<'EOF'
Usage: bash scripts/openclaw-choose-character.sh [options] <agent>

Options:
  --list, -l           List pack agents
  --search, -s QUERY   Search pack
  --force, -f          Skip confirm
  --pack-dir DIR       Override clone path
  -h, --help

Examples:
  bash scripts/openclaw-choose-character.sh --list
  bash scripts/openclaw-choose-character.sh glados
  bash scripts/openclaw-choose-character.sh bob-ross --force
EOF
}

MODE=install
while [[ $# -gt 0 ]]; do
  case "$1" in
    --list|-l) MODE=list; shift ;;
    --search|-s) MODE=search; SEARCH_Q="${2:-}"; shift 2 ;;
    --force|-f) FORCE=1; shift ;;
    --pack-dir) PACK_DIR="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    --) shift; break ;;
    -*)
      echo "unknown flag: $1" >&2
      usage >&2
      exit 2
      ;;
    *) QUERY="$1"; shift ;;
  esac
done

ensure_pack() {
  if [[ ! -d "$PACK_DIR/agents" ]]; then
    echo "Cloning openclaw-agents → $PACK_DIR"
    mkdir -p "$(dirname "$PACK_DIR")"
    git clone --depth 1 "$PACK_REPO" "$PACK_DIR"
  fi
  [[ -x "$PACK_DIR/install.sh" ]] || chmod +x "$PACK_DIR/install.sh"
}

ensure_pack

if [[ "$MODE" == "list" ]]; then
  (cd "$PACK_DIR" && ./install.sh --list)
  exit 0
fi
if [[ "$MODE" == "search" ]]; then
  (cd "$PACK_DIR" && ./install.sh --search "${SEARCH_Q:-}")
  exit 0
fi
[[ -n "$QUERY" ]] || { usage >&2; exit 2; }

# Resolve agent dir (non-interactive fuzzy)
AGENT_DIR="$(python3 - "$PACK_DIR/agents" "$QUERY" <<'PY'
import sys
from pathlib import Path
agents = Path(sys.argv[1])
q = sys.argv[2].strip().lower().replace(" ", "-")
dirs = sorted([p for p in agents.glob("*/*") if p.is_dir()])
exact = [p for p in dirs if p.name.lower() == q]
if exact:
    print(exact[0]); raise SystemExit(0)
partial = [p for p in dirs if q in p.name.lower() or q in f"{p.parent.name}/{p.name}".lower()]
if len(partial) == 1:
    print(partial[0]); raise SystemExit(0)
if not partial:
    print(f"NO_MATCH:{sys.argv[2]}", file=sys.stderr); raise SystemExit(1)
print("MULTI:", file=sys.stderr)
for i, p in enumerate(partial[:20], 1):
    print(f"  [{i}] {p.parent.name}/{p.name}", file=sys.stderr)
print("Pick a unique slug, e.g. glados or bob-ross", file=sys.stderr)
raise SystemExit(2)
PY
)" || {
  echo "REFUSE: could not resolve agent '$QUERY'" >&2
  exit 1
}

slug="$(basename "$AGENT_DIR")"
cat_name="$(basename "$(dirname "$AGENT_DIR")")"
name="$slug"
emoji="🤖"
if [[ -f "$AGENT_DIR/metadata.json" ]]; then
  name="$(python3 -c "import json;print(json.load(open('$AGENT_DIR/metadata.json')).get('name','$slug'))")"
  emoji="$(python3 -c "import json;print(json.load(open('$AGENT_DIR/metadata.json')).get('emoji','🤖'))")"
fi

echo "Agent: $emoji $name  ($cat_name/$slug)"
echo "Target workspace: $WS"
if [[ ! -f "$AGENT_DIR/SOUL.md" && ! -f "$AGENT_DIR/IDENTITY.md" ]]; then
  echo "REFUSE: no SOUL.md/IDENTITY.md in $AGENT_DIR" >&2
  exit 1
fi

if [[ "$FORCE" -ne 1 ]]; then
  read -rp "Install persona (SOUL+IDENTITY only; keep GZMO AGENTS contract)? [Y/n] " confirm
  if [[ "${confirm,,}" == "n" ]]; then
    echo "Cancelled"
    exit 0
  fi
fi

mkdir -p "$WS"
# Backup previous persona
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
bak="$WS/.persona-bak/$stamp"
mkdir -p "$bak"
for f in SOUL.md IDENTITY.md CHARACTER.md; do
  [[ -f "$WS/$f" ]] && cp "$WS/$f" "$bak/$f"
done

[[ -f "$AGENT_DIR/SOUL.md" ]] && cp "$AGENT_DIR/SOUL.md" "$WS/SOUL.md" && echo "OK SOUL.md"
[[ -f "$AGENT_DIR/IDENTITY.md" ]] && cp "$AGENT_DIR/IDENTITY.md" "$WS/IDENTITY.md" && echo "OK IDENTITY.md"
# Do NOT overwrite AGENTS.md — store pack agent file as CHARACTER.md overlay
if [[ -f "$AGENT_DIR/AGENTS.md" ]]; then
  cp "$AGENT_DIR/AGENTS.md" "$WS/CHARACTER.md"
  echo "OK CHARACTER.md (pack AGENTS.md saved; workspace AGENTS.md kept)"
fi

# Record selection
python3 - "$WS" "$slug" "$cat_name" "$name" "$emoji" "$AGENT_DIR" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
ws = Path(sys.argv[1])
payload = {
  "schema": "gzmo.openclaw.character/v1",
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "slug": sys.argv[2],
  "category": sys.argv[3],
  "name": sys.argv[4],
  "emoji": sys.argv[5],
  "source_dir": sys.argv[6],
  "pack": "will-assistant/openclaw-agents",
  "notes": "SOUL+IDENTITY applied; AGENTS.md not overwritten; GZMO sync re-applied",
}
(ws / "CHARACTER.active.json").write_text(json.dumps(payload, indent=2) + "\n")
PY

# Re-inject GZMO ecosystem contract
bash "$ROOT/scripts/sync-openclaw-workspace.sh"

# Push identity into OpenClaw agent config
if command -v openclaw >/dev/null; then
  openclaw agents set-identity --workspace "$WS" --from-identity --json 2>&1 | tail -20 || true
fi

echo
echo "[OK] Persona $emoji $name installed with GZMO ecosystem intact."
echo "     Start a new OpenClaw session (/new) to load SOUL/IDENTITY."
echo "     Ecosystem: ECOSYSTEM.md · living attach unchanged."
echo "     Previous persona backup: $bak"
