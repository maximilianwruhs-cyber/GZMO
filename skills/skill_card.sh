#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /card [type] — Forge a random Magic: The Gathering card
# ═══════════════════════════════════════════════════════════════════
# Uses baked knowledge from cardforge.toml (Color Pie, types, rarities)
# and the LLM to generate mechanically coherent, flavorfully rich cards.
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

CARD_TYPE="${1:-}"
CARDFORGE="$SKILLS_DIR/cardforge.toml"

# ─── Random selections ──────────────────────────────────────────
COLORS=("white" "blue" "black" "red" "green")
COLOR_SYMBOLS=("☀️" "💧" "💀" "🔥" "🌿")
COLOR_LETTERS=("W" "U" "B" "R" "G")
RARITY_NAMES=("Common" "Uncommon" "Rare" "Mythic Rare")
RARITY_ICONS=("⚪" "🔵" "🟡" "🟠")

# Pick random color
C_IDX=$(chaos_int 0 4)
COLOR="${COLORS[$C_IDX]}"
COLOR_SYM="${COLOR_SYMBOLS[$C_IDX]}"
COLOR_LET="${COLOR_LETTERS[$C_IDX]}"

# Pick random rarity (weighted: 45% common, 30% uncommon, 18% rare, 7% mythic)
RARITY_ROLL=$(chaos_int 1 100)
if [ $RARITY_ROLL -le 45 ]; then
    R_IDX=0
elif [ $RARITY_ROLL -le 75 ]; then
    R_IDX=1
elif [ $RARITY_ROLL -le 93 ]; then
    R_IDX=2
else
    R_IDX=3
fi
RARITY="${RARITY_NAMES[$R_IDX]}"
RARITY_ICON="${RARITY_ICONS[$R_IDX]}"

# Card type
TYPES=("Creature" "Instant" "Sorcery" "Enchantment" "Artifact")
if [ -n "$CARD_TYPE" ]; then
    # Normalize user input
    CARD_TYPE="${CARD_TYPE,,}"
    CARD_TYPE="${CARD_TYPE^}"
    # Validate
    valid=false
    for t in "${TYPES[@]}"; do
        if [ "${t,,}" = "${CARD_TYPE,,}" ]; then
            CARD_TYPE="$t"
            valid=true
            break
        fi
    done
    if [ "$valid" = false ]; then
        echo -e "${C_RED}✗ Unknown card type: $CARD_TYPE${C_RESET}"
        echo -e "  Valid types: creature, instant, sorcery, enchantment, artifact"
        exit 1
    fi
else
    T_IDX=$(chaos_int 0 4)
    CARD_TYPE="${TYPES[$T_IDX]}"
fi

# ─── Extract color philosophy from cardforge.toml ────────────────
PHILOSOPHY=""
STRENGTHS=""
FLAVOR_TONE=""
if [ -f "$CARDFORGE" ]; then
    # Simple extraction
    PHILOSOPHY=$(grep -A1 "^\[colors\.${COLOR}\]" "$CARDFORGE" | grep 'philosophy' | sed 's/.*= "//;s/"$//' || true)
    FLAVOR_TONE=$(grep -A10 "^\[colors\.${COLOR}\]" "$CARDFORGE" | grep 'flavor_tone' | sed 's/.*= "//;s/"$//' || true)
fi

# ─── Generate via LLM ───────────────────────────────────────────
SYSTEM_PROMPT="You are a Magic: The Gathering card designer following the Vision Design → Set Design → Play Design methodology.

COLOR IDENTITY: ${COLOR^} (${COLOR_LET}) — ${PHILOSOPHY}
CARD TYPE: ${CARD_TYPE}
RARITY: ${RARITY}

DESIGN RULES:
- Follow the Color Pie strictly. This ${COLOR} card must only use abilities appropriate to ${COLOR}.
- Rarity determines complexity: Common=simple, Uncommon=moderate, Rare=complex, Mythic=splashy and game-warping.
- Use strict MTG templating for rules text (identical effects = identical phrasing).
- Flavor text must perform heavy narrative lifting — convey vast philosophical tenets concisely.
- Flavor tone for ${COLOR}: ${FLAVOR_TONE}
- If Creature: assign a relevant creature type, and Power/Toughness balanced for the mana cost.
- Mana cost must be balanced: higher cost = more powerful effect.
- Do NOT overcrowd — max 2 abilities for Common/Uncommon, max 3 for Rare/Mythic.

OUTPUT FORMAT (exactly this, no other text):
NAME: [card name]
COST: [mana cost like {2}{${COLOR_LET}} or {3}{${COLOR_LET}}{${COLOR_LET}}]
TYPE: [full type line like 'Creature — Human Wizard' or 'Instant']
RARITY: ${RARITY}
RULES: [rules text, use | for line breaks between abilities]
FLAVOR: [italic flavor text, max 2 sentences]
PT: [Power/Toughness like 3/4, or NONE if not a creature]"

USER_PROMPT="Design one original Magic: The Gathering card. Make it memorable."

CARD=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Forging a ${RARITY} ${COLOR} ${CARD_TYPE}..." 0.9 384)

if [ -z "$CARD" ]; then
    echo -e "${C_RED}✗ LLM offline. The Card Forge lies cold.${C_RESET}"
    exit 1
fi

# ─── Parse result ────────────────────────────────────────────────
CARD_NAME=$(echo "$CARD" | grep '^NAME:' | sed 's/^NAME: *//')
CARD_COST=$(echo "$CARD" | grep '^COST:' | sed 's/^COST: *//')
CARD_TYPELINE=$(echo "$CARD" | grep '^TYPE:' | sed 's/^TYPE: *//')
CARD_RULES=$(echo "$CARD" | grep '^RULES:' | sed 's/^RULES: *//')
CARD_FLAVOR=$(echo "$CARD" | grep '^FLAVOR:' | sed 's/^FLAVOR: *//')
CARD_PT=$(echo "$CARD" | grep '^PT:' | sed 's/^PT: *//')

# Fallback for unparseable output
if [ -z "$CARD_NAME" ]; then
    CARD_NAME="Unnamed ${CARD_TYPE}"
fi

# ─── Render ASCII Card Frame ────────────────────────────────────
BORDER_COLOR="$C_WHITE"
case "$COLOR" in
    white) BORDER_COLOR="$C_WHITE" ;;
    blue)  BORDER_COLOR="$C_BLUE" ;;
    black) BORDER_COLOR="$C_DIM" ;;
    red)   BORDER_COLOR="$C_RED" ;;
    green) BORDER_COLOR="$C_GREEN" ;;
esac

echo ""
echo -e "${BORDER_COLOR}  ╔═══════════════════════════════════════════╗${C_RESET}"
echo -e "${BORDER_COLOR}  ║${C_RESET} ${C_BOLD}${CARD_NAME}${C_RESET}"
printf "${BORDER_COLOR}  ║${C_RESET} %45s\n" "${CARD_COST}"
echo -e "${BORDER_COLOR}  ╠═══════════════════════════════════════════╣${C_RESET}"
echo -e "${BORDER_COLOR}  ║${C_RESET}"
echo -e "${BORDER_COLOR}  ║${C_RESET}  ${COLOR_SYM} ${C_DIM}${CARD_TYPELINE}${C_RESET}"
echo -e "${BORDER_COLOR}  ║${C_RESET}  ${RARITY_ICON} ${C_DIM}${RARITY}${C_RESET}"
echo -e "${BORDER_COLOR}  ║${C_RESET}"
echo -e "${BORDER_COLOR}  ╠═══════════════════════════════════════════╣${C_RESET}"

# Rules text (split on |)
IFS='|' read -ra RULES_LINES <<< "$CARD_RULES"
for rline in "${RULES_LINES[@]}"; do
    rline=$(echo "$rline" | sed 's/^ *//;s/ *$//')
    echo -e "${BORDER_COLOR}  ║${C_RESET}  ${C_WHITE}${rline}${C_RESET}"
done

echo -e "${BORDER_COLOR}  ║${C_RESET}"

# Flavor text
if [ -n "$CARD_FLAVOR" ]; then
    echo -e "${BORDER_COLOR}  ║${C_RESET}  ${C_DIM}${C_MAGENTA}${CARD_FLAVOR}${C_RESET}"
    echo -e "${BORDER_COLOR}  ║${C_RESET}"
fi

# P/T (bottom right for creatures)
if [ -n "$CARD_PT" ] && [ "$CARD_PT" != "NONE" ] && [ "$CARD_PT" != "N/A" ]; then
    printf "${BORDER_COLOR}  ║${C_RESET}%42s ${C_BOLD}[${CARD_PT}]${C_RESET}\n"
fi

echo -e "${BORDER_COLOR}  ╚═══════════════════════════════════════════╝${C_RESET}"
echo ""
