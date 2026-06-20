//! # Sovereign Configuration
//!
//! Loads `gzmo.toml` — the single source of truth for all agent settings.
//!
//! ## Example gzmo.toml
//!
//! ```toml
//! [identity]
//! soul_path = "SOUL.md"
//! persona_name = "GZMO"
//!
//! [memory]
//! directory = "memory"
//! vault_db = "data/vault.db"
//!
//! [skills]
//! directory = "skills"
//! dreams_path = "DREAMS.md"
//!
//! [api_keys]
//! serpapi = "..."
//! openrouter = "..."
//! gemini = "..."
//!
//! [engine]
//! active_mode = "local"
//!
//! [engine.local]
//! provider = "local"
//! url = "http://localhost:8000/v1"
//! model = "qwen2.5-7b-instruct.Q3_K_M.gguf"
//! temperature = 0.3
//! top_p = 0.95
//! max_tokens = 8192
//!
//! [engine.cloud]
//! provider = "openrouter"
//! url = "https://openrouter.ai/api/v1"
//! model = "openrouter/free"
//! api_key = "sk-or-..."
//! temperature = 0.4
//! top_p = 0.95
//! max_tokens = 8192
//! fallback_provider = "gemini"
//! fallback_url = "https://generativelanguage.googleapis.com/v1beta/openai"
//! fallback_model = "gemini-2.5-flash"
//! fallback_api_key = "AIza..."
//!
//! [agent]
//! max_tool_iterations = 10
//! heartbeat_interval_secs = 1800
//!
//! [[mcp_servers]]
//! name = "filesystem"
//! command = "npx"
//! args = ["-y", "@modelcontextprotocol/server-filesystem", "/home/user"]
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
            Self::PedagogyInternal => "local",
        }
    }

    /// Whether this task runs in the autonomous GZMO loop (daemon/CLI cognition)
    /// rather than interactive chat. Background tasks are eligible for
    /// cloud-first routing (`[routing] cloud_first_background`); `Chat` is not,
    /// so chat-spawned subagents stay on the active engine.
    pub fn is_background(&self) -> bool {
        matches!(
            self,
            Self::Daemon
                | Self::DreamExtract
                | Self::DreamVerify
                | Self::SparkHypothesis
                | Self::SparkVerify
                | Self::IngestExtract
                | Self::IngestVerify
                | Self::DistillExtract
                | Self::DistillVerify
                | Self::DistillSummary
        )
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
    fn default() -> Self { Self::Local }
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

// ─── Top-level Config ───────────────────────────────────────────────────

/// Top-level configuration loaded from gzmo.toml
#[derive(Debug, Deserialize, Clone, Default)]
pub struct GzmoConfig {
    #[serde(default)]
    pub identity: IdentityConfig,

    #[serde(default)]
    pub memory: MemoryConfig,

    #[serde(default)]
    pub skills: SkillsConfig,

    #[serde(default)]
    pub engine: EngineSection,

    #[serde(default)]
    pub agent: AgentConfig,

    /// Centralized API key management
    #[serde(default)]
    pub api_keys: ApiKeysConfig,

    /// MCP server declarations — each is spawned as a child process on startup
    #[serde(default)]
    pub mcp_servers: Vec<McpServerEntry>,

    /// Background orchestration: scheduled cron jobs
    #[serde(default)]
    pub orchestration: OrchestrationConfig,

    /// autoDream consolidation settings (verification, confidence gating)
    #[serde(default)]
    pub dreams: DreamsConfig,

    /// Serendipitous recall (SparkEngine) — L3 hypotheses only, never auto-facts at 1.0
    #[serde(default)]
    pub spark: SparkConfig,

    /// Gated knowledge-folder ingest (IngestEngine) — replaces headless watcher prompts
    #[serde(default)]
    pub ingest: IngestConfig,

    /// Chat session → SessionDistill vault + rich episodic for dream.
    #[serde(default)]
    pub session_distill: SessionDistillConfig,

    /// Local embedding server for vault vectors (`/v1/embeddings`).
    #[serde(default)]
    pub embeddings: EmbeddingsConfig,

    /// Chaos engine configuration (Lorenz attractor, Thought Cabinet physics)
    #[serde(default)]
    pub chaos: Option<toml::Value>,

    /// Startup probe settings (`gzmo health`, daemon boot).
    #[serde(default)]
    pub health: HealthConfig,

    /// LXC101 Qdrant mirror for vault vectors (`scripts/sync-vault-to-qdrant.py`).
    #[serde(default)]
    pub qdrant: QdrantConfig,

    /// Retired VM200 librarian LLM (`:8083`). Kept for back-compat; session
    /// distill now runs on Prime via `[routing.mappings]`. Leave disabled.
    #[serde(default)]
    pub librarian: LibrarianConfig,

    /// VM200 reranker preset on the unified retrieval router (`:8081`);
    /// post-filters vault recall.
    #[serde(default)]
    pub rerank: RerankConfig,

    /// Obolus: static task → engine routing table.
    #[serde(default)]
    pub routing: RoutingConfig,

    /// Redis scratch cache + distill job queue (LXC101 :6379).
    #[serde(default)]
    pub redis: RedisConfig,

    /// Hot context archive threshold + scratch token budget.
    #[serde(default)]
    pub context_memory: ContextMemoryConfig,

    /// Hot context compression settings (Headroom).
    #[serde(default)]
    pub context_compress: ContextCompressConfig,

    /// Subagent delegation limits (SubagentRunner Lite).
    #[serde(default)]
    pub subagent: SubagentConfig,

    /// Cross-collection platform search (honeypot + Pi knowledge Qdrant).
    #[serde(default)]
    pub platform_search: PlatformSearchConfig,

    /// Neo4j ontology reconciliation (canonicalize entity/relation types).
    #[serde(default)]
    pub kg_reconcile: KgReconcileConfig,

    /// Read-only Pi Synapse event pull into episodic (append-only bus preserved).
    #[serde(default)]
    pub synapse_pull: SynapsePullConfig,

    /// Git-tracked markdown wiki layer (WikiEngine). Emit-only retrieval.
    #[serde(default)]
    pub wiki: WikiConfig,

    /// Agentic Teacher pedagogy orchestrator, learner profile, EDF trail.
    #[serde(default)]
    pub pedagogy: PedagogyConfig,

    /// `/dice` skill — autopoietic follow-up roll scheduling.
    #[serde(default)]
    pub dice: DiceConfig,

    /// Kurator monitor (phase 1: read-only + spawn.recommended).
    #[serde(default)]
    pub kurator: KuratorConfig,

    /// Bibliothek stable-promotion gate (dream cycle counter).
    #[serde(default)]
    pub bibliothek: BibliothekConfig,

    /// Synapse Writer gate for chaos skill CLI dispatch.
    #[serde(default)]
    pub synapse_writer: SynapseWriterConfig,

    /// Token ledger + efficiency analytics for Prime (:8000). Distinct from `[routing]`.
    #[serde(default)]
    pub obolus_analytics: ObolusAnalyticsConfig,

    /// ARCH-DIR sovereignty compliance (verify script + policy knobs).
    #[serde(default)]
    pub compliance: ComplianceConfig,

    /// Obolus runtime energy governance (ObolusGate decisions).
    #[serde(default)]
    pub obolus_governance: ObolusGovernanceConfig,
}

// ─── Dice autopoietic loop ──────────────────────────────────────────────

/// Schedule follow-up `/dice` rolls from each outcome (interval ∝ roll value).
#[derive(Debug, Deserialize, Clone)]
pub struct DiceConfig {
    #[serde(default)]
    pub r#loop: DiceLoopConfig,
    #[serde(default)]
    pub cascade: DiceCascadeConfig,
}

impl Default for DiceConfig {
    fn default() -> Self {
        Self {
            r#loop: DiceLoopConfig::default(),
            cascade: DiceCascadeConfig::default(),
        }
    }
}

/// Wild magic — after each roll, dispatch a pantheon skill from the tier pool.
#[derive(Debug, Deserialize, Clone)]
pub struct DiceCascadeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Extra skills to exclude beyond `data/dice_cascade.toml` defaults.
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for DiceCascadeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            exclude: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DiceLoopConfig {
    /// When true, each roll schedules the next automatic `/dice` for the daemon.
    #[serde(default)]
    pub enabled: bool,

    /// Minimum delay (minutes) — mapped from roll 1.
    #[serde(default = "default_dice_loop_min")]
    pub min_minutes: u32,

    /// Maximum delay (minutes) — mapped from roll max on the active die.
    #[serde(default = "default_dice_loop_max")]
    pub max_minutes: u32,

    /// Stop scheduling after this many chained auto-rolls (0 = unlimited).
    #[serde(default)]
    pub max_chain_depth: u32,

    /// When true, roll 1 clears any pending follow-up.
    #[serde(default = "default_true")]
    pub cancel_on_nat_1: bool,
}

impl Default for DiceLoopConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_minutes: default_dice_loop_min(),
            max_minutes: default_dice_loop_max(),
            max_chain_depth: 0,
            cancel_on_nat_1: true,
        }
    }
}

fn default_dice_loop_min() -> u32 {
    5
}

fn default_dice_loop_max() -> u32 {
    120
}

// ─── Dreams ─────────────────────────────────────────────────────────────

/// Settings for the autoDream consolidation engine.
///
/// These knobs are the front line against hallucinated knowledge entering
/// permanent memory. When `verify` is on, every extracted entity/relation is
/// fact-checked against the source log before it is written to the knowledge
/// graph; anything scoring below `min_confidence` (or unsupported by a quotable
/// span) is dropped instead of promoted.
#[derive(Debug, Deserialize, Clone)]
pub struct DreamsConfig {
    /// Master switch — when false, daemon skips nightly consolidation (CLI `gzmo dream` still runs).
    #[serde(default = "default_dream_enabled")]
    pub enabled: bool,

    /// If true, run a second LLM pass that fact-checks each extracted claim
    /// against the source before writing it to the knowledge graph / vault.
    #[serde(default = "default_dream_verify")]
    pub verify: bool,

    /// Minimum verified confidence (0.0–1.0) required to promote a claim.
    /// Matches the vault's quarantine threshold so the graph and vault agree.
    #[serde(default = "default_dream_min_confidence")]
    pub min_confidence: f64,

    /// Temperature for the verification pass. Kept low for near-deterministic
    /// fact-checking, independent of the engine's creative default.
    #[serde(default = "default_dream_verify_temperature")]
    pub verify_temperature: f32,

    /// Hour (UTC) when the daemon runs consolidation for **yesterday's** episodic log.
    #[serde(default = "default_dream_cron_hour")]
    pub cron_hour: u32,

    /// Minute (UTC) within `cron_hour` for the nightly dream tick.
    #[serde(default = "default_dream_cron_minute")]
    pub cron_minute: u32,

    /// Supported claims must include a quotable evidence span (≥12 chars).
    #[serde(default = "default_kg_require_evidence")]
    pub require_evidence: bool,

    /// Abort if Neo4j MCP writes fewer nodes/edges than verified (no silent partial promote).
    #[serde(default = "default_kg_strict")]
    pub strict_kg: bool,

    /// Max chars per REM/verify chunk for large daily logs (paragraph-aware split).
    #[serde(default = "default_pipeline_chunk_chars")]
    pub chunk_chars: usize,

    /// Drop episodic `### 🧠 INTERNAL` sections whose body matches any of these (case-insensitive).
    #[serde(default = "default_dream_exclude_episodic_substrings")]
    pub exclude_episodic_substrings: Vec<String>,

    /// Skip REM/KG when filtered episodic text is shorter than this (ops-only days).
    #[serde(default = "default_dream_min_consolidation_chars")]
    pub min_consolidation_chars: usize,

    /// M3: fold honeypot distillates + vector associations into REM (not episodic-only).
    #[serde(default = "default_dream_honeypot_rem_enabled")]
    pub honeypot_rem_enabled: bool,

    /// Anchor facts sampled from honeypot for association expansion.
    #[serde(default = "default_honeypot_rem_anchor_limit")]
    pub honeypot_rem_anchor_limit: usize,

    /// Similar honeypot facts retrieved per anchor (vector decay search).
    #[serde(default = "default_honeypot_rem_associate_k")]
    pub honeypot_rem_associate_k: usize,
}

fn default_dream_honeypot_rem_enabled() -> bool {
    true
}

fn default_honeypot_rem_anchor_limit() -> usize {
    4
}

fn default_honeypot_rem_associate_k() -> usize {
    6
}

fn default_dream_exclude_episodic_substrings() -> Vec<String> {
    vec![
        "sys_janitor".to_string(),
        "[job: sys_janitor]".to_string(),
        "[spark ".to_string(),
        "## spark —".to_string(),
        "[ingest:".to_string(),
        "ingested `".to_string(),
        "filesystem utilization".to_string(),
        "root filesystem".to_string(),
        "[hypothesis ".to_string(),
        // "promoted=false" removed (2026-06-15) — spark summaries often contain entity info worth dreaming on
    ]
}

fn default_dream_min_consolidation_chars() -> usize {
    200  // lowered from 400 (2026-06-15) — ops-only days were skipping REM too aggressively
}

impl DreamsConfig {
    pub fn kg_gate(&self) -> crate::memory::kg_extract::KgGateConfig {
        crate::memory::kg_extract::KgGateConfig {
            verify: self.verify,
            min_confidence: self.min_confidence,
            verify_temperature: self.verify_temperature,
            require_evidence: self.require_evidence,
            strict_kg: self.strict_kg,
        }
    }
}

impl Default for DreamsConfig {
    fn default() -> Self {
        Self {
            enabled: default_dream_enabled(),
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            verify_temperature: default_dream_verify_temperature(),
            cron_hour: default_dream_cron_hour(),
            cron_minute: default_dream_cron_minute(),
            require_evidence: default_kg_require_evidence(),
            strict_kg: default_kg_strict(),
            chunk_chars: default_pipeline_chunk_chars(),
            exclude_episodic_substrings: default_dream_exclude_episodic_substrings(),
            min_consolidation_chars: default_dream_min_consolidation_chars(),
            honeypot_rem_enabled: true,
            honeypot_rem_anchor_limit: default_honeypot_rem_anchor_limit(),
            honeypot_rem_associate_k: default_honeypot_rem_associate_k(),
        }
    }
}

fn default_pipeline_chunk_chars() -> usize {
    28_000
}

// ─── Ingest ─────────────────────────────────────────────────────────────

/// Settings for gated document ingest (knowledge watcher / `gzmo ingest`).
#[derive(Debug, Deserialize, Clone)]
pub struct IngestConfig {
    /// When true, watchers use IngestEngine instead of headless tool loops.
    #[serde(default = "default_ingest_enabled")]
    pub enabled: bool,

    #[serde(default = "default_dream_verify")]
    pub verify: bool,

    #[serde(default = "default_dream_min_confidence")]
    pub min_confidence: f64,

    #[serde(default = "default_dream_verify_temperature")]
    pub verify_temperature: f32,

    #[serde(default = "default_kg_require_evidence")]
    pub require_evidence: bool,

    #[serde(default = "default_kg_strict")]
    pub strict_kg: bool,

    #[serde(default = "default_ingest_max_source_chars")]
    pub max_source_chars: usize,

    #[serde(default = "default_pipeline_chunk_chars")]
    pub chunk_chars: usize,
}

fn default_ingest_max_source_chars() -> usize {
    120_000
}

impl IngestConfig {
    pub fn kg_gate(&self) -> crate::memory::kg_extract::KgGateConfig {
        crate::memory::kg_extract::KgGateConfig {
            verify: self.verify,
            min_confidence: self.min_confidence,
            verify_temperature: self.verify_temperature,
            require_evidence: self.require_evidence,
            strict_kg: self.strict_kg,
        }
    }
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            enabled: default_ingest_enabled(),
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            verify_temperature: default_dream_verify_temperature(),
            require_evidence: default_kg_require_evidence(),
            strict_kg: default_kg_strict(),
            max_source_chars: default_ingest_max_source_chars(),
            chunk_chars: default_pipeline_chunk_chars(),
        }
    }
}

// ─── Wiki layer ────────────────────────────────────────────────────────────

/// Settings for the git-tracked markdown wiki layer (`WikiEngine`).
///
/// The wiki is a browsable, compounding markdown synthesis layer that sits
/// between raw RAG retrieval and `DREAMS.md`. Pages are derived from already
/// verified vault facts, so retrieval is **emit-only**: `WikiEngine::search`
/// greps over `wiki/*.md` and pages are never re-ingested into the honeypot
/// (which would create circular facts). See `WIKI.md` and `docs/WIKI_LAYER.md`.
#[derive(Debug, Deserialize, Clone)]
pub struct WikiConfig {
    #[serde(default = "default_wiki_enabled")]
    pub enabled: bool,

    #[serde(default = "default_wiki_directory")]
    pub directory: String,

    #[serde(default = "default_wiki_index_path")]
    pub index_path: String,

    #[serde(default = "default_wiki_log_path")]
    pub log_path: String,

    #[serde(default = "default_wiki_schema_path")]
    pub schema_path: String,

    /// When true, `IngestEngine` emits a `wiki/sources/` page on promotion.
    #[serde(default = "default_wiki_emit_on_ingest")]
    pub emit_on_ingest: bool,

    /// Daemon "Knowledge Gardener" sync loop (UTC hour/minute).
    #[serde(default = "default_wiki_sync_cron_hour")]
    pub sync_cron_hour: u32,
    #[serde(default = "default_wiki_sync_cron_minute")]
    pub sync_cron_minute: u32,

    /// Daemon weekly lint loop (UTC weekday 0=Sun, hour).
    #[serde(default = "default_wiki_lint_cron_dow")]
    pub lint_cron_dow: u32,
    #[serde(default = "default_wiki_lint_cron_hour")]
    pub lint_cron_hour: u32,
}

fn default_wiki_enabled() -> bool { true }
fn default_wiki_directory() -> String { "wiki".to_string() }
fn default_wiki_index_path() -> String { "wiki/index.md".to_string() }
fn default_wiki_log_path() -> String { "wiki/log.md".to_string() }
fn default_wiki_schema_path() -> String { "WIKI.md".to_string() }
fn default_wiki_emit_on_ingest() -> bool { true }
fn default_wiki_sync_cron_hour() -> u32 { 5 }
fn default_wiki_sync_cron_minute() -> u32 { 30 }
fn default_wiki_lint_cron_dow() -> u32 { 0 }
fn default_wiki_lint_cron_hour() -> u32 { 6 }

impl Default for WikiConfig {
    fn default() -> Self {
        Self {
            enabled: default_wiki_enabled(),
            directory: default_wiki_directory(),
            index_path: default_wiki_index_path(),
            log_path: default_wiki_log_path(),
            schema_path: default_wiki_schema_path(),
            emit_on_ingest: default_wiki_emit_on_ingest(),
            sync_cron_hour: default_wiki_sync_cron_hour(),
            sync_cron_minute: default_wiki_sync_cron_minute(),
            lint_cron_dow: default_wiki_lint_cron_dow(),
            lint_cron_hour: default_wiki_lint_cron_hour(),
        }
    }
}

impl WikiConfig {
    /// Absolute-ish paths relative to the agent working directory.
    pub fn entities_dir(&self) -> String { format!("{}/entities", self.directory) }
    pub fn concepts_dir(&self) -> String { format!("{}/concepts", self.directory) }
    pub fn sources_dir(&self) -> String { format!("{}/sources", self.directory) }
}

// ─── Pedagogy (Agentic Teacher) ───────────────────────────────────────────

/// Default interaction mode when pedagogy orchestrator is enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PedagogyDefaultMode {
    /// Socratic mentor via internal 4-agent orchestrator.
    #[default]
    Mentor,
    /// Direct execution (legacy ops daemon behavior).
    Ops,
}

/// Settings for the Agentic Teacher stack.
#[derive(Debug, Deserialize, Clone)]
pub struct PedagogyConfig {
    #[serde(default = "default_pedagogy_enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub default_mode: PedagogyDefaultMode,

    #[serde(default = "default_learner_data_dir")]
    pub learner_data_dir: String,

    #[serde(default = "default_prereq_graphs_dir")]
    pub prerequisite_graphs_dir: String,

    #[serde(default = "default_edf_log_path")]
    pub edf_log_path: String,

    #[serde(default = "default_max_hint_level")]
    pub max_hint_level: u8,

    #[serde(default = "default_solution_leakage_penalty")]
    pub solution_leakage_penalty: f64,

    /// Max tokens for internal agent calls (Diagnoser, Planner, etc.).
    #[serde(default = "default_pedagogy_internal_max_tokens")]
    pub internal_max_tokens: u32,

    /// Teaching turns between teachback checkpoints (0 = disabled).
    #[serde(default = "default_teachback_interval")]
    pub teachback_interval: u32,

    /// Active learner ID (set at CLI boot from `--learner` / `GZMO_LEARNER_ID`).
    #[serde(skip)]
    pub active_learner_id: Option<String>,

    /// Unix-socket headless mentor API (daemon + `gzmo mentor` client).
    #[serde(default = "default_mentor_api_enabled")]
    pub mentor_api_enabled: bool,

    #[serde(default = "default_mentor_socket")]
    pub mentor_socket: String,

    #[serde(default)]
    pub sandbox: SandboxConfig,

    /// Autonomous Socratic dialogue when tension drops below threshold.
    #[serde(default)]
    pub low_tension_dialogue: LowTensionDialogueConfig,

    /// Path to `gzmo_skills` (pi-mentor-discovery scripts).
    #[serde(default = "default_discovery_scripts_root")]
    pub discovery_scripts_root: String,

    /// Structured chaos_val oscillation for discovery sprints (0.9→0.5→0.9).
    #[serde(default)]
    pub tension_oscillation: TensionOscillationConfig,
}

/// One phase in a pedagogy tension oscillation cycle.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct TensionOscillationStepConfig {
    pub target: f64,
    pub duration_secs: u32,
    #[serde(default)]
    pub label: String,
}

/// Pedagogy chaos_val setpoint controller (PulseLoop integration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TensionOscillationScheduleMode {
    /// CLI / inbox trigger only (v1 default).
    #[default]
    Manual,
    /// Reserved: daemon cron (not wired in v1).
    Cron,
}

/// Pedagogy chaos_val setpoint controller (PulseLoop integration).
#[derive(Debug, Deserialize, Clone)]
pub struct TensionOscillationConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub schedule_mode: TensionOscillationScheduleMode,

    /// Reserved for future daemon cron when `schedule_mode = "cron"`.
    #[serde(default)]
    pub cron_hours: Vec<u32>,

    #[serde(default = "default_tension_oscillation_spawn_discovery")]
    pub spawn_discovery_on_low: bool,

    #[serde(default = "default_tension_oscillation_low_threshold")]
    pub low_phase_threshold: f64,

    #[serde(default = "default_tension_oscillation_cooldown_secs")]
    pub cooldown_secs: u64,

    #[serde(default = "default_tension_oscillation_blend_ticks")]
    pub blend_ticks: u64,

    #[serde(default = "default_tension_oscillation_sequence")]
    pub sequence: Vec<TensionOscillationStepConfig>,
}

fn default_tension_oscillation_spawn_discovery() -> bool {
    true
}

fn default_tension_oscillation_low_threshold() -> f64 {
    0.55
}

fn default_tension_oscillation_cooldown_secs() -> u64 {
    3600
}

fn default_tension_oscillation_blend_ticks() -> u64 {
    8
}

fn default_tension_oscillation_sequence() -> Vec<TensionOscillationStepConfig> {
    vec![
        TensionOscillationStepConfig {
            target: 0.9,
            duration_secs: 60,
            label: "High tension — confirmation machine".to_string(),
        },
        TensionOscillationStepConfig {
            target: 0.5,
            duration_secs: 60,
            label: "Low tension — discovery machine".to_string(),
        },
        TensionOscillationStepConfig {
            target: 0.9,
            duration_secs: 60,
            label: "High tension — confirmation machine".to_string(),
        },
    ]
}

impl Default for TensionOscillationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            schedule_mode: TensionOscillationScheduleMode::default(),
            cron_hours: Vec::new(),
            spawn_discovery_on_low: default_tension_oscillation_spawn_discovery(),
            low_phase_threshold: default_tension_oscillation_low_threshold(),
            cooldown_secs: default_tension_oscillation_cooldown_secs(),
            blend_ticks: default_tension_oscillation_blend_ticks(),
            sequence: default_tension_oscillation_sequence(),
        }
    }
}

impl TensionOscillationConfig {
    /// Convert to gzmo-chaos PulseLoop settings.
    pub fn to_chaos_settings(&self) -> gzmo_chaos::pedagogy_oscillator::PedagogyOscillationSettings {
        gzmo_chaos::pedagogy_oscillator::PedagogyOscillationSettings {
            enabled: self.enabled,
            spawn_discovery_on_low: self.spawn_discovery_on_low,
            low_phase_threshold: self.low_phase_threshold,
            cooldown_secs: self.cooldown_secs,
            blend_ticks: self.blend_ticks,
            sequence: self
                .sequence
                .iter()
                .map(|s| gzmo_chaos::pedagogy_oscillator::OscillationStep {
                    target: s.target,
                    duration_secs: s.duration_secs,
                    label: s.label.clone(),
                })
                .collect(),
        }
    }
}

/// Daemon-initiated mentor turn when chaos tension is very low.
#[derive(Debug, Deserialize, Clone)]
pub struct LowTensionDialogueConfig {
    #[serde(default)]
    pub enabled: bool,

    /// Fire when tension crosses below this value (edge-triggered).
    #[serde(default = "default_low_tension_threshold")]
    pub threshold: f64,

    /// Minimum seconds between autonomous dialogue turns.
    #[serde(default = "default_low_tension_cooldown")]
    pub cooldown_secs: u64,

    /// Seed message for bare `maybe_teach` — placeholders: `{tension}`, `{tick}`, `{phase}`.
    /// Ignored when `discovery_cycle` is true.
    #[serde(default = "default_low_tension_opening")]
    pub opening_template: String,

    /// Run full pi-mentor-discovery cycle (pillar probe + cycle report) instead of bare mentor teach.
    #[serde(default = "default_low_tension_discovery_cycle")]
    pub discovery_cycle: bool,

    /// Minimum ticks of low-tension plateau to fire dialogue trigger.
    #[serde(default)]
    pub idle_ticks_threshold: Option<u64>,

    /// Discovery queue config (max pending, max concurrent, session priority).
    #[serde(default)]
    pub discovery_queue: DiscoveryQueueConfig,
}

/// Discovery queue configuration for AUTO Socratic cycles.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct DiscoveryQueueConfig {
    /// Maximum pending discovery cycles in the queue.
    #[serde(default = "default_discovery_max_pending")]
    pub max_pending: usize,

    /// Maximum concurrent discovery cycles.
    #[serde(default = "default_discovery_max_concurrent")]
    pub max_concurrent: usize,

    /// Prioritize session-bound cycles over AUTO cycles.
    #[serde(default = "default_discovery_session_priority")]
    pub session_priority: bool,
}

fn default_discovery_max_pending() -> usize { 2 }
fn default_discovery_max_concurrent() -> usize { 1 }
fn default_discovery_session_priority() -> bool { true }

impl Default for LowTensionDialogueConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: default_low_tension_threshold(),
            cooldown_secs: default_low_tension_cooldown(),
            opening_template: default_low_tension_opening(),
            discovery_cycle: default_low_tension_discovery_cycle(),
            idle_ticks_threshold: None,
            discovery_queue: DiscoveryQueueConfig::default(),
        }
    }
}

fn default_low_tension_discovery_cycle() -> bool {
    true
}

fn default_discovery_scripts_root() -> String {
    std::env::var("GZMO_SKILLS_ROOT").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/maximilian-wruhs".into());
        format!("{home}/gzmo_skills")
    })
}

fn default_low_tension_threshold() -> f64 {
    15.0
}

fn default_low_tension_cooldown() -> u64 {
    300
}

fn default_low_tension_opening() -> String {
    "[AUTONOMOUS — low tension] System tension is very low (τ={tension}%, tick {tick}, phase {phase}). \
     Begin a Socratic dialogue with the learner: ask one inviting question about stillness, dormancy, \
     or what the organism should attend to when the chaos field is calm. Do not lecture; do not give the answer."
        .to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SandboxConfig {
    #[serde(default = "default_sandbox_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sandbox_max_code_chars")]
    pub max_code_chars: usize,
    #[serde(default = "default_sandbox_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_sandbox_max_output_chars")]
    pub max_output_chars: usize,
    #[serde(default = "default_sandbox_blocked_imports")]
    pub blocked_imports: Vec<String>,
    #[serde(default = "default_sandbox_orchestrator_offload")]
    pub orchestrator_offload: bool,
}

fn default_sandbox_enabled() -> bool { true }
fn default_sandbox_max_code_chars() -> usize { 2000 }
fn default_sandbox_timeout_secs() -> u64 { 10 }
fn default_sandbox_max_output_chars() -> usize { 4000 }
fn default_sandbox_blocked_imports() -> Vec<String> {
    vec![
        "os".to_string(),
        "subprocess".to_string(),
        "socket".to_string(),
        "shutil".to_string(),
        "sys".to_string(),
    ]
}
fn default_sandbox_orchestrator_offload() -> bool { false }

fn default_pedagogy_enabled() -> bool { true }
fn default_learner_data_dir() -> String { "data/learner".to_string() }
fn default_prereq_graphs_dir() -> String { "data/pedagogy/graphs".to_string() }
fn default_edf_log_path() -> String { "data/pedagogy/edf_log.jsonl".to_string() }
fn default_max_hint_level() -> u8 { 5 }
fn default_solution_leakage_penalty() -> f64 { 1.0 }
fn default_pedagogy_internal_max_tokens() -> u32 { 512 }
fn default_teachback_interval() -> u32 { 8 }
fn default_mentor_api_enabled() -> bool { true }
fn default_mentor_socket() -> String { "data/gzmo_mentor.sock".to_string() }

impl Default for PedagogyConfig {
    fn default() -> Self {
        Self {
            enabled: default_pedagogy_enabled(),
            default_mode: PedagogyDefaultMode::default(),
            learner_data_dir: default_learner_data_dir(),
            prerequisite_graphs_dir: default_prereq_graphs_dir(),
            edf_log_path: default_edf_log_path(),
            max_hint_level: default_max_hint_level(),
            solution_leakage_penalty: default_solution_leakage_penalty(),
            internal_max_tokens: default_pedagogy_internal_max_tokens(),
            teachback_interval: default_teachback_interval(),
            active_learner_id: None,
            mentor_api_enabled: default_mentor_api_enabled(),
            mentor_socket: default_mentor_socket(),
            sandbox: SandboxConfig::default(),
            low_tension_dialogue: LowTensionDialogueConfig::default(),
            discovery_scripts_root: default_discovery_scripts_root(),
            tension_oscillation: TensionOscillationConfig::default(),
        }
    }
}

impl PedagogyConfig {
    /// Resolve learner ID: `--learner` flag → `GZMO_LEARNER_ID` env → `"operator"`.
    pub fn resolve_learner_id(cli_flag: Option<&str>) -> String {
        if let Some(id) = cli_flag.filter(|s| !s.is_empty()) {
            return id.to_string();
        }
        if let Ok(id) = std::env::var("GZMO_LEARNER_ID") {
            if !id.is_empty() {
                return id;
            }
        }
        "operator".to_string()
    }

    pub fn learner_id(&self) -> &str {
        self.active_learner_id
            .as_deref()
            .unwrap_or("operator")
    }

    pub fn learner_dir(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.learner_data_dir).join(self.learner_id())
    }

    pub fn profile_path(&self) -> std::path::PathBuf {
        self.learner_dir().join("profile.json")
    }

    pub fn session_path(&self) -> std::path::PathBuf {
        self.learner_dir().join("session.json")
    }

    pub fn episodes_dir(&self) -> std::path::PathBuf {
        self.learner_dir().join("episodes")
    }

    pub fn mentor_socket_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.mentor_socket)
    }
}

// ─── Session distill ──────────────────────────────────────────────────────

/// Settings for `gzmo distill` — sessions JSON → SessionDistill vault + episodic.
#[derive(Debug, Deserialize, Clone)]
pub struct SessionDistillConfig {
    #[serde(default = "default_session_distill_enabled")]
    pub enabled: bool,

    #[serde(default = "default_sessions_dir")]
    pub sessions_dir: std::path::PathBuf,

    #[serde(default = "default_dream_verify")]
    pub verify: bool,

    #[serde(default = "default_dream_min_confidence")]
    pub min_confidence: f64,

    #[serde(default = "default_dream_verify_temperature")]
    pub verify_temperature: f32,

    #[serde(default = "default_kg_require_evidence")]
    pub require_evidence: bool,

    #[serde(default = "default_kg_strict")]
    pub strict_kg: bool,

    #[serde(default = "default_pipeline_chunk_chars")]
    pub chunk_chars: usize,

    #[serde(default = "default_session_distill_max_transcript")]
    pub max_transcript_chars: usize,

    /// Use `[librarian]` for KG extract; Prime (local engine) stays on verify.
    #[serde(default = "default_session_distill_use_librarian")]
    pub use_librarian: bool,

    /// Short narrative for episodic via librarian (falls back to entity list).
    #[serde(default = "default_session_distill_librarian_summary")]
    pub librarian_summary: bool,

    /// Run `gzmo distill` on a daily cron inside the daemon (default 02:15 UTC).
    #[serde(default = "default_session_distill_daemon_scheduled")]
    pub daemon_scheduled: bool,

    #[serde(default = "default_session_distill_cron_hour")]
    pub cron_hour: u32,

    #[serde(default = "default_session_distill_cron_minute")]
    pub cron_minute: u32,

    /// Phase 2: distill when topic embedding drifts mid-session (not implemented).
    #[serde(default)]
    pub topic_shift_enabled: bool,

    /// Cosine distance threshold for topic-shift trigger (phase 2).
    #[serde(default = "default_topic_shift_threshold")]
    pub topic_shift_threshold: f32,

    /// Use SHA-256 content hash for dedup instead of path-based tracking.
    /// When true, sessions with genuinely unique content are re-processed
    /// even after daemon restart (prevents 96% duplicate rejection rate).
    #[serde(default = "default_content_hash_dedup")]
    pub content_hash_dedup: bool,
}

fn default_content_hash_dedup() -> bool { true }

fn default_topic_shift_threshold() -> f32 {
    0.35
}

fn default_session_distill_enabled() -> bool {
    true
}

fn default_sessions_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("data/sessions")
}

fn default_session_distill_max_transcript() -> usize {
    28_000
}

fn default_session_distill_use_librarian() -> bool {
    true
}

fn default_session_distill_librarian_summary() -> bool {
    true
}

fn default_session_distill_daemon_scheduled() -> bool {
    true
}

fn default_session_distill_cron_hour() -> u32 {
    2
}

fn default_session_distill_cron_minute() -> u32 {
    15
}

impl SessionDistillConfig {
    pub fn kg_gate(&self) -> crate::memory::kg_extract::KgGateConfig {
        crate::memory::kg_extract::KgGateConfig {
            verify: self.verify,
            min_confidence: self.min_confidence,
            verify_temperature: self.verify_temperature,
            require_evidence: self.require_evidence,
            strict_kg: self.strict_kg,
        }
    }
}

impl Default for SessionDistillConfig {
    fn default() -> Self {
        Self {
            enabled: default_session_distill_enabled(),
            topic_shift_enabled: false,
            topic_shift_threshold: default_topic_shift_threshold(),
            sessions_dir: default_sessions_dir(),
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            verify_temperature: default_dream_verify_temperature(),
            require_evidence: default_kg_require_evidence(),
            strict_kg: default_kg_strict(),
            chunk_chars: default_pipeline_chunk_chars(),
            max_transcript_chars: default_session_distill_max_transcript(),
            use_librarian: default_session_distill_use_librarian(),
            librarian_summary: default_session_distill_librarian_summary(),
            daemon_scheduled: default_session_distill_daemon_scheduled(),
            cron_hour: default_session_distill_cron_hour(),
            cron_minute: default_session_distill_cron_minute(),
            content_hash_dedup: default_content_hash_dedup(),
        }
    }
}

// ─── Spark ──────────────────────────────────────────────────────────────

/// When to run spark: fixed UTC cron slots or dice-scheduler jitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SparkScheduleMode {
    #[default]
    Cron,
    Dice,
}

/// Settings for the SparkEngine (serendipitous recall).
#[derive(Debug, Deserialize, Clone)]
pub struct SparkConfig {
    /// Master switch — when false, daemon and CLI skip spark runs.
    #[serde(default = "default_spark_enabled")]
    pub enabled: bool,

    #[serde(default = "default_dream_verify")]
    pub verify: bool,

    #[serde(default = "default_dream_min_confidence")]
    pub min_confidence: f64,

    #[serde(default = "default_spark_hypothesis_temperature")]
    pub hypothesis_temperature: f32,

    #[serde(default = "default_dream_verify_temperature")]
    pub verify_temperature: f32,

    /// How many stale vault facts to consider before picking the anchor.
    #[serde(default = "default_spark_candidate_limit")]
    pub candidate_limit: usize,

    /// How many recent facts to offer as contrast context.
    #[serde(default = "default_spark_recent_limit")]
    pub recent_limit: usize,

    /// Confidence for optional quarantine audit entries (always below vault gate).
    #[serde(default = "default_spark_quarantine_confidence")]
    pub quarantine_confidence: f64,

    /// Hours (UTC) when the daemon runs spark, at minute `cron_minute`.
    #[serde(default = "default_spark_cron_hours")]
    pub cron_hours: Vec<u32>,

    #[serde(default = "default_spark_cron_minute")]
    pub cron_minute: u32,

    /// `cron` = fixed hours above; `dice` = d6 jitter between min/max minutes.
    #[serde(default)]
    pub schedule_mode: SparkScheduleMode,

    #[serde(default = "default_spark_dice_min")]
    pub dice_min_minutes: u32,

    #[serde(default = "default_spark_dice_max")]
    pub dice_max_minutes: u32,

    /// LCG seed for dice rolls (defaults from `[chaos].seed` in daemon when unset).
    #[serde(default)]
    pub dice_seed: Option<u64>,

    /// Shallow job caps — keep spark off the deep-research path (AI-Q pattern).
    #[serde(default = "default_spark_max_tokens_hypothesis")]
    pub max_tokens_hypothesis: u32,

    #[serde(default = "default_spark_max_tokens_verify")]
    pub max_tokens_verify: u32,

    #[serde(default = "default_spark_max_connection_chars")]
    pub max_connection_chars: usize,

    /// Minimum quotable span length per anchor (LDR / dream firewall).
    #[serde(default = "default_spark_min_citation_chars")]
    pub min_citation_chars: usize,

    /// Substrings that disqualify a vault fact from being a spark anchor.
    #[serde(default = "default_spark_exclude_anchor_substrings")]
    pub exclude_anchor_substrings: Vec<String>,

    /// Vault `decay_class` values eligible for spark anchors (curated wisdom, not ops noise).
    #[serde(default = "default_spark_anchor_decay_classes")]
    pub anchor_decay_classes: Vec<String>,

    /// Minimum days since `last_accessed_at` before an anchor is considered stale enough.
    #[serde(default = "default_spark_anchor_min_stale_days")]
    pub anchor_min_stale_days: u32,

    /// Maximum days since `created_at` for anchor candidacy (avoid ancient junk).
    #[serde(default = "default_spark_anchor_max_stale_days")]
    pub anchor_max_stale_days: u32,

    /// Anchors must be at least this many hours old (separates fresh ingest slab from recent pool).
    #[serde(default = "default_spark_anchor_min_age_hours")]
    pub anchor_min_age_hours: u32,

    /// Recent pool: only facts created within this many hours (ingest window).
    #[serde(default = "default_spark_recent_max_age_hours")]
    pub recent_max_age_hours: u32,

    /// Minimum embedding cosine between anchor and at least one recent fact (or shared concept tag).
    #[serde(default = "default_spark_min_anchor_recent_similarity")]
    pub min_anchor_recent_similarity: f64,

    /// Drop near-duplicate recent facts above this cosine similarity.
    #[serde(default = "default_spark_recent_dedupe_similarity")]
    pub recent_dedupe_similarity: f64,

    /// Session anchors older than this many days are skipped (parsed from `[Session YYYY-MM-DD …]`).
    #[serde(default = "default_spark_max_session_anchor_age_days")]
    pub max_session_anchor_age_days: u32,

    /// Before each daemon spark run, refresh honeypot `promoted_at` when recent pool is thin.
    #[serde(default = "default_spark_anchor_refresh_enabled")]
    pub anchor_refresh_enabled: bool,

    /// Minimum recent-pool facts required; refresh runs when count is below this.
    #[serde(default = "default_spark_anchor_refresh_min_recent")]
    pub anchor_refresh_min_recent: usize,
}

fn default_spark_anchor_decay_classes() -> Vec<String> {
    vec!["CuratedVault".to_string(), "SessionDistill".to_string()]
}

fn default_spark_anchor_min_stale_days() -> u32 {
    0
}

fn default_spark_anchor_max_stale_days() -> u32 {
    60
}

fn default_spark_anchor_min_age_hours() -> u32 {
    6
}

fn default_spark_recent_max_age_hours() -> u32 {
    72
}

fn default_spark_min_anchor_recent_similarity() -> f64 {
    0.35
}

fn default_spark_recent_dedupe_similarity() -> f64 {
    0.92
}

fn default_spark_exclude_anchor_substrings() -> Vec<String> {
    vec![
        "[Session ".to_string(),
        "Topics discussed: GZMO, open sovereign.toml".to_string(),
        "filesystem utilization".to_string(),
        "sys_janitor".to_string(),
        "[ingest:".to_string(),
        "Root filesystem".to_string(),
        "CPU | RAM".to_string(),
    ]
}

fn default_spark_max_session_anchor_age_days() -> u32 {
    14
}

fn default_spark_dice_min() -> u32 {
    20
}
fn default_spark_dice_max() -> u32 {
    180
}
fn default_spark_max_tokens_hypothesis() -> u32 {
    2048
}
fn default_spark_max_tokens_verify() -> u32 {
    1024
}
fn default_spark_max_connection_chars() -> usize {
    1200
}
fn default_spark_min_citation_chars() -> usize {
    12
}

fn default_spark_anchor_refresh_enabled() -> bool {
    true
}

fn default_spark_anchor_refresh_min_recent() -> usize {
    4
}

impl Default for SparkConfig {
    fn default() -> Self {
        Self {
            enabled: default_spark_enabled(),
            verify: default_dream_verify(),
            min_confidence: default_dream_min_confidence(),
            hypothesis_temperature: default_spark_hypothesis_temperature(),
            verify_temperature: default_dream_verify_temperature(),
            candidate_limit: default_spark_candidate_limit(),
            recent_limit: default_spark_recent_limit(),
            quarantine_confidence: default_spark_quarantine_confidence(),
            cron_hours: default_spark_cron_hours(),
            cron_minute: default_spark_cron_minute(),
            schedule_mode: SparkScheduleMode::default(),
            dice_min_minutes: default_spark_dice_min(),
            dice_max_minutes: default_spark_dice_max(),
            dice_seed: None,
            max_tokens_hypothesis: default_spark_max_tokens_hypothesis(),
            max_tokens_verify: default_spark_max_tokens_verify(),
            max_connection_chars: default_spark_max_connection_chars(),
            min_citation_chars: default_spark_min_citation_chars(),
            exclude_anchor_substrings: default_spark_exclude_anchor_substrings(),
            anchor_decay_classes: default_spark_anchor_decay_classes(),
            anchor_min_stale_days: default_spark_anchor_min_stale_days(),
            anchor_max_stale_days: default_spark_anchor_max_stale_days(),
            anchor_min_age_hours: default_spark_anchor_min_age_hours(),
            recent_max_age_hours: default_spark_recent_max_age_hours(),
            min_anchor_recent_similarity: default_spark_min_anchor_recent_similarity(),
            recent_dedupe_similarity: default_spark_recent_dedupe_similarity(),
            max_session_anchor_age_days: default_spark_max_session_anchor_age_days(),
            anchor_refresh_enabled: default_spark_anchor_refresh_enabled(),
            anchor_refresh_min_recent: default_spark_anchor_refresh_min_recent(),
        }
    }
}

// ─── Health ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct HealthConfig {
    /// When true, daemon aborts if Prime/embed/MCP probes fail (Sovereign probe is advisory).
    #[serde(default)]
    pub strict_startup: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            strict_startup: false,
        }
    }
}

// ─── Embeddings ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_embeddings_url")]
    pub url: String,

    #[serde(default = "default_embeddings_model")]
    pub model: String,

    #[serde(default)]
    pub api_key: String,

    /// When true and `[redis].enabled`, cache vectors in Redis (24h TTL by default).
    #[serde(default = "default_true")]
    pub cache_enabled: bool,

    #[serde(default = "default_embed_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
}

fn default_true() -> bool {
    true
}

fn default_embed_cache_ttl_secs() -> u64 {
    86_400
}

fn default_embeddings_url() -> String {
    "http://localhost:8002/v1".to_string()
}

fn default_embeddings_model() -> String {
    "Qwen3-Embedding-0.6B".to_string()
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: default_embeddings_url(),
            model: default_embeddings_model(),
            api_key: String::new(),
            cache_enabled: true,
            cache_ttl_secs: default_embed_cache_ttl_secs(),
        }
    }
}

// ─── Qdrant (vault mirror on LXC101) ────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct QdrantConfig {
    /// When true, `gzmo health` probes collection reachability (SQLite remains source of truth).
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_qdrant_url")]
    pub url: String,

    #[serde(default = "default_qdrant_collection")]
    pub collection: String,

    /// Daemon runs `scripts/sync-vault-to-qdrant.py` on schedule.
    #[serde(default)]
    pub sync_enabled: bool,

    #[serde(default = "default_qdrant_sync_cron_hour")]
    pub sync_cron_hour: u32,

    #[serde(default = "default_qdrant_sync_cron_minute")]
    pub sync_cron_minute: u32,
}

fn default_qdrant_url() -> String {
    "http://192.168.31.202:6333".to_string()
}

fn default_qdrant_collection() -> String {
    "honeypot".to_string()
}

fn default_qdrant_sync_cron_hour() -> u32 {
    1
}

fn default_qdrant_sync_cron_minute() -> u32 {
    45
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: default_qdrant_url(),
            collection: default_qdrant_collection(),
            sync_enabled: false,
            sync_cron_hour: default_qdrant_sync_cron_hour(),
            sync_cron_minute: default_qdrant_sync_cron_minute(),
        }
    }
}

// ─── Platform search (cross-collection RAG) ─────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformSearchConfig {
    /// When true, `gzmo_memory_search` also queries the Pi `knowledge` Qdrant collection.
    #[serde(default = "default_platform_search_enabled")]
    pub include_knowledge_collection: bool,

    /// Qdrant collection name for Pi knowledge docs (legacy mirror, read-only).
    #[serde(default = "default_knowledge_collection")]
    pub knowledge_collection: String,

    /// Prefetch multiplier for knowledge vector hits before rerank merge.
    #[serde(default = "default_knowledge_prefetch")]
    pub knowledge_prefetch: usize,

    /// Append Neo4j graph compact search results to unified recall text.
    #[serde(default = "default_neo4j_graph_search")]
    pub neo4j_graph_search: bool,

    /// Path to `uvx` for Neo4j MCP CLI search (falls back to `NEO4J_MCP_UVX` env).
    #[serde(default)]
    pub neo4j_mcp_uvx: Option<String>,

    /// Local fork path passed to `uvx --from` (falls back to `NEO4J_MCP_FROM` env).
    #[serde(default)]
    pub neo4j_mcp_from: Option<String>,

    /// Bolt URI for Neo4j CLI subprocess (falls back to `NEO4J_URI` / `NEO4J_URL` env).
    #[serde(default)]
    pub neo4j_uri: Option<String>,

    /// Neo4j username for CLI subprocess (falls back to env).
    #[serde(default)]
    pub neo4j_username: Option<String>,

    /// Neo4j password for CLI subprocess (falls back to `NEO4J_PASSWORD` env).
    #[serde(default)]
    pub neo4j_password: Option<String>,
}

fn default_platform_search_enabled() -> bool {
    true
}

fn default_knowledge_collection() -> String {
    "knowledge".to_string()
}

fn default_knowledge_prefetch() -> usize {
    12
}

fn default_neo4j_graph_search() -> bool {
    true
}

impl Default for PlatformSearchConfig {
    fn default() -> Self {
        Self {
            include_knowledge_collection: default_platform_search_enabled(),
            knowledge_collection: default_knowledge_collection(),
            knowledge_prefetch: default_knowledge_prefetch(),
            neo4j_graph_search: default_neo4j_graph_search(),
            neo4j_mcp_uvx: None,
            neo4j_mcp_from: None,
            neo4j_uri: None,
            neo4j_username: None,
            neo4j_password: None,
        }
    }
}

// ─── KG reconcile (shared Neo4j ontology) ───────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct KgReconcileConfig {
    #[serde(default)]
    pub enabled: bool,

    /// UTC hour for weekly reconcile (default Sunday 04:00 if cron not set).
    #[serde(default = "default_kg_reconcile_hour")]
    pub cron_hour: u32,

    #[serde(default)]
    pub cron_minute: u32,

    /// Dry-run: log planned changes without MCP writes.
    #[serde(default)]
    pub dry_run: bool,
}

fn default_kg_reconcile_hour() -> u32 {
    4
}

impl Default for KgReconcileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cron_hour: default_kg_reconcile_hour(),
            cron_minute: 0,
            dry_run: true,
        }
    }
}

// ─── Synapse pull (read-only Pi event ingest) ───────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct SynapsePullConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_synapse_pull_hour")]
    pub cron_hour: u32,

    #[serde(default = "default_synapse_pull_minute")]
    pub cron_minute: u32,

    /// Max Pi events to summarize per pull cycle.
    #[serde(default = "default_synapse_pull_max_events")]
    pub max_events: usize,

    /// Path to append-only bus (relative to project root).
    #[serde(default = "default_synapse_bus_path")]
    pub bus_path: std::path::PathBuf,

    /// Run `gzmo distill pi <session.jsonl>` when Pi emits `session_end` on the bus.
    #[serde(default = "default_synapse_distill_on_session_end")]
    pub distill_on_session_end: bool,
}

fn default_synapse_distill_on_session_end() -> bool {
    true
}

fn default_synapse_pull_hour() -> u32 {
    2
}

fn default_synapse_pull_minute() -> u32 {
    45
}

fn default_synapse_pull_max_events() -> usize {
    50
}

fn default_synapse_bus_path() -> std::path::PathBuf {
    std::path::PathBuf::from("data/Synapse/events.jsonl")
}

impl Default for SynapsePullConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cron_hour: default_synapse_pull_hour(),
            cron_minute: default_synapse_pull_minute(),
            max_events: default_synapse_pull_max_events(),
            bus_path: default_synapse_bus_path(),
            distill_on_session_end: default_synapse_distill_on_session_end(),
        }
    }
}

// ─── Kurator monitor (phase 1) ───────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct KuratorConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_kurator_max_turns")]
    pub max_turns_before_recommend: u32,

    #[serde(default = "default_kurator_max_session_tokens")]
    pub max_session_tokens: u64,

    #[serde(default = "default_kurator_skill_error_rate")]
    pub skill_error_rate_threshold: f64,

    #[serde(default = "default_kurator_max_dice_loops")]
    pub max_dice_loops_per_hour: u32,

    #[serde(default = "default_kurator_agent_profile")]
    pub default_agent_profile: String,

    /// Phase 2: `gzmo kurator approve` spawns a governed sub-agent.
    #[serde(default = "default_kurator_approve_enabled")]
    pub approve_spawns_subagent: bool,

    /// Phase 3: daemon autospawns on `spawn.recommended` (no operator).
    #[serde(default = "default_kurator_auto_spawn")]
    pub auto_spawn_on_recommend: bool,

    #[serde(default = "default_kurator_spawn_brief_max")]
    pub spawn_brief_max_chars: usize,

    /// Spawn epimetheus/fixer sub-agent when discovery reports contain actionable items
    /// (FAIL/GAP markers or Recommended next actions).
    #[serde(default = "default_discovery_fixer_enabled")]
    pub discovery_fixer_enabled: bool,

    #[serde(default = "default_kurator_fixer_profile")]
    pub fixer_agent_profile: String,

    /// After deterministic probes, spawn agent to write code/config remediations.
    #[serde(default = "default_discovery_code_implementer_enabled")]
    pub discovery_code_implementer_enabled: bool,

    #[serde(default = "default_code_implementer_agent_profile")]
    pub code_implementer_agent_profile: String,

    #[serde(default = "default_discovery_code_implementer_max_iterations")]
    pub discovery_code_implementer_max_iterations: usize,

    #[serde(default = "default_discovery_code_implementer_max_retries")]
    pub discovery_code_implementer_max_retries: u32,

    /// Spawn plan agent to write Forum-2 plan dossier before fixer execute.
    #[serde(default = "default_discovery_plan_agent_enabled")]
    pub discovery_plan_agent_enabled: bool,

    #[serde(default = "default_discovery_plan_agent_profile")]
    pub discovery_plan_agent_profile: String,

    #[serde(default = "default_discovery_plan_max_iterations")]
    pub discovery_plan_max_iterations: usize,

    #[serde(default = "default_discovery_plan_max_retries")]
    pub discovery_plan_max_retries: u32,

    /// Minimum actionable items (FAIL + GAP + ACTION) before emitting a fixer spawn.
    #[serde(default = "default_discovery_fixer_min_findings")]
    pub discovery_fixer_min_findings: u32,

    /// Max tool-call rounds for discovery fixer sub-agents (epimetheus remediation).
    #[serde(default = "default_discovery_fixer_max_iterations")]
    pub discovery_fixer_max_iterations: usize,

    /// Extra single-finding retry spawns after verify_gate FAIL (0 = no retries).
    #[serde(default = "default_discovery_fixer_max_retries")]
    pub discovery_fixer_max_retries: u32,

    /// Extra shell binaries for discovery fixer only (merged with [subagent].shell_extra_commands).
    #[serde(default = "default_discovery_fixer_shell_extra")]
    pub discovery_fixer_shell_extra_commands: Vec<String>,

    /// Spawn gate — central autospawn policy (rate limits, tiers).
    #[serde(default)]
    pub spawn_gate: SpawnGateConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpawnGateConfig {
    #[serde(default = "default_spawn_gate_enabled")]
    pub enabled: bool,

    #[serde(default = "default_max_autospawns_per_hour")]
    pub max_autospawns_per_hour: u32,

    #[serde(default = "default_spawn_cooldown_secs")]
    pub spawn_cooldown_secs: u64,

    /// Block prometheus/session_triage autospawn while discovery_fix is pending or in cooldown.
    #[serde(default = "default_prometheus_requires_idle")]
    pub prometheus_requires_idle: bool,

    #[serde(default = "default_duplicate_reason_max_per_hour")]
    pub duplicate_reason_max_per_hour: u32,

    /// Autospawn epimetheus discovery fixer (separate from session_triage).
    #[serde(default = "default_auto_spawn_discovery_fix")]
    pub auto_spawn_discovery_fix: bool,

    /// Redis hourly Prime slot budget before sub-agent spawn (protects :8000).
    #[serde(default = "default_prime_budget_enabled")]
    pub prime_budget_enabled: bool,

    #[serde(default = "default_prime_spawn_budget_per_hour")]
    pub prime_spawn_budget_per_hour: u32,

    /// When Redis is unreachable, allow spawn and log (homelab default).
    #[serde(default = "default_prime_budget_fail_open")]
    pub prime_budget_fail_open: bool,

    #[serde(default)]
    pub prime_budget_key_prefix: Option<String>,

    #[serde(default = "default_prime_budget_ttl_secs")]
    pub prime_budget_ttl_secs: u64,
}

fn default_spawn_gate_enabled() -> bool {
    true
}

fn default_max_autospawns_per_hour() -> u32 {
    3
}

fn default_spawn_cooldown_secs() -> u64 {
    600
}

fn default_prometheus_requires_idle() -> bool {
    true
}

fn default_duplicate_reason_max_per_hour() -> u32 {
    3
}

fn default_auto_spawn_discovery_fix() -> bool {
    true
}

fn default_prime_budget_enabled() -> bool {
    true
}

fn default_prime_spawn_budget_per_hour() -> u32 {
    3
}

fn default_prime_budget_fail_open() -> bool {
    true
}

fn default_prime_budget_ttl_secs() -> u64 {
    7200
}

impl Default for SpawnGateConfig {
    fn default() -> Self {
        Self {
            enabled: default_spawn_gate_enabled(),
            max_autospawns_per_hour: default_max_autospawns_per_hour(),
            spawn_cooldown_secs: default_spawn_cooldown_secs(),
            prometheus_requires_idle: default_prometheus_requires_idle(),
            duplicate_reason_max_per_hour: default_duplicate_reason_max_per_hour(),
            auto_spawn_discovery_fix: default_auto_spawn_discovery_fix(),
            prime_budget_enabled: default_prime_budget_enabled(),
            prime_spawn_budget_per_hour: default_prime_spawn_budget_per_hour(),
            prime_budget_fail_open: default_prime_budget_fail_open(),
            prime_budget_key_prefix: None,
            prime_budget_ttl_secs: default_prime_budget_ttl_secs(),
        }
    }
}

fn default_kurator_auto_spawn() -> bool {
    true
}

fn default_kurator_max_turns() -> u32 {
    40
}

fn default_kurator_max_session_tokens() -> u64 {
    500_000
}

fn default_kurator_skill_error_rate() -> f64 {
    0.3
}

fn default_kurator_max_dice_loops() -> u32 {
    12
}

fn default_kurator_agent_profile() -> String {
    "prometheus".to_string()
}

fn default_kurator_approve_enabled() -> bool {
    true
}

fn default_kurator_spawn_brief_max() -> usize {
    4000
}

fn default_discovery_fixer_enabled() -> bool {
    true
}

fn default_discovery_code_implementer_enabled() -> bool {
    true
}

fn default_code_implementer_agent_profile() -> String {
    "epimetheus".to_string()
}

fn default_discovery_code_implementer_max_iterations() -> usize {
    40
}

fn default_discovery_code_implementer_max_retries() -> u32 {
    2
}

fn default_discovery_plan_agent_enabled() -> bool {
    true
}

fn default_discovery_plan_agent_profile() -> String {
    "epimetheus".to_string()
}

fn default_discovery_plan_max_iterations() -> usize {
    50
}

fn default_discovery_plan_max_retries() -> u32 {
    1
}

fn default_kurator_fixer_profile() -> String {
    "epimetheus".to_string()
}

fn default_discovery_fixer_min_findings() -> u32 {
    1
}

fn default_discovery_fixer_max_iterations() -> usize {
    35
}

fn default_discovery_fixer_max_retries() -> u32 {
    1
}

fn default_discovery_fixer_shell_extra() -> Vec<String> {
    vec!["bash".into(), "sh".into()]
}

impl Default for KuratorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_turns_before_recommend: default_kurator_max_turns(),
            max_session_tokens: default_kurator_max_session_tokens(),
            skill_error_rate_threshold: default_kurator_skill_error_rate(),
            max_dice_loops_per_hour: default_kurator_max_dice_loops(),
            default_agent_profile: default_kurator_agent_profile(),
            approve_spawns_subagent: default_kurator_approve_enabled(),
            auto_spawn_on_recommend: default_kurator_auto_spawn(),
            spawn_brief_max_chars: default_kurator_spawn_brief_max(),
            discovery_fixer_enabled: default_discovery_fixer_enabled(),
            fixer_agent_profile: default_kurator_fixer_profile(),
            discovery_code_implementer_enabled: default_discovery_code_implementer_enabled(),
            code_implementer_agent_profile: default_code_implementer_agent_profile(),
            discovery_code_implementer_max_iterations: default_discovery_code_implementer_max_iterations(),
            discovery_code_implementer_max_retries: default_discovery_code_implementer_max_retries(),
            discovery_plan_agent_enabled: default_discovery_plan_agent_enabled(),
            discovery_plan_agent_profile: default_discovery_plan_agent_profile(),
            discovery_plan_max_iterations: default_discovery_plan_max_iterations(),
            discovery_plan_max_retries: default_discovery_plan_max_retries(),
            discovery_fixer_min_findings: default_discovery_fixer_min_findings(),
            discovery_fixer_max_iterations: default_discovery_fixer_max_iterations(),
            discovery_fixer_max_retries: default_discovery_fixer_max_retries(),
            discovery_fixer_shell_extra_commands: default_discovery_fixer_shell_extra(),
            spawn_gate: SpawnGateConfig::default(),
        }
    }
}

// ─── Synapse Writer gate ────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct SynapseWriterConfig {
    /// When true, `gzmo chaos skill` requires a matching Pi `skill.invoke` on the bus.
    #[serde(default)]
    pub gate_enabled: bool,

    #[serde(default = "default_synapse_writer_invoke_window")]
    pub invoke_window_secs: u64,

    #[serde(default = "default_synapse_writer_tail_scan")]
    pub tail_scan_lines: usize,
}

fn default_synapse_writer_invoke_window() -> u64 {
    120
}

fn default_synapse_writer_tail_scan() -> usize {
    2000
}

impl Default for SynapseWriterConfig {
    fn default() -> Self {
        Self {
            gate_enabled: false,
            invoke_window_secs: default_synapse_writer_invoke_window(),
            tail_scan_lines: default_synapse_writer_tail_scan(),
        }
    }
}

// ─── Compliance (ARCH-DIR) ───────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ComplianceConfig {
    #[serde(default = "default_compliance_mode")]
    pub mode: String,

    #[serde(default)]
    pub require_obolus_governance: bool,

    #[serde(default)]
    pub allow_cloud_tools: bool,

    #[serde(default)]
    pub allow_cloud_engine: bool,

    #[serde(default = "default_trusted_cidrs")]
    pub trusted_cidrs: Vec<String>,

    #[serde(default = "default_max_binary_mb")]
    pub max_binary_mb: u64,

    #[serde(default = "default_max_workspace_deps")]
    pub max_workspace_deps: usize,

    /// Permanent network exception tier (Tier 2 SND): always allowed outbound even in sovereign mode.
    #[serde(default = "default_network_exceptions")]
    pub network_exceptions: Vec<String>,
}

fn default_network_exceptions() -> Vec<String> {
    vec![
        "web_search".into(),
        "web_read".into(),
        "agent-reach".into(),
        "arxiv".into(),
    ]
}

fn default_compliance_mode() -> String {
    "sovereign".to_string()
}

fn default_trusted_cidrs() -> Vec<String> {
    vec!["127.0.0.0/8".into(), "192.168.31.0/24".into()]
}

fn default_max_binary_mb() -> u64 {
    80
}

fn default_max_workspace_deps() -> usize {
    25
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            mode: default_compliance_mode(),
            require_obolus_governance: true,
            allow_cloud_tools: false,
            allow_cloud_engine: false,
            trusted_cidrs: default_trusted_cidrs(),
            max_binary_mb: default_max_binary_mb(),
            max_workspace_deps: default_max_workspace_deps(),
            network_exceptions: default_network_exceptions(),
        }
    }
}

// ─── Obolus governance (runtime energy bilanz) ───────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ObolusGovernanceConfig {
    #[serde(default = "default_obolus_governance_enabled")]
    pub enabled: bool,

    #[serde(default = "default_max_e_total_per_hour")]
    pub max_e_total_per_hour: u64,

    #[serde(default = "default_max_ctx_pressure_pct")]
    pub max_ctx_pressure_pct: f64,

    #[serde(default = "default_spawn_discovery_fix_reserve_obl")]
    pub spawn_discovery_fix_reserve_obl: u64,

    #[serde(default = "default_spawn_session_triage_reserve_obl")]
    pub spawn_session_triage_reserve_obl: u64,

    #[serde(default = "default_dice_loop_reserve_obl")]
    pub dice_loop_reserve_obl: u64,

    #[serde(default = "default_on_budget_exceeded")]
    pub on_budget_exceeded: String,

    #[serde(default = "default_operator_warn_only")]
    pub operator_warn_only: bool,

    #[serde(default)]
    pub fail_open_if_ledger_unreadable: bool,
}

fn default_obolus_governance_enabled() -> bool {
    true
}

fn default_max_e_total_per_hour() -> u64 {
    2_000_000
}

fn default_max_ctx_pressure_pct() -> f64 {
    400.0
}

fn default_spawn_discovery_fix_reserve_obl() -> u64 {
    50
}

fn default_spawn_session_triage_reserve_obl() -> u64 {
    20
}

fn default_dice_loop_reserve_obl() -> u64 {
    10
}

fn default_on_budget_exceeded() -> String {
    "deny".to_string()
}

fn default_operator_warn_only() -> bool {
    true
}

impl Default for ObolusGovernanceConfig {
    fn default() -> Self {
        Self {
            enabled: default_obolus_governance_enabled(),
            max_e_total_per_hour: default_max_e_total_per_hour(),
            max_ctx_pressure_pct: default_max_ctx_pressure_pct(),
            spawn_discovery_fix_reserve_obl: default_spawn_discovery_fix_reserve_obl(),
            spawn_session_triage_reserve_obl: default_spawn_session_triage_reserve_obl(),
            dice_loop_reserve_obl: default_dice_loop_reserve_obl(),
            on_budget_exceeded: default_on_budget_exceeded(),
            operator_warn_only: default_operator_warn_only(),
            fail_open_if_ledger_unreadable: false,
        }
    }
}

// ─── Obolus analytics (token ledger on Prime) ─────────────────────────────

/// Token consumption ledger for Prime LLM calls. Not the routing table (`[routing]`).
#[derive(Debug, Deserialize, Clone)]
pub struct ObolusAnalyticsConfig {
    #[serde(default = "default_obolus_analytics_enabled")]
    pub enabled: bool,

    #[serde(default = "default_obolus_ledger_path")]
    pub ledger_path: String,

    /// llama-server log file or journalctl wrapper script for external client reconcile.
    #[serde(default)]
    pub llama_log_path: String,

    #[serde(default = "default_obolus_prime_port")]
    pub prime_port: u16,

    #[serde(default = "default_obolus_prime_context_tokens")]
    pub prime_context_tokens: u64,

    #[serde(default = "default_obolus_reconcile_interval_secs")]
    pub reconcile_interval_secs: u64,

    #[serde(default = "default_obolus_tokens_per_obl")]
    pub tokens_per_obl: u64,

    #[serde(default = "default_obolus_rollup_retention_days")]
    pub rollup_retention_days: u32,

    #[serde(default = "default_obolus_writer_batch_size")]
    pub writer_batch_size: usize,

    #[serde(default = "default_obolus_writer_flush_ms")]
    pub writer_flush_ms: u64,

    /// Emit `obolus.efficiency_tick` on reconcile when an hour bucket changes.
    #[serde(default)]
    pub efficiency_tick_enabled: bool,

    /// Sample CPU RAPL + GPU power into `power_ledger_path` on reconcile ticks.
    #[serde(default)]
    pub energy_sampler_enabled: bool,

    #[serde(default = "default_obolus_power_ledger_path")]
    pub power_ledger_path: String,

    #[serde(default = "default_obolus_rapl_energy_path")]
    pub rapl_energy_path: String,

    #[serde(default = "default_obolus_energy_sample_min_interval_secs")]
    pub energy_sample_min_interval_secs: u64,

    #[serde(default = "default_obolus_hsp_state_url")]
    pub hsp_state_url: String,

    #[serde(default = "default_obolus_nvidia_smi_fallback")]
    pub nvidia_smi_fallback: bool,

    #[serde(default = "default_obolus_gpu_joules_integration")]
    pub gpu_joules_integration: bool,
}

fn default_obolus_analytics_enabled() -> bool {
    true
}

fn default_obolus_ledger_path() -> String {
    "data/Obolus/ledger.jsonl".to_string()
}

fn default_obolus_prime_port() -> u16 {
    8000
}

fn default_obolus_prime_context_tokens() -> u64 {
    131_072
}

fn default_obolus_reconcile_interval_secs() -> u64 {
    60
}

fn default_obolus_tokens_per_obl() -> u64 {
    1000
}

fn default_obolus_rollup_retention_days() -> u32 {
    90
}

fn default_obolus_writer_batch_size() -> usize {
    50
}

fn default_obolus_writer_flush_ms() -> u64 {
    200
}

fn default_obolus_power_ledger_path() -> String {
    "data/Obolus/power.jsonl".to_string()
}

fn default_obolus_rapl_energy_path() -> String {
    "/sys/class/powercap/intel-rapl:0/energy_uj".to_string()
}

fn default_obolus_energy_sample_min_interval_secs() -> u64 {
    55
}

fn default_obolus_hsp_state_url() -> String {
    "http://127.0.0.1:8001/state".to_string()
}

fn default_obolus_nvidia_smi_fallback() -> bool {
    true
}

fn default_obolus_gpu_joules_integration() -> bool {
    true
}

impl Default for ObolusAnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: default_obolus_analytics_enabled(),
            ledger_path: default_obolus_ledger_path(),
            llama_log_path: String::new(),
            prime_port: default_obolus_prime_port(),
            prime_context_tokens: default_obolus_prime_context_tokens(),
            reconcile_interval_secs: default_obolus_reconcile_interval_secs(),
            tokens_per_obl: default_obolus_tokens_per_obl(),
            rollup_retention_days: default_obolus_rollup_retention_days(),
            writer_batch_size: default_obolus_writer_batch_size(),
            writer_flush_ms: default_obolus_writer_flush_ms(),
            efficiency_tick_enabled: false,
            energy_sampler_enabled: false,
            power_ledger_path: default_obolus_power_ledger_path(),
            rapl_energy_path: default_obolus_rapl_energy_path(),
            energy_sample_min_interval_secs: default_obolus_energy_sample_min_interval_secs(),
            hsp_state_url: default_obolus_hsp_state_url(),
            nvidia_smi_fallback: default_obolus_nvidia_smi_fallback(),
            gpu_joules_integration: default_obolus_gpu_joules_integration(),
        }
    }
}

// ─── Bibliothek promotion gate ──────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct BibliothekConfig {
    /// Skip vault/KG promotion until this many dream cycles completed.
    #[serde(default = "default_bibliothek_min_dream_cycles")]
    pub min_dream_cycles: u32,
}

fn default_bibliothek_min_dream_cycles() -> u32 {
    50
}

impl Default for BibliothekConfig {
    fn default() -> Self {
        Self {
            min_dream_cycles: default_bibliothek_min_dream_cycles(),
        }
    }
}

// ─── Librarian (VM200 light LLM) ────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct LibrarianConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_librarian_url")]
    pub url: String,

    #[serde(default = "default_librarian_model")]
    pub model: String,

    #[serde(default)]
    pub api_key: String,
}

fn default_librarian_url() -> String {
    "http://192.168.31.110:8083/v1".to_string()
}

fn default_librarian_model() -> String {
    "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf".to_string()
}

impl Default for LibrarianConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: default_librarian_url(),
            model: default_librarian_model(),
            api_key: String::new(),
        }
    }
}

impl LibrarianConfig {
    /// Engine profile for the retired VM200 librarian (`:8083`). Only used if
    /// `[librarian].enabled = true`; steady state distills on Prime instead.
    pub fn to_engine_profile(&self) -> EngineProfileConfig {
        EngineProfileConfig {
            provider: "local".into(),
            url: self.url.clone(),
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            temperature: 0.2,
            top_p: 0.9,
            max_tokens: 4096,
        }
    }
}

// ─── Rerank (VM200 retrieval router :8081, gzmo-rerank preset) ───────────

#[derive(Debug, Deserialize, Clone)]
pub struct RerankConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_rerank_url")]
    pub url: String,

    #[serde(default = "default_rerank_model")]
    pub model: String,

    #[serde(default)]
    pub api_key: String,

    /// Over-fetch decay/BM25 hits before reranking (final limit unchanged).
    #[serde(default = "default_rerank_prefetch_multiplier")]
    pub prefetch_multiplier: usize,
}

fn default_rerank_url() -> String {
    "http://192.168.31.110:8081/v1".to_string()
}

fn default_rerank_model() -> String {
    "gzmo-rerank".to_string()
}

fn default_rerank_prefetch_multiplier() -> usize {
    4
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: default_rerank_url(),
            model: default_rerank_model(),
            api_key: String::new(),
            prefetch_multiplier: default_rerank_prefetch_multiplier(),
        }
    }
}

// ─── Redis scratch + distill queue ───────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    #[serde(default = "default_redis_enabled")]
    pub enabled: bool,

    #[serde(default = "default_redis_url")]
    pub url: String,

    #[serde(default = "default_distill_queue")]
    pub distill_queue: String,

    /// Fallback directory when Redis is down (`data/distill-queue/`).
    #[serde(default = "default_distill_fallback_dir")]
    pub distill_fallback_dir: PathBuf,
}

fn default_redis_enabled() -> bool {
    true
}
fn default_redis_url() -> String {
    "redis://192.168.31.202:6379".to_string()
}
fn default_distill_queue() -> String {
    "gzmo:distill:pending".to_string()
}
fn default_distill_fallback_dir() -> PathBuf {
    PathBuf::from("data/distill-queue")
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            enabled: default_redis_enabled(),
            url: default_redis_url(),
            distill_queue: default_distill_queue(),
            distill_fallback_dir: default_distill_fallback_dir(),
        }
    }
}

// ─── Context memory (archive @ 90%, scratch budget) ─────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ContextMemoryConfig {
    /// Fraction of hot budget that triggers archival (default 0.90).
    #[serde(default = "default_archive_threshold")]
    pub archive_threshold: f64,

    /// Reserve for model response (default 0.10).
    #[serde(default = "default_response_reserve")]
    pub response_reserve: f64,

    /// Max tokens injected from scratch recall per turn.
    #[serde(default = "default_scratch_max_tokens")]
    pub scratch_max_tokens: usize,

    /// Model context length for hot budget; 0 = use 131072.
    #[serde(default)]
    pub context_length: usize,
}

fn default_archive_threshold() -> f64 {
    0.90
}
fn default_response_reserve() -> f64 {
    0.10
}
fn default_scratch_max_tokens() -> usize {
    2000
}

impl Default for ContextMemoryConfig {
    fn default() -> Self {
        Self {
            archive_threshold: default_archive_threshold(),
            response_reserve: default_response_reserve(),
            scratch_max_tokens: default_scratch_max_tokens(),
            context_length: 0,
        }
    }
}

impl ContextMemoryConfig {
    /// Hot token budget: (context_length * (1 - reserve)) * archive_threshold applied in context.rs.
    pub fn hot_budget_tokens(&self) -> usize {
        let ctx = if self.context_length > 0 {
            self.context_length
        } else {
            131_072
        };
        let after_reserve = (ctx as f64 * (1.0 - self.response_reserve)) as usize;
        after_reserve
    }

    pub fn archive_trigger_tokens(&self) -> usize {
        (self.hot_budget_tokens() as f64 * self.archive_threshold) as usize
    }
}

// ─── Context compression (Headroom) ──────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ContextCompressConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_ccr_ttl_secs")]
    pub ccr_ttl_secs: u64,

    #[serde(default = "default_tool_output_max_tokens")]
    pub tool_output_max_tokens: usize,

    #[serde(default = "default_recall_compress_budget")]
    pub recall_compress_budget: usize,

    #[serde(default = "default_json_array_row_cap")]
    pub json_array_row_cap: usize,

    #[serde(default = "default_log_line_cap")]
    pub log_line_cap: usize,
}

fn default_ccr_ttl_secs() -> u64 {
    3600
}

fn default_tool_output_max_tokens() -> usize {
    4000
}

fn default_recall_compress_budget() -> usize {
    2000
}

fn default_json_array_row_cap() -> usize {
    20
}

fn default_log_line_cap() -> usize {
    500
}

impl Default for ContextCompressConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ccr_ttl_secs: default_ccr_ttl_secs(),
            tool_output_max_tokens: default_tool_output_max_tokens(),
            recall_compress_budget: default_recall_compress_budget(),
            json_array_row_cap: default_json_array_row_cap(),
            log_line_cap: default_log_line_cap(),
        }
    }
}

// ─── Subagent runner ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct SubagentConfig {
    #[serde(default = "default_subagent_enabled")]
    pub enabled: bool,

    #[serde(default = "default_subagent_max_concurrent")]
    pub max_concurrent: usize,

    #[serde(default = "default_subagent_max_depth")]
    pub max_depth: u8,

    #[serde(default = "default_subagent_context_budget")]
    pub context_budget_tokens: usize,

    #[serde(default = "default_subagent_summary_max")]
    pub summary_max_tokens: usize,

    /// Extra shell first-token binaries for governed sub-agents (discovery fixer, etc.).
    #[serde(default = "default_subagent_shell_extra")]
    pub shell_extra_commands: Vec<String>,
}

fn default_subagent_shell_extra() -> Vec<String> {
    vec![
        "cd".into(),
        "mkdir".into(),
        "cp".into(),
        "mv".into(),
        "touch".into(),
        "ln".into(),
        "chmod".into(),
        "install".into(),
    ]
}

fn default_subagent_enabled() -> bool {
    true
}
fn default_subagent_max_concurrent() -> usize {
    2
}
fn default_subagent_max_depth() -> u8 {
    2
}
fn default_subagent_context_budget() -> usize {
    32_768
}
fn default_subagent_summary_max() -> usize {
    800
}

impl Default for SubagentConfig {
    fn default() -> Self {
        Self {
            enabled: default_subagent_enabled(),
            max_concurrent: default_subagent_max_concurrent(),
            max_depth: default_subagent_max_depth(),
            context_budget_tokens: default_subagent_context_budget(),
            summary_max_tokens: default_subagent_summary_max(),
            shell_extra_commands: default_subagent_shell_extra(),
        }
    }
}

// ─── API Keys ───────────────────────────────────────────────────────────

/// Centralized API key store. Env vars take precedence over config values.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ApiKeysConfig {
    #[serde(default)]
    pub serpapi: String,
    #[serde(default)]
    pub openrouter: String,
    #[serde(default)]
    pub gemini: String,
}

impl ApiKeysConfig {
    /// Resolve a key with env-var override: GZMO_<NAME>_KEY > toml value > empty
    pub fn serpapi_key(&self) -> String {
        std::env::var("GZMO_SERPAPI_KEY").unwrap_or_else(|_| self.serpapi.clone())
    }
    pub fn openrouter_key(&self) -> String {
        std::env::var("GZMO_OPENROUTER_KEY").unwrap_or_else(|_| self.openrouter.clone())
    }
    pub fn gemini_key(&self) -> String {
        std::env::var("GZMO_GEMINI_KEY").unwrap_or_else(|_| self.gemini.clone())
    }
}

// ─── Engine Section ─────────────────────────────────────────────────────

/// The `[engine]` section supports two layouts:
/// 1. **New (dual-profile)**: `[engine] active_mode` + `[engine.local]` + `[engine.cloud]`
/// 2. **Legacy (flat)**: `[engine] provider`, `url`, `model`, etc. — treated as local profile
#[derive(Debug, Deserialize, Clone)]
pub struct EngineSection {
    /// Which profile is active: "local" or "cloud"
    #[serde(default)]
    pub active_mode: EngineMode,

    /// New-style local profile
    #[serde(default)]
    pub local: Option<EngineProfileConfig>,

    /// New-style cloud profile (with optional fallback fields)
    #[serde(default)]
    pub cloud: Option<CloudEngineConfig>,

    /// Sovereign FrankenMoE (`llama-server` :8010) — optional until GGUF exists.
    #[serde(default)]
    pub sovereign: Option<EngineProfileConfig>,

    // ── Legacy flat fields (backward compat) ────────────────────────
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

impl Default for EngineSection {
    fn default() -> Self {
        Self {
            active_mode: EngineMode::Local,
            local: None,
            cloud: None,
            sovereign: None,
            provider: None,
            url: None,
            model: None,
            api_key: None,
            temperature: None,
            top_p: None,
            max_tokens: None,
        }
    }
}

impl EngineSection {
    /// Resolve the active engine config based on `active_mode`.
    /// Falls back to legacy flat fields if no profiles are defined.
    pub fn active_engine(&self) -> EngineProfileConfig {
        match self.active_mode {
            EngineMode::Local => {
                if let Some(ref local) = self.local {
                    local.clone()
                } else {
                    // Legacy: build from flat fields
                    EngineProfileConfig {
                        provider: self.provider.clone().unwrap_or_else(default_provider),
                        url: self.url.clone().unwrap_or_else(default_engine_url),
                        model: self.model.clone().unwrap_or_else(default_model_name),
                        api_key: self.api_key.clone().unwrap_or_default(),
                        temperature: self.temperature.unwrap_or_else(default_temperature),
                        top_p: self.top_p.unwrap_or_else(default_top_p),
                        max_tokens: self.max_tokens.unwrap_or_else(default_max_tokens),
                    }
                }
            }
            EngineMode::Cloud => {
                if let Some(ref cloud) = self.cloud {
                    EngineProfileConfig {
                        provider: cloud.provider.clone(),
                        url: cloud.url.clone(),
                        model: cloud.model.clone(),
                        api_key: cloud.api_key.clone(),
                        temperature: cloud.temperature,
                        top_p: cloud.top_p,
                        max_tokens: cloud.max_tokens,
                    }
                } else {
                    // No cloud profile — fall back to local
                    tracing::warn!("Cloud mode requested but no [engine.cloud] defined — falling back to local");
                    self.active_engine_for_mode(EngineMode::Local)
                }
            }
            EngineMode::Sovereign => {
                if let Some(ref sovereign) = self.sovereign {
                    sovereign.clone()
                } else {
                    tracing::warn!("Sovereign mode requested but no [engine.sovereign] — falling back to local");
                    self.active_engine_for_mode(EngineMode::Local)
                }
            }
        }
    }

    /// Get engine config for a specific mode (without changing active_mode).
    pub fn active_engine_for_mode(&self, mode: EngineMode) -> EngineProfileConfig {
        match mode {
            EngineMode::Local => {
                self.local.clone().unwrap_or_else(|| EngineProfileConfig {
                    provider: self.provider.clone().unwrap_or_else(default_provider),
                    url: self.url.clone().unwrap_or_else(default_engine_url),
                    model: self.model.clone().unwrap_or_else(default_model_name),
                    api_key: self.api_key.clone().unwrap_or_default(),
                    temperature: self.temperature.unwrap_or_else(default_temperature),
                    top_p: self.top_p.unwrap_or_else(default_top_p),
                    max_tokens: self.max_tokens.unwrap_or_else(default_max_tokens),
                })
            }
            EngineMode::Cloud => {
                if let Some(ref cloud) = self.cloud {
                    EngineProfileConfig {
                        provider: cloud.provider.clone(),
                        url: cloud.url.clone(),
                        model: cloud.model.clone(),
                        api_key: cloud.api_key.clone(),
                        temperature: cloud.temperature,
                        top_p: cloud.top_p,
                        max_tokens: cloud.max_tokens,
                    }
                } else {
                    EngineProfileConfig::default()
                }
            }
            EngineMode::Sovereign => self
                .sovereign
                .clone()
                .unwrap_or_else(EngineProfileConfig::default),
        }
    }

    /// Get the cloud fallback config when fully configured (non-empty url,
    /// model, and api_key). Returns `None` otherwise so callers do not add a
    /// doomed fallback hop with missing credentials.
    pub fn cloud_fallback(&self) -> Option<EngineProfileConfig> {
        self.cloud.as_ref().and_then(|c| {
            let url = c.fallback_url.as_ref().filter(|s| !s.is_empty())?;
            let model = c.fallback_model.as_ref().filter(|s| !s.is_empty())?;
            let key = c.fallback_api_key.as_ref().filter(|s| !s.is_empty())?;
            Some(EngineProfileConfig {
                provider: c.fallback_provider.clone().unwrap_or_else(|| "gemini".to_string()),
                url: url.clone(),
                model: model.clone(),
                api_key: key.clone(),
                temperature: c.temperature,
                top_p: c.top_p,
                max_tokens: c.max_tokens,
            })
        })
    }
}

/// A single engine profile (used for both local and cloud).
#[derive(Debug, Deserialize, Clone)]
pub struct EngineProfileConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_engine_url")]
    pub url: String,
    #[serde(default = "default_model_name")]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

impl Default for EngineProfileConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            url: default_engine_url(),
            model: default_model_name(),
            api_key: String::new(),
            temperature: default_temperature(),
            top_p: default_top_p(),
            max_tokens: default_max_tokens(),
        }
    }
}

/// Cloud engine profile with fallback fields.
#[derive(Debug, Deserialize, Clone)]
pub struct CloudEngineConfig {
    #[serde(default = "default_cloud_provider")]
    pub provider: String,
    #[serde(default = "default_cloud_url")]
    pub url: String,
    #[serde(default = "default_cloud_model")]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    // Fallback engine (activated if primary cloud endpoint fails)
    #[serde(default)]
    pub fallback_provider: Option<String>,
    #[serde(default)]
    pub fallback_url: Option<String>,
    #[serde(default)]
    pub fallback_model: Option<String>,
    #[serde(default)]
    pub fallback_api_key: Option<String>,
}

// ─── Obolus Routing Config ──────────────────────────────────────────────

/// Static task → engine routing table (Obolus, the Economy Organ).
///
/// Maps each `TaskKind` to a named engine profile. The named profiles
/// are resolved by `GatewayRouter` into actual `Arc<dyn LlmGateway>`
/// instances pointing at the configured endpoint.
///
/// Example gzmo.toml:
/// ```toml
/// [routing]
/// default_engine = "local"
///
/// [routing.mappings]
/// # Steady state: all cognition on Prime (:8000).
/// dream_extract = "local"
/// distill_extract = "local"
/// distill_summary = "local"
/// spark_hypothesis = "local"
/// ```
#[derive(Debug, Deserialize, Clone, Default)]
pub struct RoutingConfig {
    /// Default engine name when no explicit mapping exists for a task kind.
    #[serde(default = "default_routing_engine")]
    pub default_engine: String,

    /// When true, every background `TaskKind` (all except `Chat`) is routed
    /// cloud-first: the cloud profile is tried first and the task's legacy
    /// profile (from `mappings`) is used as automatic fallback. Interactive
    /// chat is unaffected.
    #[serde(default)]
    pub cloud_first_background: bool,

    /// Task-kind → engine-name mappings. Keys are snake_case task kind names;
    /// values are engine profile names ("local", "librarian", "cloud", "sovereign").
    #[serde(default)]
    pub mappings: HashMap<String, String>,

    /// Inline engine profile overrides. Keys are profile names;
    /// values are full `EngineProfileConfig` structs.
    /// Used for non-standard profiles like "librarian".
    #[serde(default)]
    pub profiles: HashMap<String, EngineProfileConfig>,
}

fn default_routing_engine() -> String {
    "local".to_string()
}

impl RoutingConfig {
    /// Resolve the engine name for a given task kind.
    /// Falls back to `default_engine` when no mapping exists.
    pub fn resolve(&self, task: TaskKind) -> &str {
        let key = task.to_string();
        self.mappings
            .get(&key)
            .map(|s| s.as_str())
            .unwrap_or_else(|| &self.default_engine)
    }

    /// Get a named engine profile. Returns `None` if the profile is not
    /// defined inline — the caller should fall back to the standard engine
    /// sections (`engine.local`, `engine.cloud`, etc.).
    pub fn get_profile(&self, name: &str) -> Option<&EngineProfileConfig> {
        self.profiles.get(name)
    }

    /// Resolve a full `EngineProfileConfig` for a task kind.
    /// Checks inline profiles first, then falls back to standard engine sections.
    pub fn resolve_profile(&self, task: TaskKind, engine: &EngineSection) -> EngineProfileConfig {
        let profile_name = self.resolve(task);

        // Check inline profiles first
        if let Some(inline) = self.get_profile(profile_name) {
            return inline.clone();
        }

        // Fall back to standard engine sections by name
        match profile_name {
            "local" => engine.active_engine(),
            "cloud" => {
                if let Some(ref cloud) = engine.cloud {
                    EngineProfileConfig {
                        provider: cloud.provider.clone(),
                        url: cloud.url.clone(),
                        model: cloud.model.clone(),
                        api_key: cloud.api_key.clone(),
                        temperature: cloud.temperature,
                        top_p: cloud.top_p,
                        max_tokens: cloud.max_tokens,
                    }
                } else {
                    tracing::warn!("Routing to 'cloud' but no [engine.cloud] — falling back to local");
                    engine.active_engine()
                }
            }
            "sovereign" => {
                engine
                    .sovereign
                    .clone()
                    .unwrap_or_else(|| {
                        tracing::warn!("Routing to 'sovereign' but no [engine.sovereign] — falling back to local");
                        engine.active_engine()
                    })
            }
            name => {
                tracing::warn!(
                    profile = name,
                    "Unknown routing profile — falling back to active engine"
                );
                engine.active_engine()
            }
        }
    }
}

// ─── Remaining config structs (unchanged) ───────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct IdentityConfig {
    /// Path to the SOUL.md file
    #[serde(default = "default_soul_path")]
    pub soul_path: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MemoryConfig {
    /// Directory for episodic memory logs (YYYY-MM-DD.md)
    #[serde(default = "default_memory_dir")]
    pub directory: PathBuf,

    /// Path to the SQLite vault database
    #[serde(default = "default_vault_db")]
    pub vault_db: PathBuf,

    /// `sqlite` (production) or `qdrant` (not implemented — fails fast at connect).
    #[serde(default = "default_vault_backend")]
    pub vault_backend: String,
}

fn default_vault_backend() -> String {
    "sqlite".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct SkillsConfig {
    /// Directory containing .skillsrc skill definitions
    #[serde(default = "default_skills_dir")]
    pub directory: PathBuf,

    /// Path to the dreams configuration
    #[serde(default = "default_dreams_path")]
    pub dreams_path: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    /// Max tool-call iterations before forcing a text response
    #[serde(default = "default_max_iterations")]
    pub max_tool_iterations: usize,

    /// Heartbeat interval in seconds (for daemon mode)
    #[serde(default = "default_heartbeat_secs")]
    pub heartbeat_interval_secs: u64,
}

/// A scheduled background job declared in gzmo.toml.
///
/// Supports two modes:
/// - **Simple**: just `prompt` — fires a single headless agent loop.
/// - **Pipeline**: `steps` — a multi-step cognitive pipeline with dependency-aware
///   wave execution, per-step tool limits, and result forwarding.
#[derive(Debug, Deserialize, Clone)]
pub struct JobConfig {
    /// Cron expression (6-field: sec min hour day month weekday)
    pub cron: String,

    /// The prompt to send in simple mode (ignored if `steps` is non-empty)
    #[serde(default)]
    pub prompt: String,

    /// Multi-step pipeline. If non-empty, `prompt` is ignored and each step
    /// runs as a separate headless agent loop with results flowing downstream.
    #[serde(default)]
    pub steps: Vec<JobStep>,

    /// If true, this job is disabled (not scheduled on startup)
    #[serde(default)]
    pub disabled: bool,

    /// Maximum retry attempts on failure (0 = no retry)
    #[serde(default)]
    pub max_retries: u32,

    /// If true, store the final result in the semantic vault for long-term recall
    #[serde(default)]
    pub persist_results: bool,
}

/// A single step within a multi-step job pipeline.
#[derive(Debug, Deserialize, Clone)]
pub struct JobStep {
    /// Human-readable step name (must be unique within the job)
    pub name: String,

    /// The prompt for this step. Prior step results are injected as system context.
    pub prompt: String,

    /// Names of steps this depends on. The step won't run until all dependencies complete.
    /// Steps with no dependencies run in the first wave (parallel).
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Max tool-call iterations for this step (default: 5)
    #[serde(default = "default_step_iterations")]
    pub max_iterations: usize,
}

/// A directory watcher declared in gzmo.toml
#[derive(Debug, Deserialize, Clone)]
pub struct WatcherConfig {
    /// The directory to watch
    pub directory: String,

    /// Glob pattern for matching files (e.g., "*.csv", "invoice_*.pdf")
    #[serde(default)]
    pub pattern: Option<String>,

    /// The prompt to send when a file matches. Supports `{file_path}` template.
    pub prompt: String,

    /// If true, this watcher is disabled
    #[serde(default)]
    pub disabled: bool,

    /// Wait this many seconds after the last write event before ingest (coalesce saves).
    #[serde(default = "default_watcher_debounce_secs")]
    pub debounce_secs: u64,
}

fn default_watcher_debounce_secs() -> u64 {
    3
}

/// Orchestration configuration for background autonomous tasks
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OrchestrationConfig {
    /// Named background jobs
    #[serde(default)]
    pub jobs: HashMap<String, JobConfig>,

    /// Named directory watchers
    #[serde(default)]
    pub watchers: HashMap<String, WatcherConfig>,
}

/// An MCP server declared in gzmo.toml
#[derive(Debug, Deserialize, Clone)]
pub struct McpServerEntry {
    /// Human-readable name (used as tool name prefix: mcp__{name}__{tool})
    pub name: String,

    /// The executable to run
    pub command: String,

    /// Arguments to pass
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables for the child process
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// If true, this server is disabled (not connected on startup)
    #[serde(default)]
    pub disabled: bool,
}

// ─── Defaults ───────────────────────────────────────────────────────────

fn default_soul_path() -> PathBuf { PathBuf::from("SOUL.md") }
fn default_memory_dir() -> PathBuf { PathBuf::from("memory") }
fn default_vault_db() -> PathBuf { PathBuf::from("data/vault.db") }
fn default_skills_dir() -> PathBuf { PathBuf::from("skills") }
fn default_dreams_path() -> PathBuf { PathBuf::from("DREAMS.md") }
fn default_provider() -> String { "local".to_string() }
/// Resolve `gzmo.toml` for CLI invocations from any cwd (e.g. Pi started in `~`).
fn discover_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("GZMO_CONFIG") {
        return PathBuf::from(path);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_config = cwd.join("gzmo.toml");
    if cwd_config.exists() {
        return cwd_config;
    }

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe;
        dir.pop(); // strip binary name
        for _ in 0..8 {
            let candidate = dir.join("gzmo.toml");
            if candidate.exists() {
                return candidate;
            }
            if !dir.pop() {
                break;
            }
        }
    }

    cwd_config
}

fn default_engine_url() -> String { "http://localhost:8000/v1".to_string() }
fn default_model_name() -> String { "qwen3.6-27b".to_string() }
fn default_temperature() -> f32 { 0.3 }
fn default_top_p() -> f32 { 0.95 }
fn default_max_tokens() -> u32 { 8192 }
fn default_max_iterations() -> usize { 10 }
fn default_heartbeat_secs() -> u64 { 1800 }
fn default_step_iterations() -> usize { 5 }
fn default_dream_enabled() -> bool { true }
fn default_dream_verify() -> bool { true }
fn default_dream_min_confidence() -> f64 { 0.85 }
fn default_dream_verify_temperature() -> f32 { 0.1 }
fn default_dream_cron_hour() -> u32 { 1 }
fn default_dream_cron_minute() -> u32 { 0 }
fn default_ingest_enabled() -> bool { true }
fn default_kg_require_evidence() -> bool { true }
fn default_kg_strict() -> bool { true }
fn default_spark_enabled() -> bool { true }
fn default_spark_hypothesis_temperature() -> f32 { 0.2 }
fn default_spark_candidate_limit() -> usize { 5 }
fn default_spark_recent_limit() -> usize { 2 }
fn default_spark_quarantine_confidence() -> f64 { 0.6 }
fn default_spark_cron_hours() -> Vec<u32> { vec![9, 14, 21] }
fn default_spark_cron_minute() -> u32 { 17 }

fn default_cloud_provider() -> String { "openrouter".to_string() }
fn default_cloud_url() -> String { "https://openrouter.ai/api/v1".to_string() }
fn default_cloud_model() -> String { "openrouter/free".to_string() }

impl Default for IdentityConfig {
    fn default() -> Self {
        Self { soul_path: default_soul_path() }
    }
}
impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            directory: default_memory_dir(),
            vault_db: default_vault_db(),
            vault_backend: default_vault_backend(),
        }
    }
}
impl Default for SkillsConfig {
    fn default() -> Self {
        Self { directory: default_skills_dir(), dreams_path: default_dreams_path() }
    }
}
impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_tool_iterations: default_max_iterations(),
            heartbeat_interval_secs: default_heartbeat_secs(),
        }
    }
}


// ─── Loader ─────────────────────────────────────────────────────────────

/// Parse `.env` from `base_dir`. Does not mutate process environment.
fn read_dotenv(base_dir: &Path) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    let path = base_dir.join(".env");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return vars;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim().trim_matches('"').trim_matches('\'');
        if !key.is_empty() {
            vars.insert(key.to_string(), val.to_string());
        }
    }
    vars
}

fn env_or_dotenv(key: &str, dotenv: &HashMap<String, String>) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| dotenv.get(key).cloned().filter(|v| !v.is_empty()))
}

/// Inject cloud profile API keys from env / `[api_keys]` when `engine.cloud.api_key` is empty.
fn apply_engine_key_overrides(config: &mut GzmoConfig, dotenv: &HashMap<String, String>) {
    let Some(ref mut cloud) = config.engine.cloud else {
        return;
    };
    if !cloud.api_key.is_empty() {
        return;
    }
    let key = match cloud.provider.as_str() {
        "openrouter" => env_or_dotenv("GZMO_OPENROUTER_KEY", dotenv)
            .filter(|k| !k.is_empty())
            .or_else(|| {
                let k = config.api_keys.openrouter_key();
                if k.is_empty() { None } else { Some(k) }
            }),
        "gemini" => env_or_dotenv("GZMO_GEMINI_KEY", dotenv)
            .filter(|k| !k.is_empty())
            .or_else(|| {
                let k = config.api_keys.gemini_key();
                if k.is_empty() { None } else { Some(k) }
            }),
        _ => None,
    };
    if let Some(key) = key {
        cloud.api_key = key;
    }

    // Inject the cloud->cloud fallback key (e.g. Gemini) when configured only in
    // the environment, so `cloud_fallback()` can activate without storing the
    // secret in gzmo.toml.
    if cloud
        .fallback_api_key
        .as_ref()
        .map(|k| k.is_empty())
        .unwrap_or(true)
    {
        let fb_provider = cloud.fallback_provider.as_deref().unwrap_or("gemini");
        let fb_key = match fb_provider {
            "gemini" => env_or_dotenv("GZMO_GEMINI_KEY", dotenv).filter(|k| !k.is_empty()),
            "openrouter" => {
                env_or_dotenv("GZMO_OPENROUTER_KEY", dotenv).filter(|k| !k.is_empty())
            }
            _ => None,
        };
        if let Some(fb_key) = fb_key {
            cloud.fallback_api_key = Some(fb_key);
        }
    }
}

/// Overlay MCP child-process env from process environment or `.env`.
fn apply_mcp_env_overrides(config: &mut GzmoConfig, dotenv: &HashMap<String, String>) {
    const KEYS: &[&str] = &[
        "NEO4J_URL",
        "NEO4J_USERNAME",
        "NEO4J_PASSWORD",
        "NEO4J_DATABASE",
    ];
    for server in &mut config.mcp_servers {
        for key in KEYS {
            if let Some(val) = env_or_dotenv(key, dotenv) {
                server.env.insert(key.to_string(), val);
            }
        }
    }
}

impl GzmoConfig {
    /// Load configuration from a TOML file.
    ///
    /// If the file doesn't exist, returns the default configuration
    /// (all hardcoded defaults — zero-config startup).
    pub fn load(path: &Path) -> Result<Self> {
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let dotenv = read_dotenv(base_dir);
        let resolve = |p: &PathBuf| -> PathBuf {
            if p.is_absolute() { p.clone() } else { base_dir.join(p) }
        };

        if !path.exists() {
            tracing::info!(
                path = %path.display(),
                "No gzmo.toml found — using defaults"
            );
            let mut cfg = Self::default();
            cfg.identity.soul_path = resolve(&cfg.identity.soul_path);
            cfg.memory.directory = resolve(&cfg.memory.directory);
            cfg.memory.vault_db = resolve(&cfg.memory.vault_db);
            cfg.skills.directory = resolve(&cfg.skills.directory);
            cfg.skills.dreams_path = resolve(&cfg.skills.dreams_path);
            return Ok(cfg);
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let mut config: GzmoConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        // Anchor all relative paths to the location of the config file
        config.identity.soul_path = resolve(&config.identity.soul_path);
        config.memory.directory = resolve(&config.memory.directory);
        config.memory.vault_db = resolve(&config.memory.vault_db);
        config.skills.directory = resolve(&config.skills.directory);
        config.skills.dreams_path = resolve(&config.skills.dreams_path);
        apply_mcp_env_overrides(&mut config, &dotenv);
        apply_engine_key_overrides(&mut config, &dotenv);

        let active = config.engine.active_engine();
        tracing::info!(
            path = %path.display(),
            mcp_servers = config.mcp_servers.len(),
            mode = %config.engine.active_mode,
            engine_url = %active.url,
            model = %active.model,
            "Loaded gzmo.toml"
        );

        Ok(config)
    }

    /// Load from `gzmo.toml` via `GZMO_CONFIG`, cwd, or by walking up from the executable
    /// (e.g. `.../survey_GZMO/target/release/gzmo` → repo-root `gzmo.toml`).
    pub fn load_auto() -> Result<Self> {
        let path = discover_config_path();
        Self::load(&path)
    }

    /// Get active (non-disabled) MCP server entries.
    pub fn active_mcp_servers(&self) -> impl Iterator<Item = &McpServerEntry> {
        self.mcp_servers.iter().filter(|s| !s.disabled)
    }

    /// Persist the active_mode to gzmo.toml on disk.
    /// Uses a regex replacement to update the field without clobbering the rest of the file.
    pub fn persist_active_mode(&self, config_path: &Path, mode: EngineMode) -> Result<()> {
        let content = std::fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read {} for mode persistence", config_path.display()))?;

        // Replace the active_mode line under [engine]
        let updated = if content.contains("active_mode") {
            content
                .lines()
                .map(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with("active_mode") && trimmed.contains('=') {
                        format!("active_mode = \"{}\"", mode)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            // No active_mode line found — add it after [engine]
            content.replace("[engine]", &format!("[engine]\nactive_mode = \"{}\"", mode))
        };

        std::fs::write(config_path, updated)
            .with_context(|| format!("Failed to write {}", config_path.display()))?;

        tracing::info!(mode = %mode, "Persisted active_mode to gzmo.toml");
        Ok(())
    }
}
