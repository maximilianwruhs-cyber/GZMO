#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /language [code] — Switch output language for all generative skills
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

LANG_CODE="${1:-}"

if [ -z "$LANG_CODE" ]; then
    # Show current and reset
    CURRENT=$(get_language)
    echo -e "${C_DIM}Current language: ${C_BOLD}${CURRENT}${C_RESET}"
    echo -e "${C_DIM}Usage: /language <code>  (en, de, ja, fr, es, it, pt, zh, ko, ru, ar, hi)${C_RESET}"
    echo -e "${C_DIM}Reset: /language en${C_RESET}"
    exit 0
fi

# Validate (loose — accept any 2-3 letter code)
LANG_CODE="${LANG_CODE,,}"
if [[ ! "$LANG_CODE" =~ ^[a-z]{2,3}$ ]]; then
    echo -e "${C_RED}✗ Invalid language code: $LANG_CODE. Use BCP-47 codes (en, de, ja, fr, etc.)${C_RESET}"
    exit 1
fi

# Language name lookup for display
declare -A LANG_NAMES=(
    [en]="English" [de]="Deutsch" [ja]="日本語" [fr]="Français"
    [es]="Español" [it]="Italiano" [pt]="Português" [zh]="中文"
    [ko]="한국어" [ru]="Русский" [ar]="العربية" [hi]="हिन्दी"
    [nl]="Nederlands" [pl]="Polski" [sv]="Svenska" [tr]="Türkçe"
)

LANG_NAME="${LANG_NAMES[$LANG_CODE]:-$LANG_CODE}"

# Write state
echo "$LANG_CODE" > "$LANG_STATE"

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_GREEN}  🌍 LANGUAGE SWITCHED${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "  ${C_BOLD}${LANG_CODE}${C_RESET} — ${LANG_NAME}"
echo ""
echo -e "  ${C_DIM}All generative commands will now respond in${C_RESET}"
echo -e "  ${C_BOLD}${LANG_NAME}${C_RESET}${C_DIM}.${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
