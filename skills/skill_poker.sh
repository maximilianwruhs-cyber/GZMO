#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /poker — Generate a random 5-card poker hand and evaluate rank
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

# ─── Deck definition ────────────────────────────────────────────
SUITS=("♠" "♥" "♦" "♣")
SUIT_COLORS=("${C_WHITE}" "${C_RED}" "${C_RED}" "${C_WHITE}")
RANKS=("2" "3" "4" "5" "6" "7" "8" "9" "10" "J" "Q" "K" "A")
RANK_VALUES=(2 3 4 5 6 7 8 9 10 11 12 13 14)

# Build deck (52 cards as "rank|suit|value" triples)
declare -a DECK=()
for s in "${!SUITS[@]}"; do
    for r in "${!RANKS[@]}"; do
        DECK+=("${RANKS[$r]}|${SUITS[$s]}|${RANK_VALUES[$r]}|${s}")
    done
done

# ─── Shuffle and deal 5 ─────────────────────────────────────────
declare -a HAND=()
declare -a USED=()
for i in $(seq 1 5); do
    while true; do
        IDX=$(chaos_int 0 51)
        # Check if already dealt
        local_used=false
        for u in "${USED[@]}"; do
            if [ "$u" = "$IDX" ]; then local_used=true; break; fi
        done
        if [ "$local_used" = false ]; then
            USED+=("$IDX")
            HAND+=("${DECK[$IDX]}")
            break
        fi
    done
done

# ─── Parse hand ──────────────────────────────────────────────────
declare -a HAND_RANKS=()
declare -a HAND_SUITS=()
declare -a HAND_VALUES=()
declare -a HAND_SUIT_IDX=()
declare -a DISPLAY_CARDS=()

for card in "${HAND[@]}"; do
    IFS='|' read -r rank suit value sidx <<< "$card"
    HAND_RANKS+=("$rank")
    HAND_SUITS+=("$suit")
    HAND_VALUES+=("$value")
    HAND_SUIT_IDX+=("$sidx")
    COLOR="${SUIT_COLORS[$sidx]}"
    DISPLAY_CARDS+=("${COLOR}${rank}${suit}${C_RESET}")
done

# Sort values
IFS=$'\n' SORTED_VALUES=($(sort -n <<< "${HAND_VALUES[*]}")); unset IFS

# ─── Evaluate hand ───────────────────────────────────────────────
is_flush() {
    local first="${HAND_SUITS[0]}"
    for s in "${HAND_SUITS[@]}"; do
        [ "$s" != "$first" ] && return 1
    done
    return 0
}

is_straight() {
    for i in $(seq 1 4); do
        local diff=$(( SORTED_VALUES[i] - SORTED_VALUES[i-1] ))
        [ "$diff" -ne 1 ] && return 1
    done
    return 0
}

# Count rank occurrences
declare -A RANK_COUNT=()
for v in "${HAND_VALUES[@]}"; do
    RANK_COUNT[$v]=$(( ${RANK_COUNT[$v]:-0} + 1 ))
done

# Get sorted counts
COUNTS=$(printf '%s\n' "${RANK_COUNT[@]}" | sort -rn | tr '\n' ' ')

# Determine hand rank
FLUSH=false; is_flush && FLUSH=true
STRAIGHT=false; is_straight && STRAIGHT=true

# Check for ace-low straight (A-2-3-4-5)
if [ "$STRAIGHT" = false ] && [ "$FLUSH" = false ]; then
    if [ "${SORTED_VALUES[0]}" = "2" ] && [ "${SORTED_VALUES[1]}" = "3" ] && \
       [ "${SORTED_VALUES[2]}" = "4" ] && [ "${SORTED_VALUES[3]}" = "5" ] && \
       [ "${SORTED_VALUES[4]}" = "14" ]; then
        STRAIGHT=true
    fi
fi

HAND_NAME=""
HAND_ICON=""

if [ "$FLUSH" = true ] && [ "$STRAIGHT" = true ]; then
    if [ "${SORTED_VALUES[0]}" = "10" ]; then
        HAND_NAME="ROYAL FLUSH"
        HAND_ICON="👑"
    else
        HAND_NAME="STRAIGHT FLUSH"
        HAND_ICON="🌟"
    fi
elif [[ "$COUNTS" == 4\ * ]]; then
    HAND_NAME="FOUR OF A KIND"
    HAND_ICON="💎"
elif [[ "$COUNTS" == 3\ 2\ * ]]; then
    HAND_NAME="FULL HOUSE"
    HAND_ICON="🏠"
elif [ "$FLUSH" = true ]; then
    HAND_NAME="FLUSH"
    HAND_ICON="♦️"
elif [ "$STRAIGHT" = true ]; then
    HAND_NAME="STRAIGHT"
    HAND_ICON="📏"
elif [[ "$COUNTS" == 3\ * ]]; then
    HAND_NAME="THREE OF A KIND"
    HAND_ICON="🎯"
elif [[ "$COUNTS" == 2\ 2\ * ]]; then
    HAND_NAME="TWO PAIR"
    HAND_ICON="✌️"
elif [[ "$COUNTS" == 2\ * ]]; then
    HAND_NAME="ONE PAIR"
    HAND_ICON="👫"
else
    HAND_NAME="HIGH CARD"
    HAND_ICON="🃏"
fi

# ─── Output ──────────────────────────────────────────────────────
CARDS_DISPLAY="${DISPLAY_CARDS[0]}  ${DISPLAY_CARDS[1]}  ${DISPLAY_CARDS[2]}  ${DISPLAY_CARDS[3]}  ${DISPLAY_CARDS[4]}"

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_CYAN}  🃏 POKER HAND${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "     ${CARDS_DISPLAY}"
echo ""
echo -e "  ${C_BOLD}${C_YELLOW}  ${HAND_ICON} ${HAND_NAME}${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
