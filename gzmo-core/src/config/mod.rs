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
//! max_tool_iterations = 40
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

mod defaults;
mod dice;
mod distill;
mod dreams;
mod engine;
mod ingest;
mod metabolism;
mod pedagogy;
mod runtime;
mod sections;
mod services;
mod spark;
mod task_kind;
mod wiki;

pub use dice::*;
pub use distill::*;
pub use dreams::*;
pub use engine::*;
pub use ingest::*;
pub use metabolism::*;
pub use pedagogy::*;
pub use runtime::*;
pub use sections::*;
pub use services::*;
pub use spark::*;
pub use task_kind::*;
pub use wiki::*;

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

    /// Progressive-disclosure engineering workflow skills (`skills/workflows/*/SKILL.md`).
    #[serde(default)]
    pub workflow_skills: WorkflowSkillsConfig,

    /// Tool capability profiles + workspace jail.
    #[serde(default)]
    pub tools: ToolsConfig,

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

    /// Overnight metabolism cron slots for `gzmo serve` (ADR-0003).
    #[serde(default)]
    pub metabolism: MetabolismConfig,

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

    /// VM200 fast LLM for summaries (optional; `:8083` when deployed).
    #[serde(default)]
    pub librarian: LibrarianConfig,

    /// VM200 cross-encoder reranker (`:8082`); post-filters vault recall.
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

    /// Per-loop backend routing (inline engine vs Little Tools Lab recipe).
    /// Only honored when `GZMO_INSTANCE=next`; defaults to all-Inline (CT101-safe).
    #[serde(default)]
    pub assembly: crate::assembly::AssemblyConfig,

    /// Operator custom cron jobs (`gzmo cron`) executed by `gzmo serve`.
    #[serde(default)]
    pub cron: CronConfig,

    /// Agentic Teacher / Unix mentor API (`gzmo daemon` socket + discovery teach).
    #[serde(default)]
    pub pedagogy: PedagogyConfig,

    /// `/dice` skill — cascade (+ optional loop, default off).
    #[serde(default)]
    pub dice: DiceConfig,

    /// Unix-socket owner plane (`gzmo serve` / `gzmo daemon`). Default socket is
    /// `{vault_db.parent()}/gzmo.sock`.
    #[serde(default)]
    pub control_plane: ControlPlaneConfig,
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
                if k.is_empty() {
                    None
                } else {
                    Some(k)
                }
            }),
        "gemini" => env_or_dotenv("GZMO_GEMINI_KEY", dotenv)
            .filter(|k| !k.is_empty())
            .or_else(|| {
                let k = config.api_keys.gemini_key();
                if k.is_empty() {
                    None
                } else {
                    Some(k)
                }
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
            "openrouter" => env_or_dotenv("GZMO_OPENROUTER_KEY", dotenv).filter(|k| !k.is_empty()),
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
            if p.is_absolute() {
                p.clone()
            } else {
                base_dir.join(p)
            }
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
            cfg.workflow_skills.dir = resolve(&cfg.workflow_skills.dir);
            cfg.workflow_skills.handoff_dir = resolve(&cfg.workflow_skills.handoff_dir);
            if !cfg.control_plane.socket_path.as_os_str().is_empty() {
                cfg.control_plane.socket_path = resolve(&cfg.control_plane.socket_path);
            }
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
        config.session_distill.sessions_dir = resolve(&config.session_distill.sessions_dir);
        config.redis.distill_fallback_dir = resolve(&config.redis.distill_fallback_dir);
        config.workflow_skills.dir = resolve(&config.workflow_skills.dir);
        config.workflow_skills.handoff_dir = resolve(&config.workflow_skills.handoff_dir);
        if !config.control_plane.socket_path.as_os_str().is_empty() {
            config.control_plane.socket_path = resolve(&config.control_plane.socket_path);
        }
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

    /// Load config.
    ///
    /// Order: `GZMO_CONFIG` → `~/.gzmo/gzmo.toml` (product) → `config/gzmo.toml` →
    /// `config/gzmo-next.toml` → cwd/`gzmo.toml` → exe-dir variants.
    pub fn load_auto() -> Result<Self> {
        let path = if let Ok(p) = std::env::var("GZMO_CONFIG") {
            PathBuf::from(p)
        } else {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let mut exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
            exe.pop();
            let product =
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".gzmo").join("gzmo.toml"));
            let mut candidates: Vec<PathBuf> = Vec::new();
            if let Some(p) = product {
                candidates.push(p);
            }
            candidates.extend([
                cwd.join("config/gzmo.toml"),
                cwd.join("config/gzmo-next.toml"),
                exe.join("config/gzmo.toml"),
                exe.join("config/gzmo-next.toml"),
                cwd.join("gzmo.toml"),
                exe.join("gzmo.toml"),
            ]);
            candidates
                .into_iter()
                .find(|p| p.exists())
                .unwrap_or_else(|| cwd.join("gzmo.toml"))
        };

        Self::load(&path)
    }

    /// Get active (non-disabled) MCP server entries.
    pub fn active_mcp_servers(&self) -> impl Iterator<Item = &McpServerEntry> {
        self.mcp_servers.iter().filter(|s| !s.disabled)
    }

    /// Persist the active_mode to gzmo.toml on disk.
    /// Uses a regex replacement to update the field without clobbering the rest of the file.
    pub fn persist_active_mode(&self, config_path: &Path, mode: EngineMode) -> Result<()> {
        let content = std::fs::read_to_string(config_path).with_context(|| {
            format!(
                "Failed to read {} for mode persistence",
                config_path.display()
            )
        })?;

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

#[cfg(test)]
mod assembly_config_tests {
    use super::*;
    use crate::assembly::AssemblyBackend;

    #[test]
    fn parses_assembly_section() {
        let toml = r#"
            [assembly]
            distill = "lab"
            dream = "lab"
            spark = "inline"
        "#;
        let cfg: GzmoConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.assembly.distill, AssemblyBackend::Lab);
        assert_eq!(cfg.assembly.dream, AssemblyBackend::Lab);
        assert_eq!(cfg.assembly.spark, AssemblyBackend::Inline);
        // Unlisted loops default Inline
        assert_eq!(cfg.assembly.ops_health, AssemblyBackend::Inline);
    }

    #[test]
    fn absent_assembly_defaults_all_inline() {
        let cfg: GzmoConfig = toml::from_str("").unwrap();
        assert!(!cfg.assembly.distill.is_lab());
        assert!(!cfg.assembly.config_handoff.is_lab());
    }

    #[test]
    fn gzmo_next_toml_parses() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../config/gzmo-next.toml");
        let cfg = GzmoConfig::load(&path).unwrap();
        assert!(cfg.assembly.distill.is_lab());
        assert!(cfg.assembly.dream.is_lab());
        assert!(cfg.assembly.spark.is_lab());
        assert!(cfg.assembly.ops_health.is_lab());
        assert!(cfg.assembly.config_handoff.is_lab());
        assert!(cfg.memory.vault_db.ends_with("data-next/vault.db"));
    }
}
