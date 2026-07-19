//! `/dice` Wild Magic — tier-indexed pantheon skill cascade.
//!
//! After each roll, chaos state selects a skill from the roll's tier pool in
//! `data/dice_cascade.toml`, builds attractor-derived args, and dispatches through
//! the full skill registry (generative skills need gateway in context).

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::Result;
use gzmo_chaos::feedback::{ChaosEvent, ThoughtSeed};
use gzmo_chaos::pulse::ChaosSnapshot;
use serde::Deserialize;

use crate::config::DiceCascadeConfig;

use super::dice_corpus::corpus;
use super::dispatch::{self, load_live_chaos_snapshot};
use super::persona::load_characters;
use super::{SkillContext, SkillOutput, SkillRegistry};

const EMBEDDED_TOML: &str = include_str!("../../../data/dice_cascade.toml");

#[derive(Debug, Clone, Deserialize)]
struct CascadeMeta {
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
    pub meta: CascadeMeta,
    pub d20: Vec<CascadeBand>,
    pub d6: Vec<CascadeBand>,
    pub skill_prereqs: HashMap<String, String>,
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

/// Planned wild-magic dispatch for a dice outcome.
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
        ^ (snap.tick)
        ^ ((roll as u64) << 8)
        ^ ((variant as u64) << 12)
        ^ inv;
    (hash % pool_len as u64) as usize
}

fn is_excluded(skill: &str, cfg: &DiceCascadeConfig, tables: &CascadeTables) -> bool {
    if cfg.exclude.iter().any(|e| e.eq_ignore_ascii_case(skill)) {
        return true;
    }
    tables
        .meta
        .exclude
        .iter()
        .any(|e| e.eq_ignore_ascii_case(skill))
}

pub fn plan_cascade(
    cfg: &DiceCascadeConfig,
    max: u8,
    roll: u8,
    variant: usize,
    inv: u64,
    snap: &ChaosSnapshot,
    skills_dir: &Path,
) -> Option<CascadePlan> {
    if !cfg.enabled {
        return None;
    }

    let band = tables().band_for_roll(max, roll)?;
    let mut pool: Vec<String> = band
        .skills
        .iter()
        .filter(|s| !is_excluded(s, cfg, tables()))
        .cloned()
        .collect();
    if pool.is_empty() {
        return None;
    }

    let idx = pick_pool_index(snap, roll, variant, inv, pool.len());
    let skill = pool.remove(idx);
    let args = build_cascade_args(&skill, roll, max, variant, inv, snap, skills_dir);
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
    skills_dir: &Path,
) -> String {
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
            let pkm_types = &["pokemon", "trainer", "energy"];
            let i = (roll as usize + variant + inv as usize) % pkm_types.len();
            pkm_types[i].to_string()
        }
        "transform" => transform_arg(snap, roll, inv, skills_dir),
        "calculate" => {
            let exp = (snap.tick % 4) + 2;
            format!("{roll}^{exp}")
        }
        "visual" => format!("roll-{roll}"),
        _ => String::new(),
    };
    raw.trim().to_string()
}

fn transform_arg(snap: &ChaosSnapshot, roll: u8, inv: u64, skills_dir: &Path) -> String {
    let path = skills_dir.join("characters.toml");
    if let Ok(file) = load_characters(&path) {
        if !file.characters.is_empty() {
            let i = pick_pool_index(snap, roll, 0, inv, file.characters.len());
            return file.characters[i].name.clone();
        }
    }
    "Heaviside".to_string()
}

const GOLD: &str = "\x1b[38;2;212;175;55m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

pub fn format_cascade_header(plan: &CascadePlan, roll: u8, max: u8) -> String {
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
         {GOLD}  ╚═════════════════════════════════════════╝{RESET}\n"
    )
}

/// Plan-only rendering used when a caller did not supply nested dispatch.
pub fn format_cascade_plan(plan: &CascadePlan, roll: u8, max: u8) -> String {
    format!(
        "{}{DIM}  (plan only — nested dispatch unavailable){RESET}\n",
        format_cascade_header(plan, roll, max)
    )
}

pub fn format_cascade_failure(plan: &CascadePlan, err: &str) -> String {
    format!(
        "\n{GOLD}  ╔═════════════════════════════════════════╗{RESET}\n\
         {GOLD}  ║{RESET} {BOLD}◆ WILD MAGIC{RESET} {DIM}(fizzled){RESET}              {GOLD}║{RESET}\n\
         {GOLD}  ╠═════════════════════════════════════════╣{RESET}\n\
         {GOLD}  ║{RESET} /{} {} — {DIM}{}{RESET} {GOLD}║{RESET}\n\
         {GOLD}  ╚═════════════════════════════════════════╝{RESET}\n",
        plan.skill,
        plan.args,
        err.chars().take(40).collect::<String>()
    )
}

pub fn cascade_feedback_event(plan: &CascadePlan, roll: u8) -> ChaosEvent {
    ChaosEvent::Custom {
        tension_delta: -0.5,
        energy_delta: 3.0,
        thought_seed: Some(ThoughtSeed {
            category: "dice_cascade".to_string(),
            text: format!(
                "Wild magic: roll {roll} invoked /{} {}",
                plan.skill, plan.args
            ),
        }),
    }
}

/// Execute wild magic if context carries nested dispatch (full registry + profile).
pub async fn execute_cascade(
    ctx: &SkillContext<'_>,
    plan: &CascadePlan,
) -> Result<(SkillOutput, CascadeEventMeta)> {
    if ctx.nested.depth >= 2 {
        anyhow::bail!("cascade depth limit reached");
    }
    let registry = ctx
        .nested
        .registry
        .ok_or_else(|| anyhow::anyhow!("cascade requires nested registry"))?;
    let profile = ctx
        .nested
        .profile
        .ok_or_else(|| anyhow::anyhow!("cascade requires nested profile"))?;

    let fresh = load_live_chaos_snapshot(ctx.data_dir, ctx.chaos);
    let nested_ctx = dispatch::skill_context(
        &fresh,
        ctx.feedback_tx,
        &plan.args,
        ctx.gateway,
        ctx.router,
        ctx.config,
        super::NestedDispatch {
            registry: Some(registry),
            profile: Some(profile),
            depth: ctx.nested.depth.saturating_add(1),
        },
    );

    let result = dispatch::dispatch_skill(registry, &plan.skill, nested_ctx, profile).await?;

    Ok((
        result.output,
        CascadeEventMeta {
            skill: plan.skill.clone(),
            args: plan.args.clone(),
            used_shell: result.used_shell,
        },
    ))
}

#[derive(Debug, Clone)]
pub struct CascadeEventMeta {
    pub skill: String,
    pub args: String,
    pub used_shell: bool,
}

pub fn cascade_evidence_json(
    plan: &CascadePlan,
    meta: &CascadeEventMeta,
    output: &SkillOutput,
    ok: bool,
    error: Option<&str>,
) -> serde_json::Value {
    let mut value = serde_json::json!({
        "enabled": true,
        "band": plan.band_label,
        "tier": plan.tier_name,
        "pool_index": plan.pool_index,
        "skill": plan.skill,
        "args": plan.args,
        "ok": ok,
        "error": error,
        "used_shell": meta.used_shell,
        "display_chars": output.display.len(),
        "feedback_count": output.feedback.len(),
        "nested_evidence": output.evidence,
    });
    if !output.display.is_empty() {
        value["display_plain"] =
            serde_json::Value::String(crate::text_util::pi_skill_display(&output.display));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use gzmo_chaos::chaos::Phase;

    fn snap(tick: u64) -> ChaosSnapshot {
        ChaosSnapshot {
            tick,
            x: 1.1,
            y: -2.2,
            z: 0.3,
            chaos_val: 0.5,
            energy: 50.0,
            tension: 40.0,
            phase: Phase::Idle,
            ..Default::default()
        }
    }

    #[test]
    fn embedded_tables_cover_all_d20_rolls() {
        let t = tables();
        for roll in 1..=20u8 {
            assert!(
                t.band_for_roll(20, roll).is_some(),
                "missing d20 band for roll {roll}"
            );
        }
    }

    #[test]
    fn embedded_tables_cover_all_d6_rolls() {
        let t = tables();
        for roll in 1..=6u8 {
            assert!(
                t.band_for_roll(6, roll).is_some(),
                "missing d6 band for roll {roll}"
            );
        }
    }

    #[test]
    fn plan_legendary_includes_generative_skill() {
        let cfg = DiceCascadeConfig::default();
        let plan =
            plan_cascade(&cfg, 20, 20, 2, 7, &snap(100), Path::new("/tmp/skills")).expect("plan");
        assert!(
            matches!(plan.skill.as_str(), "story" | "card" | "poem" | "joke"),
            "got {}",
            plan.skill
        );
    }

    #[test]
    fn nat1_catastrophe_pool() {
        let cfg = DiceCascadeConfig::default();
        let plan = plan_cascade(&cfg, 20, 1, 0, 1, &snap(1), Path::new("/tmp/skills")).unwrap();
        assert!(matches!(plan.skill.as_str(), "sound" | "stabilize"));
        assert_eq!(plan.band_label, "catastrophe");
    }

    #[test]
    fn build_calculate_args_has_roll() {
        let args = build_cascade_args("calculate", 14, 20, 1, 3, &snap(50), Path::new("."));
        assert!(args.contains('^'));
    }
}
