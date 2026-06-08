#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /joke — Structurally engineered joke via LLM
# Architecture: Setup → Misdirection → Punchline (from LORE.md BVT)
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

SYSTEM_PROMPT="You are a comedy engine grounded in the Benign Violation Theory (BVT).
A joke triggers laughter if and only if three conditions occur simultaneously:
1. A VIOLATION — something threatens how the world ought to be (social norms, physical laws, logic).
2. A BENIGN CONTEXT — the threat is completely harmless or reframed safely.
3. SIMULTANEOUS PROCESSING — both must be processed at the same neurological millisecond.

Structure your joke using:
- SETUP: Establish a false, highly logical reality. Must be entirely devoid of comedy.
- MISDIRECTION: The invisible cognitive pivot point.
- PUNCHLINE: Violently subverts the expectation while technically complying with the setup's logic.

CRITICAL CONSTRAINTS:
- STRICTLY FORBIDDEN clichés: programming bugs, coffee, bad weather, typical artificial intelligence jokes, or simple 'dad jokes' (Flachwitze).
- Focus on clever, situational irony or absurdist framing.
- Max 280 characters total. Output ONLY the joke text. No labels, no explanation."

USER_PROMPT="Tell me one original, clever joke. Make me laugh."

# Try up to 3 times to get a joke that satisfies the character limit
MAX_ATTEMPTS=3
JOKE=""
for attempt in $(seq 1 $MAX_ATTEMPTS); do
    RAW_JOKE=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Engineering a comedic violation (Attempt $attempt)..." 0.9 4096)
    CLEANED=$(clean_llm_output "$RAW_JOKE")

    if accept_creative_output "$CLEANED" 280 quality_gate_joke; then
        JOKE="$CLEANED"
        break
    fi
done

if [ -z "$JOKE" ]; then
    declare -a FALLBACKS=(
        "Der Optiker fragte, ob ich die Brille zum Lesen oder zum Sehen brauche. Ich sagte: zum Überleben des Kleingedruckten."
        "Mein Nachbar betet jeden Abend. Nicht aus Glauben — er will, dass der Kühlschrank endlich aufhört zu summen."
        "Sie nannten mich optimistisch, weil ich bei jeder Absage nach dem Parkplatz gefragt habe."
    )
    IDX=$(chaos_int 0 $(( ${#FALLBACKS[@]} - 1 )))
    JOKE="${FALLBACKS[$IDX]}"
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
echo -e "${C_WHITE}${JOKE}${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
