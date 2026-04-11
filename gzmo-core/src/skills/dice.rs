//! # Dice Skill — `/dice`
//!
//! Chaos-driven dice rolls with narrative event pools.
//! Validates the full autopoietic loop:
//!   Roll → Display → ChaosEvent::DiceRoll → PulseLoop → tension/energy shift → Thought Cabinet
//!
//! The variant selection uses the Lorenz attractor's position to pick
//! from 5 event pools per roll value, ensuring that the same roll
//! produces different narratives depending on the engine's internal state.

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::ChaosEvent;
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

        // Send feedback to chaos engine
        let feedback_event = ChaosEvent::DiceRoll { value: roll, max };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
        })
    }
}

/// Derive a dice roll from the chaos snapshot.
/// Uses the logistic map value (chaos_val) mapped to [1, max].
fn chaos_roll(snap: &ChaosSnapshot, max: u8) -> u8 {
    // Use fractional part of x*y*z for additional entropy
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

/// Get a narrative event string for a given roll, die type, and variant.
fn get_event(roll: u8, max: u8, variant: usize) -> String {
    if max == 20 {
        d20_event(roll, variant)
    } else {
        d6_event(roll, variant)
    }
}

fn d20_event(roll: u8, variant: usize) -> String {
    let events: &[&[&str]] = match roll {
        1 => &[
            &["💀 CRITICAL FAILURE — The dice crumble to dust in your hands"],
            &["💀 FUMBLE — Reality itself flinches at your incompetence"],
            &["💀 CATASTROPHE — The universe laughs, and it's not kind"],
            &["💀 DISASTER — Every thread of fate snaps simultaneously"],
            &["💀 ANNIHILATION — Even the dice gods look away in shame"],
        ],
        2..=5 => &[
            &["⚠ A shadow crosses your path. Something stirs..."],
            &["⚠ The ground shifts beneath you. Unstable footing."],
            &["⚠ A distant rumble. Something is coming."],
            &["⚠ Your instincts scream wrong. You hesitate."],
            &["⚠ The air thickens with dread. Visibility drops."],
        ],
        6..=10 => &[
            &["🎲 A mundane outcome. Nothing ventured, nothing gained."],
            &["🎲 The path continues, unremarkable. You persist."],
            &["🎲 Neither fortune nor misfortune. The universe shrugs."],
            &["🎲 A forgettable moment in an infinite timeline."],
            &["🎲 The cosmos is indifferent. You are unchanged."],
        ],
        11..=15 => &[
            &["✨ A glimmer of opportunity reveals itself"],
            &["✨ The wind shifts in your favor. Subtle but real."],
            &["✨ A door you hadn't noticed opens slightly."],
            &["✨ Fortune smiles, gently. A small advantage appears."],
            &["✨ The current bends toward you. Progress beckons."],
        ],
        16..=19 => &[
            &["🔥 Adrenaline surges! The odds bow in your favor!"],
            &["🔥 Reality warps — and it warps FOR you!"],
            &["🔥 The stars align! Everything clicks into place!"],
            &["🔥 Time dilates — you see the perfect move!"],
            &["🔥 Pure flow state. Movement becomes precision art."],
        ],
        20 => &[
            &["👑 NATURAL 20 — The universe bends to your will!"],
            &["👑 PERFECTION — You transcend mortal limitations!"],
            &["👑 LEGENDARY — Songs will be sung of this moment!"],
            &["👑 DIVINE INTERVENTION — Fate itself applauds!"],
            &["👑 APOTHEOSIS — You briefly touch the infinite!"],
        ],
        _ => &[&["🎲 A roll beyond comprehension."]],
    };

    let pool = events[variant.min(events.len() - 1)];
    pool[0].to_string()
}

fn d6_event(roll: u8, variant: usize) -> String {
    let events: &[&[&str]] = match roll {
        1 => &[
            &["⚫ Snake eyes energy. The coin lands in the gutter."],
            &["⚫ Rock bottom. But there's only one way from here."],
            &["⚫ The dice barely roll. Gravity wins this round."],
        ],
        2..=3 => &[
            &["🔵 Below average. The machine chugs on."],
            &["🔵 Mediocre at best. Keep moving."],
            &["🔵 Not great, not terrible. Adequate."],
        ],
        4..=5 => &[
            &["🟢 Solid roll. The gears turn smoothly."],
            &["🟢 Above average. Momentum builds."],
            &["🟢 A competent result. Reliable."],
        ],
        6 => &[
            &["🔴 MAX! The die explodes with energy!"],
            &["🔴 SIXER! Pure diesel power!"],
            &["🔴 PERFECTION! The small die punches above its weight!"],
        ],
        _ => &[&["🎲 A roll."]],
    };

    let pool_idx = variant.min(events.len() - 1);
    let pool = events[pool_idx];
    pool[0].to_string()
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
