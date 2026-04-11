#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /help — Display all available slash commands
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

REGISTRY="$SKILLS_DIR/skills.toml"

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_CYAN}  ❓ GZMO SKILL REGISTRY — All Available Commands${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────────────────────┤${C_RESET}"
echo ""

# Parse skills.toml for command entries
if [ -f "$REGISTRY" ]; then
    current_cmd=""
    current_desc=""
    current_args=""
    current_icon=""
    current_type=""

    while IFS= read -r line; do
        # Match [commands.XXX]
        if [[ "$line" =~ ^\[commands\.(.*)\]$ ]]; then
            # Print previous command if exists
            if [ -n "$current_cmd" ]; then
                printf "  ${C_BOLD}%s /${current_cmd}%-12s${C_RESET} ${C_DIM}%-8s${C_RESET} %s %s\n" \
                    "$current_icon" "" "$current_type" "$current_args" ""
                echo -e "     ${C_DIM}${current_desc}${C_RESET}"
                echo ""
            fi
            current_cmd="${BASH_REMATCH[1]}"
            current_desc=""
            current_args=""
            current_icon="⚡"
            current_type=""
        elif [[ "$line" =~ ^description\ =\ \"(.*)\"$ ]]; then
            current_desc="${BASH_REMATCH[1]}"
        elif [[ "$line" =~ ^args\ =\ \"(.*)\"$ ]]; then
            current_args="${BASH_REMATCH[1]}"
        elif [[ "$line" =~ ^icon\ =\ \"(.*)\"$ ]]; then
            current_icon="${BASH_REMATCH[1]}"
        elif [[ "$line" =~ ^type\ =\ \"(.*)\"$ ]]; then
            current_type="[${BASH_REMATCH[1]}]"
        fi
    done < "$REGISTRY"

    # Print last command
    if [ -n "$current_cmd" ]; then
        printf "  ${C_BOLD}%s /${current_cmd}%-12s${C_RESET} ${C_DIM}%-8s${C_RESET} %s\n" \
            "$current_icon" "" "$current_type" "$current_args"
        echo -e "     ${C_DIM}${current_desc}${C_RESET}"
        echo ""
    fi
else
    echo -e "  ${C_RED}✗ skills.toml not found at $REGISTRY${C_RESET}"
fi

echo -e "${C_DIM}├─────────────────────────────────────────────────────────────────┤${C_RESET}"

# Show active modifiers
LANG=$(get_language)
echo -e "  ${C_DIM}Active language:${C_RESET} ${C_BOLD}${LANG}${C_RESET}"

if [ -f "$TRANSFORM_STATE" ]; then
    PERSONA_NAME=$(head -1 "$TRANSFORM_STATE" 2>/dev/null | grep -oP 'PERSONA: \K.*' || echo "Unknown")
    echo -e "  ${C_DIM}Active transform:${C_RESET} ${C_BOLD}${C_MAGENTA}${PERSONA_NAME}${C_RESET}"
else
    echo -e "  ${C_DIM}Active transform:${C_RESET} ${C_DIM}none${C_RESET}"
fi

LLM_STATUS="offline"
if llm_available; then
    LLM_STATUS="${C_GREEN}online${C_RESET}"
else
    LLM_STATUS="${C_RED}offline${C_RESET}"
fi
echo -e "  ${C_DIM}LLM status:${C_RESET} ${LLM_STATUS}"
echo ""
echo -e "  ${C_DIM}Usage: ./skills/skill_dispatch.sh <command> [args]${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────────────────────┘${C_RESET}"
