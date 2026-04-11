#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /poem — Short poem generation (max 180 characters)
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

SYSTEM_PROMPT="You are a poet. Write one short, original poem.
Rules:
- Maximum 180 characters total
- No title
- Can be any style: haiku, free verse, couplet, limerick
- Must have emotional resonance or surprising imagery
- Output ONLY the poem text. No labels, no commentary."

USER_PROMPT="Write a short poem."

POEM=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Composing verse..." 0.95 128)

if [ -z "$POEM" ]; then
    # Hardcoded fallback haiku
    declare -a FALLBACKS=(
        "The Lorenz wings fold—\na butterfly's breath disturbs\nthe silent machine."
        "Entropy whispers\nthrough corridors of pure math—\nchaos wears a crown."
        "One point zero six.\nThe seed that split the weather.\nDeterministic fire."
        "In the phase space dark,\ntwo attractors nearly kiss—\nthen diverge forever."
    )
    IDX=$(chaos_int 0 $(( ${#FALLBACKS[@]} - 1 )))
    POEM="${FALLBACKS[$IDX]}"
fi

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_MAGENTA}  🖋️  POEM${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "  ${C_WHITE}${POEM}${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
