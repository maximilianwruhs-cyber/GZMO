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
use gzmo_chaos::feedback_ipc::event_to_json_value;
use gzmo_chaos::pulse::ChaosSnapshot;

use super::attractor_common::next_call_serial;
use super::dice_cascade::{
    cascade_evidence_json, cascade_feedback_event, execute_cascade, format_cascade_failure,
    format_cascade_header, plan_cascade, CascadeEventMeta,
};
use super::dice_corpus::{corpus, dice_event};
use super::dispatch;
use super::{Skill, SkillContext, SkillOutput, SkillType};
use crate::dice_loop::{self, DiceLoopScheduleStatus};
use crate::pedagogy::graph::PrerequisiteGraph;
use crate::pedagogy::learner::LearnerStore;
use crate::pedagogy::session::PedagogySession;

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
    fn description(&self) -> &str { "Roll chaos-driven dice (D6/D20) + wild magic pantheon cascade" }
    fn skill_type(&self) -> SkillType { SkillType::Mechanical }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let max = match parse_die_max(ctx.args) {
            Ok(m) => m,
            Err(other) => {
                return Ok(SkillOutput::new(
                    format!("  {RED}Unknown die: {other}. Use d6 or d20.{RESET}"),
                    vec![],
                    false,
                ));
            }
        };

        // Roll using Lorenz-derived chaos value
        let roll = chaos_roll(ctx.chaos, max);
        let pool_size = if max == 6 { 3 } else { 5 };
        let variant = pick_variant(ctx.chaos, pool_size);
        let event = dice_event(max, roll, variant);

        let inv = next_call_serial(&ctx.skills_dir.join(".dice_inv")).unwrap_or(ctx.chaos.tick);

        let chain_depth = if is_loop_roll(ctx.args) {
            dice_loop::load_state(ctx.data_dir)
                .map(|s| s.chain_depth.saturating_add(1))
                .unwrap_or(1)
        } else {
            0
        };

        let auto_on = PedagogySession::load(&ctx.config.pedagogy)
            .await
            .map(|s| s.auto_triggers_enabled)
            .unwrap_or(true);

        let loop_status = if auto_on {
            schedule_dice_loop(&ctx, roll, max, inv, chain_depth)
        } else {
            Some(DiceLoopScheduleStatus {
                scheduled: false,
                cancelled: false,
                delay_minutes: None,
                fire_at_utc: None,
                chain_depth,
                skipped_reason: Some("auto triggers off (/ops AUTO)".into()),
            })
        };

        // Build display
        let mut display = format_roll(roll, max, &event, ctx.chaos, inv);
        if let Some(ref status) = loop_status {
            if status.scheduled {
                if let (Some(m), Some(at)) = (status.delay_minutes, &status.fire_at_utc) {
                    display.push_str(&format!(
                        "  {DIM}⏱ Next /dice in {m}m (roll {roll} → interval) — {at}{RESET}\n"
                    ));
                }
            } else if status.cancelled {
                display.push_str(&format!(
                    "  {DIM}⏱ Dice loop cancelled (nat 1){RESET}\n"
                ));
            }
        }

        // Send base feedback to chaos engine
        let feedback_event = ChaosEvent::DiceRoll { value: roll, max };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        let mut feedback = vec![feedback_event];

        // Per-tier mechanical effects — makes narrative text REAL in the chaos engine
        if max == 20 {
            if let Some(tier_event) = tier_mechanical_effect(roll) {
                let _ = ctx.feedback_tx.send(tier_event.clone()).await;
                feedback.push(tier_event);
            }
        } else if let Some(tier_event) = d6_mechanical_effect(roll) {
            let _ = ctx.feedback_tx.send(tier_event.clone()).await;
            feedback.push(tier_event);
        }

        // ◆ Wild Magic — dispatch pantheon skill from tier pool (depth 0 only)
        let cascade_evidence = if ctx.nested.depth == 0 {
            run_wild_magic_cascade(&ctx, roll, max, variant, inv, &mut display, &mut feedback)
                .await
        } else {
            None
        };

        let evidence = build_dice_evidence(
            roll,
            max,
            variant,
            inv,
            &event,
            ctx.chaos,
            &feedback,
            loop_status.as_ref(),
            chain_depth,
            cascade_evidence,
        );

        Ok(SkillOutput {
            display,
            feedback,
            inject_to_conversation: true,
            evidence: Some(evidence),
        })
    }
}

/// True when this invocation is an automatic follow-up roll from the dice loop.
pub fn is_loop_roll(args: &str) -> bool {
    args.split_whitespace().any(|t| t == "--loop")
}

/// Parse die type from skill args (strips `--json` and `--loop`).
fn parse_die_max(args: &str) -> Result<u8, String> {
    let token = args
        .split_whitespace()
        .find(|t| *t != "--json" && *t != "--loop")
        .unwrap_or("");
    match token.to_lowercase().as_str() {
        "d6" | "6" => Ok(6),
        "d20" | "20" | "" => Ok(20),
        other if other.is_empty() => Ok(20),
        other => Err(other.to_string()),
    }
}

/// Schedule the next automatic `/dice` from this roll's outcome.
pub fn schedule_dice_loop(
    ctx: &SkillContext<'_>,
    roll: u8,
    max: u8,
    inv: u64,
    chain_depth: u32,
) -> Option<DiceLoopScheduleStatus> {
    if !ctx.config.dice.r#loop.enabled {
        return None;
    }
    Some(dice_loop::schedule_from_roll(
        ctx.data_dir,
        &ctx.config.dice.r#loop,
        roll,
        max,
        inv,
        chain_depth,
    ))
}

/// Structured evidence for Pi probes, discovery cycles, and `--json` CLI output.
pub fn build_dice_evidence(
    roll: u8,
    max: u8,
    variant: usize,
    inv: u64,
    event: &str,
    snap: &ChaosSnapshot,
    feedback: &[ChaosEvent],
    loop_status: Option<&DiceLoopScheduleStatus>,
    chain_depth: u32,
    cascade: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut value = serde_json::json!({
        "skill": "dice",
        "inv": inv,
        "roll": roll,
        "max": max,
        "variant": variant,
        "label": roll_label(roll, max),
        "tier": corpus().tier_name(max, roll),
        "event": event,
        "chain_depth": chain_depth,
        "wild_magic": cascade,
        "chaos": {
            "tick": snap.tick,
            "chaos_val": snap.chaos_val,
            "energy": snap.energy,
            "tension": snap.tension,
            "rho_effective": snap.rho_effective,
            "phase": snap.phase.to_string(),
        },
        "feedback": feedback.iter().map(event_to_json_value).collect::<Vec<_>>(),
    });
    if let Some(status) = loop_status {
        value["dice_loop"] = serde_json::json!({
            "scheduled": status.scheduled,
            "cancelled": status.cancelled,
            "delay_minutes": status.delay_minutes,
            "fire_at_utc": status.fire_at_utc,
            "chain_depth": status.chain_depth,
            "daemon_running": dispatch::daemon_running(),
            "skipped_reason": status.skipped_reason,
        });
    }
    value
}

fn roll_label(roll: u8, max: u8) -> &'static str {
    if roll == 1 {
        "CRITICAL FAIL"
    } else if roll == max {
        "CRITICAL SUCCESS"
    } else if roll as f64 > max as f64 * 0.75 {
        "STRONG"
    } else if (roll as f64) < max as f64 * 0.25 {
        "WEAK"
    } else {
        "NEUTRAL"
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

/// Pick an event variant based on Lorenz position and pool size.
fn pick_variant(snap: &ChaosSnapshot, pool_size: usize) -> usize {
    let size = pool_size.max(1) as u64;
    let hash = ((snap.x.abs() * 1000.0) as u64
        ^ (snap.y.abs() * 1000.0) as u64
        ^ snap.tick)
        % size;
    hash as usize
}

/// Per-tier mechanical effects for D6 rolls (halved D20 mirror).
fn d6_mechanical_effect(roll: u8) -> Option<ChaosEvent> {
    match roll {
        1 => Some(ChaosEvent::Custom {
            tension_delta: 5.0,
            energy_delta: -3.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_catastrophe".to_string(),
                text: "Snake eyes — the entropy well deepens.".to_string(),
            }),
        }),
        2 => Some(ChaosEvent::Custom {
            tension_delta: 3.0,
            energy_delta: -1.0,
            thought_seed: None,
        }),
        3 => Some(ChaosEvent::Custom {
            tension_delta: -1.0,
            energy_delta: 0.0,
            thought_seed: None,
        }),
        4 => Some(ChaosEvent::Custom {
            tension_delta: -1.0,
            energy_delta: 2.0,
            thought_seed: None,
        }),
        5 => Some(ChaosEvent::Custom {
            tension_delta: -2.0,
            energy_delta: 3.0,
            thought_seed: None,
        }),
        6 => Some(ChaosEvent::Custom {
            tension_delta: -3.0,
            energy_delta: 8.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_legendary".to_string(),
                text: "Perfect D6 — the attractor sings in resonance.".to_string(),
            }),
        }),
        _ => None,
    }
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
        // Bad: rho decay, predictability
        4 => Some(ChaosEvent::Custom {
            tension_delta: 2.0,
            energy_delta: -1.0,
            thought_seed: None,
        }),
        // Misty: disorientation, z-axis fog
        5 => Some(ChaosEvent::Custom {
            tension_delta: 1.0,
            energy_delta: -2.0,
            thought_seed: None,
        }),
        // Minor setback: maintenance cost
        6 => Some(ChaosEvent::Custom {
            tension_delta: 1.0,
            energy_delta: -3.0,
            thought_seed: None,
        }),
        // Turbulent: orbital shift
        7 => Some(ChaosEvent::Custom {
            tension_delta: 2.0,
            energy_delta: 0.0,
            thought_seed: None,
        }),
        // Gentle: energy regen
        8 => Some(ChaosEvent::Custom {
            tension_delta: -2.0,
            energy_delta: 5.0,
            thought_seed: None,
        }),
        // Oracle: whispered insight
        9 => Some(ChaosEvent::Custom {
            tension_delta: -1.0,
            energy_delta: 0.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_oracle".to_string(),
                text: "The chaos oracle whispered from the noise floor.".to_string(),
            }),
        }),
        // Equilibrium: rare calm
        10 => Some(ChaosEvent::Custom {
            tension_delta: -1.0,
            energy_delta: 0.0,
            thought_seed: None,
        }),
        // Clearing: significant energy regen
        11 => Some(ChaosEvent::Custom {
            tension_delta: -3.0,
            energy_delta: 10.0,
            thought_seed: None,
        }),
        // Static: sigma spike, capacitive charge
        12 => Some(ChaosEvent::Custom {
            tension_delta: 4.0,
            energy_delta: -1.0,
            thought_seed: None,
        }),
        // Magnetic: phase portrait contracts
        13 => Some(ChaosEvent::Custom {
            tension_delta: 2.0,
            energy_delta: -1.0,
            thought_seed: None,
        }),
        // Spark: thermodynamic spike, creativity
        14 => Some(ChaosEvent::Custom {
            tension_delta: 3.0,
            energy_delta: 2.0,
            thought_seed: Some(ThoughtSeed {
                category: "dice_spark".to_string(),
                text: "A spark ignited in the chaos field. Creativity amplifies.".to_string(),
            }),
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
        // Lock-on: corridor of stability
        16 => Some(ChaosEvent::Custom {
            tension_delta: -4.0,
            energy_delta: 1.0,
            thought_seed: None,
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

/// Format the full dice roll display with ASCII art frame.
async fn load_dice_readiness(ctx: &SkillContext<'_>) -> (Option<PrerequisiteGraph>, Vec<String>) {
    let graphs_dir = std::path::Path::new(&ctx.config.pedagogy.prerequisite_graphs_dir);
    let graph = PrerequisiteGraph::load_dir(graphs_dir).ok();
    let mastered = LearnerStore::new(&ctx.config.pedagogy)
        .load()
        .await
        .map(|p| p.semantic.mastery_vectors)
        .unwrap_or_default();
    (graph, mastered)
}

/// Format the full dice roll display with ASCII art frame.
async fn run_wild_magic_cascade(
    ctx: &SkillContext<'_>,
    roll: u8,
    max: u8,
    variant: usize,
    inv: u64,
    display: &mut String,
    feedback: &mut Vec<ChaosEvent>,
) -> Option<serde_json::Value> {
    let (graph, mastered) = load_dice_readiness(ctx).await;
    let readiness = graph.as_ref().map(|g| (g, mastered.as_slice()));
    let plan = plan_cascade(
        &ctx.config.dice.cascade,
        max,
        roll,
        variant,
        inv,
        ctx.chaos,
        ctx.skills_dir,
        readiness,
    )?;

    if ctx.nested.registry.is_none() || ctx.nested.profile.is_none() {
        display.push_str(&format_cascade_header(&plan, roll, max));
        display.push_str(&format!(
            "  {DIM}⚠ Wild magic planned but nested registry unavailable (upgrade caller).{RESET}\n"
        ));
        return Some(cascade_evidence_json(
            &plan,
            &CascadeEventMeta {
                skill: plan.skill.clone(),
                args: plan.args.clone(),
                used_shell: false,
            },
            &SkillOutput::new(String::new(), vec![], false),
            false,
            Some("nested registry missing"),
        ));
    }

    display.push_str(&format_cascade_header(&plan, roll, max));

    match execute_cascade(ctx, &plan).await {
        Ok((output, meta)) => {
            if !output.display.is_empty() {
                display.push_str(&output.display);
                if !output.display.ends_with('\n') {
                    display.push('\n');
                }
            }
            for event in &output.feedback {
                let _ = ctx.feedback_tx.send(event.clone()).await;
            }
            feedback.extend(output.feedback.clone());
            let cascade_event = cascade_feedback_event(&plan, roll);
            let _ = ctx.feedback_tx.send(cascade_event.clone()).await;
            feedback.push(cascade_event);
            Some(cascade_evidence_json(&plan, &meta, &output, true, None))
        }
        Err(e) => {
            display.push_str(&format_cascade_failure(&plan, &e.to_string()));
            Some(cascade_evidence_json(
                &plan,
                &CascadeEventMeta {
                    skill: plan.skill.clone(),
                    args: plan.args.clone(),
                    used_shell: false,
                },
                &SkillOutput::new(String::new(), vec![], false),
                false,
                Some(&e.to_string()),
            ))
        }
    }
}

fn format_roll(roll: u8, max: u8, event: &str, snap: &ChaosSnapshot, inv: u64) -> String {
    let die_face = match max {
        20 => format_d20_face(roll),
        6 => format_d6_face(roll),
        _ => format!("  [{}]", roll),
    };

    let label = roll_label(roll, max);
    let roll_color = match label {
        "CRITICAL FAIL" | "WEAK" => RED,
        "CRITICAL SUCCESS" => GREEN,
        "STRONG" => CYAN,
        _ => DIM,
    };

    format!(
        "\n{GOLD}  ┌─────────────────────────────────────────┐{RESET}\n\
         {GOLD}  │{RESET} {BOLD}⚄ D{max} ROLL{RESET}  {DIM}inv #{inv}{RESET}  {DIM}tick {}{RESET} {GOLD}│{RESET}\n\
         {GOLD}  ├─────────────────────────────────────────┤{RESET}\n\
         {die_face}\n\
         {GOLD}  │{RESET}  {roll_color}{BOLD}{label}: {roll}{RESET}{DIM}/{max}{RESET}                       {GOLD}│{RESET}\n\
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

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;

    fn test_snap(tick: u64, x: f64, y: f64) -> ChaosSnapshot {
        ChaosSnapshot {
            tick,
            x,
            y,
            z: 0.0,
            chaos_val: 0.42,
            energy: 50.0,
            tension: 40.0,
            phase: Phase::Idle,
            ..Default::default()
        }
    }

    #[test]
    fn chaos_roll_is_deterministic_and_in_range() {
        let snap = test_snap(10, 1.2, -3.4);
        let a = chaos_roll(&snap, 20);
        let b = chaos_roll(&snap, 20);
        assert_eq!(a, b);
        assert!((1..=20).contains(&a));
    }

    #[test]
    fn pick_variant_respects_d6_pool_size() {
        let snap = test_snap(7, 2.0, 3.0);
        for _ in 0..20 {
            let v = pick_variant(&snap, 3);
            assert!(v < 3);
        }
    }

    #[test]
    fn d20_tier_20_emits_legendary_seed() {
        let event = tier_mechanical_effect(20).expect("tier 20");
        let seed = event.thought_seed().expect("legendary seed");
        assert_eq!(seed.category, "dice_legendary");
    }

    #[test]
    fn d6_perfect_roll_emits_legendary_seed() {
        let event = d6_mechanical_effect(6).expect("d6 6");
        let seed = event.thought_seed().expect("legendary seed");
        assert_eq!(seed.category, "dice_legendary");
    }

    #[test]
    fn all_d20_tiers_have_mechanical_effects() {
        for roll in 1..=20u8 {
            assert!(
                tier_mechanical_effect(roll).is_some(),
                "missing tier effect for D20 roll {roll}"
            );
        }
    }

    #[test]
    fn all_d6_tiers_have_mechanical_effects() {
        for roll in 1..=6u8 {
            assert!(
                d6_mechanical_effect(roll).is_some(),
                "missing tier effect for D6 roll {roll}"
            );
        }
    }

    #[test]
    fn tier_14_emits_spark_seed() {
        let event = tier_mechanical_effect(14).expect("tier 14");
        assert_eq!(event.thought_seed().unwrap().category, "dice_spark");
    }

    #[test]
    fn build_dice_evidence_includes_feedback() {
        let snap = test_snap(5, 1.0, 2.0);
        let fb = vec![ChaosEvent::DiceRoll { value: 12, max: 20 }];
        let ev = build_dice_evidence(12, 20, 1, 7, "⚡ static", &snap, &fb, None, 0, None);
        assert_eq!(ev["skill"], "dice");
        assert_eq!(ev["inv"], 7);
        assert_eq!(ev["feedback"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn format_roll_includes_inv_counter() {
        let out = format_roll(10, 20, "⚖️ equilibrium", &test_snap(1, 0.0, 0.0), 42);
        assert!(out.contains("inv #42"));
    }
}
