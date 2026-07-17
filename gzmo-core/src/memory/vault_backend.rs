//! # Vault backend abstraction (DRAFT / SCAFFOLD)
//!
//! `SqliteVault` is production; this trait is the seam for a future `QdrantVault`
//! on the sidecar. The `SqliteVault` adapter below delegates to inherent methods.
//!
//! ## Deferred: full Qdrant backend
//! `SqliteVault` bakes decay, reinforcement, and the `<0.85` quarantine barrier
//! into SQL. Qdrant would need those re-expressed as payload fields + client rescore.
//! Callers still use `SqliteVault` directly; switch to `Arc<dyn VaultBackend>` when
//! Qdrant bodies are implemented (see `todo!()` on `QdrantVault`).

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::types::{ExtractedTruth, SemanticFact};

/// A scored fact returned by a search (fact + decayed relevance score).
pub type ScoredFact = (SemanticFact, f64);

/// The behaviour every vault backend must provide. Mirrors the inherent
/// `SqliteVault` API so swapping the backend is a type change, not a rewrite.
///
/// Async because a remote Qdrant backend does network I/O; the SQLite impl just
/// wraps its synchronous, pooled calls (optionally via `spawn_blocking`).
#[async_trait]
pub trait VaultBackend: Send + Sync {
    /// Store a fact with an embedding. Confidence `< 0.85` must be quarantined.
    async fn store(&self, fact: &SemanticFact) -> Result<()>;

    /// Store a plain-text fact (no embedding) under a decay class.
    async fn store_text(&self, content: &str, decay_class: &str, confidence: f64) -> Result<()>;

    /// Hybrid vector + keyword search with temporal decay applied.
    async fn search_with_decay(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<ScoredFact>>;

    /// Keyword-only search (no embedding needed).
    async fn keyword_search(&self, query_text: &str, limit: usize) -> Result<Vec<ScoredFact>>;

    /// Promote dream-extracted truths; corroborate duplicates by content.
    async fn promote_truths(&self, truths: &[ExtractedTruth]) -> Result<()>;

    /// Reinforce a fact: bump confirmation count + reset its decay clock.
    async fn reinforce(&self, fact_id: Uuid) -> Result<()>;

    /// Metacognitive guard: recall past failures matching a description.
    async fn recall_failures(&self, description: &str) -> Result<Vec<String>>;

    /// Most-recent N fact contents (for context injection).
    async fn recent(&self, limit: usize) -> Result<Vec<String>>;

    /// Total fact count.
    async fn count(&self) -> Result<usize>;

    // ── Candidate selection for the chaos-free "spark" job ──────────────────
    // Surfacing an OLD / under-connected fact is the one behaviour the nightly
    // dream lacks. Expose it here so spark can query it from any backend.
    /// Return up to `limit` facts that are stale (oldest `last_accessed_at`) yet
    /// not fully decayed — i.e. worth revisiting. Qdrant: scroll ordered by
    /// `last_accessed_at`; SQLite: `ORDER BY last_accessed_at ASC`.
    async fn stale_candidates(&self, limit: usize) -> Result<Vec<SemanticFact>>;
}

// ───────────────────────────────────────────────────────────────────────────
// Qdrant backend — SKELETON. Bodies are intentionally `todo!()`.
// ───────────────────────────────────────────────────────────────────────────

/// Shared semantic vault backed by Qdrant on the sidecar (LXC101 :6333).
///
/// Collection layout (proposed):
///   - vectors: the fact embedding (cosine distance)
///   - payload: { content, half_life_days, confidence, confirmation_count,
///                decay_class, created_at, last_accessed_at }
/// Quarantine = a second collection (or `confidence < 0.85` payload filter).
pub struct QdrantVault {
    // client: qdrant_client::Qdrant,
    // collection: String,
    // quarantine_collection: String,
}

impl QdrantVault {
    fn not_implemented<T>(method: &str) -> Result<T> {
        anyhow::bail!(
            "QdrantVault::{method} is not implemented — use [memory] vault_backend = \"sqlite\""
        )
    }

    /// Connect to Qdrant and ensure the collections/indexes exist.
    /// e.g. `QdrantVault::connect("http://192.168.31.202:6333", "gzmo_vault").await`
    pub async fn connect(_url: &str, _collection: &str) -> Result<Self> {
        anyhow::bail!(
            "QdrantVault is not implemented — set [memory] vault_backend = \"sqlite\" in gzmo.toml"
        )
    }

    /// Re-implements SqliteVault's decay so Qdrant's raw similarity becomes the
    /// same decayed score the rest of the daemon expects.
    ///   decayed = raw * 0.5^( max(days_since_access - confirmations*5, 0) / half_life )
    fn _apply_decay(_raw_score: f64, _fact: &SemanticFact) -> f64 {
        unimplemented!("QdrantVault::_apply_decay — use sqlite backend")
    }
}

#[async_trait]
impl VaultBackend for QdrantVault {
    async fn store(&self, _fact: &SemanticFact) -> Result<()> {
        Self::not_implemented("store")
    }
    async fn store_text(&self, _content: &str, _decay_class: &str, _confidence: f64) -> Result<()> {
        Self::not_implemented("store_text")
    }
    async fn search_with_decay(
        &self,
        _q_emb: &[f32],
        _q_text: &str,
        _limit: usize,
    ) -> Result<Vec<ScoredFact>> {
        Self::not_implemented("search_with_decay")
    }
    async fn keyword_search(&self, _q_text: &str, _limit: usize) -> Result<Vec<ScoredFact>> {
        Self::not_implemented("keyword_search")
    }
    async fn promote_truths(&self, _truths: &[ExtractedTruth]) -> Result<()> {
        Self::not_implemented("promote_truths")
    }
    async fn reinforce(&self, _fact_id: Uuid) -> Result<()> {
        Self::not_implemented("reinforce")
    }
    async fn recall_failures(&self, _description: &str) -> Result<Vec<String>> {
        Self::not_implemented("recall_failures")
    }
    async fn recent(&self, _limit: usize) -> Result<Vec<String>> {
        Self::not_implemented("recent")
    }
    async fn count(&self) -> Result<usize> {
        Self::not_implemented("count")
    }
    async fn stale_candidates(&self, _limit: usize) -> Result<Vec<SemanticFact>> {
        Self::not_implemented("stale_candidates")
    }
}

// ───────────────────────────────────────────────────────────────────────────
// SqliteVault adapter — makes the existing local vault satisfy the trait.
// (Thin delegation to the inherent methods already in vault.rs.)
// ───────────────────────────────────────────────────────────────────────────

use crate::memory::vault::SqliteVault;

#[async_trait]
impl VaultBackend for SqliteVault {
    async fn store(&self, fact: &SemanticFact) -> Result<()> {
        SqliteVault::store(self, fact)
    }
    async fn store_text(&self, content: &str, decay_class: &str, confidence: f64) -> Result<()> {
        SqliteVault::store_text(self, content, decay_class, confidence)
    }
    async fn search_with_decay(
        &self,
        q_emb: &[f32],
        q_text: &str,
        limit: usize,
    ) -> Result<Vec<ScoredFact>> {
        if self.rerank_enabled() {
            SqliteVault::search_with_decay_reranked(self, q_emb, q_text, limit).await
        } else {
            SqliteVault::search_with_decay(self, q_emb, q_text, limit)
        }
    }
    async fn keyword_search(&self, q_text: &str, limit: usize) -> Result<Vec<ScoredFact>> {
        SqliteVault::keyword_search(self, q_text, limit)
    }
    async fn promote_truths(&self, truths: &[ExtractedTruth]) -> Result<()> {
        SqliteVault::promote_truths(self, truths).await
    }
    async fn reinforce(&self, fact_id: Uuid) -> Result<()> {
        SqliteVault::reinforce(self, fact_id)
    }
    async fn recall_failures(&self, description: &str) -> Result<Vec<String>> {
        SqliteVault::recall_failures(self, description)
    }
    async fn recent(&self, limit: usize) -> Result<Vec<String>> {
        SqliteVault::recent(self, limit)
    }
    async fn count(&self) -> Result<usize> {
        SqliteVault::count(self)
    }
    async fn stale_candidates(&self, limit: usize) -> Result<Vec<SemanticFact>> {
        SqliteVault::stale_candidates(self, limit)
    }
}
