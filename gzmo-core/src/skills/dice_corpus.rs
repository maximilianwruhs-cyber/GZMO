//! `/dice` narrative event corpus — loaded from `data/dice_events.toml`.
//!
//! The TOML file is the editable source of truth for the 118 event strings.
//! Embedded at compile time so headless `gzmo chaos skill` works without cwd assumptions.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

const EMBEDDED_TOML: &str = include_str!("../../../data/dice_events.toml");

#[derive(Debug, Clone, Deserialize)]
struct Meta {
    version: u32,
    d20_tiers: u32,
    d6_tiers: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct TierEntry {
    #[serde(default)]
    tier: String,
    variants: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DiceCorpusFile {
    meta: Meta,
    #[serde(default)]
    d20: HashMap<String, TierEntry>,
    #[serde(default)]
    d6: HashMap<String, TierEntry>,
}

#[derive(Debug, Clone)]
pub struct DiceCorpus {
    pub meta: Meta,
    pub d20: HashMap<String, TierEntry>,
    pub d6: HashMap<String, TierEntry>,
}

impl DiceCorpus {
    fn parse(toml_src: &str) -> anyhow::Result<Self> {
        let raw: DiceCorpusFile = toml::from_str(toml_src)?;
        Ok(Self {
            meta: raw.meta,
            d20: raw.d20,
            d6: raw.d6,
        })
    }

    pub fn total_event_strings(&self) -> usize {
        self.d20.values().map(|t| t.variants.len()).sum::<usize>()
            + self.d6.values().map(|t| t.variants.len()).sum::<usize>()
    }

    pub fn tier_name(&self, max: u8, roll: u8) -> Option<&str> {
        let map = if max == 6 { &self.d6 } else { &self.d20 };
        map.get(&roll.to_string()).map(|t| t.tier.as_str()).filter(|s| !s.is_empty())
    }

    pub fn event(&self, max: u8, roll: u8, variant: usize) -> String {
        let map = if max == 6 { &self.d6 } else { &self.d20 };
        match map.get(&roll.to_string()) {
            Some(entry) if !entry.variants.is_empty() => {
                let idx = variant.min(entry.variants.len() - 1);
                entry.variants[idx].clone()
            }
            _ => if max == 6 {
                "🎲 A roll.".to_string()
            } else {
                "🎲 A roll beyond comprehension.".to_string()
            },
        }
    }
}

static CORPUS: OnceLock<DiceCorpus> = OnceLock::new();

pub fn corpus() -> &'static DiceCorpus {
    CORPUS.get_or_init(|| {
        DiceCorpus::parse(EMBEDDED_TOML).unwrap_or_else(|e| {
            panic!("invalid embedded data/dice_events.toml: {e}")
        })
    })
}

/// Resolve narrative line for a roll + variant index.
pub fn dice_event(max: u8, roll: u8, variant: usize) -> String {
    corpus().event(max, roll, variant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_corpus_has_118_events() {
        let c = corpus();
        assert_eq!(c.meta.d20_tiers, 20);
        assert_eq!(c.meta.d6_tiers, 6);
        assert_eq!(c.total_event_strings(), 118);
    }

    #[test]
    fn d20_roll_20_variant_0_is_legendary() {
        let text = dice_event(20, 20, 0);
        assert!(text.contains("CRITICAL SUCCESS") || text.contains("LEGENDARY"));
    }

    #[test]
    fn tier_names_present_for_d20() {
        let c = corpus();
        assert_eq!(c.tier_name(20, 1).unwrap(), "CATASTROPHIC");
        assert_eq!(c.tier_name(20, 20).unwrap(), "LEGENDARY");
    }
}
