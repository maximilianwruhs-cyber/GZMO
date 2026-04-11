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
//! [engine]
//! url = "http://localhost:1234/v1"
//! model = "qwen2.5-7b-instruct.Q3_K_M.gguf"
//! temperature = 0.3
//! top_p = 0.95
//! max_tokens = 4096
//!
//! [agent]
//! max_tool_iterations = 10
//! heartbeat_interval_secs = 1800
//!
//! [[mcp_servers]]
//! name = "filesystem"
//! command = "npx"
//! args = ["-y", "@modelcontextprotocol/server-filesystem", "/home/user"]
//!
//! [[mcp_servers]]
//! name = "browser"
//! command = "npx"
//! args = ["-y", "@anthropic/mcp-server-puppeteer"]
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

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
    pub engine: EngineConfig,

    #[serde(default)]
    pub agent: AgentConfig,

    /// MCP server declarations — each is spawned as a child process on startup
    #[serde(default)]
    pub mcp_servers: Vec<McpServerEntry>,

    /// Background orchestration: scheduled cron jobs
    #[serde(default)]
    pub orchestration: OrchestrationConfig,
}

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
pub struct EngineConfig {
    /// Provider type: "local", "openai", "anthropic", "gemini"
    /// "local" works with any OpenAI-compatible endpoint (llama.cpp, Ollama, LM Studio, vLLM)
    /// "openai" is identical to "local" but with auth (works with OpenAI, Groq, Together, etc.)
    #[serde(default = "default_provider")]
    pub provider: String,

    /// API endpoint URL
    #[serde(default = "default_engine_url")]
    pub url: String,

    /// Model name or identifier
    #[serde(default = "default_model_name")]
    pub model: String,

    /// API key (required for cloud providers, empty for local)
    #[serde(default)]
    pub api_key: String,

    /// Sampling temperature (lower = more deterministic)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Top-p nucleus sampling
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// Maximum tokens to generate per response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
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
fn default_engine_url() -> String { "http://localhost:1234/v1".to_string() }
fn default_model_name() -> String { "qwen2.5-7b-instruct.Q3_K_M.gguf".to_string() }
fn default_temperature() -> f32 { 0.3 }
fn default_top_p() -> f32 { 0.95 }
fn default_max_tokens() -> u32 { 4096 }
fn default_max_iterations() -> usize { 10 }
fn default_heartbeat_secs() -> u64 { 1800 }
fn default_step_iterations() -> usize { 5 }

impl Default for IdentityConfig {
    fn default() -> Self {
        Self { soul_path: default_soul_path() }
    }
}
impl Default for MemoryConfig {
    fn default() -> Self {
        Self { directory: default_memory_dir(), vault_db: default_vault_db() }
    }
}
impl Default for SkillsConfig {
    fn default() -> Self {
        Self { directory: default_skills_dir(), dreams_path: default_dreams_path() }
    }
}
impl Default for EngineConfig {
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
impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_tool_iterations: default_max_iterations(),
            heartbeat_interval_secs: default_heartbeat_secs(),
        }
    }
}


// ─── Loader ─────────────────────────────────────────────────────────────

impl GzmoConfig {
    /// Load configuration from a TOML file.
    ///
    /// If the file doesn't exist, returns the default configuration
    /// (all hardcoded defaults — zero-config startup).
    pub fn load(path: &Path) -> Result<Self> {
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
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

        tracing::info!(
            path = %path.display(),
            mcp_servers = config.mcp_servers.len(),
            engine_url = %config.engine.url,
            model = %config.engine.model,
            "Loaded gzmo.toml"
        );

        Ok(config)
    }

    /// Load from `gzmo.toml` anchored to the executable directory, 
    /// or an explicit path via the `GZMO_CONFIG` environment variable.
    pub fn load_auto() -> Result<Self> {
        let path = std::env::var("GZMO_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let cwd_path = std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("gzmo.toml");
                if cwd_path.exists() {
                    return cwd_path;
                }
                
                // Portable mode fallback: anchor to the physical location of the executable
                let mut exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
                exe.pop(); // Remove executable name
                exe.join("gzmo.toml")
            });

        Self::load(&path)
    }

    /// Get active (non-disabled) MCP server entries.
    pub fn active_mcp_servers(&self) -> impl Iterator<Item = &McpServerEntry> {
        self.mcp_servers.iter().filter(|s| !s.disabled)
    }
}
