#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /quote — Random verified historical quote from the Lore Pool
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

LORE_FILE=$(resolve_lore_file) || {
    echo -e "${C_RED}✗ lore.toml not found (checked data/lore.toml and legacy paths)${C_RESET}"
    exit 1
}

# Extract all quotes (text + author pairs)
mapfile -t TEXTS < <(grep -A1 '^\[\[quotes\]\]' "$LORE_FILE" | grep '^text = ' | sed 's/^text = "//;s/"$//')
mapfile -t AUTHORS < <(grep -A2 '^\[\[quotes\]\]' "$LORE_FILE" | grep '^author = ' | sed 's/^author = "//;s/"$//')

COUNT=${#TEXTS[@]}

if [ "$COUNT" -eq 0 ]; then
    echo -e "${C_RED}✗ No quotes found in lore.toml${C_RESET}"
    exit 1
fi

IDX=$(chaos_int 0 $((COUNT - 1)))
QUOTE="${TEXTS[$IDX]}"
AUTHOR="${AUTHORS[$IDX]:-Unknown}"

# If language override is active, translate via LLM
LANG=$(get_language)
if [ "$LANG" != "en" ] && llm_available; then
    TRANSLATED=$(llm_call \
        "You are a precise translator. Translate the following quote to language code: $LANG. Output ONLY the translated quote, nothing else." \
        "$QUOTE" \
        0.3 256)
    if [ -n "$TRANSLATED" ]; then
        QUOTE="$TRANSLATED"
    fi
fi

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_CYAN}  📜 QUOTE${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "  ${C_WHITE}\"${QUOTE}\"${C_RESET}"
echo ""
echo -e "  ${C_DIM}— ${AUTHOR}${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
