#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /joke — Structurally engineered joke via LLM
# Architecture: Setup → Misdirection → Punchline (from LORE.md BVT)
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

SYSTEM_PROMPT="You are a comedy engine grounded in the Benign Violation Theory (BVT).
A joke triggers laughter if and only if three conditions occur simultaneously:
1. A VIOLATION — something threatens how the world ought to be
2. A BENIGN CONTEXT — the threat is completely harmless
3. SIMULTANEOUS PROCESSING — both must be processed at the same neurological millisecond

Structure your joke using:
- SETUP: Establish a false, highly logical reality. Must be entirely devoid of comedy.
- MISDIRECTION: The invisible cognitive pivot point.
- PUNCHLINE: Violently subverts the expectation while technically complying with the setup's logic.

Apply the Rule of Three: First item (premise), second item (confirms pattern), third item (destroys it).
Max 280 characters total. Output ONLY the joke text. No labels, no explanation."

USER_PROMPT="Tell me one original, clever joke. Make me laugh."

JOKE=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Engineering a comedic violation..." 0.95 256)

if [ -z "$JOKE" ]; then
    # Fallback: random joke from lore.toml
    LORE_FILE="$RANDOMIZER_ROOT/lore.toml"
    if [ -f "$LORE_FILE" ]; then
        mapfile -t JOKES < <(grep -A1 '^\[\[jokes\]\]' "$LORE_FILE" | grep '^text = ' | sed 's/^text = "//;s/"$//')
        COUNT=${#JOKES[@]}
        if [ "$COUNT" -gt 0 ]; then
            IDX=$(chaos_int 0 $((COUNT - 1)))
            JOKE="${JOKES[$IDX]}"
        fi
    fi
fi

if [ -z "$JOKE" ]; then
    echo -e "${C_RED}✗ LLM offline and no fallback jokes available.${C_RESET}"
    exit 1
fi

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_YELLOW}  😂 JOKE${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "  ${C_WHITE}${JOKE}${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
