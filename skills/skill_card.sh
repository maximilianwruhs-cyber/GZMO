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

# ─── LLM config: prefer explicit env, then GZMO-next defaults ───
LLM_URL="${GZMO_LLM_URL:-http://localhost:8000/v1}"
export GZMO_LLM_URL="$LLM_URL"

# Resolve model: env > GZMO-next config > server's registered model
if [ -z "${GZMO_LLM_MODEL:-}" ]; then
    if [ -f "$SKILLS_DIR/../config/gzmo-next.toml" ]; then
        GZMO_LLM_MODEL=$(python3 -c "
import tomllib, sys, shlex
try:
    with open('$SKILLS_DIR/../config/gzmo-next.toml','rb') as f:
        cfg = tomllib.load(f)
    print(cfg.get('engine',{}).get('local',{}).get('model',''))
except: print('')
" 2>/dev/null)
    fi
fi
LLM_MODEL="${GZMO_LLM_MODEL:-/home/gzmo/models/ornith-35b-GGUF/ornith-35b-Q4_K_M.gguf}"
export GZMO_LLM_MODEL="$LLM_MODEL"

# ─── Random selections ──────────────────────────────────────────
COLORS=("white" "blue" "black" "red" "green")
COLOR_SYMBOLS=("☀️" "💧" "💀" "🔥" "🌿")
COLOR_LETTERS=("W" "U" "B" "R" "G")
RARITY_NAMES=("Common" "Uncommon" "Rare" "Mythic Rare")
RARITY_ICONS=("⚪" "🔵" "🟡" "🟠")

C_IDX=$(chaos_int 0 4)
COLOR="${COLORS[$C_IDX]}"
COLOR_SYM="${COLOR_SYMBOLS[$C_IDX]}"
COLOR_LET="${COLOR_LETTERS[$C_IDX]}"

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

TYPES=("Creature" "Instant" "Sorcery" "Enchantment" "Artifact")
if [ -n "$CARD_TYPE" ]; then
    CARD_TYPE_LOWER="${CARD_TYPE,,}"
    CARD_TYPE="$CARD_TYPE_LOWER"
    valid=false
    for t in "${TYPES[@]}"; do
        if [ "${t,,}" = "${CARD_TYPE_LOWER}" ]; then
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
FLAVOR_TONE=""
if [ -f "$CARDFORGE" ]; then
    eval "$(python3 -c "
import tomllib, sys, shlex

try:
    with open('$CARDFORGE','rb') as f:
        cfg = tomllib.load(f)
    colors = cfg.get('colors',{})
    c = '$COLOR'
    section = colors.get(c, {})
    phil = section.get('philosophy', '')
    tone = section.get('flavor_tone', '')
    print(f'PHILOSOPHY={shlex.quote(phil)}')
    print(f'FLAVOR_TONE={shlex.quote(tone)}')
except Exception as e:
    print('PHILOSOPHY=' + shlex.quote(''))
    print('FLAVOR_TONE=' + shlex.quote(''))
" 2>/dev/null)"
fi

# ─── Build prompts ──────────────────────────────────────────────
SYSTEM_PROMPT="You are a Magic: The Gathering card designer following the Vision Design → Set Design → Play Design methodology.

COLOR IDENTITY: ${COLOR^} (${COLOR_LET})
PHILOSOPHY: ${PHILOSOPHY:-Color Pie default}
CARD TYPE: ${CARD_TYPE}
RARITY: ${RARITY}

DESIGN RULES:
- Follow the Color Pie strictly. This ${COLOR} card must only use abilities appropriate to ${COLOR}.
- Rarity determines complexity: Common=simple, Uncommon=moderate, Rare=complex, Mythic=splashy and game-warping.
- Use strict MTG templating for rules text (identical effects = identical phrasing).
- Flavor text must perform heavy narrative lifting — convey vast philosophical tenets concisely.
- Flavor tone for ${COLOR}: ${FLAVOR_TONE:-balanced}
- If Creature: assign a relevant creature type, and Power/Toughness balanced for the mana cost.
- Mana cost must be balanced: higher cost = more powerful effect.
- Do NOT overcrowd — max 2 abilities for Common/Uncommon, max 3 for Rare/Mythic.

OUTPUT FORMAT (exactly this, no other text — do NOT wrap in code blocks, do NOT explain your reasoning):
NAME: [card name]
COST: [mana cost like {2}{W} or {3}{R}{R}]
TYPE: [full type line like 'Creature — Human Wizard' or 'Instant']
RARITY: ${RARITY}
RULES: [rules text, use | for line breaks between abilities]
FLAVOR: [italic flavor text, max 2 sentences]
PT: [Power/Toughness like 3/4, or NONE if not a creature]"

USER_PROMPT="Design one original Magic: The Gathering card. Make it memorable."

# Structured output — skip transform persona / language injection.
export GZMO_SKILL_STRUCTURED=1

# ─── LLM call with retry ────────────────────────────────────────
CARD=""
MAX_ATTEMPTS=3
LLM_TEMP=0.9
LLM_MAX=4096
for attempt in $(seq 1 $MAX_ATTEMPTS); do
    CARD=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Forging a ${RARITY} ${COLOR} ${CARD_TYPE}..." "$LLM_TEMP" "$LLM_MAX")

    if [ -z "$CARD" ]; then
        echo -e "${C_RED}✗ LLM call failed (attempt $attempt/3). The Card Forge lies cold.${C_RESET}"
        exit 1
    fi

    # Validate structured output (tolerates reasoning before the final block)
    eval "$(printf '%s' "$CARD" | parse_structured_card_fields)"

    if [ -n "$CARD_NAME" ] && [ -n "$CARD_COST" ] && [ -n "$CARD_TYPELINE" ]; then
        break  # Good parse
    fi

    # Retry with a simpler prompt and lower temperature
    if [ $attempt -lt $MAX_ATTEMPTS ]; then
        LLM_TEMP=0.5
        SYSTEM_PROMPT="You are a Magic: The Gathering card designer.

COLOR: ${COLOR^} (${COLOR_LET})
TYPE: ${CARD_TYPE}
RARITY: ${RARITY}

Follow the Color Pie. Balance mana cost with power. Use strict MTG templating.

Output exactly:
NAME: [name]
COST: [mana cost]
TYPE: [type line]
RARITY: ${RARITY}
RULES: [rules text, use | for line breaks]
FLAVOR: [flavor text]
PT: [Power/Toughness or NONE]"
        USER_PROMPT="Design one MTG card."
    fi
done

if [ -z "$CARD_NAME" ]; then
    echo -e "${C_RED}✗ LLM output did not match expected format after $MAX_ATTEMPTS attempts.${C_RESET}"
    echo -e "  Raw output: ${CARD:0:200}..."
    exit 1
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
