#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /sound — Full-frame ASCII sound visualizer with heartbeat-reactive
#          category selection and matched system audio
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

# ─── Read Heartbeat for Category Selection ───────────────────────
HEARTBEAT="$RANDOMIZER_ROOT/HEARTBEAT.md"
HB_TENSION=50
HB_ENERGY=50
HB_PHASE="Idle"

if [ -f "$HEARTBEAT" ]; then
    HB_TENSION=$(grep -oP 'Tension: \K[0-9.]+' "$HEARTBEAT" 2>/dev/null || echo "50")
    HB_ENERGY=$(grep -oP '^\*\*Energy:\*\* \K[0-9.]+' "$HEARTBEAT" 2>/dev/null || echo "50")
    HB_PHASE=$(grep -oP '^\*\*Phase:\*\* \K\w+' "$HEARTBEAT" 2>/dev/null || echo "Idle")
fi

TENSION_INT=${HB_TENSION%.*}
TENSION_INT=${TENSION_INT:-50}

# ─── ANSI Colors ─────────────────────────────────────────────────
R="\033[31m"
G="\033[32m"
Y="\033[33m"
B="\033[34m"
M="\033[35m"
C="\033[36m"
W="\033[97m"
BLD="\033[1m"
DIM="\033[2m"
RST="\033[0m"
BG_BLK="\033[40m"
OR="\033[38;2;255;140;0m"  # orange
PK="\033[38;2;255;105;180m" # pink
SK="\033[38;2;135;206;235m" # sky blue
GLD="\033[38;2;212;175;55m" # gold

# ─── Sound Categories (heartbeat-reactive) ───────────────────────
# High tension (>60%) = aggressive sounds
# Mid tension (30-60%) = ambient/neutral
# Low tension (<30%) = calm/ethereal
if [ "$TENSION_INT" -gt 60 ]; then
    CATEGORY_POOL=("explosion" "thunder" "alarm" "roar")
elif [ "$TENSION_INT" -gt 30 ]; then
    CATEGORY_POOL=("bell" "guitar" "drum" "wave")
else
    CATEGORY_POOL=("chime" "piano" "wind" "hum")
fi

# Pick category
CAT_IDX=$(chaos_int 0 $(( ${#CATEGORY_POOL[@]} - 1 )))
CATEGORY="${CATEGORY_POOL[$CAT_IDX]}"

# ─── Animation Frame Delay ───────────────────────────────────────
frame_delay() { sleep "${1:-0.06}"; }
clear_frame() { printf "\033[${1}A" 2>/dev/null; }

# ─── VISUAL RENDERERS ───────────────────────────────────────────

render_explosion() {
    local name="BOOM"
    local desc="A thunderous detonation rips through the phase space"

    echo ""
    echo -e "  ${DIM}${R}                    .  .${RST}"
    frame_delay
    echo -e "  ${R}               . .  :  . .${RST}"
    frame_delay
    echo -e "  ${OR}            .  :  .  .  :  .${RST}"
    frame_delay
    echo -e "  ${Y}          .  :  . ${BLD}${W}💥${RST}${Y} .  :  .${RST}"
    frame_delay
    echo -e "  ${OR}         : .  . : . : .  . :${RST}"
    frame_delay
    echo -e "  ${R}          . .  : ${BLD}BOOM${RST}${R} :  . .${RST}"
    frame_delay
    echo -e "  ${OR}            .  :  .  .  :  .${RST}"
    frame_delay
    echo -e "  ${R}               . .  :  . .${RST}"
    frame_delay
    echo -e "  ${DIM}${R}                    .  .${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_thunder() {
    local name="KRAKOOM"
    local desc="Lightning splits the bifurcation diagram clean in half"

    echo ""
    echo -e "  ${DIM}${B}  ░░░▒▒▒▓▓▓████▓▓▓▒▒▒░░░░░▒▒▓▓████▓▓▒▒░░${RST}"
    frame_delay
    echo -e "  ${W}${BLD}                  ╲${RST}"
    frame_delay
    echo -e "  ${Y}${BLD}                   ╲${RST}"
    frame_delay
    echo -e "  ${Y}${BLD}                  ╱╲${RST}"
    frame_delay
    echo -e "  ${W}${BLD}                 ╱${RST}"
    frame_delay
    echo -e "  ${Y}${BLD}                ╱${RST}"
    frame_delay
    echo -e "  ${Y}${BLD}               ╲╱${RST}"
    frame_delay
    echo -e "  ${W}${BLD}                ╲   ⚡ KRAKOOM${RST}"
    frame_delay
    echo -e "  ${Y}${BLD}                 ╲${RST}"
    frame_delay
    echo -e "  ${DIM}${Y}                  ┊${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_alarm() {
    local name="ALERT"
    local desc="The chaos engine redlines — all governors blown"

    echo ""
    for i in 1 2 3; do
        echo -e "  ${R}${BLD}  ▐██████████████████████████████████████▌${RST}"
        echo -e "  ${W}${BLD}  ▐█  ░░ ${R}▓▓ ${W}░░ ${R}▓▓ ${W}░░ ALERT ░░ ${R}▓▓ ${W}░░ ${R}▓▓ ${W}░░ █▌${RST}"
        echo -e "  ${R}${BLD}  ▐██████████████████████████████████████▌${RST}"
        frame_delay 0.15
        echo -e "  ${W}${BLD}  ▐██████████████████████████████████████▌${RST}"
        echo -e "  ${R}${BLD}  ▐█  ▓▓ ${W}░░ ${R}▓▓ ${W}░░ ${R}▓▓ ALERT ▓▓ ${W}░░ ${R}▓▓ ${W}░░ ${R}▓▓ █▌${RST}"
        echo -e "  ${W}${BLD}  ▐██████████████████████████████████████▌${RST}"
        frame_delay 0.15
    done
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_roar() {
    local name="RAWWWR"
    local desc="Something ancient stirs in the attractor's core"

    echo ""
    echo -e "  ${R}${BLD}          ╭──────────────────╮${RST}"
    frame_delay
    echo -e "  ${OR}${BLD}         ╱ ▲  ▲  ▲  ▲  ▲  ▲ ╲${RST}"
    frame_delay
    echo -e "  ${R}${BLD}        ╱                      ╲${RST}"
    frame_delay
    echo -e "  ${OR}${BLD}       │   ●              ●    │${RST}"
    frame_delay
    echo -e "  ${R}${BLD}       │          ◆◆           │${RST}"
    frame_delay
    echo -e "  ${OR}${BLD}       │     ╲____________╱    │${RST}"
    frame_delay
    echo -e "  ${R}${BLD}        ╲  ▼  ▼  ▼  ▼  ▼  ▼  ╱${RST}"
    frame_delay
    echo -e "  ${OR}${BLD}         ╰──────────────────╯${RST}"
    echo -e "  ${R}${BLD}          R  A  W  W  W  R${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_bell() {
    local name="DING"
    local desc="A crystalline bell chime echoes from the attractor core"

    echo ""
    echo -e "  ${GLD}                   ╱╲${RST}"
    frame_delay
    echo -e "  ${GLD}                  ╱  ╲${RST}"
    frame_delay
    echo -e "  ${GLD}                 ╱    ╲${RST}"
    frame_delay
    echo -e "  ${GLD}${BLD}                │ ░░░░ │${RST}"
    frame_delay
    echo -e "  ${GLD}${BLD}                │ ▓▓▓▓ │${RST}"
    frame_delay
    echo -e "  ${GLD}${BLD}               ╱ ██${W}◉${GLD}██ ╲${RST}"
    frame_delay
    echo -e "  ${GLD}${BLD}              ╱________╲${RST}"
    frame_delay
    echo -e "  ${Y}          ─ ─ ─ ${BLD}D I N G${RST}${Y} ─ ─ ─${RST}"
    echo -e "  ${DIM}${Y}            ∿  ∿  ∿  ∿  ∿  ∿${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_guitar() {
    local name="TWANG"
    local desc="A cosmic string vibrates at the Feigenbaum frequency"

    echo ""
    echo -e "  ${OR}${BLD}              ┌─────┐${RST}"
    frame_delay
    echo -e "  ${OR}              │ ○ ○ ○│${RST}"
    frame_delay
    echo -e "  ${OR}              │     ╱│${RST}"
    echo -e "  ${OR}              │    ╱ │${RST}"
    echo -e "  ${OR}              │   ╱  │${RST}"
    frame_delay
    echo -e "  ${OR}${BLD}              │  ╱   │${RST}"
    echo -e "  ${OR}${BLD}              │ ┃ ◎  │${RST}"
    echo -e "  ${OR}${BLD}              │ ┃    │${RST}"
    frame_delay
    echo -e "  ${OR}              └──┃──┘${RST}"
    echo -e "  ${Y}        ∼∼∼∿∿∿${BLD} TWANG ${RST}${Y}∿∿∿∼∼∼${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_drum() {
    local name="BA DUM TSS"
    local desc="The chaos engine drops a rimshot — perfect timing"

    echo ""
    echo -e "  ${W}${BLD}       ╭─────────────────────────╮${RST}"
    frame_delay
    echo -e "  ${W}       │░░░░░░░░░░░░░░░░░░░░░░░░░│${RST}"
    frame_delay
    echo -e "  ${W}${BLD}       │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│${RST}"
    frame_delay
    echo -e "  ${W}       ╰┬───────────────────────┬╯${RST}"
    frame_delay
    echo -e "  ${W}        │  ${OR}╱${W}               ${OR}╲${W}  │${RST}"
    echo -e "  ${W}        │ ${OR}╱${W}                 ${OR}╲${W} │${RST}"
    frame_delay
    echo -e "  ${W}        ╰─────────────────────────╯${RST}"
    echo ""
    echo -e "  ${Y}${BLD}          BA${RST}  ${OR}${BLD}DUM${RST}  ${R}${BLD}TSS${RST}  🥁"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_wave() {
    local name="CRASHHH"
    local desc="A wave of entropy crashes against the system's shore"

    echo ""
    echo -e "  ${B}${DIM}                                    ░░${RST}"
    frame_delay
    echo -e "  ${B}                                  ░░▒▒${RST}"
    frame_delay
    echo -e "  ${SK}                               ░░▒▒▓▓${RST}"
    frame_delay
    echo -e "  ${C}${BLD}                    ╱╲       ░░▒▒▓▓██${RST}"
    frame_delay
    echo -e "  ${SK}${BLD}               ╱╲╱  ╲╱╲  ░░▒▒▓▓████${RST}"
    frame_delay
    echo -e "  ${B}${BLD}          ╱╲╱╲╱       ╲╱▒▒▓▓████████${RST}"
    frame_delay
    echo -e "  ${C}  ~~~~~~╱                ▓▓██████████████${RST}"
    frame_delay
    echo -e "  ${DIM}${B}  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████████████████████${RST}"
    echo ""
    echo -e "  ${C}${BLD}               C R A S H H H${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_chime() {
    local name="✧ CHIME ✧"
    local desc="Windchimes stir in the entropy breeze"

    echo ""
    echo -e "  ${SK}${DIM}            ═══════════════════${RST}"
    frame_delay
    echo -e "  ${SK}              │   │   │   │${RST}"
    frame_delay
    echo -e "  ${W}              ┃   ╽   ╿   ┃${RST}"
    frame_delay
    echo -e "  ${SK}              ┃   ╽   ╿   ┃${RST}"
    frame_delay
    echo -e "  ${W}              ○   ◇   ○   ◇${RST}"
    echo ""
    echo -e "  ${SK}       ✧  ✧  ✧ ${BLD}CHIME${RST}${SK} ✧  ✧  ✧${RST}"
    echo -e "  ${DIM}${SK}         ∿    ∿    ∿    ∿    ∿${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_piano() {
    local name="PLINK"
    local desc="A single key pressed by the ghost of Edward Lorenz"

    echo ""
    echo -e "  ${W}${BLD}  ┌─┬─┬┬─┬─┬─┬─┬┬─┬┬─┬─┬─┬─┬┬─┬─┐${RST}"
    frame_delay
    echo -e "  ${W}  │ │ │${DIM}█${RST}${W}│ │ │ │${DIM}█${RST}${W}│${DIM}█${RST}${W}│ │ │ │${DIM}█${RST}${W}│ │${RST}"
    frame_delay
    echo -e "  ${W}  │ │ │${DIM}█${RST}${W}│ │ │ │${DIM}█${RST}${W}│${DIM}█${RST}${W}│ │ │ │${DIM}█${RST}${W}│ │${RST}"
    frame_delay
    echo -e "  ${W}  │ └┬┘└┬┘ │ └┬┘└┬┘└┬┘ │ └┬┘└┬┘ │${RST}"
    frame_delay
    echo -e "  ${W}  │  │  │  │  │  │  │  │  │  │  │${RST}"
    frame_delay
    echo -e "  ${W}  │  │  │${Y}${BLD}▓▓${RST}${W}│  │  │  │  │  │  │${RST}"
    frame_delay
    echo -e "  ${W}  └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘${RST}"
    echo ""
    echo -e "  ${GLD}${BLD}              P L I N K${RST}"
    echo -e "  ${DIM}${GLD}            ♩  ♪  ♫  ♬${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_wind() {
    local name="WHOOSH"
    local desc="A cold front sweeps through the Lorenz orbital plane"

    echo ""
    echo -e "  ${SK}${DIM}  ~ ~ ~ ~ ~ ~${RST}"
    frame_delay
    echo -e "  ${W}      ~ ~ ~ ~ ~ ~ ~ ~ ~${RST}"
    frame_delay
    echo -e "  ${SK}  ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~${RST}"
    frame_delay
    echo -e "  ${C}${BLD}       ≈ ≈ ≈ WHOOSH ≈ ≈ ≈${RST}"
    frame_delay
    echo -e "  ${SK}  ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~${RST}"
    frame_delay
    echo -e "  ${W}      ~ ~ ~ ~ ~ ~ ~ ~ ~${RST}"
    frame_delay
    echo -e "  ${SK}${DIM}  ~ ~ ~ ~ ~ ~${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

render_hum() {
    local name="HMMMMM"
    local desc="A billion logistic map iterations per second — pure resonance"

    echo ""
    echo -e "  ${M}${DIM}  ─────────────────────────────────────${RST}"
    frame_delay
    echo -e "  ${M}     ╱╲   ╱╲   ╱╲   ╱╲   ╱╲   ╱╲${RST}"
    frame_delay
    echo -e "  ${M}${BLD}    ╱  ╲ ╱  ╲ ╱  ╲ ╱  ╲ ╱  ╲ ╱  ╲${RST}"
    frame_delay
    echo -e "  ${PK}${BLD}   ╱    ╳    ╳    ╳    ╳    ╳    ╲${RST}"
    frame_delay
    echo -e "  ${M}${BLD}    ╲  ╱ ╲  ╱ ╲  ╱ ╲  ╱ ╲  ╱ ╲  ╱${RST}"
    frame_delay
    echo -e "  ${M}     ╲╱   ╲╱   ╲╱   ╲╱   ╲╱   ╲╱${RST}"
    frame_delay
    echo -e "  ${M}${DIM}  ─────────────────────────────────────${RST}"
    echo ""
    echo -e "  ${M}${BLD}          H  M  M  M  M  M${RST}"
    echo ""
    echo -e "  ${DIM}${desc}${RST}"
}

# ─── Audio Synthesis (matched to visual) ─────────────────────────
play_matched_audio() {
    local category="$1"

    # Try sox/play first (best audio synthesis)
    if command -v play &>/dev/null; then
        case "$category" in
            explosion) play -qn synth 0.4 noise vol 0.5 &>/dev/null & ;;
            thunder)   play -qn synth 0.6 brownnoise synth 0.1 sine 100 vol 0.4 &>/dev/null & ;;
            alarm)     play -qn synth 0.15 sine 880 synth 0.15 sine 660 repeat 3 vol 0.3 &>/dev/null & ;;
            roar)      play -qn synth 0.5 brownnoise tremolo 5 80 vol 0.4 &>/dev/null & ;;
            bell)      play -qn synth 0.8 sine 1200 fade 0 0.8 0.5 vol 0.3 &>/dev/null & ;;
            guitar)    play -qn synth 0.6 pluck 330 vol 0.4 &>/dev/null & ;;
            drum)      play -qn synth 0.05 noise synth 0.3 sine 80 vol 0.4 &>/dev/null & ;;
            wave)      play -qn synth 1.0 pinknoise tremolo 0.5 60 vol 0.3 &>/dev/null & ;;
            chime)     play -qn synth 0.5 sine 2000 fade 0 0.5 0.3 vol 0.2 &>/dev/null & ;;
            piano)     play -qn synth 0.5 pluck 440 vol 0.3 &>/dev/null & ;;
            wind)      play -qn synth 0.8 pinknoise vol 0.2 &>/dev/null & ;;
            hum)       play -qn synth 0.6 sine 220 tremolo 8 50 vol 0.2 &>/dev/null & ;;
        esac
        return
    fi

    # Fallback: try matching Yaru system sounds
    if command -v paplay &>/dev/null; then
        local SOUND_DIR="/usr/share/sounds/Yaru/stereo"
        case "$category" in
            explosion|thunder|alarm|roar)
                paplay "$SOUND_DIR/dialog-error.oga" &>/dev/null 2>&1 & ;;
            bell|chime)
                paplay "$SOUND_DIR/message-new-instant.oga" &>/dev/null 2>&1 & ;;
            drum|guitar)
                paplay "$SOUND_DIR/audio-volume-change.oga" &>/dev/null 2>&1 & ;;
            wave|wind|hum|piano)
                paplay "$SOUND_DIR/system-ready.oga" &>/dev/null 2>&1 & ;;
        esac
    fi
}

# ─── Dispatch Visual ─────────────────────────────────────────────
echo ""
echo -e "${DIM}┌─────────────────────────────────────────────────┐${RST}"
echo -e "${BLD}${Y}  🔊 SOUND EFFECT${RST}  ${DIM}[${CATEGORY}]${RST}"
echo -e "${DIM}├─────────────────────────────────────────────────┤${RST}"

# Play matched audio in background
play_matched_audio "$CATEGORY"

# Render the visual
case "$CATEGORY" in
    explosion) render_explosion ;;
    thunder)   render_thunder ;;
    alarm)     render_alarm ;;
    roar)      render_roar ;;
    bell)      render_bell ;;
    guitar)    render_guitar ;;
    drum)      render_drum ;;
    wave)      render_wave ;;
    chime)     render_chime ;;
    piano)     render_piano ;;
    wind)      render_wind ;;
    hum)       render_hum ;;
esac

# Heartbeat state footer
echo ""
echo -e "  ${DIM}⚙ Tension:${HB_TENSION}% Energy:${HB_ENERGY} Phase:${HB_PHASE}${RST}"
echo -e "${DIM}└─────────────────────────────────────────────────┘${RST}"
