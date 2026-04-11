//! Core types shared across all openclaw crates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

/// The immutable SOUL context loaded from SOUL.md.
/// Exempt from temporal decay. Acts as the agent's DNA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulContext {
    pub persona_name: String,
    pub core_directives: Vec<String>,
    pub ethical_guardrails: Vec<String>,
    pub raw_markdown: String,
    pub loaded_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

/// A single semantic fact persisted in the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    pub id: Uuid,
    pub content: String,
    pub embedding: Vec<f32>,
    pub half_life_days: f64,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    pub confirmation_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
}

fn default_confidence() -> f64 { 1.0 }

/// Classification of memory decay rates (Atkinson-Shiffrin model).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DecayClass {
    /// Raw tool outputs, chat transcripts — 30 day half-life
    Episodic,
    /// Partially structured research — 60 day half-life
    CuratedVault,
    /// Job titles, current project — 139 day half-life
    FlexibleIdentity,
    /// Birthdate, legal name — 693 day half-life
    AbsoluteIdentity,
    /// SOUL.md — infinite, never decays
    Structural,
}

impl DecayClass {
    pub fn half_life_days(&self) -> f64 {
        match self {
            Self::Episodic => 30.0,
            Self::CuratedVault => 60.0,
            Self::FlexibleIdentity => 139.0,
            Self::AbsoluteIdentity => 693.0,
            Self::Structural => f64::INFINITY,
        }
    }
}

/// A single entry in the episodic daily log (memory/YYYY-MM-DD.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEntry {
    pub timestamp: DateTime<Utc>,
    pub source: EpisodicSource,
    pub content: String,
    pub is_silent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EpisodicSource {
    UserChat,
    HeartbeatCheck,
    ToolExecution { tool_name: String },
    InternalMonologue,
}

// ---------------------------------------------------------------------------
// Dreaming
// ---------------------------------------------------------------------------

/// A "Truth" extracted during the Deep Phase of the autoDream cycle.
/// This is the atomic unit of permanent semantic knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTruth {
    pub id: Uuid,
    pub content: String,
    pub confidence: f32,
    pub mmr_score: f32,
    pub source_date: chrono::NaiveDate,
    pub decay_class: DecayClass,
}

// ---------------------------------------------------------------------------
// Skills
// ---------------------------------------------------------------------------

/// Compressed metadata for a single skill (< 100 chars target).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub file_path: String,
    pub isolation: IsolationLevel,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum IsolationLevel {
    /// Direct host execution (fastest, least secure)
    #[default]
    Host,
    /// Read-only workspace mount
    ReadOnly,
    /// Read-write workspace mount
    ReadWrite,
    /// Ephemeral Docker container per invocation
    SessionContainer,
    /// Persistent Docker container per agent
    AgentContainer,
}

// ---------------------------------------------------------------------------
// LLM Messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    /// If true, this is a meta-injection (skill payload) hidden from the user.
    pub is_meta: bool,
    /// For Assistant messages: structured tool calls the model requested.
    /// Serialized as the OpenAI-compatible tool_calls array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<MessageToolCall>>,
    /// For Tool messages: the ID of the tool call this result responds to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// A tool call record stored in message history.
/// Matches the OpenAI chat completions tool_calls format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageToolCall {
    pub id: String,
    pub r#type: String,
    pub function: MessageToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
