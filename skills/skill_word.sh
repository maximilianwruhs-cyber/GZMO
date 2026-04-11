#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /word — Invent a brand new word with definition and example
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

SYSTEM_PROMPT="You are a neologist — an inventor of words.
Create one completely new word that does not exist in any language.
The word should:
- Sound natural and pronounceable
- Have a specific, useful meaning that fills a gap in language
- Come with a believable etymology

Format your response EXACTLY like this (3 lines only):
WORD: [the new word] ([pronunciation])
DEFINITION: [clear definition]
EXAMPLE: [one example sentence using the word]

No other text. No commentary."

USER_PROMPT="Invent a new word."

RESULT=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Inventing linguistics..." 0.95 256)

if [ -z "$RESULT" ]; then
    echo -e "${C_RED}✗ LLM offline. Cannot invent words without a brain.${C_RESET}"
    exit 1
fi

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_GREEN}  🔤 NEW WORD${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo "$RESULT" | while IFS= read -r line; do
    if [[ "$line" == WORD:* ]]; then
        echo -e "  ${C_BOLD}${C_CYAN}${line}${C_RESET}"
    elif [[ "$line" == DEFINITION:* ]]; then
        echo -e "  ${C_WHITE}${line}${C_RESET}"
    elif [[ "$line" == EXAMPLE:* ]]; then
        echo -e "  ${C_DIM}${line}${C_RESET}"
    else
        echo -e "  ${line}"
    fi
done
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
