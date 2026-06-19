//! # Skill Engine
//!
//! Rust-native skill system that replaces shell script dispatch.
//! Each skill implements the `Skill` trait, receives the current
//! `ChaosSnapshot`, and can emit `ChaosEvent` feedback.
//!
//! Skills are registered in a `SkillRegistry` and dispatched
//! by the REPL's slash command handler.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use gzmo_chaos::feedback::ChaosEvent;
use gzmo_chaos::pulse::ChaosSnapshot;
use tokio::sync::mpsc;

use crate::gateway::LlmGateway;

pub mod dice;
pub mod dice_corpus;
pub mod dice_cascade;
pub mod sound;
pub mod poker;
pub mod quote;
pub mod calculate;
pub mod help;
pub mod visual;
pub mod shell_bridge;
pub mod generative;
pub mod persona;
pub mod joke;
pub mod poem;
pub mod story;
pub mod story_brief;
pub mod poem_brief;
pub mod joke_brief;
pub mod attractor_common;
pub mod skill_ccl;
pub mod word;
pub mod word_brief;
pub mod define;
pub mod define_brief;
pub mod card;
pub mod card_forge;
pub mod card_forge_brief;
pub mod card_corpus;
pub mod pkm;
pub mod pkm_forge;
pub mod pkm_forge_brief;
pub mod pkm_corpus;
pub mod transform;
pub mod language;
pub mod stabilize;
pub mod ops;
pub mod learn;
pub mod discover;
pub mod discovery_ops;
pub mod implement;
pub mod fixer;
pub mod registry;
pub mod dispatch;

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
    /// Optional structured evidence (JSON) for headless runners / Pi probes
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

/// Nested skill dispatch (wild magic cascade from `/dice`).
#[derive(Clone, Copy, Default)]
pub struct NestedDispatch<'a> {
    pub registry: Option<&'a SkillRegistry>,
    pub profile: Option<&'a crate::config::EngineProfileConfig>,
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
    /// LLM gateway for generative skills (None if unavailable)
    pub gateway: Option<&'a dyn LlmGateway>,
    /// Router for pedagogy-internal gateway selection (`/learn` prep).
    pub router: Option<&'a crate::gateway::GatewayRouter>,
    /// Full GZMO config (paths, dice cascade, pedagogy).
    pub config: &'a crate::config::GzmoConfig,
    /// Path to `skills/` directory (persona, language state files)
    pub skills_dir: &'a Path,
    /// GZMO data directory (`data/`)
    pub data_dir: &'a Path,
    /// Nested dispatch for `/dice` wild magic cascade.
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
