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
LLM_URL="${GZMO_LLM_URL:-http://localhost:8000/v1/chat/completions}"
LLM_MODEL="${GZMO_LLM_MODEL:-bartowski/Qwen2.5-7B-Instruct-GGUF}"
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

# ─── Call LLM ────────────────────────────────────────────────────
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
    sys_escaped=$(echo "$full_system" | jq -Rs '.')
    local usr_escaped
    usr_escaped=$(echo "$user_prompt" | jq -Rs '.')

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
    response=$(curl -s --connect-timeout 5 --max-time 30 \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "$LLM_URL" 2>/dev/null)

    if [ $? -ne 0 ] || [ -z "$response" ]; then
        echo ""
        return 1
    fi

    # Extract the message content
    local content
    content=$(echo "$response" | jq -r '.choices[0].message.content // empty' 2>/dev/null)

    if [ -z "$content" ]; then
        echo ""
        return 1
    fi

    echo "$content"
    return 0
}

# ─── Check LLM Availability ─────────────────────────────────────
llm_available() {
    curl -s --connect-timeout 2 --max-time 3 \
        "${LLM_URL%/chat/completions}/models" >/dev/null 2>&1
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
# Strips thinking-channel wrappers, quotes/backticks; trims outer blank lines.
strip_thinking_channels() {
    local text="$1"
    local line stripped out=""

    while IFS= read -r line || [ -n "$line" ]; do
        line="${line#"${line%%[![:space:]]*}"}"
        line="${line%"${line##*[![:space:]]}"}"
        case "$line" in
            "<|channel>thought"|"<channel>thought") continue ;;
        esac
        stripped="${line#<|channel|>}"
        stripped="${stripped#<channel|>}"
        stripped="${stripped#"${stripped%%[![:space:]]*}"}"
        stripped="${stripped%"${stripped##*[![:space:]]}"}"
        if [ -n "$stripped" ]; then
            if [ -n "$out" ]; then
                out="${out}"$'\n'"${stripped}"
            else
                out="$stripped"
            fi
        fi
    done <<< "$text"
    printf '%s' "$out"
}

clean_llm_output() {
    local text="$1"
    text=$(strip_thinking_channels "$text")
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
        '(programmier|programming bug|\bcoffee\b|\bkaffee\b|artificial intelligence|\bchatgpt\b|\bki[- ]|dad joke|flachwitz|montagmorgen|\bwlan\b|\bwifi\b|\bbug\b)' \
        && return 1
    return 0
}

quality_gate_story() {
    printf '%s' "$1" | grep -qiE \
        '(once upon a time|es war einmal|happily ever after|und sie lebten|moral of the story|lehre des|m[aä]rchen)' \
        && return 1
    return 0
}

quality_gate_word() {
    printf '%s' "$1" | grep -q '^WORD:' || return 1
    printf '%s' "$1" | grep -qiE \
        '(wordsmith|neologism of the day|made-up word:|fake word:|lorem ipsum)' \
        && return 1
    return 0
}

quality_gate_card() {
    printf '%s' "$1" | grep -q '^NAME:' || return 1
    printf '%s' "$1" | grep -qiE \
        '(as an ai|i cannot|placeholder|lorem ipsum|\[card name\])' \
        && return 1
    return 0
}

quality_gate_define() {
    printf '%s' "$1" | grep -q '^DEFINITION:' || return 1
    printf '%s' "$1" | grep -qiE \
        '(as an ai|i don.t know|cannot define|no definition|lorem ipsum)' \
        && return 1
    return 0
}

# ─── Accept Creative Output ──────────────────────────────────────
# Usage: accept_creative_output TEXT MAX_CHARS quality_gate_fn
accept_creative_output() {
    local text="$1"
    local max_chars="$2"
    local gate_fn="$3"
    local count

    count=$(char_count "$text")
    if [ "$count" -le 0 ] || [ "$count" -gt "$max_chars" ]; then
        return 1
    fi
    "$gate_fn" "$text"
}

# ─── Pretty Box ──────────────────────────────────────────────────
print_box() {
    local title="$1"
    local content="$2"
    local icon="${3:-⚡}"
    local color="${4:-$C_CYAN}"

    echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
    echo -e "${C_BOLD}${color}  $icon $title${C_RESET}"
    echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
    echo -e "  $content"
    echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
}

# ─── Spinner (for LLM calls) ────────────────────────────────────
spin() {
    local pid=$1
    local msg="${2:-Thinking...}"
    local frames=("⠋" "⠙" "⠹" "⠸" "⠼" "⠴" "⠦" "⠧" "⠇" "⠏")
    local i=0
    while kill -0 "$pid" 2>/dev/null; do
        printf "\r  ${C_DIM}${frames[$i]} $msg${C_RESET}" >&2
        i=$(( (i + 1) % ${#frames[@]} ))
        sleep 0.1
    done
    printf "\r%*s\r" 60 "" >&2
}

# ─── Chaos feedback bridge (stderr → gzmo-core shell_bridge.rs) ──
# Shell skills emit structured events so Thought Cabinet can absorb output.
emit_chaos_event_json() {
    printf 'GZMO_CHAOS_EVENT:%s\n' "$1" >&2
}

emit_joke() {
    emit_chaos_event_json "$(jq -nc --arg t "$1" '{type:"JokeGenerated",text:$t}')"
}

emit_poem() {
    emit_chaos_event_json "$(jq -nc --arg t "$1" '{type:"PoemGenerated",text:$t}')"
}

emit_story() {
    emit_chaos_event_json "$(jq -nc --arg t "$1" '{type:"StoryGenerated",text:$t}')"
}

emit_card() {
    emit_chaos_event_json "$(jq -nc --arg n "$1" --arg ct "$2" '{type:"CardForged",name:$n,card_type:$ct}')"
}

emit_word() {
    emit_chaos_event_json "$(jq -nc --arg w "$1" --arg d "$2" '{type:"WordGenerated",word:$w,definition:$d}')"
}

emit_persona_shift() {
    emit_chaos_event_json "$(jq -nc --arg p "$1" '{type:"PersonaShift",persona:$p}')"
}

emit_persona_cleared() {
    emit_chaos_event_json '{"type":"PersonaCleared"}'
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
