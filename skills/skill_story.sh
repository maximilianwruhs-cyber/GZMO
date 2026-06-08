#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /story [keyword] — Short story generation from a keyword seed
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

KEYWORD="${1:-chaos}"

SYSTEM_PROMPT="You are a master of the modern short story, writing in the sparse, tense style of Ernest Hemingway or the surreal, absurd style of Franz Kafka.
Write a very short story based on the keyword provided.

RULES:
- Maximum 500 characters total.
- The story must be complete (beginning, middle, end) but have strong subtext or an unresolved, surprising ending.
- Focus on concrete sensory details, physical objects, and specific textures.
- STRICTLY FORBIDDEN: Fairy tales, happily ever after, 'once upon a time', obvious moral lessons, or cheesy clichés.
- Output ONLY the story text. No titles, no labels, no introduction, no markdown blockquotes."

USER_PROMPT="Write a short story based on the keyword: ${KEYWORD}"

# Try up to 3 times to get a story that satisfies the character limit
MAX_ATTEMPTS=3
STORY=""
for attempt in $(seq 1 $MAX_ATTEMPTS); do
    RAW_STORY=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Weaving a tale of '${KEYWORD}' (Attempt $attempt)..." 0.85 4096)
    CLEANED=$(clean_llm_output "$RAW_STORY")

    if accept_creative_output "$CLEANED" 500 quality_gate_story; then
        STORY="$CLEANED"
        break
    fi
done

if [ -z "$STORY" ]; then
    echo -e "${C_RED}✗ LLM offline or story exceeded limits. The story remains untold.${C_RESET}"
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
