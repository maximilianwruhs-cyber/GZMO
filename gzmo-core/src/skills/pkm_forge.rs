//! Pokemon Card Forge knowledge base + TCG frame renderer (from `skills/pkmforge.toml`).

use std::path::Path;

use gzmo_chaos::pulse::ChaosSnapshot;
use serde::Deserialize;

use super::attractor_common::AttractorMeta;
use super::generative::{chaos_index, line_value};
use super::pkm_forge_brief::{derive_forge_mode, ForgeMode};

const ELEMENTS: &[&str] = &[
    "fire",
    "water",
    "grass",
    "electric",
    "psychic",
    "darkness",
    "metal",
    "colorless",
];

const WHITE: &str = "\x1b[38;2;255;255;255m";
const BLUE: &str = "\x1b[34m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const MAGENTA: &str = "\x1b[35m";
const GOLD: &str = "\x1b[38;2;212;175;55m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone, Deserialize)]
pub struct ElementEntry {
    pub symbol: String,
    pub mana: String,
    pub philosophy: String,
    pub flavor_tone: String,
    #[serde(default)]
    pub strengths: Vec<String>,
    #[serde(default)]
    pub weaknesses: Vec<String>,
    #[serde(default)]
    pub personality: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ElementsSection {
    fire: ElementEntry,
    water: ElementEntry,
    grass: ElementEntry,
    electric: ElementEntry,
    psychic: ElementEntry,
    darkness: ElementEntry,
    metal: ElementEntry,
    colorless: ElementEntry,
}

#[derive(Debug, Clone, Deserialize)]
struct CategoryEntry {
    name: String,
    required: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RarityEntry {
    name: String,
    symbol: String,
    icon: String,
    complexity: String,
}

#[derive(Debug, Clone, Deserialize)]
struct NameFragments {
    #[serde(default)]
    prefixes: Vec<String>,
    #[serde(default)]
    nouns: Vec<String>,
    #[serde(default)]
    actions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AbilitiesSection {
    ex_rule: Option<String>,
    #[serde(default)]
    status_effects: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct PkmforgeMeta {
    design_method: Option<String>,
    flavor_rule: Option<String>,
    text_rule: Option<String>,
    set_prefix: Option<String>,
    hp_range: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PkmforgeFile {
    meta: Option<PkmforgeMeta>,
    elements: ElementsSection,
    #[serde(default)]
    categories: Vec<CategoryEntry>,
    #[serde(default)]
    rarities: Vec<RarityEntry>,
    abilities: Option<AbilitiesSection>,
    name_fragments: Option<NameFragments>,
}

#[derive(Debug, Clone)]
pub struct ForgeSparks {
    pub keyword: String,
    pub subtype: String,
    pub name_seed: String,
}

#[derive(Debug, Clone)]
pub struct ForgeSelection {
    pub category: String,
    pub element: String,
    pub element_entry: ElementEntry,
    pub rarity: String,
    pub rarity_icon: String,
    pub rarity_complexity: String,
    pub forge_mode: ForgeMode,
    pub sparks: ForgeSparks,
    pub set_code: String,
    pub strength_hint: String,
    pub weakness_hint: String,
}

#[derive(Debug, Clone)]
pub struct ParsedPkm {
    pub name: String,
    pub category: String,
    pub element: String,
    pub hp: Option<u32>,
    pub stage: Option<String>,
    pub rarity: String,
    pub weakness: Option<String>,
    pub retreat: Option<u32>,
    pub attack_1: Option<String>,
    pub attack_2: Option<String>,
    pub ability: Option<String>,
    pub trainer_type: Option<String>,
    pub energy_type: Option<String>,
    pub effect: Option<String>,
    pub flavor: String,
    pub raw: String,
}

pub fn load_pkmforge(skills_dir: &Path) -> Option<PkmforgeFile> {
    let path = skills_dir.join("pkmforge.toml");
    let content = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}

pub fn pick_element(snap: &ChaosSnapshot) -> (&'static str, usize) {
    let idx = chaos_index(snap, ELEMENTS.len());
    (ELEMENTS[idx], idx)
}

pub fn pick_rarity(pkmforge: &PkmforgeFile, snap: &ChaosSnapshot) -> (String, String, String) {
    let weights = [88, 24, 7, 1, 1]; // Common, Uncommon, Rare, Ultra Rare, Secret Rare
    let total: u32 = weights.iter().sum();
    let roll = (chaos_index(snap, total as usize) + 1) as u32;
    let mut cumulative = 0_u32;

    let names = ["Common", "Uncommon", "Rare", "Ultra Rare", "Secret Rare"];
    let icons = ["⚪", "🔵", "🟡", "🟠", "✨"];
    let mut complexities = vec![];
    if pkmforge.rarities.len() >= 5 {
        for r in &pkmforge.rarities {
            complexities.push(r.complexity.clone());
        }
    } else {
        complexities = vec![
            "Simple cards, basic attacks.".to_string(),
            "Minor utility, secondary attacks.".to_string(),
            "Signature attacks, standard abilities.".to_string(),
            "ex/V style rules, major impact, high HP.".to_string(),
            "Alternate representation, maximum prestige.".to_string(),
        ];
    }

    for (idx, weight) in weights.iter().enumerate() {
        cumulative += *weight as u32;
        if roll <= cumulative {
            return (
                names[idx].to_string(),
                icons[idx].to_string(),
                complexities[idx].clone(),
            );
        }
    }
    (
        names[0].to_string(),
        icons[0].to_string(),
        complexities[0].clone(),
    )
}

fn element_entry(pkmforge: &PkmforgeFile, element: &str) -> ElementEntry {
    match element {
        "water" => pkmforge.elements.water.clone(),
        "grass" => pkmforge.elements.grass.clone(),
        "electric" => pkmforge.elements.electric.clone(),
        "psychic" => pkmforge.elements.psychic.clone(),
        "darkness" => pkmforge.elements.darkness.clone(),
        "metal" => pkmforge.elements.metal.clone(),
        "colorless" => pkmforge.elements.colorless.clone(),
        _ => pkmforge.elements.fire.clone(),
    }
}

fn pick_sparks(pkmforge: &PkmforgeFile, snap: &ChaosSnapshot) -> ForgeSparks {
    let keywords = pkmforge
        .abilities
        .as_ref()
        .map(|a| a.status_effects.clone())
        .unwrap_or_else(|| vec!["Poisoned".into(), "Paralyzed".into()]);

    let keyword = keywords
        .get(chaos_index(snap, keywords.len()))
        .cloned()
        .unwrap_or_else(|| "Paralyzed".into());

    let subtype = "Basic".to_string();

    let name_seed = pkmforge
        .name_fragments
        .as_ref()
        .map(|nf| {
            let prefix = nf
                .prefixes
                .get(chaos_index(snap, nf.prefixes.len().max(1)))
                .cloned()
                .unwrap_or_else(|| "Volt".into());
            let noun = nf
                .nouns
                .get(chaos_index(snap, nf.nouns.len().max(1)))
                .cloned()
                .unwrap_or_else(|| "ix".into());
            format!("{}{}", prefix, noun)
        })
        .unwrap_or_else(|| "Voltix".into());

    ForgeSparks {
        keyword,
        subtype,
        name_seed,
    }
}

pub fn resolve_pkm_category(
    pkmforge: &PkmforgeFile,
    args: &str,
    snap: &ChaosSnapshot,
) -> Result<String, String> {
    let valid: Vec<String> = pkmforge.categories.iter().map(|c| c.name.clone()).collect();
    if args.trim().is_empty() {
        let idx = chaos_index(snap, valid.len());
        return Ok(valid[idx].clone());
    }
    let arg = capitalize_pkm_arg(args.trim());
    if valid.iter().any(|c| c.eq_ignore_ascii_case(&arg)) {
        Ok(arg)
    } else {
        Err(format!(
            "✗ Unknown category: {}\n  Valid: {}",
            args.trim(),
            valid
                .iter()
                .map(|c| c.to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

pub fn build_selection(
    pkmforge: &PkmforgeFile,
    snap: &ChaosSnapshot,
    category: &str,
    call_serial: u64,
) -> ForgeSelection {
    let (element, _) = pick_element(snap);
    let (rarity, rarity_icon, rarity_complexity) = pick_rarity(pkmforge, snap);
    let element_ent = element_entry(pkmforge, element);
    let sparks = pick_sparks(pkmforge, snap);
    let forge_mode = derive_forge_mode(snap);
    let prefix = pkmforge
        .meta
        .as_ref()
        .and_then(|m| m.set_prefix.as_deref())
        .unwrap_or("PKM");
    let set_code = format!("{prefix}{:03}", call_serial % 1000);

    let strength_hint = element_ent
        .strengths
        .get(chaos_index(snap, element_ent.strengths.len().max(1)))
        .cloned()
        .unwrap_or_else(|| "high damage".into());
    let weakness_hint = element_ent
        .weaknesses
        .get(chaos_index(snap, element_ent.weaknesses.len().max(1)))
        .cloned()
        .unwrap_or_else(|| "Water".into());

    ForgeSelection {
        category: category.to_string(),
        element: element.to_string(),
        element_entry: element_ent,
        rarity,
        rarity_icon,
        rarity_complexity,
        forge_mode,
        sparks,
        set_code,
        strength_hint,
        weakness_hint,
    }
}

pub fn build_system_prompt(pkmforge: &PkmforgeFile, sel: &ForgeSelection) -> String {
    let meta = pkmforge.meta.as_ref();
    let design = meta
        .and_then(|m| m.design_method.as_deref())
        .unwrap_or("Concept Design → Archetype Design → Balance Design");
    let flavor_rule = meta
        .and_then(|m| m.flavor_rule.as_deref())
        .unwrap_or("Flavor text should capture the essence of the Pokemon.");
    let text_rule = meta
        .and_then(|m| m.text_rule.as_deref())
        .unwrap_or("Rules text must follow standard Pokemon TCG format.");

    let personality = if sel.element_entry.personality.is_empty() {
        sel.element_entry.philosophy.clone()
    } else {
        sel.element_entry.personality.clone()
    };

    let ex_text = pkmforge
        .abilities
        .as_ref()
        .and_then(|a| a.ex_rule.as_deref())
        .unwrap_or("When your Pokemon ex is Knocked Out, your opponent takes 2 Prize cards.");

    let category_specific = match sel.category.as_str() {
        "Pokemon" => {
            format!(
                "CATEGORY: Pokemon\n\
                 ELEMENT: {}\n\
                 HP: [Pick a value between 30 and 340, ending in 0, typical for rarity: {}]\n\
                 STAGE: [Basic, Stage 1, or Stage 2]\n\
                 WEAKNESS: [{} x2]\n\
                 RETREAT: [retreat cost, typically 1 to 4]\n\
                 ATTACK1: [Name | Cost (like {{C}} or {{L}}) | damage, e.g. 30 | Description]\n\
                 ATTACK2: [Name | Cost | damage, e.g. 80 | Description (optional, omit for simple commons)]\n\
                 ABILITY: [Name | Effect text (optional, e.g. Static Charge | When you attach a {{{}}} Energy to this Pokemon, draw a card.)]",
                sel.element,
                sel.rarity,
                sel.weakness_hint,
                sel.element_entry.symbol
            )
        }
        "Trainer" => {
            "CATEGORY: Trainer\n\
             TRAINER_TYPE: [Item, Supporter, or Stadium]\n\
             EFFECT: [Rule text describing what this card does. Item cards let you perform small operations, Supporter cards let you draw cards or do major operations (limit 1 per turn), Stadium cards stay on the battlefield and affect both players.]".to_string()
        }
        "Energy" => {
            "CATEGORY: Energy\n\
             ENERGY_TYPE: [Basic or Special]\n\
             EFFECT: [Rule text if Special Energy. Basic Energy has no effect (just provides energy).]".to_string()
        }
        _ => String::new(),
    };

    format!(
        "You are a Pokemon Trading Card Game designer following {design}.\n\
         FORGE LENS: {} — {}\n\
         SET CODE: {} (collector identity for this forge)\n\
         ELEMENT IDENTITY: {} ({}) — {}\n\
         PERSONALITY: {}\n\
         CATEGORY: {} \n\
         RARITY: {} — {}\n\
         FORGE SPARKS (weave in subtly — do not paste verbatim):\n\
         - Status effect hint: {}\n\
         - Name seed (transform, do not copy): {}\n\
         DESIGN RULES:\n\
         - Lean into {} mechanics; avoid {} as a primary effect.\n\
         - Follow Pokemon TCG mechanics strictly.\n\
         - Rarity complexity: {}\n\
         - {}\n\
         - {}\n\
         - Flavor tone for {}: {}\n\
         - If designing an Ultra Rare or Secret Rare 'ex' Pokemon, always append the ex rule: {}\n\
         OUTPUT FORMAT (exactly this, no other text, no markdown block wrappers):\n\
         NAME: [card name]\n\
         {}\n\
         RARITY: {}\n\
         FLAVOR: [flavor text, max 2 sentences]\n\
         CATEGORY: {}",
        sel.forge_mode.label(),
        sel.forge_mode.directive(),
        sel.set_code,
        capitalize_pkm_arg(&sel.element),
        sel.element_entry.symbol,
        sel.element_entry.philosophy,
        personality,
        sel.category,
        sel.rarity,
        sel.rarity_complexity,
        sel.sparks.keyword,
        sel.sparks.name_seed,
        sel.strength_hint,
        sel.weakness_hint,
        sel.rarity_complexity,
        text_rule,
        flavor_rule,
        sel.element,
        sel.element_entry.flavor_tone,
        ex_text,
        category_specific,
        sel.rarity,
        sel.category
    )
}

pub fn build_user_prompt(meta: &AttractorMeta, sel: &ForgeSelection) -> String {
    let mut lines = vec![
        format!(
            "Forge one original {} {} {} card for set {}.",
            sel.rarity,
            capitalize_pkm_arg(&sel.element),
            sel.category,
            sel.set_code
        ),
        format!(
            "Forge lens: {} — {}",
            sel.forge_mode.label(),
            sel.forge_mode.directive()
        ),
        format!(
            "Attractor state: tick {}, phase {}, valence {:.2}, rho {:.2}, invocation #{}",
            meta.tick, meta.phase, meta.valence, meta.rho_effective, meta.call_serial
        ),
        format!("Nonce: {} (unique per forge)", meta.nonce),
    ];
    if let Some(echo) = &meta.cabinet_echo {
        lines.push(format!(
            "Let incubating thought \"{echo}\" inform name or flavor — do not copy verbatim."
        ));
    }
    if !meta.anti_repeat_hint.is_empty() {
        lines.push(meta.anti_repeat_hint.clone());
    }
    lines.push("Make it memorable, mechanically coherent, and worthy of the set.".to_string());
    lines.join("\n")
}

pub fn validate_forged_pokemon(text: &str, category: &str) -> bool {
    let slop = [
        "as an ai",
        "i cannot",
        "placeholder",
        "lorem ipsum",
        "[card name]",
        "[mana cost",
        "[rules text",
    ];
    if slop.iter().any(|s| text.to_lowercase().contains(s)) {
        return false;
    }

    // Check common requirements
    for prefix in ["NAME:", "CATEGORY:", "RARITY:"] {
        if !text.lines().any(|l| l.starts_with(prefix)) {
            return false;
        }
    }

    let name = line_value(text, "NAME:").unwrap_or("");
    if name.len() < 3 {
        return false;
    }

    match category {
        "Pokemon" => {
            for prefix in ["ELEMENT:", "HP:", "STAGE:", "ATTACK1:"] {
                if !text.lines().any(|l| l.starts_with(prefix)) {
                    return false;
                }
            }
            let hp_str = line_value(text, "HP:").unwrap_or("");
            let hp = hp_str.trim().parse::<u32>().unwrap_or(0);
            if hp < 30 || hp > 340 || hp % 10 != 0 {
                return false;
            }
        }
        "Trainer" => {
            for prefix in ["TRAINER_TYPE:", "EFFECT:"] {
                if !text.lines().any(|l| l.starts_with(prefix)) {
                    return false;
                }
            }
        }
        "Energy" => {
            if !text.lines().any(|l| l.starts_with("ENERGY_TYPE:")) {
                return false;
            }
        }
        _ => return false,
    }

    true
}

pub fn parse_pkm(raw: &str, fallback_category: &str) -> ParsedPkm {
    let name = line_value(raw, "NAME:")
        .unwrap_or("Unnamed Card")
        .to_string();
    let category = line_value(raw, "CATEGORY:")
        .unwrap_or(fallback_category)
        .to_string();
    let element = line_value(raw, "ELEMENT:")
        .unwrap_or("colorless")
        .to_string();
    let hp = line_value(raw, "HP:").and_then(|s| s.trim().parse::<u32>().ok());
    let stage = line_value(raw, "STAGE:").map(String::from);
    let rarity = line_value(raw, "RARITY:").unwrap_or("Common").to_string();
    let weakness = line_value(raw, "WEAKNESS:").map(String::from);
    let retreat = line_value(raw, "RETREAT:").and_then(|s| s.trim().parse::<u32>().ok());

    let attack_1 = line_value(raw, "ATTACK1:").map(String::from);
    let attack_2 = line_value(raw, "ATTACK2:").map(String::from);
    let ability = line_value(raw, "ABILITY:").map(String::from);

    let trainer_type = line_value(raw, "TRAINER_TYPE:").map(String::from);
    let energy_type = line_value(raw, "ENERGY_TYPE:").map(String::from);
    let effect = line_value(raw, "EFFECT:").map(String::from);

    let flavor = line_value(raw, "FLAVOR:").unwrap_or("").to_string();

    ParsedPkm {
        name,
        category,
        element,
        hp,
        stage,
        rarity,
        weakness,
        retreat,
        attack_1,
        attack_2,
        ability,
        trainer_type,
        energy_type,
        effect,
        flavor,
        raw: raw.to_string(),
    }
}

pub fn is_ultra_or_secret(sel: &ForgeSelection) -> bool {
    sel.rarity.eq_ignore_ascii_case("Ultra Rare") || sel.rarity.eq_ignore_ascii_case("Secret Rare")
}

pub fn render_forge_display(
    meta: &AttractorMeta,
    sel: &ForgeSelection,
    parsed: &ParsedPkm,
) -> String {
    let ultra_or_secret = is_ultra_or_secret(sel);
    let header = format!(
        "⚡ POCKET FORGE · SET {set}\n  \
         {rarity_icon} {rarity} {category} · {mode} · inv #{inv} · #{coll}\n  \
         tick {tick} · phase {phase} · valence {val:.2} · ρ {rho:.2}",
        set = sel.set_code,
        rarity_icon = sel.rarity_icon,
        rarity = sel.rarity,
        category = sel.category,
        mode = sel.forge_mode.label(),
        inv = meta.call_serial,
        coll = meta.call_serial,
        tick = meta.tick,
        phase = meta.phase,
        val = meta.valence,
        rho = meta.rho_effective,
    );

    let sparks = format!(
        "sparks: {} · {} · \"{}\"",
        sel.sparks.keyword, sel.sparks.subtype, sel.sparks.name_seed
    );

    let frame = render_pkm_frame(sel, parsed, ultra_or_secret);

    let footer = if ultra_or_secret {
        format!(
            "{GOLD}  ✦ MYTHIC EX RESONANCE — tension +3 · energy −4 · attractor ripple{RESET}\n  \
             crystallize: ~35 ticks → friction −0.03"
        )
    } else if let Some(echo) = &meta.cabinet_echo {
        let truncated = if echo.chars().count() > 40 {
            let taken: String = echo.chars().take(37).collect();
            format!("{taken}...")
        } else {
            echo.clone()
        };
        format!("incubating echo: \"{truncated}\"\n  crystallize: ~35 ticks → friction −0.03")
    } else {
        "crystallize: ~35 ticks → friction −0.03".to_string()
    };

    format!(
        "\n┌─────────────────────────────────────────────────┐\n  {header}\n  {DIM}{sparks}{RESET}\n├─────────────────────────────────────────────────┤\n{frame}\n├─────────────────────────────────────────────────┤\n  {footer}\n└─────────────────────────────────────────────────┘\n"
    )
}

fn split_ability(ability: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = ability.split('|').collect();
    if parts.len() >= 2 {
        Some((
            parts[0].trim().to_string(),
            parts[1..].join("|").trim().to_string(),
        ))
    } else {
        None
    }
}

fn split_attack(attack: &str) -> Option<(String, String, String, String)> {
    let parts: Vec<&str> = attack.split('|').collect();
    if parts.len() == 3 {
        // Name | Damage | Description
        Some((
            parts[0].trim().to_string(),
            "".to_string(),
            parts[1].trim().to_string(),
            parts[2].trim().to_string(),
        ))
    } else if parts.len() >= 4 {
        // Name | Cost | Damage | Description
        Some((
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
            parts[2].trim().to_string(),
            parts[3..].join("|").trim().to_string(),
        ))
    } else {
        None
    }
}

fn wrap_rules_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('|') {
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    lines
}

fn render_pkm_frame(sel: &ForgeSelection, card: &ParsedPkm, ultra_or_secret: bool) -> String {
    let border = match sel.element.as_str() {
        "fire" => RED,
        "water" => BLUE,
        "grass" => GREEN,
        "electric" => GOLD,
        "psychic" => MAGENTA,
        "darkness" => DIM,
        "metal" => BOLD,
        _ => WHITE,
    };

    let mut lines = vec![format!(
        "{border}  ╔═══════════════════════════════════════════╗{RESET}"
    )];
    if ultra_or_secret {
        lines.push(format!(
            "{border}  ║{RESET} {GOLD}{BOLD}✦ ULTRA EX FORGE ✦{RESET}{:>26}",
            ""
        ));
    }

    if card.category == "Pokemon" {
        let hp_str = card.hp.map(|h| format!("{h} HP")).unwrap_or_default();
        let stage_str = card.stage.as_deref().unwrap_or("Basic");
        let name_stage = format!("{} ({})", card.name, stage_str);

        let spaces = 41 - name_stage.len() - hp_str.len();
        let spaces_str = " ".repeat(spaces.max(1));
        lines.push(format!(
            "{border}  ║{RESET} {BOLD}{}{}{}{RESET}",
            name_stage, spaces_str, hp_str
        ));
    } else if card.category == "Trainer" {
        let trainer_type = card.trainer_type.as_deref().unwrap_or("Item");
        let name_str = format!("{} [{}]", card.name, trainer_type);
        lines.push(format!(
            "{border}  ║{RESET} {BOLD}{}{:<width$}{RESET}",
            name_str,
            "",
            width = (41 - name_str.len()).max(1)
        ));
    } else {
        let energy_type = card.energy_type.as_deref().unwrap_or("Basic");
        let name_str = format!("{} [{} Energy]", card.name, energy_type);
        lines.push(format!(
            "{border}  ║{RESET} {BOLD}{}{:<width$}{RESET}",
            name_str,
            "",
            width = (41 - name_str.len()).max(1)
        ));
    }

    lines.push(format!(
        "{border}  ║{RESET} {:>35} {DIM}{}{RESET}",
        "", sel.set_code
    ));
    lines.push(format!(
        "{border}  ╠═══════════════════════════════════════════╣{RESET}"
    ));

    if card.category == "Pokemon" {
        lines.push(format!(
            "{border}  ║{RESET}  {} {DIM}{}{RESET}",
            sel.element_entry.symbol, sel.rarity
        ));
        lines.push(format!("{border}  ║{RESET}"));

        if let Some(ability) = &card.ability {
            if let Some((ab_name, ab_desc)) = split_ability(ability) {
                lines.push(format!(
                    "{border}  ║{RESET}  {RED}{BOLD}Ability: {ab_name}{RESET}"
                ));
                for part in wrap_rules_text(&ab_desc, 39) {
                    lines.push(format!("{border}  ║{RESET}    {WHITE}{part}{RESET}"));
                }
                lines.push(format!("{border}  ║{RESET}"));
            }
        }

        if let Some(attack1) = &card.attack_1 {
            if let Some((atk_name, atk_cost, atk_dmg, atk_desc)) = split_attack(attack1) {
                let cost_block = format!("{} {}", atk_cost, atk_name);
                let spaces = 37 - cost_block.len() - atk_dmg.len();
                let spaces_str = " ".repeat(spaces.max(1));
                lines.push(format!(
                    "{border}  ║{RESET}  {BOLD}{}{}{}{RESET}",
                    cost_block, spaces_str, atk_dmg
                ));
                for part in wrap_rules_text(&atk_desc, 39) {
                    lines.push(format!("{border}  ║{RESET}    {WHITE}{part}{RESET}"));
                }
                lines.push(format!("{border}  ║{RESET}"));
            }
        }

        if let Some(attack2) = &card.attack_2 {
            if let Some((atk_name, atk_cost, atk_dmg, atk_desc)) = split_attack(attack2) {
                let cost_block = format!("{} {}", atk_cost, atk_name);
                let spaces = 37 - cost_block.len() - atk_dmg.len();
                let spaces_str = " ".repeat(spaces.max(1));
                lines.push(format!(
                    "{border}  ║{RESET}  {BOLD}{}{}{}{RESET}",
                    cost_block, spaces_str, atk_dmg
                ));
                for part in wrap_rules_text(&atk_desc, 39) {
                    lines.push(format!("{border}  ║{RESET}    {WHITE}{part}{RESET}"));
                }
                lines.push(format!("{border}  ║{RESET}"));
            }
        }

        let weak_str = card.weakness.as_deref().unwrap_or("None");
        let retreat_str = card
            .retreat
            .map(|r| r.to_string())
            .unwrap_or_else(|| "0".to_string());
        let weak_retreat = format!("Weakness: {}  Retreat: {}", weak_str, retreat_str);
        lines.push(format!("{border}  ║{RESET}  {DIM}{weak_retreat}{RESET}"));
    } else {
        lines.push(format!("{border}  ║{RESET}"));
        if let Some(effect) = &card.effect {
            for part in wrap_rules_text(effect, 39) {
                lines.push(format!("{border}  ║{RESET}  {WHITE}{part}{RESET}"));
            }
        }
        lines.push(format!("{border}  ║{RESET}"));
    }

    if !card.flavor.is_empty() {
        for part in wrap_rules_text(&card.flavor, 39) {
            lines.push(format!("{border}  ║{RESET}  {DIM}{MAGENTA}{part}{RESET}"));
        }
        lines.push(format!("{border}  ║{RESET}"));
    }

    lines.push(format!(
        "{border}  ╚═══════════════════════════════════════════╝{RESET}"
    ));
    lines.join("\n")
}

pub fn capitalize_pkm_arg(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
    }
}

pub fn build_pkm_evidence(
    parsed: &ParsedPkm,
    sel: &ForgeSelection,
    meta: &AttractorMeta,
    display: &str,
    body_hash: &str,
    feedback: &[ChaosEventRef<'_>],
) -> serde_json::Value {
    serde_json::json!({
        "skill": "pkm",
        "inv": meta.call_serial,
        "set_code": sel.set_code,
        "display_plain": display,
        "body_hash": body_hash,
        "ultra_or_secret": is_ultra_or_secret(sel),
        "card": {
            "name": parsed.name,
            "category": parsed.category,
            "element": parsed.element,
            "hp": parsed.hp,
            "stage": parsed.stage,
            "rarity": parsed.rarity,
            "weakness": parsed.weakness,
            "retreat": parsed.retreat,
            "attack_1": parsed.attack_1,
            "attack_2": parsed.attack_2,
            "ability": parsed.ability,
            "trainer_type": parsed.trainer_type,
            "energy_type": parsed.energy_type,
            "effect": parsed.effect,
            "flavor": parsed.flavor,
        },
        "forge": {
            "mode": sel.forge_mode.label(),
            "sparks": {
                "keyword": sel.sparks.keyword,
                "subtype": sel.sparks.subtype,
                "name_seed": sel.sparks.name_seed,
            }
        },
        "chaos": {
            "tick": meta.tick,
            "phase": meta.phase.to_string(),
            "valence": meta.valence,
            "rho_effective": meta.rho_effective,
            "nonce": meta.nonce,
        },
        "feedback": feedback,
    })
}

pub struct ChaosEventRef<'a> {
    pub kind: &'a str,
    pub detail: String,
}

impl<'a> serde::Serialize for ChaosEventRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ChaosEventRef", 2)?;
        s.serialize_field("type", self.kind)?;
        s.serialize_field("detail", &self.detail)?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pkm_fields() {
        let raw = "NAME: Voltix\nCATEGORY: Pokemon\nELEMENT: electric\nHP: 120\nSTAGE: Basic\nRARITY: Rare\nWEAKNESS: Fighting x2\nRETREAT: 1\nATTACK1: Thunder Bolt | {L}{C} | 30 | This attack does 30 damage.\nFLAVOR: Its tail stores voltage.";
        let p = parse_pkm(raw, "Pokemon");
        assert_eq!(p.name, "Voltix");
        assert_eq!(p.hp, Some(120));
        assert_eq!(p.retreat, Some(1));
    }

    #[test]
    fn validate_rejects_incomplete_pokemon() {
        assert!(!validate_forged_pokemon("NAME: X\nHP: 120", "Pokemon"));
        assert!(validate_forged_pokemon(
            "NAME: Voltix\nCATEGORY: Pokemon\nELEMENT: electric\nHP: 120\nSTAGE: Basic\nRARITY: Rare\nATTACK1: Thunder Bolt | {L} | 30 | desc",
            "Pokemon"
        ));
    }
}
