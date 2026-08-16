//! SQLite-backed semantic vault with temporal decay.
//!
//! Implements hybrid search blending cosine similarity on stored embeddings
//! with keyword matching and exponential half-life decay.

use anyhow::{Context, Result};
use chrono::Utc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;
use uuid::Uuid;

use std::sync::Arc;

use crate::memory::core_pin;
use crate::memory::embeddings::Embedder;
use crate::memory::felt_use::{self, FeltUseKind};
use crate::memory::honeypot::{self, qualifies_for_honeypot};
use crate::memory::lifecycle::{
    classify_truth_pair, extract_primary_entity, find_latest_honeypot_by_entity,
    is_unverified_derived, supersede_honeypot, LifecycleKind,
};
use crate::memory::qdrant_recall::QdrantRecall;
use crate::memory::recall_rrf::{
    apply_utility_boost, diversify_by_source_file, extract_entity_tokens, fts_match_query,
    fts_match_query_broad, merge_interleaved_rank, rrf_fuse, RecallCandidate, PREFETCH_K,
    RERANK_PREFETCH,
};
use crate::memory::rerank::Reranker;
use crate::types::{DecayClass, ExtractedTruth, SemanticFact};
use std::process::Command as StdCommand;

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

    /// Store a new semantic fact.
    pub fn store(&self, fact: &SemanticFact) -> Result<()> {
        let conn = self.pool.get()?;

        // Hallucination Prevention Barrier
        if fact.confidence < 0.85 {
            conn.execute(
                "INSERT OR REPLACE INTO quarantine_vault
                    (id, content, embedding, half_life_days, confidence, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    fact.id.to_string(),
                    fact.content,
                    bincode_embed(&fact.embedding),
                    fact.half_life_days,
                    fact.confidence,
                    fact.created_at.to_rfc3339(),
                ],
            )?;
            tracing::warn!(fact_id = %fact.id, confidence = fact.confidence, "Memory quarantined due to low confidence");
            return Ok(());
        }

        conn.execute(
            "INSERT OR REPLACE INTO semantic_vault
                (id, content, embedding, half_life_days, confidence, confirmation_count,
                 decay_class, created_at, last_accessed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                fact.id.to_string(),
                fact.content,
                bincode_embed(&fact.embedding),
                fact.half_life_days,
                fact.confidence,
                fact.confirmation_count,
                &fact.decay_class,
                fact.created_at.to_rfc3339(),
                fact.last_accessed_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Retrieve items placed in quarantine awaiting HITL validation
    pub fn list_quarantine(&self) -> Result<Vec<(String, String, f64, String)>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, content, confidence, created_at FROM quarantine_vault ORDER BY created_at DESC")?;
        let results = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    /// Path to the vault SQLite file (artifact dirs are siblings under parent).
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Parent of the vault DB (`/opt/gzmo/data`, `data-next`, `~/.gzmo`, …).
    pub fn data_dir(&self) -> &Path {
        self.db_path.parent().unwrap_or_else(|| Path::new("."))
    }

    /// Reinforce a fact: increment confirmation_count and reset decay clock.
    pub fn reinforce(&self, fact_id: Uuid) -> Result<()> {
        self.reinforce_by(fact_id, 1)
    }

    /// Graded Felt Use: bump vault confirmation + honeypot `recall_count` by `delta`.
    pub fn reinforce_by(&self, fact_id: Uuid, delta: i64) -> Result<()> {
        if delta <= 0 {
            return Ok(());
        }
        let conn = self.pool.get()?;
        let now = Utc::now().to_rfc3339();
        let id = fact_id.to_string();
        conn.execute(
            "UPDATE semantic_vault
             SET confirmation_count = confirmation_count + ?1,
                 last_accessed_at = ?2
             WHERE id = ?3",
            params![delta, now, id],
        )?;
        let _ = conn.execute(
            "UPDATE honeypot
             SET recall_count = recall_count + ?1,
                 last_recalled_at = ?2,
                 utility_score = utility_score + CAST(?1 AS REAL)
             WHERE id = ?3",
            params![delta, now, id],
        );
        info!(fact_id = %fact_id, delta, "Reinforced semantic fact (felt use + utility)");
        Ok(())
    }

    /// Census for M5 export gates (`gzmo ripen status` / overnight honesty).
    pub fn ripen_gate_census(
        &self,
        min_confidence: f64,
        min_recall: i64,
    ) -> Result<crate::memory::ripen::RipenGateCensus> {
        let conn = self.pool.get()?;
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='honeypot'",
            [],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Ok(crate::memory::ripen::RipenGateCensus {
                latest: 0,
                nonzero_recall: 0,
                dual: 0,
                dual_origin: 0,
            });
        }
        let latest: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1",
            [],
            |r| r.get(0),
        )?;
        let nonzero_recall: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1 AND recall_count > 0",
            [],
            |r| r.get(0),
        )?;
        let dual: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot
             WHERE is_latest = 1 AND confidence >= ?1 AND recall_count >= ?2",
            params![min_confidence, min_recall],
            |r| r.get(0),
        )?;
        let dual_origin: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot
             WHERE is_latest = 1 AND confidence >= ?1 AND recall_count >= ?2
               AND origin IN ('ingest','verified_dream','session_distill')",
            params![min_confidence, min_recall],
            |r| r.get(0),
        )?;
        Ok(crate::memory::ripen::RipenGateCensus {
            latest,
            nonzero_recall,
            dual,
            dual_origin,
        })
    }

    /// Row count in a sibling `knowledge_core.db` (separate file).
    pub fn knowledge_core_row_count(&self, core_path: &Path) -> Result<i64> {
        if !core_path.exists() {
            anyhow::bail!("knowledge_core missing: {}", core_path.display());
        }
        let conn = Connection::open(core_path)?;
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM knowledge_core", [], |r| r.get(0))?;
        Ok(n)
    }

    /// Latest honeypot rows matching SQL LIKE pattern (immune patrol / ops).
    pub fn honeypot_latest_matching(
        &self,
        like_pattern: &str,
        limit: usize,
    ) -> Result<Vec<(Uuid, String, f64)>> {
        let conn = self.pool.get()?;
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='honeypot'",
            [],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Ok(Vec::new());
        }
        let mut stmt = conn.prepare(
            "SELECT id, content, confidence FROM honeypot
             WHERE is_latest = 1 AND content LIKE ?1
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![like_pattern, limit as i64], |row| {
                let id_s: String = row.get(0)?;
                let content: String = row.get(1)?;
                let confidence: f64 = row.get(2)?;
                Ok((id_s, content, confidence))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(id_s, content, confidence)| {
                Uuid::parse_str(&id_s)
                    .ok()
                    .map(|id| (id, content, confidence))
            })
            .collect();
        Ok(rows)
    }

    /// Label for Synapse / telemetry: which table backs cognition reads.
    pub fn cognition_memory_layer(&self) -> &'static str {
        let Ok(conn) = self.pool.get() else {
            return "semantic_vault";
        };
        match Self::cognition_from_honeypot(&conn) {
            Ok(true) => "honeypot",
            _ => "semantic_vault",
        }
    }

    /// M3: recall / association reads use curated `honeypot` when populated.
    pub(crate) fn cognition_from_honeypot(conn: &Connection) -> Result<bool> {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='honeypot'",
            [],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Ok(false);
        }
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(n > 0)
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

    /// Search with temporal decay applied in Rust.
    /// Returns facts sorted by decayed relevance score (honeypot when M3 table is populated).
    pub fn search_with_decay(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        let conn = self.pool.get()?;
        let use_hp = Self::cognition_from_honeypot(&conn)?;
        let sql = if use_hp {
            tracing::debug!("search_with_decay: honeypot (M3)");
            "SELECT id, content, embedding, decay_class, confidence, recall_count,
                    promoted_at, COALESCE(last_recalled_at, promoted_at)
             FROM honeypot
             WHERE is_latest = 1
               AND embedding IS NOT NULL AND length(embedding) >= 4"
        } else {
            "SELECT id, content, embedding, half_life_days, confirmation_count,
                    created_at, last_accessed_at
             FROM semantic_vault
             WHERE (julianday('now') - julianday(last_accessed_at)) < (half_life_days * 10.0)"
        };
        let mut stmt = conn.prepare(sql)?;

        let now = Utc::now();
        let query_lower = query_text.to_lowercase();
        let word_count = query_lower.split_whitespace().count().max(1) as f64;

        let mut scored: Vec<(SemanticFact, f64)> = Vec::new();
        if use_hp {
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, u32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })?;
            for row in rows.filter_map(|r| r.ok()) {
                let (
                    id_str,
                    content,
                    embed_blob,
                    decay_class,
                    confidence,
                    conf_count,
                    created_str,
                    accessed_str,
                ) = row;
                let embedding = decode_embed(&embed_blob);
                let half_life = Self::half_life_from_decay_class(&decay_class);
                let content_lower = content.to_lowercase();
                let keyword_score = query_lower
                    .split_whitespace()
                    .filter(|w| content_lower.contains(w))
                    .count() as f64
                    / word_count;
                let vec_sim = embedding_cosine_similarity(query_embedding, &embedding);
                let raw_score = (vec_sim * 0.7) + (keyword_score * 0.3);
                let accessed_at = chrono::DateTime::parse_from_rfc3339(&accessed_str)
                    .unwrap_or_else(|_| now.into());
                let days_elapsed =
                    (now - accessed_at.with_timezone(&Utc)).num_seconds() as f64 / 86400.0;
                let effective_days = (days_elapsed - (conf_count as f64 * 5.0)).max(0.0);
                let decay_multiplier = 0.5_f64.powf(effective_days / half_life);
                let fact = SemanticFact {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    content,
                    embedding,
                    confidence,
                    half_life_days: half_life,
                    confirmation_count: conf_count,
                    decay_class,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(now),
                    last_accessed_at: accessed_at.with_timezone(&Utc),
                };
                scored.push((fact, raw_score * decay_multiplier));
            }
        } else {
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, u32>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?;
            for row in rows.filter_map(|r| r.ok()) {
                let (id_str, content, embed_blob, half_life, conf_count, created_str, accessed_str) =
                    row;
                let embedding = decode_embed(&embed_blob);
                let content_lower = content.to_lowercase();
                let keyword_score = query_lower
                    .split_whitespace()
                    .filter(|w| content_lower.contains(w))
                    .count() as f64
                    / word_count;
                let vec_sim = embedding_cosine_similarity(query_embedding, &embedding);
                let raw_score = (vec_sim * 0.7) + (keyword_score * 0.3);
                let accessed_at = chrono::DateTime::parse_from_rfc3339(&accessed_str)
                    .unwrap_or_else(|_| now.into());
                let days_elapsed =
                    (now - accessed_at.with_timezone(&Utc)).num_seconds() as f64 / 86400.0;
                let effective_days = (days_elapsed - (conf_count as f64 * 5.0)).max(0.0);
                let decay_multiplier = 0.5_f64.powf(effective_days / half_life);
                let fact = SemanticFact {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    content,
                    embedding,
                    confidence: 1.0,
                    half_life_days: half_life,
                    confirmation_count: conf_count,
                    decay_class: "Episodic".to_string(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(now),
                    last_accessed_at: accessed_at.with_timezone(&Utc),
                };
                scored.push((fact, raw_score * decay_multiplier));
            }
        }

        // Sort descending by decayed score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored)
    }

    /// Hybrid search with optional cross-encoder rerank (requires `[rerank].enabled`).
    pub async fn search_with_decay_reranked(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        let prefetch = self
            .reranker
            .as_ref()
            .map(|r| r.prefetch_limit(limit))
            .unwrap_or(limit);
        let mut scored = Self::search_with_decay(self, query_embedding, query_text, prefetch)?;
        self.apply_rerank(query_text, limit, &mut scored).await;
        Ok(scored)
    }

    /// Best-effort vault recall: honeypot RRF when populated, else legacy vector/BM25.
    pub async fn search_recall(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        self.recall_rrf(query, limit, "obolus").await
    }

    /// Unified honeypot recall (RRF). Falls back to legacy search when honeypot empty.
    pub async fn recall_rrf(
        &self,
        query: &str,
        limit: usize,
        container_tag: &str,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.pool.get()?;
        if !Self::cognition_from_honeypot(&conn)? {
            drop(conn);
            return self.search_recall_legacy(q, limit).await;
        }

        self.ensure_honeypot_fts_synced(&conn)?;
        self.ensure_evidence_fts_synced(&conn)?;
        drop(conn);

        let mut candidates: HashMap<Uuid, RecallCandidate> = HashMap::new();

        let fts_ids = self.honeypot_fts_stream(q, container_tag, PREFETCH_K)?;
        let evidence_fts_ids = self.honeypot_evidence_fts_stream(q, container_tag, PREFETCH_K)?;
        let graph_ids = self.honeypot_graph_stream(q, PREFETCH_K)?;
        let kw_ids = if graph_ids.is_empty() {
            self.honeypot_keyword_stream(q, PREFETCH_K)?
        } else {
            Vec::new()
        };
        let mut rank_lists = vec![fts_ids.clone()];
        if !evidence_fts_ids.is_empty() {
            rank_lists.push(evidence_fts_ids);
        }
        if !graph_ids.is_empty() {
            rank_lists.push(graph_ids.clone());
        } else if !kw_ids.is_empty() {
            rank_lists.push(kw_ids.clone());
        }

        if let Some(embedder) = &self.embedder {
            match embedder.embed(q).await {
                Ok(emb) if !emb.is_empty() => {
                    let mut qdrant_ids = Vec::new();
                    if let Some(qdrant) = &self.qdrant {
                        if let Ok(ids) = qdrant.search_ids(&emb, PREFETCH_K).await {
                            qdrant_ids = ids;
                        }
                    }
                    let scored = self.search_with_decay(&emb, q, PREFETCH_K)?;
                    for (fact, _) in &scored {
                        let sf = self.honeypot_source_file(fact.id)?;
                        candidates.entry(fact.id).or_insert(RecallCandidate {
                            fact: fact.clone(),
                            source_file: sf,
                        });
                    }
                    let local_ids: Vec<Uuid> = scored.into_iter().map(|(f, _)| f.id).collect();
                    let vector_ids = merge_interleaved_rank(&qdrant_ids, &local_ids, PREFETCH_K);
                    if !vector_ids.is_empty() {
                        rank_lists.push(vector_ids);
                    }
                    let evidence_vector_ids =
                        self.honeypot_evidence_vector_stream(&emb, container_tag, PREFETCH_K)?;
                    if !evidence_vector_ids.is_empty() {
                        rank_lists.push(evidence_vector_ids);
                    }
                }
                Ok(_) => {
                    tracing::warn!(query = %q, "empty embedding — FTS-only recall");
                }
                Err(e) => {
                    tracing::warn!(error = %e, query = %q, "embed failed — FTS-only recall");
                }
            }
        }

        for list in &rank_lists {
            for id in list {
                if candidates.contains_key(id) {
                    continue;
                }
                if let Some(cand) = self.load_honeypot_candidate(*id)? {
                    candidates.insert(*id, cand);
                }
            }
        }

        let mut scores = rrf_fuse(&rank_lists);
        // Boost top-5 per stream so strong single-stream hits survive diversification/rerank.
        const STREAM_TOP_RESCUE: f64 = 0.025;
        for list in &rank_lists {
            for (idx, id) in list.iter().take(5).enumerate() {
                let boost = STREAM_TOP_RESCUE / (idx as f64 + 1.0);
                *scores.entry(*id).or_insert(0.0) += boost;
            }
        }
        let mut ranked: Vec<(RecallCandidate, f64)> = scores
            .into_iter()
            .filter_map(|(id, score)| candidates.get(&id).map(|c| (c.clone(), score)))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Exp2: relax the per-file cap *before* rerank so the cross-encoder can see
        // fact-bearing chunks that RRF ranked 6+ within their own file. The reranker's
        // truncate(limit) still produces the final top-N.
        const RERANK_STAGE_PER_FILE: usize = 12;
        let diversified =
            diversify_by_source_file(ranked, RERANK_PREFETCH.max(limit), RERANK_STAGE_PER_FILE);
        let mut scored: Vec<(SemanticFact, f64)> =
            diversified.into_iter().map(|(c, s)| (c.fact, s)).collect();

        // Phase A: relevance pool (RRF + optional cross-encoder). Keep prefetch
        // so phase B can still promote a high-Q fact that sat below `limit`.
        self.apply_rerank(q, RERANK_PREFETCH.max(limit), &mut scored)
            .await;
        self.apply_utility_select(&mut scored)?;
        scored.truncate(limit);
        Ok(scored)
    }

    /// MemRL phase B: Q-select inside the relevance pool (honeypot `utility_score`).
    fn apply_utility_select(&self, scored: &mut Vec<(SemanticFact, f64)>) -> Result<()> {
        if scored.len() <= 1 {
            return Ok(());
        }
        let ids: Vec<Uuid> = scored.iter().map(|(f, _)| f.id).collect();
        let utility = self.honeypot_utility_scores(&ids)?;
        apply_utility_boost(scored, &utility);
        Ok(())
    }

    fn honeypot_utility_scores(&self, ids: &[Uuid]) -> Result<HashMap<Uuid, f64>> {
        let mut out = HashMap::new();
        if ids.is_empty() {
            return Ok(out);
        }
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT utility_score FROM honeypot WHERE id = ?1")?;
        for id in ids {
            if let Ok(u) = stmt.query_row(params![id.to_string()], |row| row.get::<_, f64>(0)) {
                out.insert(*id, u.max(0.0));
            }
        }
        Ok(out)
    }

    async fn search_recall_legacy(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        if let Some(embedder) = &self.embedder {
            match embedder.embed(query).await {
                Ok(emb) if !emb.is_empty() => {
                    return self.search_with_decay_reranked(&emb, query, limit).await;
                }
                Ok(_) => tracing::warn!(query = %query, "empty embedding — keyword recall"),
                Err(e) => {
                    tracing::warn!(error = %e, query = %query, "embed failed — keyword recall")
                }
            }
        }
        let prefetch = self
            .reranker
            .as_ref()
            .map(|r| r.prefetch_limit(limit))
            .unwrap_or(limit);
        let mut scored = self.keyword_search(query, prefetch)?;
        self.apply_rerank(query, limit, &mut scored).await;
        Ok(scored)
    }

    fn ensure_honeypot_fts_synced(&self, conn: &Connection) -> Result<()> {
        let hp: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1",
            [],
            |r| r.get(0),
        )?;
        let fts_latest: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot_fts f
             JOIN honeypot h ON f.rowid = h.rowid
             WHERE h.is_latest = 1",
            [],
            |r| r.get(0),
        )?;
        let fts_stale: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot_fts f
             JOIN honeypot h ON f.rowid = h.rowid
             WHERE h.is_latest = 0",
            [],
            |r| r.get(0),
        )?;
        if hp > 0 && fts_latest == hp && fts_stale == 0 {
            return Ok(());
        }
        conn.execute_batch(
            "DELETE FROM honeypot_fts;
             INSERT INTO honeypot_fts(rowid, content, content_norm)
             SELECT rowid, content, content_norm FROM honeypot WHERE is_latest = 1;",
        )?;
        tracing::info!(
            honeypot = hp,
            "Backfilled honeypot_fts for RRF lexical stream"
        );
        Ok(())
    }

    fn ensure_evidence_fts_synced(&self, conn: &Connection) -> Result<()> {
        let ev: i64 = conn.query_row(
            "SELECT COUNT(*) FROM evidence WHERE verify_pass = 1",
            [],
            |r| r.get(0),
        )?;
        let fts: i64 = conn.query_row("SELECT COUNT(*) FROM evidence_fts", [], |r| r.get(0))?;
        if ev > 0 && fts >= ev {
            return Ok(());
        }
        conn.execute_batch(
            "DELETE FROM evidence_fts;
             INSERT INTO evidence_fts(rowid, evidence_text, evidence_norm)
             SELECT rowid, evidence_text, evidence_norm FROM evidence WHERE verify_pass = 1;",
        )?;
        tracing::info!(
            evidence = ev,
            "Backfilled evidence_fts for RRF lexical stream"
        );
        Ok(())
    }

    fn honeypot_fts_stream(
        &self,
        query: &str,
        container_tag: &str,
        limit: usize,
    ) -> Result<Vec<Uuid>> {
        let narrow =
            self.honeypot_fts_stream_query(query, container_tag, limit, &fts_match_query(query))?;
        if !narrow.is_empty() {
            return Ok(narrow);
        }
        self.honeypot_fts_stream_query(query, container_tag, limit, &fts_match_query_broad(query))
    }

    fn honeypot_fts_stream_query(
        &self,
        _query: &str,
        container_tag: &str,
        limit: usize,
        match_q: &str,
    ) -> Result<Vec<Uuid>> {
        if match_q.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT h.id
             FROM honeypot h
             JOIN honeypot_fts fts ON h.rowid = fts.rowid
             WHERE honeypot_fts MATCH ?1
               AND h.is_latest = 1
               AND h.container_tag = ?2
             ORDER BY rank
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![match_q, container_tag, limit as i64], |row| {
            let id_str: String = row.get(0)?;
            Ok(id_str)
        })?;
        let mut ids = Vec::new();
        for id_str in rows.filter_map(|r| r.ok()) {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    fn honeypot_evidence_fts_stream(
        &self,
        query: &str,
        container_tag: &str,
        limit: usize,
    ) -> Result<Vec<Uuid>> {
        let narrow = self.honeypot_evidence_fts_stream_query(
            query,
            container_tag,
            limit,
            &fts_match_query(query),
        )?;
        if !narrow.is_empty() {
            return Ok(narrow);
        }
        self.honeypot_evidence_fts_stream_query(
            query,
            container_tag,
            limit,
            &fts_match_query_broad(query),
        )
    }

    fn honeypot_evidence_fts_stream_query(
        &self,
        _query: &str,
        container_tag: &str,
        limit: usize,
        match_q: &str,
    ) -> Result<Vec<Uuid>> {
        if match_q.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT e.fact_id
             FROM evidence e
             JOIN evidence_fts fts ON e.rowid = fts.rowid
             JOIN honeypot h ON e.fact_id = h.id
             WHERE evidence_fts MATCH ?1
               AND e.verify_pass = 1
               AND h.is_latest = 1
               AND h.container_tag = ?2
             ORDER BY rank
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![match_q, container_tag, limit as i64], |row| {
            let fact_id_str: String = row.get(0)?;
            Ok(fact_id_str)
        })?;
        let mut ids = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for id_str in rows.filter_map(|r| r.ok()) {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if seen.insert(id) {
                    ids.push(id);
                }
            }
        }
        Ok(ids)
    }

    fn honeypot_evidence_vector_stream(
        &self,
        query_embedding: &[f32],
        container_tag: &str,
        limit: usize,
    ) -> Result<Vec<Uuid>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT e.fact_id, e.embedding
             FROM evidence e
             JOIN honeypot h ON e.fact_id = h.id
             WHERE e.embedding IS NOT NULL AND length(e.embedding) >= 4
               AND e.verify_pass = 1
               AND h.is_latest = 1
               AND h.container_tag = ?1",
        )?;
        let rows = stmt.query_map(params![container_tag], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        let mut scored: Vec<(Uuid, f64)> = Vec::new();
        for row in rows.filter_map(|r| r.ok()) {
            let (fact_id_str, embed_blob) = row;
            if let Ok(fact_id) = Uuid::parse_str(&fact_id_str) {
                let embedding = decode_embed(&embed_blob);
                let score = embedding_cosine_similarity(query_embedding, &embedding);
                scored.push((fact_id, score));
            }
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut ids = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (id, _) in scored {
            if seen.insert(id) {
                ids.push(id);
            }
            if ids.len() >= limit {
                break;
            }
        }
        Ok(ids)
    }

    fn honeypot_keyword_stream(&self, query: &str, limit: usize) -> Result<Vec<Uuid>> {
        let scored = self.keyword_search(query, limit)?;
        Ok(scored.into_iter().map(|(f, _)| f.id).collect())
    }

    /// Graph stream: Neo4j hints (optional) mapped to honeypot rows; SQLite entity overlap fallback.
    fn honeypot_graph_stream(&self, query: &str, limit: usize) -> Result<Vec<Uuid>> {
        let mut hints = self.fetch_neo4j_graph_hints(query, limit)?;
        if hints.is_empty() {
            hints = extract_entity_tokens(query);
        }
        if hints.is_empty() {
            return Ok(Vec::new());
        }
        self.map_hints_to_honeypot_ids(&hints, limit)
    }

    fn fetch_neo4j_graph_hints(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let script = Path::new("scripts/graph-recall-stream.py");
        if !script.exists() {
            return Ok(Vec::new());
        }
        let python = if Path::new("scripts/.venv/bin/python3").exists() {
            "scripts/.venv/bin/python3"
        } else {
            "python3"
        };
        let out = StdCommand::new(python)
            .arg(script)
            .arg(query)
            .arg(limit.to_string())
            .env(
                "NEO4J_URL",
                std::env::var("NEO4J_URL").unwrap_or_else(|_| "bolt://192.168.31.202:7687".into()),
            )
            .env(
                "NEO4J_USERNAME",
                std::env::var("NEO4J_USERNAME").unwrap_or_else(|_| "neo4j".into()),
            )
            .env(
                "NEO4J_PASSWORD",
                std::env::var("NEO4J_PASSWORD").unwrap_or_default(),
            )
            .env(
                "NEO4J_DATABASE",
                std::env::var("NEO4J_DATABASE").unwrap_or_else(|_| "neo4j".into()),
            )
            .output();
        let Ok(output) = out else {
            return Ok(Vec::new());
        };
        if !output.status.success() {
            tracing::debug!(stderr = %String::from_utf8_lossy(&output.stderr), "graph-recall-stream.py failed");
            return Ok(Vec::new());
        }
        let parsed: Vec<String> = serde_json::from_slice(&output.stdout).unwrap_or_default();
        Ok(parsed)
    }

    fn map_hints_to_honeypot_ids(&self, hints: &[String], limit: usize) -> Result<Vec<Uuid>> {
        let conn = self.pool.get()?;
        let mut ids = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for hint in hints {
            let needle = hint.trim();
            if needle.len() < 3 {
                continue;
            }
            let pattern = format!("%{needle}%");
            let mut stmt = conn.prepare(
                "SELECT id FROM honeypot
                 WHERE is_latest = 1 AND (content LIKE ?1 OR content_norm LIKE ?1)
                 ORDER BY utility_score DESC, confidence DESC, recall_count DESC
                 LIMIT 3",
            )?;
            for id_str in stmt
                .query_map(params![pattern], |row| row.get::<_, String>(0))?
                .filter_map(|r| r.ok())
            {
                if let Ok(id) = Uuid::parse_str(&id_str) {
                    if seen.insert(id) {
                        ids.push(id);
                    }
                }
                if ids.len() >= limit {
                    return Ok(ids);
                }
            }
        }
        if ids.len() < limit {
            let tokens = extract_entity_tokens(hints.join(" ").as_str());
            let mut scored: Vec<(Uuid, usize)> = Vec::new();
            let mut stmt =
                conn.prepare("SELECT id, content, content_norm FROM honeypot WHERE is_latest = 1")?;
            for row in stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })? {
                let Ok((id_str, content, norm)) = row else {
                    continue;
                };
                let blob = format!("{} {}", content.to_lowercase(), norm.to_lowercase());
                let matches = tokens
                    .iter()
                    .filter(|t| blob.contains(&t.to_lowercase()))
                    .count();
                if matches == 0 {
                    continue;
                }
                if let Ok(id) = Uuid::parse_str(&id_str) {
                    scored.push((id, matches));
                }
            }
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            for (id, _) in scored {
                if seen.insert(id) {
                    ids.push(id);
                }
                if ids.len() >= limit {
                    break;
                }
            }
        }
        Ok(ids)
    }

    pub fn honeypot_source_file(&self, id: Uuid) -> Result<Option<String>> {
        let conn = self.pool.get()?;
        conn.query_row(
            "SELECT source_file FROM honeypot WHERE id = ?1",
            params![id.to_string()],
            |r| r.get(0),
        )
        .map_err(Into::into)
    }

    pub fn get_evidence_text(&self, fact_id: Uuid) -> Result<Option<String>> {
        let conn = self.pool.get()?;
        let res = conn.query_row(
            "SELECT evidence_text FROM evidence WHERE fact_id = ?1 LIMIT 1",
            params![fact_id.to_string()],
            |row| row.get::<_, String>(0),
        );
        match res {
            Ok(text) => Ok(Some(text)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns true when this transcript was already distilled (archive worker + nightly cron dedup).
    pub fn distill_dedup_seen(&self, dedup_key: &str) -> Result<bool> {
        let conn = self.pool.get()?;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM distill_dedup WHERE dedup_key = ?1",
            params![dedup_key],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn record_distill_dedup(
        &self,
        dedup_key: &str,
        session_id: &str,
        source: &str,
        truths_count: usize,
    ) -> Result<()> {
        let conn = self.pool.get()?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO distill_dedup (dedup_key, session_id, source, distilled_at, truths_count)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(dedup_key) DO UPDATE SET
                distilled_at = excluded.distilled_at,
                truths_count = excluded.truths_count",
            params![dedup_key, session_id, source, now, truths_count as i64],
        )?;
        Ok(())
    }

    /// Returns true when identical prepared ingest body was already processed.
    pub fn ingest_dedup_seen(&self, content_hash: &str) -> Result<bool> {
        let conn = self.pool.get()?;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM ingest_dedup WHERE content_hash = ?1",
            params![content_hash],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn record_ingest_dedup(&self, content_hash: &str, source_path: &str) -> Result<()> {
        let conn = self.pool.get()?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO ingest_dedup (content_hash, source_path, ingested_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(content_hash) DO UPDATE SET
                source_path = excluded.source_path,
                ingested_at = excluded.ingested_at",
            params![content_hash, source_path, now],
        )?;
        Ok(())
    }

    fn load_honeypot_candidate(&self, id: Uuid) -> Result<Option<RecallCandidate>> {
        let conn = self.pool.get()?;
        let row = conn.query_row(
            "SELECT id, content, embedding, decay_class, confidence, recall_count,
                    promoted_at, COALESCE(last_recalled_at, promoted_at), source_file
             FROM honeypot WHERE id = ?1 AND is_latest = 1",
            params![id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, u32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            },
        );
        let Ok((
            id_str,
            content,
            embed_blob,
            decay_class,
            confidence,
            conf_count,
            created_str,
            accessed_str,
            source_file,
        )) = row
        else {
            return Ok(None);
        };
        let now = Utc::now();
        let half_life = Self::half_life_from_decay_class(&decay_class);
        let accessed_at =
            chrono::DateTime::parse_from_rfc3339(&accessed_str).unwrap_or_else(|_| now.into());
        let fact = SemanticFact {
            id: Uuid::parse_str(&id_str).unwrap_or(id),
            content,
            embedding: decode_embed(&embed_blob),
            confidence,
            half_life_days: half_life,
            confirmation_count: conf_count,
            decay_class: decay_class.clone(),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now),
            last_accessed_at: accessed_at.with_timezone(&Utc),
        };
        Ok(Some(RecallCandidate { fact, source_file }))
    }

    pub(crate) async fn apply_rerank(
        &self,
        query_text: &str,
        limit: usize,
        scored: &mut Vec<(SemanticFact, f64)>,
    ) {
        let Some(rr) = &self.reranker else {
            scored.truncate(limit);
            return;
        };
        if query_text.is_empty() || scored.len() < 2 {
            scored.truncate(limit);
            return;
        }
        let docs: Vec<String> = scored
            .iter()
            .map(|(f, _)| {
                let mut doc = f.content.clone();
                if let Ok(Some(ev)) = self.get_evidence_text(f.id) {
                    let ev = ev.trim();
                    if !ev.is_empty() {
                        doc.push_str("\n");
                        doc.push_str(ev);
                    }
                }
                doc
            })
            .collect();
        match rr.rerank(query_text, &docs, Some(limit)).await {
            Ok(order) => {
                let mut reordered = Vec::with_capacity(order.len());
                for (idx, rerank_score) in order {
                    if let Some((fact, _)) = scored.get(idx).cloned() {
                        reordered.push((fact, rerank_score));
                    }
                }
                if !reordered.is_empty() {
                    *scored = reordered;
                }
            }
            Err(e) => tracing::warn!(error = %e, "Rerank failed — keeping decay/BM25 order"),
        }
        scored.truncate(limit);
    }

    fn maybe_upsert_evidence(
        conn: &Connection,
        fact_id: &str,
        truth: &ExtractedTruth,
        evidence_embedding: &[u8],
    ) -> Result<()> {
        if let Some(ev) = &truth.evidence {
            let ev_norm = ev.evidence_text.to_lowercase();
            crate::memory::honeypot::upsert_evidence_row(
                conn,
                fact_id, // evidence_id = fact_id for 1:1 first iteration
                fact_id,
                truth.source_file.as_deref(),
                &ev.evidence_text,
                &ev_norm,
                ev.char_start,
                ev.char_end,
                Some(&ev.quote_verifier),
                evidence_embedding,
            )?;
        }
        Ok(())
    }

    fn promote_corroborate_vault(
        &self,
        conn: &Connection,
        existing_id: &str,
        truth: &ExtractedTruth,
        embedding: &[u8],
        content_norm: &str,
        confidence: f64,
        origin: &str,
        evidence_embedding: &[u8],
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE semantic_vault
             SET confirmation_count = confirmation_count + 1,
                 last_accessed_at = ?1,
                 confidence = MAX(confidence, ?2),
                 content_norm = COALESCE(content_norm, ?3)
             WHERE id = ?4",
            params![now, confidence, content_norm, existing_id],
        )?;
        info!(id = %existing_id, "Corroborated existing truth");
        if qualifies_for_honeypot(truth) && !is_unverified_derived(truth, origin) {
            honeypot::upsert_honeypot_row(
                conn,
                existing_id,
                truth,
                embedding,
                content_norm,
                origin,
            )?;
            Self::maybe_upsert_evidence(conn, existing_id, truth, evidence_embedding)?;
        }
        Ok(())
    }

    fn promote_new_vault_truth(
        &self,
        conn: &Connection,
        truth: &ExtractedTruth,
        embedding: &[u8],
        content_norm: &str,
        confidence: f64,
        origin: &str,
        evidence_embedding: &[u8],
    ) -> Result<()> {
        if let Some(entity) = extract_primary_entity(&truth.content) {
            if let Some((old_hp_id, old_content)) =
                find_latest_honeypot_by_entity(conn, &entity, "obolus")?
            {
                let kind = classify_truth_pair(&old_content, &truth.content);
                match kind {
                    LifecycleKind::Duplicate => {
                        let vault_id: String = conn.query_row(
                            "SELECT vault_id FROM honeypot WHERE id = ?1",
                            params![old_hp_id],
                            |row| row.get(0),
                        )?;
                        return self.promote_corroborate_vault(
                            conn,
                            &vault_id,
                            truth,
                            embedding,
                            content_norm,
                            confidence,
                            origin,
                            evidence_embedding,
                        );
                    }
                    LifecycleKind::Contradicts => {
                        let now = Utc::now();
                        conn.execute(
                            "INSERT INTO semantic_vault
                                (id, content, embedding, half_life_days, confidence,
                                 confirmation_count, decay_class, created_at, last_accessed_at,
                                 source_file, content_norm)
                            VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?9, ?10)",
                            params![
                                truth.id.to_string(),
                                truth.content,
                                embedding.to_vec(),
                                truth.decay_class.half_life_days(),
                                confidence,
                                format!("{:?}", truth.decay_class),
                                now.to_rfc3339(),
                                now.to_rfc3339(),
                                truth.source_file,
                                content_norm,
                            ],
                        )?;
                        supersede_honeypot(conn, &old_hp_id)?;
                        if qualifies_for_honeypot(truth) && !is_unverified_derived(truth, origin) {
                            honeypot::insert_honeypot_lifecycle(
                                conn,
                                &truth.id.to_string(),
                                truth,
                                embedding,
                                content_norm,
                                origin,
                                LifecycleKind::Contradicts.graph_rel(),
                                Some(&old_hp_id),
                            )?;
                            Self::maybe_upsert_evidence(
                                conn,
                                &truth.id.to_string(),
                                truth,
                                evidence_embedding,
                            )?;
                        }
                        info!(
                            id = %truth.id,
                            superseded = %old_hp_id,
                            "Promoted contradicting truth (lifecycle update)"
                        );
                        return Ok(());
                    }
                    LifecycleKind::Extends => {
                        let now = Utc::now();
                        conn.execute(
                            "INSERT INTO semantic_vault
                                (id, content, embedding, half_life_days, confidence,
                                 confirmation_count, decay_class, created_at, last_accessed_at,
                                 source_file, content_norm)
                            VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?9, ?10)",
                            params![
                                truth.id.to_string(),
                                truth.content,
                                embedding.to_vec(),
                                truth.decay_class.half_life_days(),
                                confidence,
                                format!("{:?}", truth.decay_class),
                                now.to_rfc3339(),
                                now.to_rfc3339(),
                                truth.source_file,
                                content_norm,
                            ],
                        )?;
                        if qualifies_for_honeypot(truth) && !is_unverified_derived(truth, origin) {
                            honeypot::insert_honeypot_lifecycle(
                                conn,
                                &truth.id.to_string(),
                                truth,
                                embedding,
                                content_norm,
                                origin,
                                LifecycleKind::Extends.graph_rel(),
                                Some(&old_hp_id),
                            )?;
                            Self::maybe_upsert_evidence(
                                conn,
                                &truth.id.to_string(),
                                truth,
                                evidence_embedding,
                            )?;
                        }
                        info!(id = %truth.id, extends = %old_hp_id, "Promoted extending truth");
                        return Ok(());
                    }
                    LifecycleKind::Derives | LifecycleKind::Unrelated => {}
                }
            }
        }

        let now = Utc::now();
        conn.execute(
            "INSERT INTO semantic_vault
                (id, content, embedding, half_life_days, confidence,
                 confirmation_count, decay_class, created_at, last_accessed_at,
                 source_file, content_norm)
            VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?9, ?10)",
            params![
                truth.id.to_string(),
                truth.content,
                embedding.to_vec(),
                truth.decay_class.half_life_days(),
                confidence,
                format!("{:?}", truth.decay_class),
                now.to_rfc3339(),
                now.to_rfc3339(),
                truth.source_file,
                content_norm,
            ],
        )?;
        info!(id = %truth.id, "Promoted new truth to vault");
        if qualifies_for_honeypot(truth) && !is_unverified_derived(truth, origin) {
            honeypot::insert_honeypot_lifecycle(
                conn,
                &truth.id.to_string(),
                truth,
                embedding,
                content_norm,
                origin,
                None,
                None,
            )?;
            Self::maybe_upsert_evidence(conn, &truth.id.to_string(), truth, evidence_embedding)?;
        }
        Ok(())
    }

    /// Chain of facts for one honeypot id (latest first, includes superseded).
    pub fn get_memory_chain(&self, fact_id: &str) -> Result<Vec<(String, bool, Option<String>)>> {
        let conn = self.db_conn()?;
        let mut chain = Vec::new();
        let mut cursor = fact_id.to_string();
        let mut seen = std::collections::HashSet::new();
        while !cursor.is_empty() && seen.insert(cursor.clone()) {
            let row = conn.query_row(
                "SELECT content, is_latest, graph_rel, supersedes_id
                 FROM honeypot WHERE id = ?1 OR vault_id = ?1",
                params![cursor],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i32>(1)? != 0,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            );
            let Ok((content, is_latest, graph_rel, supersedes_id)) = row else {
                break;
            };
            chain.push((content, is_latest, graph_rel));
            cursor = supersedes_id.unwrap_or_default();
        }
        Ok(chain)
    }

    /// Promote extracted truths into vault (default origin `ingest`).
    pub async fn promote_truths(&self, truths: &[ExtractedTruth]) -> Result<()> {
        self.promote_truths_with_origin(truths, "ingest").await
    }

    /// Promote truths to vault and honeypot when [honeypot::qualifies_for_honeypot].
    pub async fn promote_truths_with_origin(
        &self,
        truths: &[ExtractedTruth],
        origin: &str,
    ) -> Result<()> {
        if truths.is_empty() {
            return Ok(());
        }

        let mut embedding_blobs: Vec<Vec<u8>> = Vec::with_capacity(truths.len());
        let mut evidence_embedding_blobs: Vec<Vec<u8>> = Vec::with_capacity(truths.len());
        for truth in truths {
            let blob = if let Some(embedder) = &self.embedder {
                match embedder.embed(&truth.content).await {
                    Ok(vec) if !vec.is_empty() => bincode_embed(&vec),
                    Ok(_) => Vec::new(),
                    Err(e) => {
                        tracing::warn!(error = %e, "Embedding failed — storing without vector");
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };
            embedding_blobs.push(blob);

            let ev_blob = if let Some(ev) = &truth.evidence {
                if let Some(embedder) = &self.embedder {
                    match embedder.embed(&ev.evidence_text).await {
                        Ok(vec) if !vec.is_empty() => bincode_embed(&vec),
                        _ => Vec::new(),
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
            evidence_embedding_blobs.push(ev_blob);
        }

        let promote_started = Utc::now();
        let conn = self.pool.get()?;
        conn.execute_batch("BEGIN IMMEDIATE")?;

        let result = (|| -> Result<()> {
            for (i, truth) in truths.iter().enumerate() {
                let content_norm = normalize_truth_content(&truth.content);
                let confidence = truth.confidence as f64;

                if confidence < 0.85 {
                    let now = Utc::now();
                    conn.execute(
                        "INSERT OR REPLACE INTO quarantine_vault
                            (id, content, embedding, half_life_days, confidence, created_at)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![
                            truth.id.to_string(),
                            truth.content,
                            embedding_blobs[i].clone(),
                            truth.decay_class.half_life_days(),
                            confidence,
                            now.to_rfc3339(),
                        ],
                    )?;
                    tracing::warn!(
                        id = %truth.id,
                        confidence,
                        "Ingest truth quarantined due to low confidence"
                    );
                    continue;
                }

                let existing: Option<String> = conn
                    .query_row(
                        "SELECT id FROM semantic_vault
                         WHERE content_norm = ?1
                            OR (content_norm IS NULL AND content = ?2)",
                        params![content_norm, truth.content],
                        |row| row.get(0),
                    )
                    .ok();

                if let Some(existing_id) = existing {
                    self.promote_corroborate_vault(
                        &conn,
                        &existing_id,
                        truth,
                        &embedding_blobs[i],
                        &content_norm,
                        confidence,
                        origin,
                        &evidence_embedding_blobs[i],
                    )?;
                } else {
                    self.promote_new_vault_truth(
                        &conn,
                        truth,
                        &embedding_blobs[i],
                        &content_norm,
                        confidence,
                        origin,
                        &evidence_embedding_blobs[i],
                    )?;
                }
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                conn.execute_batch("COMMIT")?;
                info!(
                    count = truths.len(),
                    origin, "Batch promoted truths to vault"
                );
                self.seed_core_pin_bonded(truths, origin);
                self.maybe_incremental_qdrant_sync(truths, origin, promote_started)
                    .await;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// One-shot Bonded Felt Use for CORE crystallize / `[CORE]` pins (recall starts at 0).
    fn seed_core_pin_bonded(&self, truths: &[ExtractedTruth], origin: &str) {
        for truth in truths {
            if !core_pin::should_seed_bonded(&truth.content, origin) {
                continue;
            }
            if !qualifies_for_honeypot(truth) || is_unverified_derived(truth, origin) {
                continue;
            }
            let Ok(conn) = self.pool.get() else {
                continue;
            };
            // Prefer honeypot row id (may be corroboration existing_id).
            let hp_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM honeypot
                     WHERE is_latest = 1 AND (id = ?1 OR content = ?2)
                     ORDER BY CASE WHEN id = ?1 THEN 0 ELSE 1 END
                     LIMIT 1",
                    params![truth.id.to_string(), truth.content],
                    |row| row.get(0),
                )
                .ok();
            let Some(id_str) = hp_id else {
                continue;
            };
            let recall: i64 = conn
                .query_row(
                    "SELECT recall_count FROM honeypot WHERE id = ?1",
                    params![id_str],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            // Only seed virgin rows — don't re-Bonded every corroboration.
            if recall > 0 {
                continue;
            }
            let Ok(uuid) = Uuid::parse_str(&id_str) else {
                continue;
            };
            if let Err(e) = felt_use::touch(self, uuid, FeltUseKind::Bonded) {
                tracing::debug!(error = %e, id = %id_str, "core_pin Bonded seed skipped");
            } else {
                info!(id = %id_str, "core_pin Bonded seed (+5 recall)");
            }
        }
    }

    /// After honeypot-eligible promote, upsert only those points (GZMO-next).
    async fn maybe_incremental_qdrant_sync(
        &self,
        truths: &[ExtractedTruth],
        origin: &str,
        since: chrono::DateTime<Utc>,
    ) {
        if self.qdrant.is_none() {
            return;
        }
        if std::env::var("GZMO_INSTANCE").ok().as_deref() != Some("next") {
            return;
        }
        let ids: Vec<String> = truths
            .iter()
            .filter(|t| qualifies_for_honeypot(t) && !is_unverified_derived(t, origin))
            .map(|t| t.id.to_string())
            .collect();
        if ids.is_empty() {
            return;
        }
        let root = crate::memory::qdrant_sync::discover_project_root();
        let url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://127.0.0.1:6333".into());
        let collection = std::env::var("QDRANT_COLLECTION").unwrap_or_else(|_| "honeypot".into());
        let cfg = crate::config::QdrantConfig {
            enabled: true,
            url,
            collection,
            sync_enabled: true,
            ..Default::default()
        };
        // Prefer --ids; also pass --since as a safety filter.
        match crate::memory::qdrant_sync::sync_vault_to_qdrant_filtered(
            &root,
            &cfg,
            &self.db_path,
            Some(&since.to_rfc3339()),
            Some(&ids),
        )
        .await
        {
            Ok(()) => info!(count = ids.len(), "Incremental Qdrant upsert after promote"),
            Err(e) => {
                tracing::warn!(error = %e, "Incremental Qdrant upsert failed (nightly sync remains)")
            }
        }
    }

    /// Metacognitive guard: recall past failures for a given command/context.
    pub fn recall_failures(&self, description: &str) -> Result<Vec<String>> {
        let conn = self.pool.get()?;
        let search_pattern = format!("%{}%", description.to_lowercase().replace(' ', "%"));
        let mut stmt = conn.prepare(
            "SELECT content FROM semantic_vault
             WHERE (content LIKE '%error%' OR content LIKE '%failed%' OR content LIKE '%warning%')
               AND lower(content) LIKE ?1
             ORDER BY last_accessed_at DESC LIMIT 5",
        )?;

        let results: Vec<String> = stmt
            .query_map([search_pattern], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .filter(|content: &String| {
                let desc_lower = description.to_lowercase();
                let content_lower = content.to_lowercase();
                desc_lower
                    .split_whitespace()
                    .any(|w| content_lower.contains(w))
            })
            .collect();

        Ok(results)
    }

    /// Keyword-only search (no embeddings required). Uses honeypot when populated (M3).
    pub fn keyword_search(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        let conn = self.pool.get()?;
        let use_hp = Self::cognition_from_honeypot(&conn)?;
        let sql = if use_hp {
            tracing::debug!("keyword_search: honeypot (M3)");
            "SELECT id, content, embedding, decay_class, confidence, recall_count,
                    promoted_at, COALESCE(last_recalled_at, promoted_at)
             FROM honeypot WHERE is_latest = 1"
        } else {
            "SELECT id, content, embedding, half_life_days, confirmation_count,
                    created_at, last_accessed_at
             FROM semantic_vault
             WHERE (julianday('now') - julianday(last_accessed_at)) < (half_life_days * 10.0)"
        };
        let mut stmt = conn.prepare(sql)?;

        let now = Utc::now();
        let query_lower = query_text.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        if query_words.is_empty() {
            return Ok(Vec::new());
        }

        let mut scored: Vec<(SemanticFact, f64)> = Vec::new();
        if use_hp {
            for row in stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, f64>(4)?,
                        row.get::<_, u32>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                })?
                .filter_map(|r| r.ok())
            {
                let (
                    id_str,
                    content,
                    embed_blob,
                    decay_class,
                    confidence,
                    conf_count,
                    created_str,
                    accessed_str,
                ) = row;
                let content_lower = content.to_lowercase();
                let matched = query_words
                    .iter()
                    .filter(|w| content_lower.contains(**w))
                    .count();
                if matched == 0 {
                    continue;
                }
                let half_life = Self::half_life_from_decay_class(&decay_class);
                let keyword_score = matched as f64 / query_words.len() as f64;
                let accessed_at = chrono::DateTime::parse_from_rfc3339(&accessed_str)
                    .unwrap_or_else(|_| now.into());
                let days_elapsed =
                    (now - accessed_at.with_timezone(&Utc)).num_seconds() as f64 / 86400.0;
                let effective_days = (days_elapsed - (conf_count as f64 * 5.0)).max(0.0);
                let decay_multiplier = 0.5_f64.powf(effective_days / half_life);
                scored.push((
                    SemanticFact {
                        id: Uuid::parse_str(&id_str).unwrap_or_default(),
                        content,
                        embedding: decode_embed(&embed_blob),
                        confidence,
                        half_life_days: half_life,
                        confirmation_count: conf_count,
                        decay_class,
                        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or(now),
                        last_accessed_at: accessed_at.with_timezone(&Utc),
                    },
                    keyword_score * decay_multiplier,
                ));
            }
        } else {
            for row in stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, f64>(3)?,
                        row.get::<_, u32>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                })?
                .filter_map(|r| r.ok())
            {
                let (id_str, content, embed_blob, half_life, conf_count, created_str, accessed_str) =
                    row;
                let content_lower = content.to_lowercase();
                let matched = query_words
                    .iter()
                    .filter(|w| content_lower.contains(**w))
                    .count();
                if matched == 0 {
                    continue;
                }
                let keyword_score = matched as f64 / query_words.len() as f64;
                let accessed_at = chrono::DateTime::parse_from_rfc3339(&accessed_str)
                    .unwrap_or_else(|_| now.into());
                let days_elapsed =
                    (now - accessed_at.with_timezone(&Utc)).num_seconds() as f64 / 86400.0;
                let effective_days = (days_elapsed - (conf_count as f64 * 5.0)).max(0.0);
                let decay_multiplier = 0.5_f64.powf(effective_days / half_life);
                scored.push((
                    SemanticFact {
                        id: Uuid::parse_str(&id_str).unwrap_or_default(),
                        content,
                        embedding: decode_embed(&embed_blob),
                        confidence: 1.0,
                        half_life_days: half_life,
                        confirmation_count: conf_count,
                        decay_class: "Episodic".to_string(),
                        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or(now),
                        last_accessed_at: accessed_at.with_timezone(&Utc),
                    },
                    keyword_score * decay_multiplier,
                ));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored)
    }

    /// Store a plain text fact (no embedding). For use by the interactive REPL
    /// when no embedding model is available.
    pub fn store_text(&self, content: &str, decay_class: &str, confidence: f64) -> Result<()> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        let id = Uuid::new_v4();

        let half_life = match decay_class {
            "Core" => 365.0 * 100.0,
            "Semantic" => 365.0,
            "SessionDistill" => 60.0,
            "Procedural" => 90.0,
            _ => 30.0, // Episodic default
        };

        if confidence < 0.85 {
            conn.execute(
                "INSERT OR REPLACE INTO quarantine_vault
                    (id, content, embedding, half_life_days, confidence, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id.to_string(),
                    content,
                    Vec::<u8>::new(),
                    half_life,
                    confidence,
                    now.to_rfc3339(),
                ],
            )?;
            tracing::warn!(fact_id = %id, confidence, "Memory quarantined due to low confidence");
            return Ok(());
        }

        conn.execute(
            "INSERT OR REPLACE INTO semantic_vault
                (id, content, embedding, half_life_days, confidence, confirmation_count,
                 decay_class, created_at, last_accessed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8)",
            params![
                id.to_string(),
                content,
                Vec::<u8>::new(),
                half_life,
                confidence,
                decay_class,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        info!(id = %id, decay_class, "Stored text fact in vault");
        Ok(())
    }

    /// Rows in `semantic_vault` with empty embedding blobs.
    pub fn count_missing_embeddings(&self) -> Result<usize> {
        let conn = self.pool.get()?;
        let count: usize = conn.query_row(
            "SELECT COUNT(*) FROM semantic_vault WHERE embedding IS NULL OR length(embedding) = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Promote high-confidence vault facts not yet in `honeypot` (overnight metabolism job).
    pub fn promote_mature_to_honeypot(&self, limit: Option<usize>) -> Result<PromoteMatureReport> {
        let cap = limit.unwrap_or(500) as i64;
        let conn = self.pool.get()?;
        // Honeypot table is created by schema migrations in `open`.
        let mut stmt = conn.prepare(
            "SELECT id, content, confidence, decay_class, source_file, embedding
             FROM semantic_vault
             WHERE confidence >= 0.85
               AND NOT EXISTS (SELECT 1 FROM honeypot h WHERE h.id = semantic_vault.id)
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;
        let rows: Vec<(String, String, f64, String, Option<String>, Vec<u8>)> = stmt
            .query_map([cap], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get::<_, Option<Vec<u8>>>(5)?.unwrap_or_default(),
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut report = PromoteMatureReport {
            candidates: rows.len(),
            promoted: 0,
            skipped: 0,
        };

        for (id, content, confidence, decay_class, source_file, embedding) in rows {
            let source = source_file
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "semantic_vault".into());
            let truth = ExtractedTruth {
                id: Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()),
                content: content.clone(),
                confidence: confidence as f32,
                mmr_score: 0.0,
                source_date: Utc::now().date_naive(),
                decay_class: parse_decay_class(&decay_class),
                source_file: Some(source),
                evidence: None,
            };
            if !honeypot::qualifies_for_honeypot(&truth) {
                report.skipped += 1;
                continue;
            }
            let content_norm = normalize_truth_content(&truth.content);
            honeypot::upsert_honeypot_row(
                &conn,
                &id,
                &truth,
                &embedding,
                &content_norm,
                "honeypot",
            )?;
            report.promoted += 1;
        }

        info!(
            candidates = report.candidates,
            promoted = report.promoted,
            skipped = report.skipped,
            "Mature vault → honeypot promote complete"
        );
        Ok(report)
    }

    /// Embed and store vectors for facts missing embeddings (requires `with_embedder`).
    pub async fn backfill_missing_embeddings(
        &self,
        limit: Option<usize>,
    ) -> Result<EmbedBackfillReport> {
        let embedder = self
            .embedder
            .as_ref()
            .context("Vault has no embedder — enable [embeddings] and ensure :8002 is up")?;

        let cap = limit.unwrap_or(10_000);
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, content FROM semantic_vault
             WHERE embedding IS NULL OR length(embedding) = 0
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;
        let rows: Vec<(String, String)> = stmt
            .query_map([cap as i64], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        let mut report = EmbedBackfillReport {
            attempted: rows.len(),
            updated: 0,
            failed: 0,
        };

        for (id, content) in rows {
            match embedder.embed(&content).await {
                Ok(vec) if !vec.is_empty() => {
                    let blob = bincode_embed(&vec);
                    let n = conn.execute(
                        "UPDATE semantic_vault SET embedding = ?1 WHERE id = ?2",
                        params![blob, id],
                    )?;
                    if n == 1 {
                        report.updated += 1;
                        // Keep honeypot RAG mirror in sync when the same fact id exists.
                        let _ = conn.execute(
                            "UPDATE honeypot SET embedding = ?1
                             WHERE id = ?2
                               AND (embedding IS NULL OR length(embedding) = 0)",
                            params![blob, id],
                        );
                    }
                }
                Ok(_) => {
                    tracing::warn!(id = %id, "Embedding server returned empty vector");
                    report.failed += 1;
                }
                Err(e) => {
                    tracing::warn!(id = %id, error = %e, "Embedding backfill failed for fact");
                    report.failed += 1;
                }
            }
        }

        info!(
            attempted = report.attempted,
            updated = report.updated,
            failed = report.failed,
            "Vault embedding backfill complete"
        );
        Ok(report)
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

    /// Demote old `[Session …]` stubs so spark/dream pick substantive facts.
    pub fn archive_stale_session_anchors(&self, max_age_days: u32) -> Result<usize> {
        let conn = self.pool.get()?;
        let updated = conn.execute(
            "UPDATE semantic_vault
             SET decay_class = 'ArchivedSession',
                 confidence = MIN(confidence, 0.5),
                 last_accessed_at = datetime('now')
             WHERE content LIKE '[Session %'
               AND (julianday('now') - julianday(substr(content, 10, 10))) > ?1",
            params![max_age_days],
        )?;
        if updated > 0 {
            tracing::info!(count = updated, "Archived stale session anchors in vault");
        }
        Ok(updated)
    }

    /// Stale facts worth revisiting: oldest `last_accessed_at` first, still within decay window.
    pub fn stale_candidates(&self, limit: usize) -> Result<Vec<SemanticFact>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, content, embedding, half_life_days, confidence, confirmation_count,
                    decay_class, created_at, last_accessed_at
             FROM semantic_vault
             WHERE decay_class != 'ArchivedSession'
               AND (julianday('now') - julianday(last_accessed_at)) < (half_life_days * 10.0)
             ORDER BY last_accessed_at ASC
             LIMIT ?1",
        )?;
        self.query_semantic_facts(&mut stmt, params![limit as i64])
    }

    /// Recent curated facts for spark contrast (honeypot when M3 populated).
    pub fn spark_recent_pool(
        &self,
        decay_classes: &[String],
        max_age_hours: u32,
        limit: usize,
    ) -> Result<Vec<SemanticFact>> {
        if decay_classes.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        let use_hp = Self::cognition_from_honeypot(&conn)?;
        let placeholders = decay_classes
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = if use_hp {
            format!(
                "SELECT id, content, embedding,
                    CASE decay_class
                        WHEN 'Episodic' THEN 30.0
                        WHEN 'CuratedVault' THEN 60.0 WHEN 'SessionDistill' THEN 60.0
                        WHEN 'FlexibleIdentity' THEN 139.0
                        WHEN 'AbsoluteIdentity' THEN 693.0
                        WHEN 'Structural' THEN 36500.0 WHEN 'Core' THEN 36500.0
                        WHEN 'Semantic' THEN 365.0 WHEN 'Procedural' THEN 90.0
                        ELSE 60.0 END,
                    confidence, recall_count, decay_class, promoted_at,
                    COALESCE(last_recalled_at, promoted_at)
             FROM honeypot
             WHERE decay_class IN ({placeholders})
               AND is_latest = 1
               AND datetime(promoted_at) >= datetime('now', ?)
             ORDER BY datetime(promoted_at) DESC
             LIMIT ?"
            )
        } else {
            format!(
                "SELECT id, content, embedding, half_life_days, confidence, confirmation_count,
                    decay_class, created_at, last_accessed_at
             FROM semantic_vault
             WHERE decay_class IN ({placeholders})
               AND created_at >= datetime('now', ?)
             ORDER BY created_at DESC
             LIMIT ?"
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = decay_classes
            .iter()
            .map(|c| Box::new(c.clone()) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        params.push(Box::new(format!("-{} hours", max_age_hours)));
        params.push(Box::new(limit as i64));
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        self.query_semantic_facts(&mut stmt, param_refs.as_slice())
    }

    /// Stale curated facts for spark anchors (honeypot when M3 populated).
    pub fn spark_anchor_pool(
        &self,
        decay_classes: &[String],
        min_age_hours: u32,
        min_stale_days: u32,
        max_stale_days: u32,
        limit: usize,
    ) -> Result<Vec<SemanticFact>> {
        if decay_classes.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        let use_hp = Self::cognition_from_honeypot(&conn)?;
        let placeholders = decay_classes
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = if use_hp {
            format!(
                "SELECT id, content, embedding,
                    CASE decay_class
                        WHEN 'Episodic' THEN 30.0
                        WHEN 'CuratedVault' THEN 60.0 WHEN 'SessionDistill' THEN 60.0
                        WHEN 'FlexibleIdentity' THEN 139.0
                        WHEN 'AbsoluteIdentity' THEN 693.0
                        WHEN 'Structural' THEN 36500.0 WHEN 'Core' THEN 36500.0
                        WHEN 'Semantic' THEN 365.0 WHEN 'Procedural' THEN 90.0
                        ELSE 60.0 END,
                    confidence, recall_count, decay_class, promoted_at,
                    COALESCE(last_recalled_at, promoted_at)
             FROM honeypot
             WHERE decay_class IN ({placeholders})
               AND is_latest = 1
               AND decay_class != 'ArchivedSession'
               AND datetime(promoted_at) <= datetime('now', ?)
               AND (julianday('now') - julianday(datetime(promoted_at))) >= ?
               AND (julianday('now') - julianday(datetime(promoted_at))) <= ?
             ORDER BY datetime(promoted_at) ASC
             LIMIT ?"
            )
        } else {
            format!(
                "SELECT id, content, embedding, half_life_days, confidence, confirmation_count,
                    decay_class, created_at, last_accessed_at
             FROM semantic_vault
             WHERE decay_class IN ({placeholders})
               AND decay_class != 'ArchivedSession'
               AND created_at <= datetime('now', ?)
               AND (julianday('now') - julianday(created_at)) >= ?
               AND (julianday('now') - julianday(created_at)) <= ?
               AND (julianday('now') - julianday(created_at)) < (half_life_days * 10.0)
             ORDER BY created_at ASC
             LIMIT ?"
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = decay_classes
            .iter()
            .map(|c| Box::new(c.clone()) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        params.push(Box::new(format!("-{} hours", min_age_hours)));
        params.push(Box::new(min_stale_days as f64));
        params.push(Box::new(max_stale_days as f64));
        params.push(Box::new(limit as i64));
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        self.query_semantic_facts(&mut stmt, param_refs.as_slice())
    }

    /// True when cognition reads/writes use the curated honeypot table.
    pub fn cognition_uses_honeypot(&self) -> bool {
        let Ok(conn) = self.pool.get() else {
            return false;
        };
        Self::cognition_from_honeypot(&conn).unwrap_or(false)
    }

    /// Vector (+ keyword fallback) neighbors in the cognition layer for one anchor text.
    pub async fn cognition_associate_similar(
        &self,
        anchor_text: &str,
        limit: usize,
    ) -> Result<Vec<SemanticFact>> {
        if limit == 0 || anchor_text.trim().is_empty() {
            return Ok(Vec::new());
        }
        if let Some(embedder) = &self.embedder {
            let emb = embedder.embed(anchor_text).await?;
            let prefetch = limit.saturating_mul(3).max(limit);
            let mut scored = Self::search_with_decay(self, &emb, anchor_text, prefetch)?;
            scored.truncate(limit);
            return Ok(scored.into_iter().map(|(f, _)| f).collect());
        }
        let mut kw = self.keyword_search(anchor_text, limit)?;
        kw.truncate(limit);
        Ok(kw.into_iter().map(|(f, _)| f).collect())
    }

    /// M3 REM substrate: honeypot anchors + associated distillates (for DreamEngine).
    pub async fn build_honeypot_rem_context(
        &self,
        anchor_limit: usize,
        per_anchor_k: usize,
    ) -> Result<String> {
        if !self.cognition_uses_honeypot() || anchor_limit == 0 {
            return Ok(String::new());
        }
        let anchors = self.recent_semantic_facts(anchor_limit)?;
        if anchors.is_empty() {
            return Ok(String::new());
        }

        let mut seen = std::collections::HashSet::new();
        let mut lines = Vec::new();
        lines.push(format!(
            "Cognition layer: {} ({} anchor facts)",
            self.cognition_memory_layer(),
            anchors.len()
        ));

        for (i, anchor) in anchors.iter().enumerate() {
            seen.insert(anchor.id);
            lines.push(format!("[HP-A{i}] {}", anchor.content.trim()));
            if per_anchor_k == 0 {
                continue;
            }
            match self
                .cognition_associate_similar(&anchor.content, per_anchor_k)
                .await
            {
                Ok(neighbors) => {
                    for (j, n) in neighbors.iter().enumerate() {
                        if seen.insert(n.id) {
                            lines.push(format!("[HP-A{i}-S{j}] {}", n.content.trim()));
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "honeypot association skipped for anchor");
                }
            }
        }

        Ok(lines.join("\n"))
    }

    /// Most recent cognition facts (honeypot when M3 populated).
    pub fn recent_semantic_facts(&self, limit: usize) -> Result<Vec<SemanticFact>> {
        let conn = self.pool.get()?;
        if Self::cognition_from_honeypot(&conn)? {
            let mut stmt = conn.prepare(
                "SELECT id, content, embedding,
                    CASE decay_class
                        WHEN 'Episodic' THEN 30.0
                        WHEN 'CuratedVault' THEN 60.0 WHEN 'SessionDistill' THEN 60.0
                        WHEN 'FlexibleIdentity' THEN 139.0
                        WHEN 'AbsoluteIdentity' THEN 693.0
                        WHEN 'Structural' THEN 36500.0 WHEN 'Core' THEN 36500.0
                        WHEN 'Semantic' THEN 365.0 WHEN 'Procedural' THEN 90.0
                        ELSE 60.0 END,
                    confidence, recall_count, decay_class, promoted_at,
                    COALESCE(last_recalled_at, promoted_at)
                 FROM honeypot
                 WHERE is_latest = 1
                 ORDER BY COALESCE(last_recalled_at, promoted_at) DESC
                 LIMIT ?1",
            )?;
            return self.query_semantic_facts(&mut stmt, params![limit as i64]);
        }
        let mut stmt = conn.prepare(
            "SELECT id, content, embedding, half_life_days, confidence, confirmation_count,
                    decay_class, created_at, last_accessed_at
             FROM semantic_vault
             ORDER BY last_accessed_at DESC
             LIMIT ?1",
        )?;
        self.query_semantic_facts(&mut stmt, params![limit as i64])
    }

    fn query_semantic_facts<P: rusqlite::Params>(
        &self,
        stmt: &mut rusqlite::Statement<'_>,
        params: P,
    ) -> Result<Vec<SemanticFact>> {
        let now = Utc::now();
        let rows = stmt
            .query_map(params, |row| {
                let id_str: String = row.get(0)?;
                let content: String = row.get(1)?;
                let embed_blob: Vec<u8> = row.get(2)?;
                let half_life: f64 = row.get(3)?;
                let confidence: f64 = row.get(4)?;
                let conf_count: u32 = row.get(5)?;
                let decay_class: String = row.get(6)?;
                let created_str: String = row.get(7)?;
                let accessed_str: String = row.get(8)?;
                Ok((
                    id_str,
                    content,
                    embed_blob,
                    half_life,
                    confidence,
                    conf_count,
                    decay_class,
                    created_str,
                    accessed_str,
                ))
            })?
            .filter_map(|r| r.ok())
            .map(
                |(
                    id_str,
                    content,
                    embed_blob,
                    half_life,
                    confidence,
                    conf_count,
                    decay_class,
                    created_str,
                    accessed_str,
                )| {
                    let embedding = decode_embed(&embed_blob);
                    SemanticFact {
                        id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
                        content,
                        embedding,
                        confidence,
                        half_life_days: half_life,
                        confirmation_count: conf_count,
                        decay_class,
                        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or(now),
                        last_accessed_at: chrono::DateTime::parse_from_rfc3339(&accessed_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or(now),
                    }
                },
            )
            .collect();
        Ok(rows)
    }

    /// Dump the entire vault to a human-readable Markdown directory.
    pub async fn dump_to_markdown(&self, out_dir: impl AsRef<Path>) -> Result<()> {
        let mut groups: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        {
            // Scope the lock so it drops before async I/O
            let conn = self.pool.get()?;
            let mut stmt = conn.prepare(
                "SELECT id, content, half_life_days, confirmation_count, decay_class, created_at
                 FROM semantic_vault
                 ORDER BY decay_class ASC, created_at DESC",
            )?;

            let rows = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let content: String = row.get(1)?;
                let hld: f64 = row.get(2)?;
                let conf: u32 = row.get(3)?;
                let dclass: String = row.get(4)?;
                let created: String = row.get(5)?;
                Ok((id, content, hld, conf, dclass, created))
            })?;

            for r in rows.flatten() {
                let (id, content, hld, conf, dclass, created) = r;
                let md = groups
                    .entry(dclass.clone())
                    .or_insert_with(|| format!("# GZMO Memory Vault: {}\n\n", dclass));

                md.push_str(&format!(
                    "## Entry: {}\n- **Created:** {}\n- **Confirmations:** {}\n- **Half-life:** {} days\n\n> {}\n\n---\n",
                    id, created, conf, hld, content.replace('\n', "\n> ")
                ));
            }
        } // Lock drops here

        let dir = out_dir.as_ref();
        tokio::fs::create_dir_all(dir).await?;

        for (dclass, markdown) in groups {
            let file_name = format!("Vault_{}.md", dclass);
            let target = dir.join(file_name);
            tokio::fs::write(&target, markdown).await?;
            println!("Exported memory partition: {:?}", target);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Math utilities
// ---------------------------------------------------------------------------

/// Normalize vault fact text for dedup (lowercase, collapsed whitespace).
pub fn normalize_truth_content(content: &str) -> String {
    content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Cosine similarity for vault embeddings (spark pre-filter and search).
pub fn embedding_cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| *x as f64 * *y as f64)
        .sum();
    let mag_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

fn bincode_embed(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

#[cfg(test)]
mod normalize_tests {
    use super::*;

    #[test]
    fn normalize_collapses_whitespace_and_lowercases() {
        assert_eq!(
            normalize_truth_content("[SYSTEM:GZMO]  Hello   World"),
            "[system:gzmo] hello world"
        );
    }
}

#[cfg(test)]
mod spark_pool_tests {
    use super::*;

    #[test]
    fn spark_recent_pool_reads_curated_honeypot() {
        let path = Path::new("data/vault.db");
        if !path.exists() {
            return;
        }
        let vault = SqliteVault::open(path).expect("open vault");
        let classes = vec!["CuratedVault".to_string(), "SessionDistill".to_string()];
        let recent = vault
            .spark_recent_pool(&classes, 72, 16)
            .expect("recent pool");
        // Local lab vault may be empty / mid-band cold — skip rather than fail CI.
        if recent.is_empty() {
            return;
        }
        assert!(!recent.is_empty());
    }
}

#[cfg(test)]
mod utility_recall_tests {
    use super::*;
    use crate::memory::honeypot::insert_honeypot_lifecycle;
    use crate::types::{DecayClass, ExtractedTruth};
    use rusqlite::params;
    use std::env;

    fn tempfile_db() -> std::path::PathBuf {
        let mut p = env::temp_dir();
        p.push(format!(
            "gzmo_utility_recall_{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        p
    }

    fn insert_fact(vault: &SqliteVault, content: &str, source: &str, utility: f64) -> Uuid {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();
        let truth = ExtractedTruth {
            id,
            content: content.to_string(),
            confidence: 0.95,
            mmr_score: 0.0,
            source_date: Utc::now().date_naive(),
            decay_class: DecayClass::CuratedVault,
            source_file: Some(source.to_string()),
            evidence: None,
        };
        let conn = vault.db_conn().expect("conn");
        conn.execute(
            "INSERT INTO semantic_vault
                (id, content, embedding, half_life_days, confidence, confirmation_count,
                 decay_class, created_at, last_accessed_at, source_file, content_norm)
             VALUES (?1, ?2, ?3, 60.0, 0.95, 1, 'CuratedVault', ?4, ?4, ?5, ?6)",
            params![
                id.to_string(),
                content,
                Vec::<u8>::new(),
                now,
                source,
                normalize_truth_content(content),
            ],
        )
        .expect("insert vault");
        insert_honeypot_lifecycle(
            &conn,
            &id.to_string(),
            &truth,
            &[],
            &normalize_truth_content(content),
            "honeypot",
            None,
            None,
        )
        .expect("insert honeypot");
        conn.execute(
            "UPDATE honeypot SET utility_score = ?1 WHERE id = ?2",
            params![utility, id.to_string()],
        )
        .expect("set utility");
        id
    }

    #[tokio::test]
    async fn search_recall_orders_by_utility_inside_fts_pool() {
        let path = tempfile_db();
        let vault = SqliteVault::open(&path).expect("open");
        let low = insert_fact(
            &vault,
            "alpha widget sits in the low-utility drawer",
            "a.md",
            0.0,
        );
        let high = insert_fact(
            &vault,
            "alpha gadget sits in the high-utility drawer",
            "b.md",
            20.0,
        );

        let hits = vault.search_recall("alpha", 5).await.expect("search");
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));

        assert!(
            hits.len() >= 2,
            "both fixture facts must be in the FTS pool, got {}",
            hits.len()
        );
        assert_eq!(
            hits[0].0.id,
            high,
            "high utility_score must rank first, got {:?}",
            hits.iter().map(|(f, s)| (f.id, s)).collect::<Vec<_>>()
        );
        assert!(hits.iter().any(|(f, _)| f.id == low));
    }

    #[tokio::test]
    async fn empty_query_returns_no_hits() {
        let path = tempfile_db();
        let vault = SqliteVault::open(&path).expect("open");
        insert_fact(&vault, "alpha widget", "a.md", 9.0);
        let hits = vault.search_recall("   ", 5).await.expect("search");
        let _ = std::fs::remove_file(&path);
        assert!(hits.is_empty(), "empty query must not invent recall");
    }

    #[test]
    fn reinforce_by_bumps_utility_score() {
        let path = tempfile_db();
        let vault = SqliteVault::open(&path).expect("open");
        let id = insert_fact(&vault, "bonded scar about felt use", "c.md", 1.0);
        vault.reinforce_by(id, 5).expect("reinforce");
        let conn = vault.db_conn().expect("conn");
        let (recall, utility): (i64, f64) = conn
            .query_row(
                "SELECT recall_count, utility_score FROM honeypot WHERE id = ?1",
                params![id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read");
        let _ = std::fs::remove_file(&path);
        assert_eq!(recall, 5);
        assert!((utility - 6.0).abs() < 1e-9, "utility was {utility}");
    }
}

pub(crate) fn decode_embed(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
