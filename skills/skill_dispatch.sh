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

export LC_ALL=C.UTF-8
export LANG="${LANG:-C.UTF-8}"

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
COMMAND="${1:-help}"

# Normalize: strip leading slash if present
COMMAND="${COMMAND#/}"
COMMAND="${COMMAND,,}"  # lowercase

# Ensure all skill executions route through gzmo chaos skill to prevent bypass
if [[ "${GZMO_CHAOS_SKILL_INNER:-}" != "1" ]]; then
    # We are calling skill_dispatch.sh from outside gzmo chaos skill!
    # Run it via gzmo chaos skill instead to ensure proper routing and audit.
    # Locate the gzmo binary relative to this script
    GZMO_BIN="$SKILLS_DIR/../target/release/gzmo"
    if [ ! -x "$GZMO_BIN" ]; then
        GZMO_BIN="$SKILLS_DIR/../target/debug/gzmo"
    fi
    if [ ! -x "$GZMO_BIN" ]; then
        GZMO_BIN="gzmo" # fallback to PATH
    fi
    export GZMO_CHAOS_SKILL_INNER=1
    exec "$GZMO_BIN" chaos skill "$COMMAND" "${@:2}"
fi

shift 2>/dev/null || true

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
