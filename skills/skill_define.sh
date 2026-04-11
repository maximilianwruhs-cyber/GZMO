#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /define [term] — Definition, pronunciation (IPA), and etymology
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

TERM="$*"
if [ -z "$TERM" ]; then
    echo -e "${C_RED}✗ Usage: /define <term>${C_RESET}"
    exit 1
fi

SYSTEM_PROMPT="You are a lexicographer. For the given term, provide:
1. WORD: The term
2. PRONUNCIATION: IPA notation
3. PART OF SPEECH: (noun, verb, adjective, etc.)
4. DEFINITION: Clear, precise definition
5. ETYMOLOGY: Language of origin and historical derivation
6. USAGE: One example sentence

Format each on its own line with the label prefix. No other text."

USER_PROMPT="Define: ${TERM}"

RESULT=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Consulting the lexicon for '${TERM}'..." 0.3 384)

if [ -z "$RESULT" ]; then
    # Fallback: free dictionary API
    echo -e "${C_DIM}  LLM offline — trying dictionary API...${C_RESET}"
    ENCODED=$(echo "$TERM" | jq -Rr @uri 2>/dev/null || echo "${TERM// /+}")
    API_RESULT=$(curl -s --max-time 5 "https://api.dictionaryapi.dev/api/v2/entries/en/${ENCODED}" 2>/dev/null)

    if echo "$API_RESULT" | jq -e '.[0].word' >/dev/null 2>&1; then
        WORD=$(echo "$API_RESULT" | jq -r '.[0].word')
        PHONETIC=$(echo "$API_RESULT" | jq -r '.[0].phonetic // "N/A"')
        MEANING=$(echo "$API_RESULT" | jq -r '.[0].meanings[0].definitions[0].definition // "N/A"')
        POS=$(echo "$API_RESULT" | jq -r '.[0].meanings[0].partOfSpeech // "N/A"')

        RESULT="WORD: ${WORD}
PRONUNCIATION: ${PHONETIC}
PART OF SPEECH: ${POS}
DEFINITION: ${MEANING}
ETYMOLOGY: (API fallback — etymology unavailable)
USAGE: (API fallback — example unavailable)"
    else
        echo -e "${C_RED}✗ Term not found and LLM offline.${C_RESET}"
        exit 1
    fi
fi

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_BLUE}  📚 DEFINE${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo "$RESULT" | while IFS= read -r line; do
    if [[ "$line" == WORD:* ]]; then
        echo -e "  ${C_BOLD}${C_CYAN}${line}${C_RESET}"
    elif [[ "$line" == PRONUNCIATION:* ]]; then
        echo -e "  ${C_DIM}${line}${C_RESET}"
    elif [[ "$line" == DEFINITION:* ]]; then
        echo -e "  ${C_WHITE}${line}${C_RESET}"
    elif [[ "$line" == ETYMOLOGY:* ]]; then
        echo -e "  ${C_MAGENTA}${line}${C_RESET}"
    else
        echo -e "  ${line}"
    fi
done
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
