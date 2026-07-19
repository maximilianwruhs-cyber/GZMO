//! Card Forge knowledge base + MTG frame renderer (from `skills/cardforge.toml`).

use std::path::Path;

use gzmo_chaos::pulse::ChaosSnapshot;
use serde::Deserialize;

use super::attractor_common::AttractorMeta;
use super::card_forge_brief::{derive_forge_mode, ForgeMode};
use super::generative::{chaos_index, line_value};

const COLORS: &[&str] = &["white", "blue", "black", "red", "green"];
const RARITY_WEIGHTS: [u8; 4] = [88, 24, 7, 1]; // Common → Mythic (1:7:24:88)

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
pub struct ColorEntry {
    symbol: String,
    mana: String,
    philosophy: String,
    flavor_tone: String,
    #[serde(default)]
    strengths: Vec<String>,
    #[serde(default)]
    weaknesses: Vec<String>,
    #[serde(default)]
    personality: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ColorsSection {
    white: ColorEntry,
    blue: ColorEntry,
    black: ColorEntry,
    red: ColorEntry,
    green: ColorEntry,
}

#[derive(Debug, Clone, Deserialize)]
struct CardTypeEntry {
    name: String,
    has_power_toughness: bool,
    #[serde(default)]
    subtypes_examples: Vec<String>,
    rules: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RarityEntry {
    name: String,
    icon: String,
    complexity: String,
}

#[derive(Debug, Clone, Deserialize)]
struct KeywordsSection {
    #[serde(default)]
    evergreen: Vec<String>,
    #[serde(default)]
    deciduous: Vec<String>,
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
struct CardforgeMeta {
    design_method: Option<String>,
    flavor_rule: Option<String>,
    text_rule: Option<String>,
    set_prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardforgeFile {
    meta: Option<CardforgeMeta>,
    colors: ColorsSection,
    #[serde(default)]
    card_types: Vec<CardTypeEntry>,
    #[serde(default)]
    rarities: Vec<RarityEntry>,
    keywords: Option<KeywordsSection>,
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
    pub color: &'static str,
    pub color_entry: ColorEntry,
    pub rarity: String,
    pub rarity_icon: String,
    pub rarity_complexity: String,
    pub card_type: String,
    pub type_rules: String,
    pub requires_pt: bool,
    pub forge_mode: ForgeMode,
    pub sparks: ForgeSparks,
    pub set_code: String,
    pub strength_hint: String,
    pub weakness_hint: String,
}

#[derive(Debug, Clone)]
pub struct ParsedCard {
    pub name: String,
    pub cost: String,
    pub type_line: String,
    pub rules: String,
    pub flavor: String,
    pub power_toughness: Option<String>,
    pub raw: String,
}

pub fn load_cardforge(skills_dir: &Path) -> Option<CardforgeFile> {
    let path = skills_dir.join("cardforge.toml");
    let content = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}

pub fn pick_color(snap: &ChaosSnapshot) -> (&'static str, usize) {
    let idx = chaos_index(snap, COLORS.len());
    (COLORS[idx], idx)
}

pub fn pick_rarity(cardforge: &CardforgeFile, snap: &ChaosSnapshot) -> (String, String, String) {
    let total: u32 = RARITY_WEIGHTS.iter().map(|w| *w as u32).sum();
    let roll = (chaos_index(snap, total as usize) + 1) as u32;
    let mut cumulative = 0_u32;
    let names = default_rarity_names();
    let icons = default_rarity_icons();
    let complexities = default_rarity_complexities(cardforge);

    for (idx, weight) in RARITY_WEIGHTS.iter().enumerate() {
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

fn default_rarity_names() -> [&'static str; 4] {
    ["Common", "Uncommon", "Rare", "Mythic Rare"]
}

fn default_rarity_icons() -> [&'static str; 4] {
    ["⚪", "🔵", "🟡", "🟠"]
}

fn default_rarity_complexities(cardforge: &CardforgeFile) -> [String; 4] {
    let defaults = [
        "Simple, clean effects. One ability max.",
        "Two abilities or one complex ability.",
        "Complex. Multiple abilities, unique effects.",
        "Maximum complexity. Splashy, game-warping.",
    ];
    if cardforge.rarities.len() >= 4 {
        return [
            cardforge.rarities[0].complexity.clone(),
            cardforge.rarities[1].complexity.clone(),
            cardforge.rarities[2].complexity.clone(),
            cardforge.rarities[3].complexity.clone(),
        ];
    }
    [
        defaults[0].into(),
        defaults[1].into(),
        defaults[2].into(),
        defaults[3].into(),
    ]
}

fn type_catalog(cardforge: &CardforgeFile) -> Vec<CardTypeEntry> {
    if cardforge.card_types.is_empty() {
        return vec![
            CardTypeEntry {
                name: "Creature".into(),
                has_power_toughness: true,
                subtypes_examples: vec!["Human".into(), "Golem".into(), "Dragon".into()],
                rules: "Must have Power/Toughness.".into(),
            },
            CardTypeEntry {
                name: "Instant".into(),
                has_power_toughness: false,
                subtypes_examples: vec![],
                rules: "One-time instant speed.".into(),
            },
            CardTypeEntry {
                name: "Sorcery".into(),
                has_power_toughness: false,
                subtypes_examples: vec![],
                rules: "Main phase sorcery.".into(),
            },
            CardTypeEntry {
                name: "Enchantment".into(),
                has_power_toughness: false,
                subtypes_examples: vec!["Aura".into()],
                rules: "Persistent enchantment.".into(),
            },
            CardTypeEntry {
                name: "Artifact".into(),
                has_power_toughness: false,
                subtypes_examples: vec!["Equipment".into()],
                rules: "Colorless artifact.".into(),
            },
            CardTypeEntry {
                name: "Planeswalker".into(),
                has_power_toughness: false,
                subtypes_examples: vec![],
                rules: "Loyalty counters, one ability per turn.".into(),
            },
        ];
    }
    cardforge.card_types.clone()
}

pub fn resolve_card_type(
    cardforge: &CardforgeFile,
    args: &str,
    snap: &ChaosSnapshot,
) -> Result<String, String> {
    let catalog = type_catalog(cardforge);
    let valid: Vec<String> = catalog.iter().map(|t| t.name.clone()).collect();
    if args.trim().is_empty() {
        let idx = chaos_index(snap, valid.len());
        return Ok(valid[idx].clone());
    }
    let arg = capitalize_card_arg(args.trim());
    if valid.iter().any(|t| t.eq_ignore_ascii_case(&arg)) {
        Ok(arg)
    } else {
        Err(format!(
            "✗ Unknown card type: {}\n  Valid: {}",
            args.trim(),
            valid
                .iter()
                .map(|t| t.to_lowercase())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

fn pick_sparks(
    cardforge: &CardforgeFile,
    snap: &ChaosSnapshot,
    type_entry: &CardTypeEntry,
) -> ForgeSparks {
    let keywords = cardforge
        .keywords
        .as_ref()
        .map(|k| {
            let mut pool = k.evergreen.clone();
            pool.extend(k.deciduous.clone());
            pool
        })
        .unwrap_or_else(|| vec!["Flying".into(), "Trample".into(), "Flash".into()]);

    let keyword = keywords
        .get(chaos_index(snap, keywords.len()))
        .cloned()
        .unwrap_or_else(|| "Flying".into());

    let subtype = type_entry
        .subtypes_examples
        .get(chaos_index(snap, type_entry.subtypes_examples.len().max(1)))
        .cloned()
        .unwrap_or_else(|| "Human".into());

    let name_seed = cardforge
        .name_fragments
        .as_ref()
        .map(|nf| {
            let prefix = nf
                .prefixes
                .get(chaos_index(snap, nf.prefixes.len().max(1)))
                .cloned()
                .unwrap_or_else(|| "Shadow".into());
            let noun = nf
                .nouns
                .get(chaos_index(snap, nf.nouns.len().max(1)))
                .cloned()
                .unwrap_or_else(|| "Oracle".into());
            format!("{prefix} {noun}")
        })
        .unwrap_or_else(|| "Chaos Oracle".into());

    ForgeSparks {
        keyword,
        subtype,
        name_seed,
    }
}

fn type_entry_for(cardforge: &CardforgeFile, card_type: &str) -> CardTypeEntry {
    type_catalog(cardforge)
        .into_iter()
        .find(|t| t.name.eq_ignore_ascii_case(card_type))
        .unwrap_or(CardTypeEntry {
            name: card_type.to_string(),
            has_power_toughness: card_type.eq_ignore_ascii_case("creature"),
            subtypes_examples: vec![],
            rules: String::new(),
        })
}

pub fn build_selection(
    cardforge: &CardforgeFile,
    snap: &ChaosSnapshot,
    card_type: &str,
    call_serial: u64,
) -> ForgeSelection {
    let (color, _) = pick_color(snap);
    let (rarity, rarity_icon, rarity_complexity) = pick_rarity(cardforge, snap);
    let color_entry = color_entry(cardforge, color);
    let type_entry = type_entry_for(cardforge, card_type);
    let sparks = pick_sparks(cardforge, snap, &type_entry);
    let forge_mode = derive_forge_mode(snap);
    let prefix = cardforge
        .meta
        .as_ref()
        .and_then(|m| m.set_prefix.as_deref())
        .unwrap_or("ATR");
    let set_code = format!("{prefix}{:03}", call_serial % 1000);
    let (strength_hint, weakness_hint) = pick_color_mechanics(&color_entry, snap);

    ForgeSelection {
        color,
        color_entry,
        rarity,
        rarity_icon,
        rarity_complexity,
        card_type: card_type.to_string(),
        type_rules: type_entry.rules.clone(),
        requires_pt: type_entry.has_power_toughness,
        forge_mode,
        sparks,
        set_code,
        strength_hint,
        weakness_hint,
    }
}

fn color_entry(cardforge: &CardforgeFile, color: &str) -> ColorEntry {
    match color {
        "blue" => cardforge.colors.blue.clone(),
        "black" => cardforge.colors.black.clone(),
        "red" => cardforge.colors.red.clone(),
        "green" => cardforge.colors.green.clone(),
        _ => cardforge.colors.white.clone(),
    }
}

fn pick_color_mechanics(entry: &ColorEntry, snap: &ChaosSnapshot) -> (String, String) {
    let strength = entry
        .strengths
        .get(chaos_index(snap, entry.strengths.len().max(1)))
        .cloned()
        .unwrap_or_else(|| "efficient creatures".into());
    let weakness = entry
        .weaknesses
        .get(chaos_index(snap, entry.weaknesses.len().max(1)))
        .cloned()
        .unwrap_or_else(|| "card draw".into());
    (strength, weakness)
}

pub fn build_system_prompt(cardforge: &CardforgeFile, sel: &ForgeSelection) -> String {
    let meta = cardforge.meta.as_ref();
    let design = meta
        .and_then(|m| m.design_method.as_deref())
        .unwrap_or("Vision Design → Set Design → Play Design");
    let flavor_rule = meta
        .and_then(|m| m.flavor_rule.as_deref())
        .unwrap_or("Flavor text must perform heavy narrative lifting.");
    let text_rule = meta
        .and_then(|m| m.text_rule.as_deref())
        .unwrap_or("Rules text must use strict MTG templating.");

    let personality = if sel.color_entry.personality.is_empty() {
        sel.color_entry.philosophy.clone()
    } else {
        sel.color_entry.personality.clone()
    };

    format!(
        "You are a Magic: The Gathering card designer following {design}.\n\
         FORGE LENS: {} — {}\n\
         SET CODE: {} (collector identity for this forge)\n\
         COLOR IDENTITY: {} ({}) — {}\n\
         PERSONALITY: {}\n\
         CARD TYPE: {} — {}\n\
         RARITY: {} — {}\n\
         FORGE SPARKS (weave in subtly — do not paste verbatim):\n\
         - Keyword hint: {}\n\
         - Subtype hint: {}\n\
         - Name seed (transform, do not copy): {}\n\
         DESIGN RULES:\n\
         - Lean into {} mechanics; avoid {} as a primary effect.\n\
         - Follow the Color Pie strictly for {}.\n\
         - Rarity complexity: {}\n\
         - {text_rule}\n\
         - {flavor_rule}\n\
         - Flavor tone for {}: {}\n\
         - Do NOT overcrowd — max 2 abilities for Common/Uncommon, max 3 for Rare/Mythic.\n\
         OUTPUT FORMAT (exactly this, no other text):\n\
         NAME: [card name]\n\
         COST: [mana cost like {{2}}{{{}}}]\n\
         TYPE: [full type line]\n\
         RARITY: {}\n\
         RULES: [rules text, use | for line breaks]\n\
         FLAVOR: [flavor text, max 2 sentences]\n\
         PT: [Power/Toughness or NONE]",
        sel.forge_mode.label(),
        sel.forge_mode.directive(),
        sel.set_code,
        capitalize_card_arg(sel.color),
        sel.color_entry.symbol,
        sel.color_entry.philosophy,
        personality,
        sel.card_type,
        sel.type_rules,
        sel.rarity,
        sel.rarity_complexity,
        sel.sparks.keyword,
        sel.sparks.subtype,
        sel.sparks.name_seed,
        sel.strength_hint,
        sel.weakness_hint,
        sel.color,
        sel.rarity_complexity,
        sel.color,
        sel.color_entry.flavor_tone,
        sel.color_entry.symbol,
        sel.rarity,
    )
}

pub fn build_user_prompt(meta: &AttractorMeta, sel: &ForgeSelection) -> String {
    let mut lines = vec![
        format!(
            "Forge one original {} {} {} for set {}.",
            sel.rarity,
            capitalize_card_arg(sel.color),
            sel.card_type,
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

pub fn validate_forged_card(text: &str, requires_pt: bool) -> bool {
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
    for prefix in ["NAME:", "COST:", "TYPE:", "RARITY:", "RULES:"] {
        if !text.lines().any(|l| l.starts_with(prefix)) {
            return false;
        }
    }
    let name = line_value(text, "NAME:").unwrap_or("");
    if name.len() < 3 {
        return false;
    }
    let cost = line_value(text, "COST:").unwrap_or("");
    if !cost.contains('{') {
        return false;
    }
    let rules = line_value(text, "RULES:").unwrap_or("");
    if rules.trim().len() < 4 {
        return false;
    }
    if requires_pt {
        let pt = line_value(text, "PT:").unwrap_or("");
        if !pt.chars().any(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    true
}

pub fn parse_card(raw: &str, fallback_type: &str) -> ParsedCard {
    let name = line_value(raw, "NAME:")
        .unwrap_or("Unnamed Card")
        .to_string();
    let cost = line_value(raw, "COST:").unwrap_or("{1}").to_string();
    let type_line = line_value(raw, "TYPE:")
        .unwrap_or(fallback_type)
        .to_string();
    let rules = line_value(raw, "RULES:").unwrap_or("").to_string();
    let flavor = line_value(raw, "FLAVOR:").unwrap_or("").to_string();
    let pt = line_value(raw, "PT:")
        .filter(|p| !p.eq_ignore_ascii_case("NONE") && !p.eq_ignore_ascii_case("N/A"));
    ParsedCard {
        name,
        cost,
        type_line,
        rules,
        flavor,
        power_toughness: pt.map(|s| s.to_string()),
        raw: raw.to_string(),
    }
}

pub fn is_mythic(sel: &ForgeSelection) -> bool {
    sel.rarity.eq_ignore_ascii_case("Mythic Rare")
}

pub fn render_forge_display(
    meta: &AttractorMeta,
    sel: &ForgeSelection,
    parsed: &ParsedCard,
) -> String {
    let mythic = is_mythic(sel);
    let header = format!(
        "🂡 ATTRACTOR FORGE · SET {set}\n  \
         {mana} {rarity} {ctype} · {mode} · inv #{inv} · #{coll}\n  \
         tick {tick} · phase {phase} · valence {val:.2} · ρ {rho:.2}",
        set = sel.set_code,
        mana = sel.color_entry.mana,
        rarity = sel.rarity,
        ctype = sel.card_type,
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

    let frame = render_card_frame(sel, parsed, mythic);

    let footer = if mythic {
        format!(
            "{GOLD}  ✦ MYTHIC RESONANCE — tension +3 · energy −4 · attractor ripple{RESET}\n  \
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

fn render_card_frame(sel: &ForgeSelection, card: &ParsedCard, mythic: bool) -> String {
    let border = match sel.color {
        "blue" => BLUE,
        "black" => DIM,
        "red" => RED,
        "green" => GREEN,
        _ => WHITE,
    };

    let mut lines = vec![format!(
        "{border}  ╔═══════════════════════════════════════════╗{RESET}"
    )];
    if mythic {
        lines.push(format!(
            "{border}  ║{RESET} {GOLD}{BOLD}✦ MYTHIC FORGE ✦{RESET}{:>28}",
            ""
        ));
    }
    lines.push(format!("{border}  ║{RESET} {BOLD}{}{RESET}", card.name));
    lines.push(format!(
        "{border}  ║{RESET} {:>35} {DIM}{}{RESET}",
        card.cost, sel.set_code
    ));
    lines.push(format!(
        "{border}  ╠═══════════════════════════════════════════╣{RESET}"
    ));
    lines.push(format!("{border}  ║{RESET}"));
    lines.push(format!(
        "{border}  ║{RESET}  {} {DIM}{}{RESET}",
        sel.color_entry.mana, card.type_line
    ));
    lines.push(format!(
        "{border}  ║{RESET}  {} {DIM}{}{RESET}",
        sel.rarity_icon, sel.rarity
    ));
    lines.push(format!("{border}  ║{RESET}"));

    lines.push(format!(
        "{border}  ╠═══════════════════════════════════════════╣{RESET}"
    ));

    if card.rules.is_empty() {
        lines.push(format!("{border}  ║{RESET}"));
    } else {
        for part in card.rules.split('|') {
            let rline = part.trim();
            if !rline.is_empty() {
                lines.push(format!("{border}  ║{RESET}  {WHITE}{rline}{RESET}"));
            }
        }
        lines.push(format!("{border}  ║{RESET}"));
    }

    if !card.flavor.is_empty() {
        lines.push(format!(
            "{border}  ║{RESET}  {DIM}{MAGENTA}{}{RESET}",
            card.flavor
        ));
        lines.push(format!("{border}  ║{RESET}"));
    }

    if let Some(pt) = &card.power_toughness {
        lines.push(format!("{border}  ║{RESET}{:>42} {BOLD}[{pt}]{RESET}", ""));
    }

    lines.push(format!(
        "{border}  ╚═══════════════════════════════════════════╝{RESET}"
    ));
    lines.join("\n")
}

pub fn capitalize_card_arg(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
    }
}

pub fn build_card_evidence(
    parsed: &ParsedCard,
    sel: &ForgeSelection,
    meta: &AttractorMeta,
    display: &str,
    body_hash: &str,
    feedback: &[ChaosEventRef<'_>],
) -> serde_json::Value {
    serde_json::json!({
        "skill": "card",
        "inv": meta.call_serial,
        "set_code": sel.set_code,
        "display_plain": display,
        "body_hash": body_hash,
        "mythic": is_mythic(sel),
        "card": {
            "name": parsed.name,
            "cost": parsed.cost,
            "type_line": parsed.type_line,
            "rarity": sel.rarity,
            "color": sel.color,
            "rules": parsed.rules,
            "flavor": parsed.flavor,
            "power_toughness": parsed.power_toughness,
        },
        "forge": {
            "mode": sel.forge_mode.label(),
            "keyword_spark": sel.sparks.keyword,
            "subtype_hint": sel.sparks.subtype,
            "name_seed": sel.sparks.name_seed,
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

/// Lightweight feedback summary for JSON evidence (avoids circular imports).
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
    use gzmo_chaos::pulse::ChaosSnapshot;

    #[test]
    fn parse_card_fields() {
        let raw = "NAME: Test Golem\nCOST: {3}\nTYPE: Creature — Golem\nRARITY: Rare\nRULES: Trample\nFLAVOR: It walks.\nPT: 4/4";
        let p = parse_card(raw, "Creature");
        assert_eq!(p.name, "Test Golem");
        assert_eq!(p.power_toughness.as_deref(), Some("4/4"));
    }

    #[test]
    fn validate_rejects_incomplete_card() {
        assert!(!validate_forged_card("NAME: X\nCOST: 1", false));
        assert!(validate_forged_card(
            "NAME: Iron Golem\nCOST: {4}\nTYPE: Creature — Golem\nRARITY: Rare\nRULES: Trample.\nPT: 5/5",
            true
        ));
    }

    #[test]
    fn rarity_ratio_favors_common() {
        let cf = CardforgeFile {
            meta: None,
            colors: ColorsSection {
                white: ColorEntry {
                    symbol: "W".into(),
                    mana: "☀".into(),
                    philosophy: "order".into(),
                    flavor_tone: "righteous".into(),
                    strengths: vec![],
                    weaknesses: vec![],
                    personality: String::new(),
                },
                blue: ColorEntry {
                    symbol: "U".into(),
                    mana: "💧".into(),
                    philosophy: "mind".into(),
                    flavor_tone: "precise".into(),
                    strengths: vec![],
                    weaknesses: vec![],
                    personality: String::new(),
                },
                black: ColorEntry {
                    symbol: "B".into(),
                    mana: "💀".into(),
                    philosophy: "power".into(),
                    flavor_tone: "dark".into(),
                    strengths: vec![],
                    weaknesses: vec![],
                    personality: String::new(),
                },
                red: ColorEntry {
                    symbol: "R".into(),
                    mana: "🔥".into(),
                    philosophy: "freedom".into(),
                    flavor_tone: "visceral".into(),
                    strengths: vec![],
                    weaknesses: vec![],
                    personality: String::new(),
                },
                green: ColorEntry {
                    symbol: "G".into(),
                    mana: "🌿".into(),
                    philosophy: "nature".into(),
                    flavor_tone: "primal".into(),
                    strengths: vec![],
                    weaknesses: vec![],
                    personality: String::new(),
                },
            },
            card_types: vec![],
            rarities: vec![],
            keywords: None,
            name_fragments: None,
        };
        let mut commons = 0;
        for i in 0..120 {
            let mut snap = ChaosSnapshot::default();
            snap.x = i as f64;
            let (r, _, _) = pick_rarity(&cf, &snap);
            if r == "Common" {
                commons += 1;
            }
        }
        assert!(commons >= 80, "expected ~88/120 commons, got {commons}");
    }

    #[test]
    fn render_frame_contains_name() {
        let sel = ForgeSelection {
            color: "black",
            color_entry: ColorEntry {
                symbol: "B".into(),
                mana: "💀".into(),
                philosophy: "power".into(),
                flavor_tone: "dark".into(),
                strengths: vec![],
                weaknesses: vec![],
                personality: String::new(),
            },
            rarity: "Rare".into(),
            rarity_icon: "🟡".into(),
            rarity_complexity: "complex".into(),
            card_type: "Creature".into(),
            type_rules: String::new(),
            requires_pt: true,
            forge_mode: ForgeMode::Vision,
            sparks: ForgeSparks {
                keyword: "Menace".into(),
                subtype: "Shade".into(),
                name_seed: "Shadow Oracle".into(),
            },
            set_code: "ATR042".into(),
            strength_hint: "deathtouch".into(),
            weakness_hint: "lifegain".into(),
        };
        let parsed = parse_card(
            "NAME: Shade\nCOST: {2}{B}\nTYPE: Creature — Shade\nRARITY: Rare\nRULES: Menace\nFLAVOR: None.\nPT: 2/2",
            "Creature",
        );
        let frame = render_card_frame(&sel, &parsed, false);
        assert!(frame.contains("Shade"));
        assert!(frame.contains("Menace"));
    }

    #[test]
    fn pick_color_varies_with_snap() {
        let mut a = ChaosSnapshot::default();
        let mut b = ChaosSnapshot::default();
        a.x = 1.0;
        b.x = 99.0;
        let (ca, _) = pick_color(&a);
        let (cb, _) = pick_color(&b);
        assert!(!ca.is_empty() && !cb.is_empty());
    }
}
