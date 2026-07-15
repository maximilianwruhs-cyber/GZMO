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

pub mod dice;
pub mod sound;
pub mod poker;
pub mod quote;
pub mod calculate;
pub mod help;
pub mod status;
pub mod visual;

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
}

/// Context provided to every skill execution.
pub struct SkillContext<'a> {
    /// Current chaos engine state
    pub chaos: &'a ChaosSnapshot,
    /// Feedback channel to the chaos engine
    pub feedback_tx: &'a mpsc::Sender<ChaosEvent>,
    /// Arguments provided after the slash command
    pub args: &'a str,
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
