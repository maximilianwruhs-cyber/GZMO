//! `/dice` Wild Magic — Slice A.1 plan-only cascade.
//!
//! Selects a pantheon skill from `data/dice_cascade.toml` after a roll and
//! surfaces it in the dice display. Does **not** nested-dispatch (needs
//! `dispatch` + fuller `SkillContext` — Slice A full).

use std::collections::HashMap;
use std::sync::OnceLock;

use gzmo_chaos::feedback::{ChaosEvent, ThoughtSeed};
use gzmo_chaos::pulse::ChaosSnapshot;
use serde::Deserialize;

use super::dice_corpus::corpus;

const EMBEDDED_TOML: &str = include_str!("../../../data/dice_cascade.toml");

#[derive(Debug, Clone, Deserialize)]
struct CascadeMeta {
    #[allow(dead_code)]
    version: u32,
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CascadeBand {
    label: String,
    rolls: Vec<u8>,
    skills: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CascadeFile {
    meta: CascadeMeta,
    #[serde(default)]
    d20: Vec<CascadeBand>,
    #[serde(default)]
    d6: Vec<CascadeBand>,
    #[serde(default)]
    skill_prereqs: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CascadeTables {
    meta: CascadeMeta,
    d20: Vec<CascadeBand>,
    d6: Vec<CascadeBand>,
    #[allow(dead_code)]
    skill_prereqs: HashMap<String, String>,
}

impl CascadeTables {
    fn parse(toml_src: &str) -> anyhow::Result<Self> {
        let raw: CascadeFile = toml::from_str(toml_src)?;
        Ok(Self {
            meta: raw.meta,
            d20: raw.d20,
            d6: raw.d6,
            skill_prereqs: raw.skill_prereqs,
        })
    }

    fn band_for_roll(&self, max: u8, roll: u8) -> Option<&CascadeBand> {
        let bands = if max == 6 { &self.d6 } else { &self.d20 };
        bands.iter().find(|b| b.rolls.contains(&roll))
    }
}

static TABLES: OnceLock<CascadeTables> = OnceLock::new();

pub fn tables() -> &'static CascadeTables {
    TABLES.get_or_init(|| {
        CascadeTables::parse(EMBEDDED_TOML)
            .unwrap_or_else(|e| panic!("invalid embedded data/dice_cascade.toml: {e}"))
    })
}

/// Planned wild-magic suggestion for a dice outcome (not executed on main yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CascadePlan {
    pub skill: String,
    pub args: String,
    pub band_label: String,
    pub pool_index: usize,
    pub tier_name: Option<String>,
}

pub fn pick_pool_index(
    snap: &ChaosSnapshot,
    roll: u8,
    variant: usize,
    inv: u64,
    pool_len: usize,
) -> usize {
    if pool_len == 0 {
        return 0;
    }
    let hash = ((snap.x.abs() * 1000.0) as u64)
        ^ ((snap.y.abs() * 1000.0) as u64)
        ^ ((snap.z.abs() * 100.0) as u64)
        ^ snap.tick
        ^ ((roll as u64) << 8)
        ^ ((variant as u64) << 12)
        ^ inv;
    (hash % pool_len as u64) as usize
}

fn is_excluded(skill: &str, tables: &CascadeTables) -> bool {
    tables
        .meta
        .exclude
        .iter()
        .any(|e| e.eq_ignore_ascii_case(skill))
}

/// Plan a cascade skill from TOML pools (always enabled in Slice A.1).
pub fn plan_cascade(
    max: u8,
    roll: u8,
    variant: usize,
    inv: u64,
    snap: &ChaosSnapshot,
) -> Option<CascadePlan> {
    let band = tables().band_for_roll(max, roll)?;
    let pool: Vec<String> = band
        .skills
        .iter()
        .filter(|s| !is_excluded(s, tables()))
        .cloned()
        .collect();
    if pool.is_empty() {
        return None;
    }

    let idx = pick_pool_index(snap, roll, variant, inv, pool.len());
    let skill = pool[idx].clone();
    let args = build_cascade_args(&skill, roll, max, variant, inv, snap);
    Some(CascadePlan {
        skill,
        args,
        band_label: band.label.clone(),
        pool_index: idx,
        tier_name: corpus().tier_name(max, roll).map(String::from),
    })
}

const STORY_SEEDS: &[&str] = &[
    "lorenz",
    "attractor",
    "butterfly",
    "entropy",
    "resonance",
    "bifurcation",
    "oracle",
    "crystallize",
    "phase",
    "strange-loop",
];
const JOKE_TOPICS: &[&str] = &[
    "chaos",
    "physics",
    "dice",
    "entropy",
    "attractor",
    "determinism",
    "oracle",
];
const POEM_MOTIFS: &[&str] = &[
    "verse", "static", "spiral", "orbit", "whisper", "tide", "ember",
];
const DEFINE_TERMS: &[&str] = &[
    "entropy",
    "attractor",
    "bifurcation",
    "lyapunov",
    "strange-loop",
    "resonance",
    "equilibrium",
    "chaos",
];
const CARD_TYPES: &[&str] = &["creature", "instant", "enchantment", "artifact", "sorcery"];

fn pick_from<'a>(list: &'a [&str], snap: &ChaosSnapshot, roll: u8, inv: u64) -> &'a str {
    let i = pick_pool_index(snap, roll, 0, inv, list.len());
    list[i]
}

pub fn build_cascade_args(
    skill: &str,
    roll: u8,
    max: u8,
    variant: usize,
    inv: u64,
    snap: &ChaosSnapshot,
) -> String {
    let _ = max;
    let raw = match skill {
        "story" => pick_from(STORY_SEEDS, snap, roll, inv).to_string(),
        "joke" => pick_from(JOKE_TOPICS, snap, roll, inv).to_string(),
        "poem" => pick_from(POEM_MOTIFS, snap, roll, inv).to_string(),
        "define" => pick_from(DEFINE_TERMS, snap, roll, inv).to_string(),
        "card" => {
            let i = (roll as usize + variant + inv as usize) % CARD_TYPES.len();
            CARD_TYPES[i].to_string()
        }
        "pkm" => {
            let pkm_types = ["pokemon", "trainer", "energy"];
            let i = (roll as usize + variant + inv as usize) % pkm_types.len();
            pkm_types[i].to_string()
        }
        "transform" => "Heaviside".to_string(),
        "calculate" => {
            let exp = (snap.tick % 4) + 2;
            format!("{roll}^{exp}")
        }
        "visual" => format!("roll-{roll}"),
        _ => String::new(),
    };
    raw.trim().to_string()
}

const GOLD: &str = "\x1b[38;2;212;175;55m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Display block for a planned (not executed) cascade.
pub fn format_cascade_plan(plan: &CascadePlan, roll: u8, max: u8) -> String {
    let tier = plan
        .tier_name
        .as_deref()
        .unwrap_or(&plan.band_label)
        .to_ascii_uppercase();
    let cmd = if plan.args.is_empty() {
        format!("/{}", plan.skill)
    } else {
        format!("/{} {}", plan.skill, plan.args)
    };
    format!(
        "\n{GOLD}  ╔═════════════════════════════════════════╗{RESET}\n\
         {GOLD}  ║{RESET} {BOLD}◆ WILD MAGIC{RESET}  {DIM}tier {tier}{RESET}               {GOLD}║{RESET}\n\
         {GOLD}  ╠═════════════════════════════════════════╣{RESET}\n\
         {GOLD}  ║{RESET} Roll {CYAN}{roll}{RESET}/{max} → {BOLD}{cmd}{RESET}          {GOLD}║{RESET}\n\
         {GOLD}  ║{RESET} {DIM}(plan only — nested dispatch = Slice A full){RESET} {GOLD}║{RESET}\n\
         {GOLD}  ╚═════════════════════════════════════════╝{RESET}\n"
    )
}

pub fn cascade_feedback_event(plan: &CascadePlan, roll: u8) -> ChaosEvent {
    ChaosEvent::Custom {
        tension_delta: -0.5,
        energy_delta: 3.0,
        thought_seed: Some(ThoughtSeed {
            category: "dice_cascade".to_string(),
            text: format!(
                "Wild magic plan: roll {roll} suggests /{} {}",
                plan.skill, plan.args
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> ChaosSnapshot {
        ChaosSnapshot {
            tick: 42,
            x: 1.2,
            y: -0.5,
            z: 3.0,
            chaos_val: 0.4,
            ..ChaosSnapshot::default()
        }
    }

    #[test]
    fn embedded_tables_cover_d20_and_d6() {
        let t = tables();
        assert!(!t.d20.is_empty());
        assert!(!t.d6.is_empty());
        assert!(t.band_for_roll(20, 1).is_some());
        assert!(t.band_for_roll(6, 6).is_some());
    }

    #[test]
    fn plan_cascade_picks_skill() {
        let plan = plan_cascade(20, 1, 0, 7, &snap()).expect("catastrophe band");
        assert!(!plan.skill.is_empty());
        assert_eq!(plan.band_label, "catastrophe");
    }

    #[test]
    fn calculate_args_include_roll() {
        let args = build_cascade_args("calculate", 15, 20, 0, 1, &snap());
        assert!(args.starts_with("15^"));
    }
}
