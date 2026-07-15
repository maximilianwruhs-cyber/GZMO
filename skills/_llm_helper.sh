#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────
# _llm_helper.sh — Shared LLM call infrastructure for GZMO skills
# Sources: skill scripts source this file, never execute directly.
# ─────────────────────────────────────────────────────────────────

SKILLS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SKILLS_DIR")"
RANDOMIZER_ROOT="$(dirname "$PROJECT_ROOT")/Randomizer"

# Locale-safe counting and matching (systemd/cron often lack UTF-8 locale)
export LC_ALL=C.UTF-8
export LANG="${LANG:-C.UTF-8}"

# ─── LLM Endpoint Configuration ─────────────────────────────────
LLM_URL="${GZMO_LLM_URL:-http://localhost:8000/v1}"
LLM_MODEL="${GZMO_LLM_MODEL:-/home/gzmo/models/ornith-35b-GGUF/ornith-35b-Q4_K_M.gguf}"
LLM_TEMPERATURE="${GZMO_LLM_TEMP:-0.8}"
LLM_MAX_TOKENS="${GZMO_LLM_MAX_TOKENS:-512}"

# ─── State Files ─────────────────────────────────────────────────
LANG_STATE="$SKILLS_DIR/.language"
TRANSFORM_STATE="$SKILLS_DIR/.transform_persona"

# ─── Color Codes ─────────────────────────────────────────────────
C_RESET="\033[0m"
C_BOLD="\033[1m"
C_DIM="\033[2m"
C_RED="\033[31m"
C_GREEN="\033[32m"
C_YELLOW="\033[33m"
C_BLUE="\033[34m"
C_MAGENTA="\033[35m"
C_CYAN="\033[36m"
C_WHITE="\033[97m"
C_BG_BLACK="\033[40m"

# ─── Read Active Language ────────────────────────────────────────
get_language() {
    if [ -f "$LANG_STATE" ]; then
        cat "$LANG_STATE"
    else
        echo "en"
    fi
}

# ─── Read Active Transform Persona ──────────────────────────────
get_persona_prompt() {
    if [ -f "$TRANSFORM_STATE" ]; then
        cat "$TRANSFORM_STATE"
    else
        echo ""
    fi
}

# ─── Build System Prompt with Language + Persona ─────────────────
build_system_prompt() {
    local base_prompt="$1"

    if [ "${GZMO_SKILL_STRUCTURED:-0}" = "1" ]; then
        echo -e "$base_prompt"
        return
    fi

    local lang=$(get_language)
    local persona=$(get_persona_prompt)

    local full_prompt="$base_prompt"

    # Inject language directive
    if [ "$lang" != "en" ]; then
        full_prompt="$full_prompt\n\nCRITICAL: You MUST respond entirely in language code: $lang. Do not use English."
    fi

    # Inject persona transform
    if [ -n "$persona" ]; then
        full_prompt="$full_prompt\n\nCHARACTER TRANSFORM ACTIVE:\n$persona\nYou MUST adopt this character's speech patterns, vocabulary, and personality in your response."
    fi

    echo -e "$full_prompt"
}

# ─── Strip thinking tags from LLM output ────────────────────────
strip_thinking_tags() {
    local text="$1"
    # Remove <think>...</think> blocks (with any whitespace/newlines inside)
    text=$(printf '%s' "$text" | sed -E ':a;s/<think[^>]*>.*?<\/think>//Ig;ta' | sed -E 's/<think[^>]*>//Ig; s/<\/think>//Ig')
    # Remove any remaining stray tags
    text=$(printf '%s' "$text" | sed -E 's/<\/?[a-z]+>//Ig')
    # Trim leading/trailing whitespace and blank lines
    text=$(printf '%s' "$text" | sed -e '/./,$!d' -e :a -e '/^\s*$/{$d;N;ba' -e '}')
    text="${text#"${text%%[![:space:]]*}"}"
    text="${text%"${text##*[![:space:]]}"}"
    printf '%s' "$text"
}

# ─── Call LLM (chat completions API for llama.cpp) ──────────────
# Usage: llm_call "system_prompt" "user_prompt" [temperature] [max_tokens]
# Returns: the raw text response, or "" on failure
llm_call() {
    local system_prompt="$1"
    local user_prompt="$2"
    local temp="${3:-$LLM_TEMPERATURE}"
    local max_tok="${4:-$LLM_MAX_TOKENS}"

    # Build the full system prompt with language/persona injections
    local full_system
    full_system=$(build_system_prompt "$system_prompt")

    # Escape for JSON
    local sys_escaped
    sys_escaped=$(printf '%s' "$full_system" | jq -Rs '.')
    local usr_escaped
    usr_escaped=$(printf '%s' "$user_prompt" | jq -Rs '.')

    local payload
    payload=$(cat <<JSONEOF
{
  "model": "$LLM_MODEL",
  "messages": [
    {"role": "system", "content": $sys_escaped},
    {"role": "user", "content": $usr_escaped}
  ],
  "temperature": $temp,
  "max_tokens": $max_tok,
  "stream": false
}
JSONEOF
)

    local response
    response=$(curl -s --connect-timeout 5 --max-time 120 \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "${LLM_URL%/v1}/v1/chat/completions" 2>/dev/null)

    if [ $? -ne 0 ] || [ -z "$response" ]; then
        echo ""
        return 1
    fi

    # Extract content — fall back to reasoning_content if content is empty
    # (Ornith/llama.cpp often puts output in reasoning_content)
    local content
    content=$(echo "$response" | jq -r '.choices[0].message.content // empty' 2>/dev/null)

    if [ -z "$content" ]; then
        content=$(echo "$response" | jq -r '.choices[0].message.reasoning_content // empty' 2>/dev/null)
    fi

    if [ -z "$content" ]; then
        echo ""
        return 1
    fi

    # Strip thinking tags and clean up
    content=$(strip_thinking_tags "$content")

    echo "$content"
    return 0
}

# ─── Check LLM Availability ─────────────────────────────────────
llm_available() {
    curl -s --connect-timeout 2 --max-time 3 \
        "${LLM_URL%/v1}/v1/models" >/dev/null 2>&1
    return $?
}

# ─── Chaos Seed ──────────────────────────────────────────────────
# Read chaos_val from Randomizer HEARTBEAT.md or fallback to $RANDOM
get_chaos_seed() {
    local heartbeat="$RANDOMIZER_ROOT/HEARTBEAT.md"
    if [ -f "$heartbeat" ]; then
        # Try to extract tension value as a chaos proxy
        local tension
        tension=$(grep -oP 'Tension: \K[0-9.]+' "$heartbeat" 2>/dev/null)
        if [ -n "$tension" ]; then
            echo "$tension"
            return
        fi
    fi
    # Fallback: pseudo-random float
    echo "$(( RANDOM % 100 )).$(( RANDOM % 100 ))"
}

# ─── Random Integer ──────────────────────────────────────────────
# Usage: chaos_int MIN MAX
chaos_int() {
    local min=$1
    local max=$2
    local range=$(( max - min + 1 ))
    echo $(( (RANDOM % range) + min ))
}

# ─── Lore Pool Path ──────────────────────────────────────────────
# Usage: resolve_lore_file  → prints path or exits 1
resolve_lore_file() {
    local candidates=(
        "$PROJECT_ROOT/data/lore.toml"
        "$RANDOMIZER_ROOT/lore.toml"
        "$PROJECT_ROOT/lore.toml"
    )
    local f
    for f in "${candidates[@]}"; do
        if [ -f "$f" ]; then
            echo "$f"
            return 0
        fi
    done
    return 1
}

# ─── LLM Output Cleanup ──────────────────────────────────────────
# Strips wrapping quotes/backticks; trims outer blank lines; keeps internal newlines.
clean_llm_output() {
    local text="$1"
    text="${text//\`/}"
    text=$(printf '%s' "$text" | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
    text=$(printf '%s' "$text" | sed -e '/./,$!d' -e :a -e '/^\s*$/{$d;N;ba' -e '}')
    text="${text#"${text%%[![:space:]]*}"}"
    text="${text%"${text##*[![:space:]]}"}"
    text=$(printf '%s' "$text" | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
    printf '%s' "$text"
}

# ─── Character Count (UTF-8 safe) ────────────────────────────────
char_count() {
    printf '%s' "$1" | wc -m | awk '{print $1}'
}

# ─── Creative Output Quality Gates ───────────────────────────────
# Return 0 if text passes, 1 if it hits a banned pattern.
quality_gate_poem() {
    printf '%s' "$1" | grep -qiE \
        '(seele|schicksal|ewigkeit|tr[aä]nen|schatten|fl[uü]stern|herz.*schmerz|schmerz.*herz|\bsoul\b|\bfate\b|\beternity\b|\bwhisper\b|\bshadows\b|\btears\b|\bdance\b)' \
        && return 1
    return 0
}

quality_gate_joke() {
    printf '%s' "$1" | grep -qiE \
        '(programmier|programming bug|\bcoffee\b|\bkaffee\b|artificial intelligence|\bchatgpt\b|\bopenai\b|\bclaude\b|\bdeepseek\b)' \
        && return 1
    return 0
}

# ─── Spinner ─────────────────────────────────────────────────────
# Usage: spin PID MESSAGE
spin() {
    local pid=$1
    local msg="${2:-Working...}"
    local spin_chars="⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"
    local i=0
    while kill -0 "$pid" 2>/dev/null; do
        local char="${spin_chars:$((i % ${#spin_chars})):1}"
        printf "\r  ${C_CYAN}%s${C_RESET} %s" "$char" "$msg" >&2
        i=$((i + 1))
        sleep 0.1
    done
    printf "\r  ${C_GREEN}✓${C_RESET} %s\n" "$msg" >&2
}

# ─── Parse structured skill output (NAME:/COST:/TYPE: blocks) ─────
# Reads LLM text from stdin; prints shell assignments for card fields.
parse_structured_card_fields() {
    python3 -c "$(cat <<'PY'
import re
import shlex
import sys

text = sys.stdin.read()
fields = {}

anchors = list(re.finditer(r"^NAME:\s*.+$", text, re.MULTILINE | re.IGNORECASE))
if anchors:
    block = text[anchors[-1].start():]
    for key in ("NAME", "COST", "TYPE", "RARITY", "RULES", "FLAVOR", "PT"):
        match = re.search(rf"^{key}:\s*(.*)$", block, re.MULTILINE | re.IGNORECASE)
        if match:
            fields[key] = match.group(1).strip()

if not fields.get("NAME"):
    match = re.search(r"^\*\*(.+?)\*\*\s*$", text, re.MULTILINE)
    if match:
        fields["NAME"] = match.group(1).strip()

if not fields.get("COST"):
    match = re.search(r"(\{[WUBRG\d/]+\}(?:\{[WUBRG\d/]+\})*)", text)
    if match:
        fields["COST"] = match.group(1)

if not fields.get("TYPE"):
    match = re.search(
        r"^((?:Legendary\s+)?(?:Creature|Instant|Sorcery|Enchantment|Artifact)(?:\s*[—–-]\s*.+)?)\s*$",
        text,
        re.MULTILINE | re.IGNORECASE,
    )
    if match:
        fields["TYPE"] = match.group(1).strip()

for key, value in fields.items():
    shell_key = {
        "NAME": "CARD_NAME",
        "COST": "CARD_COST",
        "TYPE": "CARD_TYPELINE",
        "RARITY": "CARD_RARITY",
        "RULES": "CARD_RULES",
        "FLAVOR": "CARD_FLAVOR",
        "PT": "CARD_PT",
    }.get(key, key)
    print(f"{shell_key}={shlex.quote(value)}")
PY
)"
}

# ─── LLM Call with Spinner ───────────────────────────────────────
llm_call_pretty() {
    local system_prompt="$1"
    local user_prompt="$2"
    local spin_msg="${3:-Channeling chaos...}"
    local temp="${4:-$LLM_TEMPERATURE}"
    local max_tok="${5:-$LLM_MAX_TOKENS}"

    # Run LLM call in background
    local tmpfile
    tmpfile=$(mktemp)
    (llm_call "$system_prompt" "$user_prompt" "$temp" "$max_tok" > "$tmpfile") &
    local bg_pid=$!
    spin $bg_pid "$spin_msg"
    wait $bg_pid
    local result
    result=$(cat "$tmpfile")
    rm -f "$tmpfile"
    echo "$result"
}
