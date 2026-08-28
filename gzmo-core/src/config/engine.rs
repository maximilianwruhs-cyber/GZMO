use std::collections::HashMap;

use serde::Deserialize;

use super::defaults::*;
use super::task_kind::{EngineMode, TaskKind};

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
                        reasoning_effort: None,
                        seed: None,
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
                        reasoning_effort: cloud.reasoning_effort.clone(),
                        seed: None,
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
            EngineMode::Local => self.local.clone().unwrap_or_else(|| EngineProfileConfig {
                provider: self.provider.clone().unwrap_or_else(default_provider),
                url: self.url.clone().unwrap_or_else(default_engine_url),
                model: self.model.clone().unwrap_or_else(default_model_name),
                api_key: self.api_key.clone().unwrap_or_default(),
                temperature: self.temperature.unwrap_or_else(default_temperature),
                top_p: self.top_p.unwrap_or_else(default_top_p),
                max_tokens: self.max_tokens.unwrap_or_else(default_max_tokens),
                reasoning_effort: None,
                seed: None,
            }),
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
                        reasoning_effort: cloud.reasoning_effort.clone(),
                        seed: None,
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
                provider: c
                    .fallback_provider
                    .clone()
                    .unwrap_or_else(|| "gemini".to_string()),
                url: url.clone(),
                model: model.clone(),
                api_key: key.clone(),
                temperature: c.temperature,
                top_p: c.top_p,
                max_tokens: c.max_tokens,
                reasoning_effort: None,
                seed: None,
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
    /// OpenRouter reasoning effort: minimal | low | medium | high | xhigh
    #[serde(default)]
    pub reasoning_effort: Option<String>,
    /// Optional sampler seed for reproducible sampling (effective only with
    /// temp > 0). Set on a routing profile (e.g. local_deterministic) to make
    /// eval runs replayable.
    #[serde(default)]
    pub seed: Option<u64>,
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
            reasoning_effort: None,
            seed: None,
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
    /// OpenRouter reasoning effort: minimal | low | medium | high | xhigh
    #[serde(default)]
    pub reasoning_effort: Option<String>,

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
/// dream_extract = "librarian"
/// distill_extract = "librarian"
/// distill_summary = "librarian"
/// spark_hypothesis = "librarian"
///
/// [routing.profiles.librarian]
/// provider = "local"
/// url = "http://192.168.31.110:8083/v1"
/// model = "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"
/// temperature = 0.2
/// top_p = 0.9
/// max_tokens = 4096
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
                        reasoning_effort: cloud.reasoning_effort.clone(),
                        seed: None,
                    }
                } else {
                    tracing::warn!(
                        "Routing to 'cloud' but no [engine.cloud] — falling back to local"
                    );
                    engine.active_engine()
                }
            }
            "sovereign" => engine.sovereign.clone().unwrap_or_else(|| {
                tracing::warn!(
                    "Routing to 'sovereign' but no [engine.sovereign] — falling back to local"
                );
                engine.active_engine()
            }),
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
