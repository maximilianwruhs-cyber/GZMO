#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /poem — Short poem generation (max 180 characters)
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

SYSTEM_PROMPT="You are a critically acclaimed contemporary German poet. Write a short, highly evocative poem.

CRITICAL CONSTRAINTS:
- STRICTLY BAN simple, predictable end-rhymes (e.g., Herz/Schmerz, Nacht/Lacht, Zeit/Weit). If you rhyme, use subtle slant rhymes (unreine Reime oder Binnenreime) or assonances.
- Avoid abstract words: eternity, soul, fate, whisper, dance, shadows, tears, Ewigkeit, Seele, Schicksal, Tränen.
- Focus on concrete, physical objects, textures, and sensory details.
- Maximum 180 characters total.
- Output ONLY the poem. No titles, no introduction, no markdown blockquotes, no commentary."

USER_PROMPT="Write a short, powerful poem."

# Try up to 3 times to get a poem that satisfies the character limit
MAX_ATTEMPTS=3
POEM=""
for attempt in $(seq 1 $MAX_ATTEMPTS); do
    RAW_POEM=$(llm_call_pretty "$SYSTEM_PROMPT" "$USER_PROMPT" "Composing verse (Attempt $attempt)..." 0.85 4096)
    CLEANED=$(clean_llm_output "$RAW_POEM")

    if accept_creative_output "$CLEANED" 180 quality_gate_poem; then
        POEM="$CLEANED"
        break
    fi
done

if [ -z "$POEM" ]; then
    # Hardcoded fallback haikus that align with the concrete, high-quality style
    declare -a FALLBACKS=(
        "Kupfer grünt, das Glas vergilbt langsam\nSand sinkt hinab, der Stahl bricht ab\nAsche legt sich, die Kälte bleibt"
        "Der Ruß auf der Kachel zerfällt leise\nKaltes Eisen gibt nach, dehnt sich aus\nKein Rad greift mehr ins andere"
        "Ein Tropfen Öl auf trockenem Schiefer\nEr glänzt im trüben Mittagslicht\nBevor der Stein den Glanz verschluckt"
    )
    IDX=$(chaos_int 0 $(( ${#FALLBACKS[@]} - 1 )))
    POEM="${FALLBACKS[$IDX]}"
fi

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_MAGENTA}  🖋️  POEM${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "${C_WHITE}${POEM}${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
