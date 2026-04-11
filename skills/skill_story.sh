#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /story [keyword] — Short story generation from a keyword seed
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

KEYWORD="${1:-chaos}"

SYSTEM_PROMPT="You are a master storyteller. Write a very short story.
Rules:
- Maximum 500 characters
- The story must be complete: beginning, middle, end
- Must be based on the keyword provided
- Vivid imagery. Surprising ending.
- Output ONLY the story text. No title, no labels."

USER_PROMPT="Write a short story based on the keyword: ${KEYWORD}"

STORY=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Weaving a tale of '${KEYWORD}'..." 0.9 384)

if [ -z "$STORY" ]; then
    echo -e "${C_RED}✗ LLM offline. The story remains untold.${C_RESET}"
    exit 1
fi

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_BLUE}  📖 STORY — seed: \"${KEYWORD}\"${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo "$STORY" | fold -s -w 50 | while IFS= read -r line; do
    echo -e "  ${C_WHITE}${line}${C_RESET}"
done
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
