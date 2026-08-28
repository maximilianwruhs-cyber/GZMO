//! SQLite-backed semantic vault with temporal decay.
//!
//! Implements hybrid search blending cosine similarity on stored embeddings
//! with keyword matching and exponential half-life decay.

use crate::memory::embeddings::Embedder;
use crate::memory::qdrant_recall::QdrantRecall;
use crate::memory::rerank::Reranker;
use crate::types::DecayClass;
use anyhow::{Context, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

mod cognition;
mod dedup;
mod embed;
mod failure_case;
mod fts;
mod inspect;
mod promote;
mod search;
mod spark_pool;
mod store;
mod text;

#[cfg(test)]
mod utility_recall_tests;

pub(crate) use embed::decode_embed;
pub use embed::embedding_cosine_similarity;
pub use text::normalize_truth_content;

/// Cap on Memento failure-case retrieve (never dump the table into a prompt).
pub const FAILURE_CASE_RECALL_LIMIT: usize = 3;

/// A verify/gate refusal recalled beside honeypot hits — not a promoted fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureCaseHit {
    pub kind: String,
    pub content: String,
    pub related_fact_id: Option<String>,
}

/// Result of `SqliteVault::backfill_missing_embeddings`.
#[derive(Debug, Clone, Copy)]
pub struct EmbedBackfillReport {
    pub attempted: usize,
    pub updated: usize,
    pub failed: usize,
}

/// Result of `SqliteVault::promote_mature_to_honeypot`.
#[derive(Debug, Clone, Copy, Default)]
pub struct PromoteMatureReport {
    pub candidates: usize,
    pub promoted: usize,
    pub skipped: usize,
}

fn parse_decay_class(s: &str) -> DecayClass {
    match s {
        "CuratedVault" | "curated_vault" => DecayClass::CuratedVault,
        "SessionDistill" | "session_distill" => DecayClass::SessionDistill,
        "FlexibleIdentity" | "flexible_identity" => DecayClass::FlexibleIdentity,
        "AbsoluteIdentity" | "absolute_identity" => DecayClass::AbsoluteIdentity,
        "Structural" | "structural" => DecayClass::Structural,
        _ => DecayClass::Episodic,
    }
}

/// The permanent semantic vault backed by SQLite.
#[derive(Clone)]
pub struct SqliteVault {
    pool: Pool<SqliteConnectionManager>,
    db_path: std::path::PathBuf,
    embedder: Option<Arc<Embedder>>,
    reranker: Option<Arc<Reranker>>,
    qdrant: Option<Arc<QdrantRecall>>,
}

impl SqliteVault {
    pub(crate) fn db_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().map_err(Into::into)
    }

    /// Open or create the vault database.
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
        let init_conn = Connection::open(db_path.as_ref())
            .with_context(|| "Failed to open semantic vault database")?;

        // Initialize schema
        init_conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS semantic_vault (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB,
                half_life_days REAL NOT NULL DEFAULT 30.0,
                confidence REAL NOT NULL DEFAULT 1.0,
                confirmation_count INTEGER NOT NULL DEFAULT 0,
                decay_class TEXT NOT NULL DEFAULT 'Episodic',
                created_at TEXT NOT NULL,
                last_accessed_at TEXT NOT NULL,
                source_file TEXT,
                content_norm TEXT
            );

            CREATE TABLE IF NOT EXISTS quarantine_vault (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB,
                half_life_days REAL NOT NULL DEFAULT 30.0,
                confidence REAL NOT NULL DEFAULT 1.0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS memory_index (
                id TEXT PRIMARY KEY,
                summary TEXT NOT NULL,
                fact_id TEXT NOT NULL,
                FOREIGN KEY (fact_id) REFERENCES semantic_vault(id)
            );

            CREATE INDEX IF NOT EXISTS idx_vault_decay
                ON semantic_vault(last_accessed_at, half_life_days);",
        )?;

        // Enable WAL mode for concurrent reader safety during background writes
        init_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;

        // Non-destructive schema migration — using PRAGMA user_version
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 1 {
            match init_conn.execute(
                "ALTER TABLE semantic_vault ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0",
                [],
            ) {
                Ok(_) => info!("Applied schema migration: added confidence column"),
                Err(e) if e.to_string().contains("duplicate column") => { /* already exists */ }
                Err(e) => {
                    tracing::error!(error = %e, "Schema migration failed unexpectedly");
                    return Err(e.into());
                }
            }
            init_conn.execute_batch("PRAGMA user_version = 1")?;
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 2 {
            for sql in [
                "ALTER TABLE semantic_vault ADD COLUMN source_file TEXT",
                "ALTER TABLE semantic_vault ADD COLUMN content_norm TEXT",
            ] {
                match init_conn.execute(sql, []) {
                    Ok(_) => info!(migration = sql, "Applied schema migration v2"),
                    Err(e) if e.to_string().contains("duplicate column") => {}
                    Err(e) => {
                        tracing::error!(error = %e, migration = sql, "Schema migration v2 failed");
                        return Err(e.into());
                    }
                }
            }
            init_conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_vault_content_norm ON semantic_vault(content_norm);
                 UPDATE semantic_vault SET content_norm = lower(content) WHERE content_norm IS NULL;",
            )?;
            init_conn.execute_batch("PRAGMA user_version = 2")?;
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 3 {
            init_conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS honeypot (
                    id TEXT PRIMARY KEY,
                    vault_id TEXT NOT NULL,
                    content TEXT NOT NULL,
                    content_norm TEXT NOT NULL,
                    embedding BLOB,
                    origin TEXT NOT NULL DEFAULT 'ingest',
                    memory_type TEXT NOT NULL DEFAULT 'fact',
                    graph_rel TEXT,
                    supersedes_id TEXT,
                    is_latest INTEGER NOT NULL DEFAULT 1,
                    verify_pass INTEGER NOT NULL DEFAULT 1,
                    confidence REAL NOT NULL,
                    decay_class TEXT NOT NULL DEFAULT 'Semantic',
                    source_file TEXT,
                    container_tag TEXT NOT NULL DEFAULT 'obolus',
                    promoted_at TEXT NOT NULL,
                    last_recalled_at TEXT,
                    recall_count INTEGER NOT NULL DEFAULT 0,
                    FOREIGN KEY (vault_id) REFERENCES semantic_vault(id)
                );
                CREATE INDEX IF NOT EXISTS idx_honeypot_latest ON honeypot(is_latest, container_tag);
                CREATE INDEX IF NOT EXISTS idx_honeypot_norm ON honeypot(content_norm);
                CREATE INDEX IF NOT EXISTS idx_honeypot_source ON honeypot(source_file);
                CREATE VIRTUAL TABLE IF NOT EXISTS honeypot_fts USING fts5(
                    content, content_norm, tokenize='porter'
                );",
            )?;
            init_conn.execute_batch("PRAGMA user_version = 3")?;
            info!("Applied schema migration v3: honeypot + honeypot_fts");
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 4 {
            for trig in ["trg_honeypot_ai", "trg_honeypot_ad", "trg_honeypot_au"] {
                let _ = init_conn.execute_batch(&format!("DROP TRIGGER IF EXISTS {trig};"));
            }
            init_conn.execute_batch("PRAGMA user_version = 4")?;
            info!("Applied schema migration v4: drop broken honeypot FTS triggers");
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 5 {
            init_conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS evidence (
                    id TEXT PRIMARY KEY,
                    fact_id TEXT NOT NULL,
                    source_file TEXT,
                    evidence_text TEXT NOT NULL,
                    evidence_norm TEXT NOT NULL,
                    char_start INTEGER,
                    char_end INTEGER,
                    quote_verifier TEXT,
                    embedding BLOB,
                    verify_pass INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (fact_id) REFERENCES honeypot(id)
                );
                CREATE INDEX IF NOT EXISTS idx_evidence_fact ON evidence(fact_id);
                CREATE INDEX IF NOT EXISTS idx_evidence_source ON evidence(source_file);
                CREATE VIRTUAL TABLE IF NOT EXISTS evidence_fts USING fts5(
                    evidence_text, evidence_norm, tokenize='porter'
                );",
            )?;
            init_conn.execute_batch("PRAGMA user_version = 5")?;
            info!("Applied schema migration v5: evidence + evidence_fts");
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 6 {
            init_conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS distill_dedup (
                    dedup_key TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    source TEXT NOT NULL,
                    distilled_at TEXT NOT NULL,
                    truths_count INTEGER NOT NULL DEFAULT 0
                );",
            )?;
            init_conn.execute_batch("PRAGMA user_version = 6")?;
            info!("Applied schema migration v6: distill_dedup");
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 7 {
            init_conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS ingest_dedup (
                    content_hash TEXT PRIMARY KEY,
                    source_path TEXT NOT NULL,
                    ingested_at TEXT NOT NULL
                );",
            )?;
            init_conn.execute_batch("PRAGMA user_version = 7")?;
            info!("Applied schema migration v7: ingest_dedup");
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 8 {
            // MemRL-inspired utility for two-phase retrieval (semantic then value).
            match init_conn.execute(
                "ALTER TABLE honeypot ADD COLUMN utility_score REAL NOT NULL DEFAULT 0.0",
                [],
            ) {
                Ok(_) => info!("Applied schema migration v8: honeypot.utility_score"),
                Err(e) if e.to_string().contains("duplicate column") => {}
                Err(e) => {
                    tracing::error!(error = %e, "Schema migration v8 failed");
                    return Err(e.into());
                }
            }
            // Seed utility from existing Felt Use so ripen/search aren't cold-start zero.
            let _ = init_conn.execute_batch(
                "UPDATE honeypot
                 SET utility_score = CAST(recall_count AS REAL)
                 WHERE utility_score = 0.0 AND recall_count > 0;",
            );
            init_conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_honeypot_utility
                 ON honeypot(is_latest, utility_score DESC);",
            )?;
            init_conn.execute_batch("PRAGMA user_version = 8")?;
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 9 {
            // Repair: some living DBs reached user_version=8 without utility_score.
            let has_utility: bool = {
                let mut stmt = init_conn.prepare("PRAGMA table_info(honeypot)")?;
                let cols: Vec<String> = stmt
                    .query_map([], |row| row.get::<_, String>(1))?
                    .filter_map(|r| r.ok())
                    .collect();
                cols.iter().any(|c| c == "utility_score")
            };
            if !has_utility {
                init_conn.execute(
                    "ALTER TABLE honeypot ADD COLUMN utility_score REAL NOT NULL DEFAULT 0.0",
                    [],
                )?;
                let _ = init_conn.execute_batch(
                    "UPDATE honeypot
                     SET utility_score = CAST(recall_count AS REAL)
                     WHERE utility_score = 0.0 AND recall_count > 0;",
                );
                init_conn.execute_batch(
                    "CREATE INDEX IF NOT EXISTS idx_honeypot_utility
                     ON honeypot(is_latest, utility_score DESC);",
                )?;
                info!("Applied schema migration v9: repair honeypot.utility_score");
            } else {
                info!("Schema migration v9: utility_score already present");
            }
            init_conn.execute_batch("PRAGMA user_version = 9")?;
        }
        let user_version: u32 = init_conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version < 10 {
            for sql in [
                "ALTER TABLE honeypot ADD COLUMN valid_from TEXT",
                "ALTER TABLE honeypot ADD COLUMN valid_to TEXT",
                "ALTER TABLE honeypot ADD COLUMN gate_event TEXT NOT NULL DEFAULT 'promote'",
            ] {
                match init_conn.execute(sql, []) {
                    Ok(_) => info!(migration = sql, "Applied schema migration v10"),
                    Err(e) if e.to_string().contains("duplicate column") => {}
                    Err(e) => {
                        tracing::error!(error = %e, migration = sql, "Schema migration v10 failed");
                        return Err(e.into());
                    }
                }
            }
            let _ = init_conn.execute_batch(
                "UPDATE honeypot SET valid_from = COALESCE(valid_from, promoted_at);
                 UPDATE honeypot SET valid_to = promoted_at WHERE is_latest = 0 AND valid_to IS NULL;
                 CREATE TABLE IF NOT EXISTS failure_cases (
                    id TEXT PRIMARY KEY,
                    kind TEXT NOT NULL,
                    content TEXT NOT NULL,
                    related_fact_id TEXT,
                    created_at TEXT NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_honeypot_valid
                    ON honeypot(valid_from, valid_to);",
            );
            init_conn.execute_batch("PRAGMA user_version = 10")?;
            info!("Applied schema migration v10: bi-temporal + failure_cases + gate_event");
        }

        info!("Semantic vault initialized (WAL mode + r2d2 pool)");

        let path = db_path.as_ref().to_owned();
        let manager = SqliteConnectionManager::file(&path)
            .with_init(|c| c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;"));
        let pool = r2d2::Pool::builder()
            .max_size(5) // Enough for daemon and main loop
            .build(manager)
            .with_context(|| "Failed to create connection pool")?;

        Ok(Self {
            pool,
            db_path: path,
            embedder: None,
            reranker: None,
            qdrant: None,
        })
    }

    /// Attach an embedding client for vector storage on promote.
    pub fn with_embedder(mut self, embedder: Option<Arc<Embedder>>) -> Self {
        self.embedder = embedder;
        self
    }

    /// Attach Qdrant honeypot collection for RRF vector stream.
    pub fn with_qdrant(mut self, qdrant: Option<Arc<QdrantRecall>>) -> Self {
        self.qdrant = qdrant;
        self
    }

    /// Attach a rerank client for vault recall post-filtering.
    pub fn with_reranker(mut self, reranker: Option<Arc<Reranker>>) -> Self {
        self.reranker = reranker;
        self
    }

    pub fn rerank_enabled(&self) -> bool {
        self.reranker.is_some()
    }

    /// Path to the vault SQLite file (artifact dirs are siblings under parent).
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Parent of the vault DB (`/opt/gzmo/data`, `data-next`, `~/.gzmo`, …).
    pub fn data_dir(&self) -> &Path {
        self.db_path.parent().unwrap_or_else(|| Path::new("."))
    }

    pub(crate) fn half_life_from_decay_class(decay_class: &str) -> f64 {
        match decay_class {
            "Episodic" => 30.0,
            "CuratedVault" | "SessionDistill" => 60.0,
            "FlexibleIdentity" => 139.0,
            "AbsoluteIdentity" => 693.0,
            "Structural" | "Core" => 36500.0,
            "Semantic" => 365.0,
            "Procedural" => 90.0,
            _ => 60.0,
        }
    }

    /// Get the total number of facts in the vault.
    pub fn count(&self) -> Result<usize> {
        let conn = self.pool.get()?;
        let count: usize =
            conn.query_row("SELECT COUNT(*) FROM semantic_vault", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Curated honeypot rows with `is_latest=1` (RAG mirror source of truth).
    pub fn count_honeypot_latest(&self) -> Result<usize> {
        let conn = self.pool.get()?;
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='honeypot'",
            [],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Ok(0);
        }
        let count: usize = conn.query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get the most recent N facts (for context injection).
    pub fn recent(&self, limit: usize) -> Result<Vec<String>> {
        Ok(self
            .recent_semantic_facts(limit)?
            .into_iter()
            .map(|f| f.content)
            .collect())
    }
}
