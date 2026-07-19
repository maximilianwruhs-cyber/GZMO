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
//! Event text lives in `data/dice_events.toml` (embedded via `dice_corpus`).

use anyhow::Result;
use async_trait::async_trait;

use gzmo_chaos::feedback::{ChaosEvent, ThoughtSeed};
use gzmo_chaos::pulse::ChaosSnapshot;

use super::dice_cascade::{
    cascade_evidence_json, cascade_feedback_event, execute_cascade, format_cascade_failure,
    format_cascade_header, format_cascade_plan, plan_cascade,
};
use super::dice_corpus::dice_event;
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
    fn name(&self) -> &str {
        "dice"
    }
    fn description(&self) -> &str {
        "Roll chaos-driven dice (D6 or D20)"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Mechanical
    }

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
                    evidence: None,
                });
            }
        };

        // Roll using Lorenz-derived chaos value
        let roll = chaos_roll(ctx.chaos, max);
        let pool_size = if max == 6 { 3 } else { 5 };
        let variant = pick_variant(ctx.chaos, pool_size);
        let event = dice_event(max, roll, variant);
        let inv = ctx.chaos.tick;

        // Build display and select a configuration-aware wild-magic cascade.
        let mut display = format_roll(roll, max, &event, ctx.chaos);
        let cascade = plan_cascade(
            &ctx.config.dice.cascade,
            max,
            roll,
            variant,
            inv,
            ctx.chaos,
            ctx.skills_dir,
        );

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

        let mut evidence = None;
        if let Some(plan) = cascade {
            let cascade_fb = cascade_feedback_event(&plan, roll);
            let _ = ctx.feedback_tx.send(cascade_fb.clone()).await;
            feedback.push(cascade_fb);
            if ctx.nested.registry.is_some() && ctx.nested.profile.is_some() && ctx.nested.depth < 2
            {
                display.push_str(&format_cascade_header(&plan, roll, max));
                match execute_cascade(&ctx, &plan).await {
                    Ok((nested, meta)) => {
                        display.push_str(&nested.display);
                        feedback.extend(nested.feedback.clone());
                        evidence = Some(cascade_evidence_json(&plan, &meta, &nested, true, None));
                    }
                    Err(err) => display.push_str(&format_cascade_failure(&plan, &err.to_string())),
                }
            } else {
                display.push_str(&format_cascade_plan(&plan, roll, max));
            }
        }

        Ok(SkillOutput {
            display,
            feedback,
            inject_to_conversation: true,
            evidence,
        })
    }
}

/// Derive a dice roll from the chaos snapshot.
fn chaos_roll(snap: &ChaosSnapshot, max: u8) -> u8 {
    let combined =
        (snap.chaos_val * 10000.0 + snap.x.abs() * 100.0 + snap.y.abs() * 10.0 + snap.z.abs())
            .fract();
    let roll = (combined * max as f64).floor() as u8 + 1;
    roll.clamp(1, max)
}

/// Pick an event variant based on Lorenz position (pool_size is 5 for D20, 3 for D6).
fn pick_variant(snap: &ChaosSnapshot, pool_size: usize) -> usize {
    let n = pool_size.max(1) as u64;
    let hash = ((snap.x.abs() * 1000.0) as u64 ^ (snap.y.abs() * 1000.0) as u64 ^ snap.tick) % n;
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
                text: "A new thought seed crystallizes spontaneously. Gravity mod shifts -0.1."
                    .to_string(),
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
                text: "CRITICAL SUCCESS — A perfect crystallization! Thought Cabinet gains ρ +1.0."
                    .to_string(),
            }),
        }),
        _ => None,
    }
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
