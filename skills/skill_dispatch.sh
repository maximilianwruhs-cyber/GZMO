#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# skill_dispatch.sh — GZMO Skill Router
# ═══════════════════════════════════════════════════════════════════
# Usage: ./skills/skill_dispatch.sh <command> [args...]
# Example: ./skills/skill_dispatch.sh dice D20
#          ./skills/skill_dispatch.sh joke
#          ./skills/skill_dispatch.sh transform Batman
# ═══════════════════════════════════════════════════════════════════

set -euo pipefail

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
COMMAND="${1:-help}"
shift 2>/dev/null || true

# Normalize: strip leading slash if present
COMMAND="${COMMAND#/}"
COMMAND="${COMMAND,,}"  # lowercase

# Resolve handler script
HANDLER="$SKILLS_DIR/skill_${COMMAND}.sh"

if [ ! -f "$HANDLER" ]; then
    echo -e "\033[31m✗ Unknown command: /${COMMAND}\033[0m"
    echo -e "  Run \033[1m/help\033[0m to see all available commands."
    exit 1
fi

if [ ! -x "$HANDLER" ]; then
    chmod +x "$HANDLER"
fi

# Dispatch
exec "$HANDLER" "$@"
