use serde::Deserialize;

use super::defaults::*;

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
}

impl Default for SubagentConfig {
    fn default() -> Self {
        Self {
            enabled: default_subagent_enabled(),
            max_concurrent: default_subagent_max_concurrent(),
            max_depth: default_subagent_max_depth(),
            context_budget_tokens: default_subagent_context_budget(),
            summary_max_tokens: default_subagent_summary_max(),
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
