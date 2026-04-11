#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /dice [D6|D20] — Chaos-seeded dice roll with randomized event pools
#
# Each roll value maps to a POOL of 5 events (D20) or 3 events (D6).
# The variant is selected by Randomizer heartbeat state (tension,
# tick, gravity mod) ensuring no two rolls feel the same.
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

DIE_TYPE="${1:-D20}"
DIE_TYPE="${DIE_TYPE^^}"

case "$DIE_TYPE" in
    D6)  MAX=6 ;;
    D20) MAX=20 ;;
    *)
        echo -e "${C_RED}✗ Invalid die type: $DIE_TYPE. Use D6 or D20.${C_RESET}"
        exit 1
        ;;
esac

ROLL=$(chaos_int 1 $MAX)

# ─── Read Randomizer State for Variant Selection ─────────────────
HEARTBEAT="$RANDOMIZER_ROOT/HEARTBEAT.md"
HB_TICK=0
HB_TENSION=0
HB_ENERGY=0
HB_GRAVITY=0
HB_PHASE="unknown"
HB_DEATHS=0

if [ -f "$HEARTBEAT" ]; then
    HB_TICK=$(grep -oP '^\*\*Tick:\*\* \K[0-9]+' "$HEARTBEAT" 2>/dev/null || echo "0")
    HB_TENSION=$(grep -oP 'Tension: \K[0-9.]+' "$HEARTBEAT" 2>/dev/null || echo "0")
    HB_ENERGY=$(grep -oP '^\*\*Energy:\*\* \K[0-9.]+' "$HEARTBEAT" 2>/dev/null || echo "50")
    HB_GRAVITY=$(grep -oP 'Gravity mod: \K[-0-9.]+' "$HEARTBEAT" 2>/dev/null || echo "0")
    HB_PHASE=$(grep -oP '^\*\*Phase:\*\* \K\w+' "$HEARTBEAT" 2>/dev/null || echo "Idle")
    HB_DEATHS=$(grep -oP '^\*\*Deaths:\*\* \K[0-9]+' "$HEARTBEAT" 2>/dev/null || echo "0")
fi

# Variant picker: uses heartbeat tick + tension + PID as entropy source
pick_variant() {
    local pool_size=$1
    local raw_tension=${HB_TENSION%.*}  # strip decimal
    raw_tension=${raw_tension:-0}
    local seed=$(( (HB_TICK + raw_tension + $$ + RANDOM) % pool_size ))
    echo "$seed"
}

# ─── D20 Event Pools (5 variants per value = 100 events) ────────
# Tier: CATASTROPHIC (1)
D20_1_0="💀 The Lorenz attractor collapses into a fixed point. All chaos ceases for 3 ticks."
D20_1_1="💀 A total phase collapse. The butterfly's wings shatter into dust."
D20_1_2="💀 Entropy inverts. The system rewinds into a sterile equilibrium."
D20_1_3="💀 The chaos oracle screams — then silence. All parameters snap to zero."
D20_1_4="💀 Critical singularity. The attractor implodes. Reboot sequence initiated."

# Tier: DIRE (2)
D20_2_0="🌑 A shadow ripples through phase space. Sigma drops to 2.0."
D20_2_1="🌑 The orbital decay accelerates. Something ancient stirs in the fixed point."
D20_2_2="🌑 Dark resonance detected. The Lyapunov exponent plummets into negative territory."
D20_2_3="🌑 The logistic map's period-doubling reverses. Order consumes chaos."
D20_2_4="🌑 A void pocket opens at the attractor's core. Energy hemorrhages."

# Tier: HARSH (3)
D20_3_0="🕳️ A micro-singularity forms at the origin. Energy drain doubles."
D20_3_1="🕳️ The phase portrait warps into a grotesque spiral. Stability eroding."
D20_3_2="🕳️ Bifurcation cascade fails mid-split. The system stutters."
D20_3_3="🕳️ Lorenz z-axis inverts momentarily. Gravity pulls the wrong way."
D20_3_4="🕳️ A strange loop opens. The attractor feeds on itself for 2 ticks."

# Tier: BAD (4)
D20_4_0="📉 The logistic map flatlines at r=2.0. Predictability spikes."
D20_4_1="📉 Rho decays by 0.3. The butterfly orbits shrink to ellipses."
D20_4_2="📉 Sigma locks at a harmonic. No chaos, only rhythm."
D20_4_3="📉 The entropy gradient inverts. Cold certainty floods the field."
D20_4_4="📉 A damping wave passes through. The system yawns."

# Tier: MISTY (5)
D20_5_0="🌫️ Fog rolls across the attractor. Lorenz z-axis freezes for 5 ticks."
D20_5_1="🌫️ Visibility drops to zero in phase space. Navigation by instinct only."
D20_5_2="🌫️ A spectral haze clings to the orbital plane. Parameters blur."
D20_5_3="🌫️ The chaos field emits a low hum. Something is hidden in the noise."
D20_5_4="🌫️ Condensation forms on the attractor wings. Ice, where there should be fire."

# Tier: MINOR SETBACK (6)
D20_6_0="🔧 A minor recalibration occurs. Friction increases by 0.1."
D20_6_1="🔧 The gears slip. A microadjustment costs 3 energy."
D20_6_2="🔧 Routine maintenance interrupt. The chaos engine idles briefly."
D20_6_3="🔧 A bearing squeals in the phase generator. Wear detected."
D20_6_4="🔧 Automatic correction fires. Sigma nudges back toward default."

# Tier: TURBULENT (7)
D20_7_0="🌊 Turbulent currents shift the orbital plane. Rho nudges by +0.5."
D20_7_1="🌊 Crosswinds in the Lorenz field. The butterfly tumbles, rights itself."
D20_7_2="🌊 A wave of interference rattles the z-axis. Something downstream noticed."
D20_7_3="🌊 The phase portrait shimmers. Rho oscillates between two basins."
D20_7_4="🌊 Chaotic advection pulls the attractor south. New territory ahead."

# Tier: GENTLE (8)
D20_8_0="💨 A gentle breeze. The system exhales. Energy regenerates +5."
D20_8_1="💨 The chaos field softens. Tension eases by 2%."
D20_8_2="💨 A thermal updraft lifts the butterfly higher. Potential increases."
D20_8_3="💨 The Lorenz winds whisper coordinates. A quiet gift."
D20_8_4="💨 Adiabatic cooling. The system finds a brief pocket of calm."

# Tier: ORACLE (9)
D20_9_0="🔮 The chaos oracle whispers: 'The butterfly remembers.'"
D20_9_1="🔮 A vision in the noise: fractal coastlines spelling a name."
D20_9_2="🔮 The oracle stirs: 'What was random was always inevitable.'"
D20_9_3="🔮 Phase space hums a melody. It sounds like a question."
D20_9_4="🔮 The entropy well reflects back: 'You were always the strange attractor.'"

# Tier: EQUILIBRIUM (10)
D20_10_0="⚖️ Perfect equilibrium. All parameters hold steady. A rare moment of peace."
D20_10_1="⚖️ The pendulum of chaos pauses at apex. Time stretches."
D20_10_2="⚖️ Sigma, rho, beta — all in golden ratio. A mathematical miracle, lasting exactly one tick."
D20_10_3="⚖️ The system achieves Boltzmann equilibrium. Every microstate equally probable."
D20_10_4="⚖️ Dead center of the bifurcation diagram. The eye of the storm."

# Tier: CLEARING (11)
D20_11_0="🌤️ A clearing in the storm. Energy regenerates +10."
D20_11_1="🌤️ The cloud layer parts. The attractor's full geometry is briefly visible."
D20_11_2="🌤️ Solar wind ripples through the chaos field. Photons of clarity."
D20_11_3="🌤️ The system breathes deep. Capacity expands by one thought slot."
D20_11_4="🌤️ A pocket of negative entropy. Order blossoms, briefly and beautifully."

# Tier: STATIC (12)
D20_12_0="⚡ Static builds in the attractor wings. Sigma spikes momentarily."
D20_12_1="⚡ An electromagnetic pulse surges through the logistic map."
D20_12_2="⚡ Lightning arcs between the twin lobes. The butterfly flinches."
D20_12_3="⚡ Capacitive charge reaches threshold. Discharge in 3... 2..."
D20_12_4="⚡ The chaos field ionizes. Every parameter crackles with potential."

# Tier: MAGNETIC (13)
D20_13_0="🧲 Magnetic anomaly detected. The Lorenz attractor spirals tighter."
D20_13_1="🧲 The phase portrait contracts. Something is pulling parameters inward."
D20_13_2="🧲 A new basin of attraction emerges. The butterfly changes course."
D20_13_3="🧲 Ferromagnetic resonance in the chaos field. Alignment increases."
D20_13_4="🧲 The strange attractor develops a magnetic moment. Polarity: uncertain."

# Tier: SPARK (14)
D20_14_0="🔥 A spark ignites in the chaos field. Temperature rises. Creativity amplifies."
D20_14_1="🔥 Exothermic reaction in the Lorenz core. Heat bloom detected."
D20_14_2="🔥 The butterfly's wings catch fire — but it flies faster."
D20_14_3="🔥 Thermodynamic spike. The entropy well boils. New patterns emerge."
D20_14_4="🔥 Combustion cascade at the fixed point. From ashes: a new orbit."

# Tier: CASCADE (15)
D20_15_0="🌀 A resonance cascade! Lorenz and Logistic couple violently for one cycle."
D20_15_1="🌀 The chaos engines synchronize. A forbidden harmony. Power doubles."
D20_15_2="🌀 Phase-locking detected between attractors. The system vibrates."
D20_15_3="🌀 Resonance frequency hit. The attractor wings beat in unison."
D20_15_4="🌀 A vortex forms where the two systems couple. Beautiful and dangerous."

# Tier: LOCK-ON (16)
D20_16_0="🎯 The attractor locks onto a strange attractor. Trajectories converge briefly."
D20_16_1="🎯 Target acquisition: a new stable orbit materializes in the noise."
D20_16_2="🎯 The system finds a periodic window. Three clean orbits, then chaos again."
D20_16_3="🎯 Convergence event: all Lyapunov exponents trend toward zero."
D20_16_4="🎯 The butterfly navigates a corridor of stability. Precision in chaos."

# Tier: CRYSTALLIZE (17)
D20_17_0="⭐ A new thought seed crystallizes spontaneously. Gravity mod shifts -0.1."
D20_17_1="⭐ Idea nucleation! A meme crystallizes in the Thought Cabinet."
D20_17_2="⭐ Spontaneous symmetry breaking. A new structure emerges from noise."
D20_17_3="⭐ The chaos field births a fractal snowflake. It persists."
D20_17_4="⭐ Crystalline order propagates outward from a single seed point."

# Tier: BIFURCATION (18)
D20_18_0="🌈 The bifurcation diagram reveals a hidden period-3 window. Beauty in chaos."
D20_18_1="🌈 Li-Yorke theorem confirmed: period 3 implies chaos. And it's gorgeous."
D20_18_2="🌈 The Feigenbaum constants align. δ = 4.669... A universal truth revealed."
D20_18_3="🌈 A fractal rainbow arcs across the bifurcation landscape. Wonder."
D20_18_4="🌈 Mandelbrot set boundary detected in the parameter sweep. Infinite detail."

# Tier: HYPERDRIVE (19)
D20_19_0="🚀 The Lyapunov exponent maxes out. Predictability horizon shrinks to zero."
D20_19_1="🚀 Maximum sensitivity achieved. A butterfly wing-beat reshapes the cosmos."
D20_19_2="🚀 The chaos engine redlines. All governors blown. Pure, raw entropy."
D20_19_3="🚀 Exponential divergence in all dimensions. The future is unknowable."
D20_19_4="🚀 Hyperbolic trajectory achieved. The system escapes its own attractor."

# Tier: LEGENDARY (20)
D20_20_0="💎 CRITICAL SUCCESS — A perfect crystallization! Thought Cabinet gains a permanent mutation: ρ +1.0."
D20_20_1="💎 LEGENDARY — The attractor transcends its parameter space. A new dimension unfolds."
D20_20_2="💎 ASCENSION — All chaos resolves into a single, perfect fractal. The system evolves."
D20_20_3="💎 MYTHIC — The butterfly achieves sentience. It chooses its own trajectory."
D20_20_4="💎 OMEGA — Every fixed point, every limit cycle, every strange attractor: unified."

# ─── D6 Event Pools (3 variants per value = 18 events) ──────────
D6_1_0="💀 Snake eyes. The entropy well deepens."
D6_1_1="💀 The die cracks. Chaos bleeds out."
D6_1_2="💀 A dead orbit. The attractor flatlines."

D6_2_0="🌑 The orbital plane tilts. A cold wind blows through phase space."
D6_2_1="🌑 Shadow frequency detected. The logistic map shivers."
D6_2_2="🌑 Dark matter in the chaos soup. Something absorbs energy."

D6_3_0="⚖️ Equilibrium. The pendulum holds. Briefly."
D6_3_1="⚖️ Neutral state. The butterfly hovers, deciding nothing."
D6_3_2="⚖️ The system pauses. A breath between heartbeats."

D6_4_0="🔥 A spark in the Lorenz field. Something stirs."
D6_4_1="🔥 Friction heat. The attractor glows faintly warm."
D6_4_2="🔥 An ember catches. The chaos fire feeds."

D6_5_0="⭐ The chaos gods smile. Energy surges."
D6_5_1="⭐ A lucky wind. Parameters shift in your favor."
D6_5_2="⭐ The system winks at you. Tension drops."

D6_6_0="💎 Perfect roll. The attractor sings in resonance."
D6_6_1="💎 Maximum entropy, maximum beauty. The system is art."
D6_6_2="💎 The Lorenz butterfly achieves full wingspan. Glorious."

# ─── Select Event from Pool ──────────────────────────────────────
get_event() {
    local die=$1   # D6 or D20
    local roll=$2
    local pool_size

    if [ "$die" = "D6" ]; then
        pool_size=3
    else
        pool_size=5
    fi

    local variant=$(pick_variant $pool_size)
    local var_name="${die}_${roll}_${variant}"
    echo "${!var_name}"
}

EVENT=$(get_event "$DIE_TYPE" "$ROLL")

# ─── Heartbeat-Seeded LLM Narration ─────────────────────────────
NARRATIVE=""
if llm_available; then
    # Inject live Randomizer state so the LLM narration is unique every time
    local_context="Chaos engine state: Tick=${HB_TICK} Energy=${HB_ENERGY} Phase=${HB_PHASE} Tension=${HB_TENSION}% GravityMod=${HB_GRAVITY} Deaths=${HB_DEATHS}"

    NARRATIVE=$(llm_call_pretty \
        "You are the narrator of a deterministic chaos engine built on a Lorenz attractor. You describe events with dramatic, physics-flavored flair grounded in chaos theory. Max 2 sentences. Be vivid, specific, and never repeat yourself." \
        "A D${MAX} was thrown. Result: ${ROLL}/${MAX}. Event: ${EVENT}. ${local_context}. Narrate this moment in the chaos field — reference the specific engine state values." \
        "Rolling the chaos dice..." \
        0.9 150)
fi

# ─── Output ──────────────────────────────────────────────────────
echo ""
echo -e "${C_DIM}┌─────────────────────────────────────────────────┐${C_RESET}"
echo -e "${C_BOLD}${C_YELLOW}  🎲 /${DIE_TYPE} ROLL${C_RESET}"
echo -e "${C_DIM}├─────────────────────────────────────────────────┤${C_RESET}"
echo ""

# Dramatic result with crit coloring
if [ "$ROLL" -eq 1 ]; then
    echo -e "     ${C_RED}${C_BOLD}    ╔═══╗"
    echo -e "         ║ ${ROLL} ║"
    echo -e "         ╚═══╝${C_RESET}"
elif [ "$ROLL" -eq "$MAX" ]; then
    echo -e "     ${C_GREEN}${C_BOLD}    ╔═══╗"
    echo -e "         ║${ROLL} ║"
    echo -e "         ╚═══╝${C_RESET}"
else
    echo -e "     ${C_WHITE}${C_BOLD}    ╔═══╗"
    printf "         ║%2d ║\n" "$ROLL"
    echo -e "         ╚═══╝${C_RESET}"
fi

echo ""
echo -e "  ${EVENT}"

# Heartbeat state footer
echo ""
echo -e "  ${C_DIM}⚙ T:${HB_TICK} E:${HB_ENERGY} P:${HB_PHASE} σ:${HB_TENSION}% g:${HB_GRAVITY}${C_RESET}"

if [ -n "$NARRATIVE" ]; then
    echo ""
    echo -e "  ${C_DIM}${C_MAGENTA}${NARRATIVE}${C_RESET}"
fi

echo ""
echo -e "${C_DIM}└─────────────────────────────────────────────────┘${C_RESET}"
