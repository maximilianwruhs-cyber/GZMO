#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /calculate [expression] — Math solver via bc
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

EXPR="$*"
if [ -z "$EXPR" ]; then
    echo -e "${C_RED}✗ Usage: /calculate <expression>${C_RESET}"
    echo -e "  Examples: /calculate 2^10"
    echo -e "           /calculate \"sqrt(144)\""
    echo -e "           /calculate \"3.14 * 42^2\""
    exit 1
fi

# Compute via bc with math library
RESULT=$(echo "$EXPR" | bc -l 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ] || [ -z "$RESULT" ]; then
    # Try to ask LLM to parse natural language math
    if llm_available; then
        RESULT=$(llm_call \
            "You are a calculator. Given a math expression in any format, compute the answer. Output ONLY the numerical result. Nothing else." \
            "$EXPR" 0.0 64)
    fi

    if [ -z "$RESULT" ]; then
        echo -e "${C_RED}✗ Invalid expression: $EXPR${C_RESET}"
        exit 1
    fi
fi

# Trim trailing zeros for cleaner output
CLEAN=$(echo "$RESULT" | sed 's/\.\{0,1\}0*$//')

echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_GREEN}  🧮 CALCULATE${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""
echo -e "  ${C_DIM}expr: ${EXPR}${C_RESET}"
echo -e "  ${C_BOLD}${C_WHITE}  =  ${CLEAN}${C_RESET}"
echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
