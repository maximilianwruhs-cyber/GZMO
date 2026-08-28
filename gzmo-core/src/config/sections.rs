use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::defaults::*;

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

/// Owner socket for CLI/MCP attach. Empty `socket_path` → sibling of `vault_db`.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ControlPlaneConfig {
    #[serde(default)]
    pub socket_path: PathBuf,
}

impl ControlPlaneConfig {
    pub fn resolved_socket(&self, vault_db: &Path) -> PathBuf {
        if self.socket_path.as_os_str().is_empty() {
            vault_db
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("gzmo.sock")
        } else {
            self.socket_path.clone()
        }
    }
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

/// Workflow skill pack (`SKILL.md` engineering contracts).
#[derive(Debug, Deserialize, Clone)]
pub struct WorkflowSkillsConfig {
    #[serde(default = "default_workflow_skills_enabled")]
    pub enabled: bool,

    #[serde(default = "default_workflow_skills_dir")]
    pub dir: PathBuf,

    #[serde(default = "default_workflow_model_can_activate")]
    pub model_can_activate: bool,

    #[serde(default = "default_workflow_max_active")]
    pub max_active: usize,

    /// Where `/handoff` artifacts are written.
    #[serde(default = "default_workflow_handoff_dir")]
    pub handoff_dir: PathBuf,

    /// When true, handoff writes also store a one-line vault pointer.
    #[serde(default = "default_workflow_handoff_to_vault")]
    pub handoff_to_vault: bool,
}

impl Default for WorkflowSkillsConfig {
    fn default() -> Self {
        Self {
            enabled: default_workflow_skills_enabled(),
            dir: default_workflow_skills_dir(),
            model_can_activate: default_workflow_model_can_activate(),
            max_active: default_workflow_max_active(),
            handoff_dir: default_workflow_handoff_dir(),
            handoff_to_vault: default_workflow_handoff_to_vault(),
        }
    }
}

/// Interactive / subagent tool policy.
#[derive(Debug, Deserialize, Clone)]
pub struct ToolsConfig {
    /// Default profile for chat / `--repl`: read_only | developer | reviewer | operator
    #[serde(default = "default_tools_profile")]
    pub profile: String,

    /// Workspace roots for path jail (empty = cwd only).
    #[serde(default)]
    pub workspace_roots: Vec<PathBuf>,

    /// Emit structured audit log lines on every tool dispatch.
    #[serde(default = "default_tools_audit")]
    pub audit: bool,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            profile: default_tools_profile(),
            workspace_roots: Vec::new(),
            audit: default_tools_audit(),
        }
    }
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

    /// Max tool-call iterations for this step (default: 20)
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

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            soul_path: default_soul_path(),
        }
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
        Self {
            directory: default_skills_dir(),
            dreams_path: default_dreams_path(),
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
