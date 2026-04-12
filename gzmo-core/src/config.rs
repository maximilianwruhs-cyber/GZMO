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
//! url = "http://localhost:1234/v1"
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
use serde::Deserialize;

// ─── Engine Mode ────────────────────────────────────────────────────────

/// The active engine mode — local or cloud.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineMode {
    Local,
    Cloud,
}

impl Default for EngineMode {
    fn default() -> Self { Self::Local }
}

impl std::fmt::Display for EngineMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Cloud => write!(f, "cloud"),
        }
    }
}

impl std::str::FromStr for EngineMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "cloud" => Ok(Self::Cloud),
            other => anyhow::bail!("Unknown engine mode: '{}'. Use 'local' or 'cloud'.", other),
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

    /// Chaos engine configuration (Lorenz attractor, Thought Cabinet physics)
    #[serde(default)]
    pub chaos: Option<toml::Value>,
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
        }
    }

    /// Get the cloud fallback config (if defined).
    pub fn cloud_fallback(&self) -> Option<EngineProfileConfig> {
        self.cloud.as_ref().and_then(|c| {
            let url = c.fallback_url.as_ref()?;
            let model = c.fallback_model.as_ref()?;
            let key = c.fallback_api_key.as_ref()?;
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
fn default_model_name() -> String { "gemma-4-E4B-it-Q4_K_M.gguf".to_string() }
fn default_temperature() -> f32 { 0.3 }
fn default_top_p() -> f32 { 0.95 }
fn default_max_tokens() -> u32 { 8192 }
fn default_max_iterations() -> usize { 10 }
fn default_heartbeat_secs() -> u64 { 1800 }
fn default_step_iterations() -> usize { 5 }

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
        Self { directory: default_memory_dir(), vault_db: default_vault_db() }
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
