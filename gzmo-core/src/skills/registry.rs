//! Build the full chaos skill registry (Rust-native pantheon).

use std::sync::Arc;

use crate::config::PedagogyConfig;

use super::skill_ccl::{ccl_for_skill, ChaosCouplingLevel};
use super::{
    calculate::CalculateSkill,
    card::CardSkill,
    define::DefineSkill,
    dice::DiceSkill,
    discover::DiscoverSkill,
    fixer::FixerSkill,
    help::HelpSkill,
    implement::ImplementSkill,
    joke::JokeSkill,
    language::LanguageSkill,
    learn::LearnSkill,
    ops::OpsSkill,
    pkm::PkmSkill,
    poem::PoemSkill,
    poker::PokerSkill,
    quote::QuoteSkill,
    sound::SoundSkill,
    stabilize::StabilizeSkill,
    story::StorySkill,
    transform::TransformSkill,
    visual::VisualSkill,
    word::WordSkill,
    SkillRegistry, SkillType,
};

/// Build `/help` rows from registered skills (excludes `help` itself).
pub fn help_entries_for_registry(registry: &SkillRegistry) -> Vec<(String, String, &'static str, ChaosCouplingLevel)> {
    registry
        .all()
        .iter()
        .filter(|s| s.name() != "help")
        .map(|s| {
            let type_label = match s.skill_type() {
                SkillType::Mechanical => "mechanical",
                SkillType::Generative => "generative",
                SkillType::Mutation => "mutation",
                SkillType::Info => "info",
            };
            (
                s.name().to_string(),
                s.description().to_string(),
                type_label,
                ccl_for_skill(s.name()),
            )
        })
        .collect()
}

pub fn build_chaos_skill_registry(pedagogy: &PedagogyConfig) -> SkillRegistry {
    let mut registry = SkillRegistry::new();
    registry.register(Arc::new(DiceSkill));
    registry.register(Arc::new(SoundSkill));
    registry.register(Arc::new(PokerSkill));
    registry.register(Arc::new(QuoteSkill));
    registry.register(Arc::new(CalculateSkill));
    registry.register(Arc::new(VisualSkill));
    registry.register(Arc::new(JokeSkill));
    registry.register(Arc::new(PoemSkill));
    registry.register(Arc::new(StorySkill));
    registry.register(Arc::new(WordSkill));
    registry.register(Arc::new(DefineSkill));
    registry.register(Arc::new(CardSkill));
    registry.register(Arc::new(PkmSkill));
    registry.register(Arc::new(TransformSkill));
    registry.register(Arc::new(LanguageSkill));
    registry.register(Arc::new(StabilizeSkill));
    registry.register(Arc::new(OpsSkill {
        pedagogy_config: pedagogy.clone(),
    }));
    registry.register(Arc::new(LearnSkill {
        pedagogy_config: pedagogy.clone(),
    }));
    registry.register(Arc::new(DiscoverSkill {
        pedagogy_config: pedagogy.clone(),
    }));
    registry.register(Arc::new(ImplementSkill {
        pedagogy_config: pedagogy.clone(),
    }));
    registry.register(Arc::new(FixerSkill {
        pedagogy_config: pedagogy.clone(),
    }));

    let help_entries = help_entries_for_registry(&registry);
    registry.register(Arc::new(HelpSkill { entries: help_entries }));
    registry
}
