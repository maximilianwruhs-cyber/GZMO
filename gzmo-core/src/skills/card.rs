//! # Card Skill — `/card [type]`
//!
//! Forge a random Magic: The Gathering card via LLM + cardforge.toml color pie.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::pulse::ChaosSnapshot;
use serde::Deserialize;

use super::llm::{
    chaos_index, llm_chat, SkillRuntime, BLUE, BOLD, DIM, GREEN, MAGENTA, RED, RESET, WHITE,
};
use super::{Skill, SkillContext, SkillOutput, SkillType};

const CARD_TYPES: [&str; 5] = ["Creature", "Instant", "Sorcery", "Enchantment", "Artifact"];
const COLORS: [&str; 5] = ["white", "blue", "black", "red", "green"];
const COLOR_SYMBOLS: [&str; 5] = ["☀️", "💧", "💀", "🔥", "🌿"];
const COLOR_LETTERS: [&str; 5] = ["W", "U", "B", "R", "G"];
const RARITY_NAMES: [&str; 4] = ["Common", "Uncommon", "Rare", "Mythic Rare"];
const RARITY_ICONS: [&str; 4] = ["⚪", "🔵", "🟡", "🟠"];

#[derive(Debug, Deserialize)]
struct CardForgeFile {
    #[serde(default)]
    colors: HashMap<String, ColorSection>,
}

#[derive(Debug, Deserialize)]
struct ColorSection {
    #[serde(default)]
    philosophy: String,
    #[serde(default)]
    flavor_tone: String,
}

#[derive(Debug, Default)]
struct ParsedCard {
    name: String,
    cost: String,
    type_line: String,
    rarity: String,
    rules: String,
    flavor: String,
    pt: String,
}

pub struct CardSkill {
    pub rt: Arc<SkillRuntime>,
}

fn chaos_int(snap: &ChaosSnapshot, min: i32, max: i32) -> i32 {
    let range = (max - min + 1) as f64;
    min + (snap.chaos_val.fract() * range).floor() as i32
}

fn pick_rarity(snap: &ChaosSnapshot) -> usize {
    let roll = chaos_int(snap, 1, 100);
    if roll <= 45 {
        0
    } else if roll <= 75 {
        1
    } else if roll <= 93 {
        2
    } else {
        3
    }
}

fn parse_structured_card(text: &str) -> ParsedCard {
    let mut card = ParsedCard::default();
    let anchors: Vec<_> = text
        .match_indices("\nNAME:")
        .chain(text.match_indices("NAME:").filter(|(i, _)| *i == 0))
        .collect();
    let block = if let Some((start, _)) = anchors.last() {
        &text[*start..]
    } else {
        text
    };

    for line in block.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("NAME:") {
            card.name = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("COST:") {
            card.cost = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("TYPE:") {
            card.type_line = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("RARITY:") {
            card.rarity = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("RULES:") {
            card.rules = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("FLAVOR:") {
            card.flavor = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("PT:") {
            card.pt = v.trim().to_string();
        }
    }

    if card.name.is_empty() {
        if let Some(cap) = text
            .lines()
            .find(|l| l.starts_with("**") && l.ends_with("**"))
        {
            card.name = cap.trim_matches('*').trim().to_string();
        }
    }
    card
}

fn load_color_philosophy(path: &std::path::Path, color: &str) -> (String, String) {
    let content = std::fs::read_to_string(path).ok();
    let Some(content) = content else {
        return (String::new(), String::new());
    };
    let cfg: CardForgeFile = toml::from_str(&content).unwrap_or(CardForgeFile {
        colors: HashMap::new(),
    });
    let section = cfg.colors.get(color);
    (
        section.map(|s| s.philosophy.clone()).unwrap_or_default(),
        section.map(|s| s.flavor_tone.clone()).unwrap_or_default(),
    )
}

fn border_color_name(color: &str) -> &'static str {
    match color {
        "white" => WHITE,
        "blue" => BLUE,
        "black" => DIM,
        "red" => RED,
        "green" => GREEN,
        _ => WHITE,
    }
}

fn render_card(color: &str, color_sym: &str, card: &ParsedCard, rarity_icon: &str) -> String {
    let bc = border_color_name(color);
    let mut out = String::new();
    out.push_str(&format!(
        "\n{bc}  ╔═══════════════════════════════════════════╗{RESET}\n\
         {bc}  ║{RESET} {BOLD}{}{RESET}\n",
        card.name
    ));
    out.push_str(&format!("{bc}  ║{RESET} {:>45}\n", card.cost));
    out.push_str(&format!(
        "{bc}  ╠═══════════════════════════════════════════╣{RESET}\n\
         {bc}  ║{RESET}\n\
         {bc}  ║{RESET}  {color_sym} {DIM}{}{RESET}\n\
         {bc}  ║{RESET}  {rarity_icon} {DIM}{}{RESET}\n\
         {bc}  ║{RESET}\n\
         {bc}  ╠═══════════════════════════════════════════╣{RESET}\n",
        card.type_line, card.rarity
    ));

    for rline in card.rules.split('|') {
        let rline = rline.trim();
        if !rline.is_empty() {
            out.push_str(&format!("{bc}  ║{RESET}  {WHITE}{rline}{RESET}\n"));
        }
    }

    out.push_str(&format!("{bc}  ║{RESET}\n"));
    if !card.flavor.is_empty() {
        out.push_str(&format!(
            "{bc}  ║{RESET}  {DIM}{MAGENTA}{}{RESET}\n{bc}  ║{RESET}\n",
            card.flavor
        ));
    }

    if !card.pt.is_empty() && !matches!(card.pt.to_uppercase().as_str(), "NONE" | "N/A") {
        out.push_str(&format!(
            "{bc}  ║{RESET}{:>42} {BOLD}[{}]{RESET}\n",
            "", card.pt
        ));
    }

    out.push_str(&format!(
        "{bc}  ╚═══════════════════════════════════════════╝{RESET}\n"
    ));
    out
}

#[async_trait]
impl Skill for CardSkill {
    fn name(&self) -> &str {
        "card"
    }
    fn description(&self) -> &str {
        "Forge a random Magic: The Gathering card"
    }
    fn skill_type(&self) -> SkillType {
        SkillType::Generative
    }

    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput> {
        let c_idx = chaos_index(ctx.chaos, COLORS.len());
        let color = COLORS[c_idx];
        let color_sym = COLOR_SYMBOLS[c_idx];
        let color_letter = COLOR_LETTERS[c_idx];

        let r_idx = pick_rarity(ctx.chaos);
        let rarity = RARITY_NAMES[r_idx];
        let rarity_icon = RARITY_ICONS[r_idx];

        let card_type = if ctx.args.trim().is_empty() {
            let t_idx = chaos_index(ctx.chaos, CARD_TYPES.len());
            CARD_TYPES[t_idx].to_string()
        } else {
            let arg = ctx.args.trim().to_lowercase();
            match CARD_TYPES.iter().find(|t| t.to_lowercase() == arg) {
                Some(t) => (*t).to_string(),
                None => {
                    return Ok(SkillOutput {
                        display: format!(
                            "  {RED}✗ Unknown card type: {}{RESET}\n  Valid types: creature, instant, sorcery, enchantment, artifact",
                            ctx.args.trim()
                        ),
                        feedback: vec![],
                        inject_to_conversation: false,
                    });
                }
            }
        };

        let (philosophy, flavor_tone) = load_color_philosophy(&self.rt.cardforge_path(), color);

        let color_title = {
            let mut chars = color.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };

        let mut system_prompt = format!(
            "You are a Magic: The Gathering card designer following the Vision Design → Set Design → Play Design methodology.\n\n\
             COLOR IDENTITY: {color_title} ({color_letter})\n\
             PHILOSOPHY: {}\n\
             CARD TYPE: {card_type}\n\
             RARITY: {rarity}\n\n\
             DESIGN RULES:\n\
             - Follow the Color Pie strictly. This {color} card must only use abilities appropriate to {color}.\n\
             - Rarity determines complexity: Common=simple, Uncommon=moderate, Rare=complex, Mythic=splashy and game-warping.\n\
             - Use strict MTG templating for rules text (identical effects = identical phrasing).\n\
             - Flavor text must perform heavy narrative lifting — convey vast philosophical tenets concisely.\n\
             - Flavor tone for {color}: {}\n\
             - If Creature: assign a relevant creature type, and Power/Toughness balanced for the mana cost.\n\
             - Mana cost must be balanced: higher cost = more powerful effect.\n\
             - Do NOT overcrowd — max 2 abilities for Common/Uncommon, max 3 for Rare/Mythic.\n\n\
             OUTPUT FORMAT (exactly this, no other text — do NOT wrap in code blocks, do NOT explain your reasoning):\n\
             NAME: [card name]\n\
             COST: [mana cost like {{2}}{{W}} or {{3}}{{R}}{{R}}]\n\
             TYPE: [full type line like 'Creature — Human Wizard' or 'Instant']\n\
             RARITY: {rarity}\n\
             RULES: [rules text, use | for line breaks between abilities]\n\
             FLAVOR: [italic flavor text, max 2 sentences]\n\
             PT: [Power/Toughness like 3/4, or NONE if not a creature]",
            if philosophy.is_empty() {
                "Color Pie default"
            } else {
                philosophy.as_str()
            },
            if flavor_tone.is_empty() {
                "balanced"
            } else {
                flavor_tone.as_str()
            },
        );

        let user_prompt = "Design one original Magic: The Gathering card. Make it memorable.";
        let mut parsed = ParsedCard::default();
        let mut temp = 0.9f64;

        for attempt in 0..3 {
            let raw = llm_chat(&self.rt, &system_prompt, user_prompt, temp, 4096, true).await;

            match raw {
                Ok(text) if !text.is_empty() => {
                    parsed = parse_structured_card(&text);
                    if !parsed.name.is_empty()
                        && !parsed.cost.is_empty()
                        && !parsed.type_line.is_empty()
                    {
                        break;
                    }
                }
                _ if attempt == 0 => {
                    return Ok(SkillOutput {
                        display: format!(
                            "  {RED}✗ LLM call failed. The Card Forge lies cold.{RESET}"
                        ),
                        feedback: vec![],
                        inject_to_conversation: false,
                    });
                }
                _ => {}
            }

            temp = 0.5;
            system_prompt = format!(
                "You are a Magic: The Gathering card designer.\n\n\
                 COLOR: {color_title} ({color_letter})\n\
                 TYPE: {card_type}\n\
                 RARITY: {rarity}\n\n\
                 Follow the Color Pie. Balance mana cost with power. Use strict MTG templating.\n\n\
                 Output exactly:\n\
                 NAME: [name]\n\
                 COST: [mana cost]\n\
                 TYPE: [type line]\n\
                 RARITY: {rarity}\n\
                 RULES: [rules text, use | for line breaks]\n\
                 FLAVOR: [flavor text]\n\
                 PT: [Power/Toughness or NONE]"
            );
        }

        if parsed.name.is_empty() {
            return Ok(SkillOutput {
                display: format!(
                    "  {RED}✗ LLM output did not match expected format after 3 attempts.{RESET}"
                ),
                feedback: vec![],
                inject_to_conversation: false,
            });
        }

        let display = render_card(color, color_sym, &parsed, rarity_icon);

        let feedback_event = ChaosEvent::CardForged {
            name: parsed.name.clone(),
            card_type: parsed.type_line.clone(),
        };
        let _ = ctx.feedback_tx.send(feedback_event.clone()).await;

        Ok(SkillOutput {
            display,
            feedback: vec![feedback_event],
            inject_to_conversation: true,
        })
    }
}
