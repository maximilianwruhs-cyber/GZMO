//! # Skill Engine
//!
//! Rust-native skill system that replaces shell script dispatch.
//! Each skill implements the `Skill` trait, receives the current
//! `ChaosSnapshot`, and can emit `ChaosEvent` feedback.
//!
//! Skills are registered in a `SkillRegistry` and dispatched
//! by the REPL's slash command handler.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::pulse::ChaosSnapshot;
use tokio::sync::mpsc;

use crate::config::{EngineProfileConfig, GzmoConfig};
use crate::gateway::{GatewayRouter, LlmGateway};

pub mod attractor_common;
pub mod calculate;
pub mod card;
pub mod card_corpus;
pub mod card_forge;
pub mod card_forge_brief;
pub mod define;
pub mod dice;
pub mod dice_cascade;
pub mod dice_corpus;
pub mod dispatch;
pub mod generative;
pub mod help;
pub mod joke;
pub mod language;
pub mod llm;
pub mod persona;
pub mod poem;
pub mod poker;
pub mod quote;
pub mod shell_bridge;
pub mod sound;
pub mod stabilize;
pub mod status;
pub mod story;
pub mod story_brief;
pub mod transform;
pub mod visual;
pub mod word;

pub use llm::SkillRuntime;

use calculate::CalculateSkill;
use card::CardSkill;
use define::DefineSkill;
use dice::DiceSkill;
use help::HelpSkill;
use joke::JokeSkill;
use language::LanguageSkill;
use poem::PoemSkill;
use poker::PokerSkill;
use quote::QuoteSkill;
use sound::SoundSkill;
use stabilize::StabilizeSkill;
use status::StatusSkill;
use story::StorySkill;
use transform::TransformSkill;
use visual::VisualSkill;
use word::WordSkill;

/// The type of skill — affects display and feedback behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillType {
    /// Generates content via LLM (poem, joke, story, card)
    Generative,
    /// Pure mechanic — no LLM needed (dice, sound)
    Mechanical,
    /// State mutation (transform, language)
    Mutation,
    /// Information display (help, stats)
    Info,
}

/// Output from a skill execution.
pub struct SkillOutput {
    /// Terminal output (may contain ANSI escape codes)
    pub display: String,
    /// Feedback events to inject into the chaos engine
    pub feedback: Vec<ChaosEvent>,
    /// If true, also inject the display text as a system message
    /// into the conversation (so the LLM "sees" what happened)
    pub inject_to_conversation: bool,
    /// Optional structured metadata for callers that persist skill results.
    pub evidence: Option<serde_json::Value>,
}

impl SkillOutput {
    pub fn new(
        display: impl Into<String>,
        feedback: Vec<ChaosEvent>,
        inject_to_conversation: bool,
    ) -> Self {
        Self {
            display: display.into(),
            feedback,
            inject_to_conversation,
            evidence: None,
        }
    }
}

/// Capability passed to a skill that may invoke another skill.
#[derive(Default)]
pub struct NestedDispatch<'a> {
    pub registry: Option<&'a SkillRegistry>,
    pub profile: Option<&'a EngineProfileConfig>,
    pub depth: u8,
}

/// Context provided to every skill execution.
pub struct SkillContext<'a> {
    /// Current chaos engine state
    pub chaos: &'a ChaosSnapshot,
    /// Feedback channel to the chaos engine
    pub feedback_tx: &'a mpsc::Sender<ChaosEvent>,
    /// Arguments provided after the slash command
    pub args: &'a str,
    /// LLM gateway for generative skills.
    pub gateway: Option<&'a dyn LlmGateway>,
    /// Router for skill dispatch callers that have one.
    pub router: Option<&'a GatewayRouter>,
    /// Loaded operator configuration.
    pub config: &'a GzmoConfig,
    /// Directory containing shell skill definitions.
    pub skills_dir: &'a std::path::Path,
    /// Runtime data directory.
    pub data_dir: &'a std::path::Path,
    /// Optional nested-dispatch capability.
    pub nested: NestedDispatch<'a>,
}

/// Core trait for all GZMO skills.
#[async_trait]
pub trait Skill: Send + Sync {
    /// The slash command name (without the leading `/`)
    fn name(&self) -> &str;

    /// Short description for `/help`
    fn description(&self) -> &str;

    /// Skill type — affects feedback and display behavior
    fn skill_type(&self) -> SkillType;

    /// Execute the skill with the given context.
    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput>;
}

/// Registry of all available skills, indexed by command name.
pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Register a skill. The name must be unique.
    pub fn register(&mut self, skill: Arc<dyn Skill>) {
        let name = skill.name().to_string();
        self.skills.insert(name, skill);
    }

    /// Look up a skill by command name.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Skill>> {
        self.skills.get(name)
    }

    /// Check if a command name is registered as a Rust skill.
    pub fn has(&self, name: &str) -> bool {
        self.skills.contains_key(name)
    }

    /// Get all registered skill names (for /help).
    pub fn names(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered skills (for /help descriptions).
    pub fn all(&self) -> Vec<&Arc<dyn Skill>> {
        self.skills.values().collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build shared LLM runtime from config.
pub fn build_runtime(config: &GzmoConfig) -> Arc<SkillRuntime> {
    Arc::new(SkillRuntime::from_config(config))
}

/// Register the full pantheon of Rust-native skills (including `/help` last).
pub fn register_pantheon(registry: &mut SkillRegistry, config: &GzmoConfig) {
    let rt = build_runtime(config);

    registry.register(Arc::new(DiceSkill));
    registry.register(Arc::new(SoundSkill));
    registry.register(Arc::new(PokerSkill));
    registry.register(Arc::new(QuoteSkill));
    registry.register(Arc::new(CalculateSkill));
    registry.register(Arc::new(VisualSkill));
    registry.register(Arc::new(StatusSkill {
        config: config.clone(),
    }));
    registry.register(Arc::new(StabilizeSkill {
        rt: Arc::clone(&rt),
    }));

    registry.register(Arc::new(JokeSkill {
        rt: Arc::clone(&rt),
    }));
    registry.register(Arc::new(PoemSkill {
        rt: Arc::clone(&rt),
    }));
    registry.register(Arc::new(WordSkill {
        rt: Arc::clone(&rt),
    }));
    registry.register(Arc::new(StorySkill));
    registry.register(Arc::new(DefineSkill {
        rt: Arc::clone(&rt),
    }));
    registry.register(Arc::new(CardSkill));
    registry.register(Arc::new(TransformSkill {
        rt: Arc::clone(&rt),
    }));
    registry.register(Arc::new(LanguageSkill {
        rt: Arc::clone(&rt),
    }));

    let help_entries: Vec<(String, String, &'static str)> = registry
        .all()
        .iter()
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
            )
        })
        .collect();
    registry.register(Arc::new(HelpSkill {
        entries: help_entries,
    }));
}

#[cfg(test)]
mod skill_smoke {
    use super::*;
    use futures_util::FutureExt;
    use gzmo_chaos::pulse::ChaosSnapshot;
    use std::panic::AssertUnwindSafe;
    use std::path::PathBuf;

    fn test_config() -> GzmoConfig {
        let mut config = GzmoConfig::default();
        // Anchor to repo skills/ regardless of cargo package cwd.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("skills");
        config.skills.directory = root.canonicalize().unwrap_or(root);
        config
    }

    async fn exec_catch(
        reg: &SkillRegistry,
        snap: &ChaosSnapshot,
        tx: &mpsc::Sender<ChaosEvent>,
        name: &str,
        args: &str,
    ) {
        let config = test_config();
        let ctx = dispatch::skill_context(
            snap,
            tx,
            args,
            None,
            None,
            &config,
            NestedDispatch::default(),
        );
        let fut = reg.get(name).expect(name).execute(ctx);
        match AssertUnwindSafe(fut).catch_unwind().await {
            Ok(Ok(_)) | Ok(Err(_)) => {}
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("non-string panic");
                panic!("skill /{name} PANICKED: {msg}");
            }
        }
    }

    #[tokio::test]
    async fn mechanical_skills_do_not_panic() {
        let config = test_config();
        let mut reg = SkillRegistry::new();
        register_pantheon(&mut reg, &config);
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move { while rx.recv().await.is_some() {} });

        let mut snap = ChaosSnapshot::default();
        snap.tick = 42;
        snap.x = -12.3;
        snap.y = 18.7;
        snap.z = 27.1;
        snap.chaos_val = 0.73;
        snap.tension = 55.0;
        snap.energy = 80.0;

        for (name, args) in [
            ("dice", ""),
            ("dice", "d6"),
            ("poker", ""),
            ("quote", ""),
            ("sound", ""),
            ("calculate", "2+2"),
            ("help", ""),
            ("stabilize", ""),
            ("language", ""),
            ("language", "de"),
            ("transform", ""),
            ("status", ""),
            ("visual", "lorenz"),
        ] {
            exec_catch(&reg, &snap, &tx, name, args).await;
        }
    }

    #[tokio::test]
    async fn generative_skills_do_not_panic() {
        let config = test_config();
        let mut reg = SkillRegistry::new();
        register_pantheon(&mut reg, &config);
        let (tx, mut rx) = mpsc::channel(64);
        tokio::spawn(async move { while rx.recv().await.is_some() {} });
        let mut snap = ChaosSnapshot::default();
        snap.chaos_val = 0.42;
        snap.x = 1.0;
        snap.y = 2.0;
        snap.z = 30.0;
        for (name, args) in [("story", ""), ("card", "")] {
            exec_catch(&reg, &snap, &tx, name, args).await;
        }
    }
}
