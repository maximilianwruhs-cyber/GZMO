use anyhow::Result;
use serde::Deserialize;

// ─── Task Kind (Obolus routing classification) ──────────────────────────

/// Categories used by the Obolus routing table to dispatch LLM calls.
/// Each variant maps to a named engine profile in `[routing]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// Interactive chat / tool-use reasoning (always Prime for quality).
    Chat,
    /// Daemon main loop — fallback when no specific mapping exists.
    Daemon,
    /// Dream consolidation: KG extraction from episodic log.
    DreamExtract,
    /// Dream consolidation: fact-checking / verification pass.
    DreamVerify,
    /// Spark cycle: hypothesis generation (light model OK).
    SparkHypothesis,
    /// Spark cycle: link verification (heavier model for accuracy).
    SparkVerify,
    /// Ingest: KG extraction from document chunk.
    IngestExtract,
    /// Ingest: fact-checking / verification pass (decoupled from dream_verify).
    IngestVerify,
    /// Distill: KG extraction from session transcript.
    DistillExtract,
    /// Distill: fact-checking / verification pass.
    DistillVerify,
    /// Distill: short narrative summary for episodic.
    DistillSummary,
    /// Agentic Teacher internal agents (Diagnoser, Planner, Affective, learn prep).
    PedagogyInternal,
}

impl std::fmt::Display for TaskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chat => write!(f, "chat"),
            Self::Daemon => write!(f, "daemon"),
            Self::DreamExtract => write!(f, "dream_extract"),
            Self::DreamVerify => write!(f, "dream_verify"),
            Self::SparkHypothesis => write!(f, "spark_hypothesis"),
            Self::SparkVerify => write!(f, "spark_verify"),
            Self::IngestExtract => write!(f, "ingest_extract"),
            Self::IngestVerify => write!(f, "ingest_verify"),
            Self::DistillExtract => write!(f, "distill_extract"),
            Self::DistillVerify => write!(f, "distill_verify"),
            Self::DistillSummary => write!(f, "distill_summary"),
            Self::PedagogyInternal => write!(f, "pedagogy_internal"),
        }
    }
}

impl TaskKind {
    /// All task kinds — used for validation.
    pub fn all() -> &'static [Self] {
        &[
            Self::Chat,
            Self::Daemon,
            Self::DreamExtract,
            Self::DreamVerify,
            Self::SparkHypothesis,
            Self::SparkVerify,
            Self::IngestExtract,
            Self::IngestVerify,
            Self::DistillExtract,
            Self::DistillVerify,
            Self::DistillSummary,
            Self::PedagogyInternal,
        ]
    }

    /// The default engine for this task kind when no mapping exists.
    pub fn default_engine(&self) -> &'static str {
        match self {
            Self::Chat => "local",
            Self::Daemon => "local",
            Self::DreamExtract => "local",
            Self::DreamVerify => "local",
            Self::SparkHypothesis => "local",
            Self::SparkVerify => "local",
            Self::IngestExtract => "local",
            Self::IngestVerify => "local",
            Self::DistillExtract => "local",
            Self::DistillVerify => "local",
            Self::DistillSummary => "local",
            // Prefer local/prime for internal pedagogy agents; cloud via routing override.
            Self::PedagogyInternal => "local",
        }
    }

    /// Whether this task runs in the autonomous GZMO loop (daemon/CLI cognition)
    /// rather than interactive chat. Background tasks are eligible for
    /// cloud-first routing (`[routing] cloud_first_background`); `Chat` is not,
    /// so chat-spawned subagents stay on the active engine.
    pub fn is_background(&self) -> bool {
        !matches!(self, Self::Chat)
    }
}

// ─── Engine Mode ────────────────────────────────────────────────────────

/// The active engine mode — local or cloud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineMode {
    Local,
    Cloud,
    /// Sovereign 3×7B FrankenMoE on :8010 (when GGUF is built).
    Sovereign,
}

impl Default for EngineMode {
    fn default() -> Self {
        Self::Local
    }
}

impl std::fmt::Display for EngineMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Cloud => write!(f, "cloud"),
            Self::Sovereign => write!(f, "sovereign"),
        }
    }
}

impl std::str::FromStr for EngineMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "cloud" => Ok(Self::Cloud),
            "sovereign" => Ok(Self::Sovereign),
            other => anyhow::bail!(
                "Unknown engine mode: '{}'. Use 'local', 'cloud', or 'sovereign'.",
                other
            ),
        }
    }
}
