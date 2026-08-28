use std::path::PathBuf;

use serde::Deserialize;

use super::defaults::*;
use super::engine::EngineProfileConfig;

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
}

impl Default for PlatformSearchConfig {
    fn default() -> Self {
        Self {
            include_knowledge_collection: default_platform_search_enabled(),
            knowledge_collection: default_knowledge_collection(),
            knowledge_prefetch: default_knowledge_prefetch(),
        }
    }
}

// ─── KG reconcile (shared Neo4j ontology) ───────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct KgReconcileConfig {
    #[serde(default)]
    pub enabled: bool,

    /// UTC hour for daily reconcile on GZMO-next (default 04:30).
    #[serde(default = "default_kg_reconcile_hour")]
    pub cron_hour: u32,

    #[serde(default = "default_kg_reconcile_minute")]
    pub cron_minute: u32,

    /// Dry-run: log planned changes without MCP writes.
    #[serde(default = "default_true")]
    pub dry_run: bool,
}

impl Default for KgReconcileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cron_hour: default_kg_reconcile_hour(),
            cron_minute: default_kg_reconcile_minute(),
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
}

impl Default for SynapsePullConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cron_hour: default_synapse_pull_hour(),
            cron_minute: default_synapse_pull_minute(),
            max_events: default_synapse_pull_max_events(),
            bus_path: default_synapse_bus_path(),
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
    /// Engine profile for structured extract / short summaries on VM200 :8083.
    pub fn to_engine_profile(&self) -> EngineProfileConfig {
        EngineProfileConfig {
            provider: "local".into(),
            url: self.url.clone(),
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            temperature: 0.2,
            top_p: 0.9,
            max_tokens: 4096,
            reasoning_effort: None,
            seed: None,
        }
    }
}

// ─── Rerank (VM200 bge-reranker) ────────────────────────────────────────

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
