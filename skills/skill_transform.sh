#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /transform [character] — Activate a character persona overlay
# ═══════════════════════════════════════════════════════════════════
# All subsequent generative skills will adopt this persona's voice.
# Call with no args to reset. Call with a name to activate.
# Profiles are baked in characters.toml — no external dependency.
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

CHARACTER="${1:-}"
CHARACTERS_FILE="$SKILLS_DIR/characters.toml"

if [ ! -f "$CHARACTERS_FILE" ]; then
    echo -e "${C_RED}✗ characters.toml not found at $CHARACTERS_FILE${C_RESET}"
    exit 1
fi

# ─── Reset if no arg ────────────────────────────────────────────
if [ -z "$CHARACTER" ]; then
    if [ -f "$TRANSFORM_STATE" ]; then
        rm -f "$TRANSFORM_STATE"
        echo ""
        echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
        echo -e "${C_BOLD}${C_GREEN}  🎭 TRANSFORM RESET${C_RESET}"
        echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
        echo ""
        echo -e "  ${C_WHITE}Persona cleared. Back to default GZMO voice.${C_RESET}"
        echo ""
        echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
    else
        # No persona active — show available characters
        echo ""
        echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
        echo -e "${C_BOLD}${C_MAGENTA}  🎭 AVAILABLE PERSONAS${C_RESET}"
        echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
        echo ""

        # Parse character names and icons from TOML
        while IFS= read -r line; do
            if [[ "$line" =~ ^name\ =\ \"(.*)\"$ ]]; then
                local_name="${BASH_REMATCH[1]}"
            fi
            if [[ "$line" =~ ^icon\ =\ \"(.*)\"$ ]]; then
                local_icon="${BASH_REMATCH[1]}"
                echo -e "  ${local_icon}  ${C_BOLD}${local_name}${C_RESET}"
            fi
        done < "$CHARACTERS_FILE"

        echo ""
        echo -e "  ${C_DIM}Usage: /transform <name>${C_RESET}"
        echo -e "  ${C_DIM}Reset: /transform${C_RESET}"
        echo ""
        echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
    fi
    exit 0
fi

# ─── Find character (case-insensitive) ──────────────────────────
CHARACTER_LOWER="${CHARACTER,,}"

found=false
char_name=""
char_icon=""
char_alter_ego=""
char_universe=""
char_speech=""
char_personality=""
char_catchphrases=""
char_system_prompt=""

# Simple TOML parser — find the matching character block
in_block=false
current_name=""

while IFS= read -r line; do
    if [[ "$line" == "[[characters]]" ]]; then
        # If we were processing the right block, we're done
        if [ "$found" = true ]; then
            break
        fi
        in_block=true
        continue
    fi

    if [ "$in_block" = true ]; then
        if [[ "$line" =~ ^name\ =\ \"(.*)\"$ ]]; then
            current_name="${BASH_REMATCH[1]}"
            current_lower="${current_name,,}"
            if [[ "$current_lower" == *"$CHARACTER_LOWER"* ]]; then
                found=true
                char_name="$current_name"
            fi
        fi

        if [ "$found" = true ]; then
            if [[ "$line" =~ ^icon\ =\ \"(.*)\"$ ]]; then
                char_icon="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^alter_ego\ =\ \"(.*)\"$ ]]; then
                char_alter_ego="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^universe\ =\ \"(.*)\"$ ]]; then
                char_universe="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^speech_style\ =\ \"(.*)\"$ ]]; then
                char_speech="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^personality\ =\ \"(.*)\"$ ]]; then
                char_personality="${BASH_REMATCH[1]}"
            elif [[ "$line" =~ ^system_prompt\ =\ \"(.*)\"$ ]]; then
                char_system_prompt="${BASH_REMATCH[1]}"
            fi
        fi
    fi
done < "$CHARACTERS_FILE"

# ─── If character not found in baked profiles, ask LLM to create one ─
if [ "$found" = false ]; then
    if llm_available; then
        echo -e "${C_DIM}  Character '${CHARACTER}' not in Pantheon — generating custom profile...${C_RESET}"
        CUSTOM_PROMPT=$(llm_call_pretty \
            "You are a character analyst. Given a fictional or real character name, create a persona profile.
Output EXACTLY in this format (5 lines, no other text):
NAME: [character name]
SPEECH: [2 sentences describing their unique speech patterns and vocabulary]
PERSONALITY: [1 sentence describing core personality trait]
CATCHPHRASE: [one iconic quote or saying]
SYSTEM_PROMPT: [A 2-sentence instruction for an AI to roleplay as this character]" \
            "Create a persona profile for: ${CHARACTER}" \
            "Analyzing '${CHARACTER}'..." \
            0.7 384)

        if [ -n "$CUSTOM_PROMPT" ]; then
            char_name="$CHARACTER"
            char_icon="🎭"
            char_system_prompt=$(echo "$CUSTOM_PROMPT" | grep '^SYSTEM_PROMPT:' | sed 's/^SYSTEM_PROMPT: *//')
            char_speech=$(echo "$CUSTOM_PROMPT" | grep '^SPEECH:' | sed 's/^SPEECH: *//')
            char_personality=$(echo "$CUSTOM_PROMPT" | grep '^PERSONALITY:' | sed 's/^PERSONALITY: *//')

            if [ -n "$char_system_prompt" ]; then
                found=true
            fi
        fi
    fi

    if [ "$found" = false ]; then
        echo -e "${C_RED}✗ Character '${CHARACTER}' not found in the Pantheon and LLM offline.${C_RESET}"
        echo -e "${C_DIM}  Available characters: Superman, Batman, Spider-Man, Wonder Woman,${C_RESET}"
        echo -e "${C_DIM}  Wolverine, Captain America, Iron Man, The Flash, Hulk, Thor${C_RESET}"
        exit 1
    fi
fi

# ─── Write transform state ──────────────────────────────────────
cat > "$TRANSFORM_STATE" <<EOF
PERSONA: ${char_name}
ICON: ${char_icon}
SPEECH: ${char_speech}
PERSONALITY: ${char_personality}
SYSTEM_PROMPT: ${char_system_prompt}
EOF

# ─── Output ──────────────────────────────────────────────────────
echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_MAGENTA}  🎭 TRANSFORM ACTIVATED${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "  ${C_BOLD}${char_icon} ${char_name}${C_RESET}"

if [ -n "$char_alter_ego" ]; then
    echo -e "  ${C_DIM}aka ${char_alter_ego} (${char_universe})${C_RESET}"
fi

echo ""
echo -e "  ${C_CYAN}Speech:${C_RESET} ${char_speech}"
echo ""
echo -e "  ${C_YELLOW}Personality:${C_RESET} ${char_personality}"
echo ""
echo -e "  ${C_DIM}All generative commands will now channel this${C_RESET}"
echo -e "  ${C_DIM}persona until you run /transform again.${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"

# ─── Demo: Let the character introduce themselves ────────────────
if llm_available; then
    echo ""
    INTRO=$(llm_call \
        "$char_system_prompt" \
        "Introduce yourself in one dramatic sentence. Stay in character." \
        0.9 128)
    if [ -n "$INTRO" ]; then
        echo -e "  ${C_BOLD}${char_icon}${C_RESET} ${C_WHITE}${INTRO}${C_RESET}"
        echo ""
    fi
fi
