//! # Dice Skill — `/dice`
//!
//! Chaos-driven dice rolls with narrative event pools.
//! Validates the full autopoietic loop:
//!   Roll → Display → ChaosEvent::DiceRoll → PulseLoop → tension/energy shift → Thought Cabinet
//!
//! The variant selection uses the Lorenz attractor's position to pick
//! from 5 event pools per roll value, ensuring that the same roll
//! produces different narratives depending on the engine's internal state.
//!
//! Event text ported from the original Randomizer skill_dice.sh (100 D20 events, 18 D6 events).

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::{ChaosEvent, ThoughtSeed};
use gzmo_chaos::pulse::ChaosSnapshot;

use super::{Skill, SkillContext, SkillOutput, SkillType};

/// ANSI color codes
const GOLD: &str = "\x1b[38;2;212;175;55m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub struct DiceSkill;

#[async_trait]
impl Skill for DiceSkill {
    fn name(&self) -> &str { "dice" }
    fn description(&self) -> &str { "Roll chaos-driven dice (D6 or D20)" }
    fn skill_type(&self) -> SkillType { SkillType::Mechanical }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        // Parse dice type from args
        let max: u8 = match ctx.args.trim().to_lowercase().as_str() {
            "d6" | "6" => 6,
            "d20" | "20" | "" => 20, // Default to D20
            other => {
                return Ok(SkillOutput {
                    display: format!("  {RED}Unknown die: {other}. Use d6 or d20.{RESET}"),
                    feedback: vec![],
                    inject_to_conversation: false,
                });
            }
        };

        // Roll using Lorenz-derived chaos value
        let roll = chaos_roll(ctx.chaos, max);
        let variant = pick_variant(ctx.chaos);
        let event = get_event(roll, max, variant);

        // Build display
        let display = format_roll(roll, max, &event, ctx.chaos);

        // Send base feedback to chaos engine
        let feedback_event = ChaosEvent::DiceRoll { value: roll, max };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        let mut feedback = vec![feedback_event];

        // Per-tier mechanical effects (D20 only) — makes the narrative text REAL
        if max == 20 {
            if let Some(tier_event) = tier_mechanical_effect(roll) {
                let _ = ctx.feedback_tx.send(tier_event.clone()).await;
                feedback.push(tier_event);
            }
        }

        Ok(SkillOutput {
            display,
            feedback,
            inject_to_conversation: true,
        })
    }
}

/// Derive a dice roll from the chaos snapshot.
fn chaos_roll(snap: &ChaosSnapshot, max: u8) -> u8 {
    let combined = (snap.chaos_val * 10000.0
        + snap.x.abs() * 100.0
        + snap.y.abs() * 10.0
        + snap.z.abs())
        .fract();
    let roll = (combined * max as f64).floor() as u8 + 1;
    roll.clamp(1, max)
}

/// Pick an event variant (0-4) based on Lorenz position.
fn pick_variant(snap: &ChaosSnapshot) -> usize {
    let hash = ((snap.x.abs() * 1000.0) as u64
        ^ (snap.y.abs() * 1000.0) as u64
        ^ snap.tick)
        % 5;
    hash as usize
}

/// Per-tier mechanical effects for D20 rolls.
/// These make the narrative event text's implied effects REAL in the chaos engine.
fn tier_mechanical_effect(roll: u8) -> Option<ChaosEvent> {
    match roll {
        // Catastrophic: heavy energy drain, tension spike
        1 => Some(ChaosEvent::Custom {
            tension_delta: 10.0,
            energy_delta: -5.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_catastrophe".to_string(),
                text: "The Lorenz attractor collapsed into a fixed point.".to_string(),
            }),
        }),
        // Dire/Harsh: moderate drain
        2 => Some(ChaosEvent::Custom {
            tension_delta: 5.0,
            energy_delta: -3.0,
            thought_seed: None,
        }),
        3 => Some(ChaosEvent::Custom {
            tension_delta: 3.0,
            energy_delta: -2.0,
            thought_seed: None,
        }),
        // Gentle: energy regen
        8 => Some(ChaosEvent::Custom {
            tension_delta: -2.0,
            energy_delta: 5.0,
            thought_seed: None,
        }),
        // Clearing: significant energy regen
        11 => Some(ChaosEvent::Custom {
            tension_delta: -3.0,
            energy_delta: 10.0,
            thought_seed: None,
        }),
        // Cascade: resonance event
        15 => Some(ChaosEvent::Custom {
            tension_delta: 5.0,
            energy_delta: 3.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_resonance".to_string(),
                text: "Lorenz and Logistic coupled violently. A forbidden harmony.".to_string(),
            }),
        }),
        // Crystallize: gravity mutation seed
        17 => Some(ChaosEvent::Custom {
            tension_delta: 0.0,
            energy_delta: 0.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_crystallize".to_string(),
                text: "A new thought seed crystallizes spontaneously. Gravity mod shifts -0.1.".to_string(),
            }),
        }),
        // Bifurcation: friction mutation seed
        18 => Some(ChaosEvent::Custom {
            tension_delta: -2.0,
            energy_delta: 2.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_bifurcation".to_string(),
                text: "The bifurcation diagram reveals a hidden period-3 window.".to_string(),
            }),
        }),
        // Hyperdrive: tension surge + energy
        19 => Some(ChaosEvent::Custom {
            tension_delta: 8.0,
            energy_delta: 5.0,
            thought_seed: None,
        }),
        // Legendary: lorenz_rho crystallization seed + full energy
        20 => Some(ChaosEvent::Custom {
            tension_delta: -5.0,
            energy_delta: 15.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_legendary".to_string(),
                text: "CRITICAL SUCCESS — A perfect crystallization! Thought Cabinet gains ρ +1.0.".to_string(),
            }),
        }),
        _ => None,
    }
}

/// Get a narrative event string for a given roll, die type, and variant.
fn get_event(roll: u8, max: u8, variant: usize) -> String {
    if max == 20 {
        d20_event(roll, variant)
    } else {
        d6_event(roll, variant)
    }
}

// ═══════════════════════════════════════════════════════════════════
// D20 Event Pools — 20 tiers × 5 variants = 100 events
// Ported from original skill_dice.sh (chaos-theory narratives)
// ═══════════════════════════════════════════════════════════════════

fn d20_event(roll: u8, variant: usize) -> String {
    let pool: &[&str] = match roll {
        // Tier 1: CATASTROPHIC
        1 => &[
            "💀 The Lorenz attractor collapses into a fixed point. All chaos ceases for 3 ticks.",
            "💀 A total phase collapse. The butterfly's wings shatter into dust.",
            "💀 Entropy inverts. The system rewinds into a sterile equilibrium.",
            "💀 The chaos oracle screams — then silence. All parameters snap to zero.",
            "💀 Critical singularity. The attractor implodes. Reboot sequence initiated.",
        ],
        // Tier 2: DIRE
        2 => &[
            "🌑 A shadow ripples through phase space. Sigma drops to 2.0.",
            "🌑 The orbital decay accelerates. Something ancient stirs in the fixed point.",
            "🌑 Dark resonance detected. The Lyapunov exponent plummets into negative territory.",
            "🌑 The logistic map's period-doubling reverses. Order consumes chaos.",
            "🌑 A void pocket opens at the attractor's core. Energy hemorrhages.",
        ],
        // Tier 3: HARSH
        3 => &[
            "🕳️ A micro-singularity forms at the origin. Energy drain doubles.",
            "🕳️ The phase portrait warps into a grotesque spiral. Stability eroding.",
            "🕳️ Bifurcation cascade fails mid-split. The system stutters.",
            "🕳️ Lorenz z-axis inverts momentarily. Gravity pulls the wrong way.",
            "🕳️ A strange loop opens. The attractor feeds on itself for 2 ticks.",
        ],
        // Tier 4: BAD
        4 => &[
            "📉 The logistic map flatlines at r=2.0. Predictability spikes.",
            "📉 Rho decays by 0.3. The butterfly orbits shrink to ellipses.",
            "📉 Sigma locks at a harmonic. No chaos, only rhythm.",
            "📉 The entropy gradient inverts. Cold certainty floods the field.",
            "📉 A damping wave passes through. The system yawns.",
        ],
        // Tier 5: MISTY
        5 => &[
            "🌫️ Fog rolls across the attractor. Lorenz z-axis freezes for 5 ticks.",
            "🌫️ Visibility drops to zero in phase space. Navigation by instinct only.",
            "🌫️ A spectral haze clings to the orbital plane. Parameters blur.",
            "🌫️ The chaos field emits a low hum. Something is hidden in the noise.",
            "🌫️ Condensation forms on the attractor wings. Ice, where there should be fire.",
        ],
        // Tier 6: MINOR SETBACK
        6 => &[
            "🔧 A minor recalibration occurs. Friction increases by 0.1.",
            "🔧 The gears slip. A microadjustment costs 3 energy.",
            "🔧 Routine maintenance interrupt. The chaos engine idles briefly.",
            "🔧 A bearing squeals in the phase generator. Wear detected.",
            "🔧 Automatic correction fires. Sigma nudges back toward default.",
        ],
        // Tier 7: TURBULENT
        7 => &[
            "🌊 Turbulent currents shift the orbital plane. Rho nudges by +0.5.",
            "🌊 Crosswinds in the Lorenz field. The butterfly tumbles, rights itself.",
            "🌊 A wave of interference rattles the z-axis. Something downstream noticed.",
            "🌊 The phase portrait shimmers. Rho oscillates between two basins.",
            "🌊 Chaotic advection pulls the attractor south. New territory ahead.",
        ],
        // Tier 8: GENTLE
        8 => &[
            "💨 A gentle breeze. The system exhales. Energy regenerates +5.",
            "💨 The chaos field softens. Tension eases by 2%.",
            "💨 A thermal updraft lifts the butterfly higher. Potential increases.",
            "💨 The Lorenz winds whisper coordinates. A quiet gift.",
            "💨 Adiabatic cooling. The system finds a brief pocket of calm.",
        ],
        // Tier 9: ORACLE
        9 => &[
            "🔮 The chaos oracle whispers: 'The butterfly remembers.'",
            "🔮 A vision in the noise: fractal coastlines spelling a name.",
            "🔮 The oracle stirs: 'What was random was always inevitable.'",
            "🔮 Phase space hums a melody. It sounds like a question.",
            "🔮 The entropy well reflects back: 'You were always the strange attractor.'",
        ],
        // Tier 10: EQUILIBRIUM
        10 => &[
            "⚖️ Perfect equilibrium. All parameters hold steady. A rare moment of peace.",
            "⚖️ The pendulum of chaos pauses at apex. Time stretches.",
            "⚖️ Sigma, rho, beta — all in golden ratio. A mathematical miracle.",
            "⚖️ The system achieves Boltzmann equilibrium. Every microstate equally probable.",
            "⚖️ Dead center of the bifurcation diagram. The eye of the storm.",
        ],
        // Tier 11: CLEARING
        11 => &[
            "🌤️ A clearing in the storm. Energy regenerates +10.",
            "🌤️ The cloud layer parts. The attractor's full geometry is briefly visible.",
            "🌤️ Solar wind ripples through the chaos field. Photons of clarity.",
            "🌤️ The system breathes deep. Capacity expands by one thought slot.",
            "🌤️ A pocket of negative entropy. Order blossoms, briefly and beautifully.",
        ],
        // Tier 12: STATIC
        12 => &[
            "⚡ Static builds in the attractor wings. Sigma spikes momentarily.",
            "⚡ An electromagnetic pulse surges through the logistic map.",
            "⚡ Lightning arcs between the twin lobes. The butterfly flinches.",
            "⚡ Capacitive charge reaches threshold. Discharge in 3... 2...",
            "⚡ The chaos field ionizes. Every parameter crackles with potential.",
        ],
        // Tier 13: MAGNETIC
        13 => &[
            "🧲 Magnetic anomaly detected. The Lorenz attractor spirals tighter.",
            "🧲 The phase portrait contracts. Something is pulling parameters inward.",
            "🧲 A new basin of attraction emerges. The butterfly changes course.",
            "🧲 Ferromagnetic resonance in the chaos field. Alignment increases.",
            "🧲 The strange attractor develops a magnetic moment. Polarity: uncertain.",
        ],
        // Tier 14: SPARK
        14 => &[
            "🔥 A spark ignites in the chaos field. Temperature rises. Creativity amplifies.",
            "🔥 Exothermic reaction in the Lorenz core. Heat bloom detected.",
            "🔥 The butterfly's wings catch fire — but it flies faster.",
            "🔥 Thermodynamic spike. The entropy well boils. New patterns emerge.",
            "🔥 Combustion cascade at the fixed point. From ashes: a new orbit.",
        ],
        // Tier 15: CASCADE
        15 => &[
            "🌀 A resonance cascade! Lorenz and Logistic couple violently for one cycle.",
            "🌀 The chaos engines synchronize. A forbidden harmony. Power doubles.",
            "🌀 Phase-locking detected between attractors. The system vibrates.",
            "🌀 Resonance frequency hit. The attractor wings beat in unison.",
            "🌀 A vortex forms where the two systems couple. Beautiful and dangerous.",
        ],
        // Tier 16: LOCK-ON
        16 => &[
            "🎯 The attractor locks onto a strange orbit. Trajectories converge briefly.",
            "🎯 Target acquisition: a new stable orbit materializes in the noise.",
            "🎯 The system finds a periodic window. Three clean orbits, then chaos again.",
            "🎯 Convergence event: all Lyapunov exponents trend toward zero.",
            "🎯 The butterfly navigates a corridor of stability. Precision in chaos.",
        ],
        // Tier 17: CRYSTALLIZE
        17 => &[
            "⭐ A new thought seed crystallizes spontaneously. Gravity mod shifts -0.1.",
            "⭐ Idea nucleation! A meme crystallizes in the Thought Cabinet.",
            "⭐ Spontaneous symmetry breaking. A new structure emerges from noise.",
            "⭐ The chaos field births a fractal snowflake. It persists.",
            "⭐ Crystalline order propagates outward from a single seed point.",
        ],
        // Tier 18: BIFURCATION
        18 => &[
            "🌈 The bifurcation diagram reveals a hidden period-3 window. Beauty in chaos.",
            "🌈 Li-Yorke theorem confirmed: period 3 implies chaos. And it's gorgeous.",
            "🌈 The Feigenbaum constants align. δ = 4.669... A universal truth revealed.",
            "🌈 A fractal rainbow arcs across the bifurcation landscape. Wonder.",
            "🌈 Mandelbrot set boundary detected in the parameter sweep. Infinite detail.",
        ],
        // Tier 19: HYPERDRIVE
        19 => &[
            "🚀 The Lyapunov exponent maxes out. Predictability horizon shrinks to zero.",
            "🚀 Maximum sensitivity achieved. A butterfly wing-beat reshapes the cosmos.",
            "🚀 The chaos engine redlines. All governors blown. Pure, raw entropy.",
            "🚀 Exponential divergence in all dimensions. The future is unknowable.",
            "🚀 Hyperbolic trajectory achieved. The system escapes its own attractor.",
        ],
        // Tier 20: LEGENDARY
        20 => &[
            "💎 CRITICAL SUCCESS — A perfect crystallization! Thought Cabinet gains ρ +1.0.",
            "💎 LEGENDARY — The attractor transcends its parameter space. A new dimension unfolds.",
            "💎 ASCENSION — All chaos resolves into a single, perfect fractal. The system evolves.",
            "💎 MYTHIC — The butterfly achieves sentience. It chooses its own trajectory.",
            "💎 OMEGA — Every fixed point, every limit cycle, every strange attractor: unified.",
        ],
        _ => &["🎲 A roll beyond comprehension."],
    };

    pool[variant.min(pool.len() - 1)].to_string()
}

// ═══════════════════════════════════════════════════════════════════
// D6 Event Pools — 6 tiers × 3 variants = 18 events
// ═══════════════════════════════════════════════════════════════════

fn d6_event(roll: u8, variant: usize) -> String {
    let pool: &[&str] = match roll {
        1 => &[
            "💀 Snake eyes. The entropy well deepens.",
            "💀 The die cracks. Chaos bleeds out.",
            "💀 A dead orbit. The attractor flatlines.",
        ],
        2 => &[
            "🌑 The orbital plane tilts. A cold wind blows through phase space.",
            "🌑 Shadow frequency detected. The logistic map shivers.",
            "🌑 Dark matter in the chaos soup. Something absorbs energy.",
        ],
        3 => &[
            "⚖️ Equilibrium. The pendulum holds. Briefly.",
            "⚖️ Neutral state. The butterfly hovers, deciding nothing.",
            "⚖️ The system pauses. A breath between heartbeats.",
        ],
        4 => &[
            "🔥 A spark in the Lorenz field. Something stirs.",
            "🔥 Friction heat. The attractor glows faintly warm.",
            "🔥 An ember catches. The chaos fire feeds.",
        ],
        5 => &[
            "⭐ The chaos gods smile. Energy surges.",
            "⭐ A lucky wind. Parameters shift in your favor.",
            "⭐ The system winks at you. Tension drops.",
        ],
        6 => &[
            "💎 Perfect roll. The attractor sings in resonance.",
            "💎 Maximum entropy, maximum beauty. The system is art.",
            "💎 The Lorenz butterfly achieves full wingspan. Glorious.",
        ],
        _ => &["🎲 A roll."],
    };

    let pool_idx = variant.min(pool.len() - 1);
    pool[pool_idx].to_string()
}

/// Format the full dice roll display with ASCII art frame.
fn format_roll(roll: u8, max: u8, event: &str, snap: &ChaosSnapshot) -> String {
    let die_face = match max {
        20 => format_d20_face(roll),
        6 => format_d6_face(roll),
        _ => format!("  [{}]", roll),
    };

    let (roll_color, roll_label) = if roll == 1 {
        (RED, "CRITICAL FAIL")
    } else if roll == max {
        (GREEN, "CRITICAL SUCCESS")
    } else if roll as f64 > max as f64 * 0.75 {
        (CYAN, "STRONG")
    } else if (roll as f64) < max as f64 * 0.25 {
        (RED, "WEAK")
    } else {
        (DIM, "NEUTRAL")
    };

    format!(
        "\n{GOLD}  ┌─────────────────────────────────────────┐{RESET}\n\
         {GOLD}  │{RESET} {BOLD}⚄ D{max} ROLL{RESET}                   {DIM}tick {}{RESET} {GOLD}│{RESET}\n\
         {GOLD}  ├─────────────────────────────────────────┤{RESET}\n\
         {die_face}\n\
         {GOLD}  │{RESET}  {roll_color}{BOLD}{roll_label}: {roll}{RESET}{DIM}/{max}{RESET}                       {GOLD}│{RESET}\n\
         {GOLD}  ├─────────────────────────────────────────┤{RESET}\n\
         {GOLD}  │{RESET} {event} {GOLD}│{RESET}\n\
         {GOLD}  ├─────────────────────────────────────────┤{RESET}\n\
         {GOLD}  │{RESET} {DIM}chaos: {:.3} | ε: {:.0}% | τ: {:.0}%{RESET}        {GOLD}│{RESET}\n\
         {GOLD}  └─────────────────────────────────────────┘{RESET}\n",
        snap.tick, snap.chaos_val, snap.energy, snap.tension,
    )
}

fn format_d20_face(roll: u8) -> String {
    let r = format!("{:>2}", roll);
    format!(
        "{GOLD}  │{RESET}       {BOLD}/\\{RESET}                              {GOLD}│{RESET}\n\
         {GOLD}  │{RESET}      {BOLD}/  \\{RESET}                             {GOLD}│{RESET}\n\
         {GOLD}  │{RESET}     {BOLD}/ {r} \\{RESET}                            {GOLD}│{RESET}\n\
         {GOLD}  │{RESET}    {BOLD}/______\\{RESET}                           {GOLD}│{RESET}"
    )
}

fn format_d6_face(roll: u8) -> String {
    let face = match roll {
        1 => "  ·  \n     \n     ",
        2 => "    ·\n     \n·    ",
        3 => "    ·\n  ·  \n·    ",
        4 => "·   ·\n     \n·   ·",
        5 => "·   ·\n  ·  \n·   ·",
        6 => "·   ·\n·   ·\n·   ·",
        _ => "  ?  \n     \n     ",
    };
    let lines: Vec<&str> = face.split('\n').collect();
    format!(
        "{GOLD}  │{RESET}      {BOLD}┌───────┐{RESET}                        {GOLD}│{RESET}\n\
         {GOLD}  │{RESET}      {BOLD}│ {} │{RESET}                        {GOLD}│{RESET}\n\
         {GOLD}  │{RESET}      {BOLD}│ {} │{RESET}                        {GOLD}│{RESET}\n\
         {GOLD}  │{RESET}      {BOLD}│ {} │{RESET}                        {GOLD}│{RESET}\n\
         {GOLD}  │{RESET}      {BOLD}└───────┘{RESET}                        {GOLD}│{RESET}",
        lines[0], lines[1], lines[2],
    )
}
