//! SQLite-backed semantic vault with temporal decay.
//!
//! Implements hybrid search blending cosine similarity on stored embeddings
//! with keyword matching and exponential half-life decay.

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;
use tracing::info;
use uuid::Uuid;

use crate::types::{ExtractedTruth, SemanticFact};

/// The permanent semantic vault backed by SQLite.
pub struct SqliteVault {
    conn: Mutex<Connection>,
}

impl SqliteVault {
    /// Open or create the vault database.
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path.as_ref())
            .with_context(|| "Failed to open semantic vault database")?;

        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS semantic_vault (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB,
                half_life_days REAL NOT NULL DEFAULT 30.0,
                confidence REAL NOT NULL DEFAULT 1.0,
                confirmation_count INTEGER NOT NULL DEFAULT 0,
                decay_class TEXT NOT NULL DEFAULT 'Episodic',
                created_at TEXT NOT NULL,
                last_accessed_at TEXT NOT NULL
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

        // Non-destructive schema migration (fails silently if column already exists)
        let _ = conn.execute("ALTER TABLE semantic_vault ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0", []);

        info!("Semantic vault initialized");
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Store a new semantic fact.
    pub fn store(&self, fact: &SemanticFact) -> Result<()> {
        let conn = self.conn.lock().unwrap();

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
                "Episodic",
                fact.created_at.to_rfc3339(),
                fact.last_accessed_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Retrieve items placed in quarantine awaiting HITL validation
    pub fn list_quarantine(&self) -> Result<Vec<(String, String, f64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, content, confidence, created_at FROM quarantine_vault ORDER BY created_at DESC")?;
        let results = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    /// Reinforce a fact: increment confirmation_count and reset decay clock.
    pub fn reinforce(&self, fact_id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE semantic_vault
             SET confirmation_count = confirmation_count + 1,
                 last_accessed_at = ?1
             WHERE id = ?2",
            params![now, fact_id.to_string()],
        )?;
        info!(fact_id = %fact_id, "Reinforced semantic fact");
        Ok(())
    }

    /// Search with temporal decay applied in Rust.
    /// Returns facts sorted by decayed relevance score.
    pub fn search_with_decay(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<(SemanticFact, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, embedding, half_life_days, confirmation_count,
                    created_at, last_accessed_at
             FROM semantic_vault",
        )?;

        let now = Utc::now();
        let query_lower = query_text.to_lowercase();

        let mut scored: Vec<(SemanticFact, f64)> = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let content: String = row.get(1)?;
                let embed_blob: Vec<u8> = row.get(2)?;
                let half_life: f64 = row.get(3)?;
                let conf_count: u32 = row.get(4)?;
                let created_str: String = row.get(5)?;
                let accessed_str: String = row.get(6)?;

                Ok((
                    id_str,
                    content,
                    embed_blob,
                    half_life,
                    conf_count,
                    created_str,
                    accessed_str,
                ))
            })?
            .filter_map(|r| r.ok())
            .map(
                |(id_str, content, embed_blob, half_life, conf_count, created_str, accessed_str)| {
                    let embedding = decode_embed(&embed_blob);

                    // Vector cosine similarity (0.7 weight)
                    let vec_sim = cosine_similarity(query_embedding, &embedding);

                    // Simple keyword match score (0.3 weight)
                    let content_lower = content.to_lowercase();
                    let keyword_score = query_lower
                        .split_whitespace()
                        .filter(|w| content_lower.contains(w))
                        .count() as f64
                        / query_lower.split_whitespace().count().max(1) as f64;

                    let raw_score = (vec_sim * 0.7) + (keyword_score * 0.3);

                    // Temporal decay: score * 0.5^(effective_days / half_life)
                    let accessed_at = chrono::DateTime::parse_from_rfc3339(&accessed_str)
                        .unwrap_or_else(|_| now.into());
                    let days_elapsed =
                        (now - accessed_at.with_timezone(&Utc)).num_seconds() as f64 / 86400.0;
                    let effective_days = (days_elapsed - (conf_count as f64 * 5.0)).max(0.0);
                    let decay_multiplier = 0.5_f64.powf(effective_days / half_life);

                    let decayed_score = raw_score * decay_multiplier;

                    let fact = SemanticFact {
                        id: Uuid::parse_str(&id_str).unwrap_or_default(),
                        content,
                        embedding,
                        confidence: 1.0,
                        half_life_days: half_life,
                        confirmation_count: conf_count,
                        created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or(now),
                        last_accessed_at: accessed_at.with_timezone(&Utc),
                    };

                    (fact, decayed_score)
                },
            )
            .collect();

        // Sort descending by decayed score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored)
    }

    /// Promote extracted truths from the dream cycle into permanent storage.
    pub fn promote_truths(&self, truths: &[ExtractedTruth]) -> Result<()> {
        for truth in truths {
            // Check if a similar fact already exists (by content hash)
            let conn = self.conn.lock().unwrap();
            let existing: Option<String> = conn
                .query_row(
                    "SELECT id FROM semantic_vault WHERE content = ?1",
                    params![truth.content],
                    |row| row.get(0),
                )
                .ok();

            if let Some(existing_id) = existing {
                // Corroboration: increment confirmation_count, reset decay
                let now = Utc::now().to_rfc3339();
                conn.execute(
                    "UPDATE semantic_vault
                     SET confirmation_count = confirmation_count + 1,
                         last_accessed_at = ?1
                     WHERE id = ?2",
                    params![now, existing_id],
                )?;
                info!(id = %existing_id, "Corroborated existing truth");
            } else {
                // New truth: insert with appropriate decay class
                let now = Utc::now();
                conn.execute(
                    "INSERT INTO semantic_vault
                        (id, content, embedding, half_life_days, confirmation_count,
                         decay_class, created_at, last_accessed_at)
                    VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?7)",
                    params![
                        truth.id.to_string(),
                        truth.content,
                        Vec::<u8>::new(), // Embedding computed later
                        truth.decay_class.half_life_days(),
                        format!("{:?}", truth.decay_class),
                        now.to_rfc3339(),
                        now.to_rfc3339(),
                    ],
                )?;
                info!(id = %truth.id, "Promoted new truth to vault");
            }
        }
        Ok(())
    }

    /// Metacognitive guard: recall past failures for a given command/context.
    pub fn recall_failures(&self, description: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT content FROM semantic_vault
             WHERE content LIKE '%error%' OR content LIKE '%failed%' OR content LIKE '%warning%'
             ORDER BY last_accessed_at DESC LIMIT 5",
        )?;

        let results: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
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

    /// Keyword-only search (no embeddings required).
    /// Returns facts where the content matches any of the query keywords,
    /// sorted by recency with temporal decay applied.
    pub fn keyword_search(&self, query_text: &str, limit: usize) -> Result<Vec<(SemanticFact, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, content, embedding, half_life_days, confirmation_count,
                    created_at, last_accessed_at
             FROM semantic_vault",
        )?;

        let now = Utc::now();
        let query_lower = query_text.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        if query_words.is_empty() {
            return Ok(Vec::new());
        }

        let mut scored: Vec<(SemanticFact, f64)> = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let content: String = row.get(1)?;
                let embed_blob: Vec<u8> = row.get(2)?;
                let half_life: f64 = row.get(3)?;
                let conf_count: u32 = row.get(4)?;
                let created_str: String = row.get(5)?;
                let accessed_str: String = row.get(6)?;
                Ok((id_str, content, embed_blob, half_life, conf_count, created_str, accessed_str))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(id_str, content, embed_blob, half_life, conf_count, created_str, accessed_str)| {
                let content_lower = content.to_lowercase();

                // BM25-style keyword matching
                let matched = query_words.iter().filter(|w| content_lower.contains(**w)).count();
                if matched == 0 {
                    return None;
                }

                let keyword_score = matched as f64 / query_words.len() as f64;

                // Temporal decay
                let accessed_at = chrono::DateTime::parse_from_rfc3339(&accessed_str)
                    .unwrap_or_else(|_| now.into());
                let days_elapsed = (now - accessed_at.with_timezone(&Utc)).num_seconds() as f64 / 86400.0;
                let effective_days = (days_elapsed - (conf_count as f64 * 5.0)).max(0.0);
                let decay_multiplier = 0.5_f64.powf(effective_days / half_life);

                let decayed_score = keyword_score * decay_multiplier;

                let embedding = decode_embed(&embed_blob);
                let fact = SemanticFact {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    content,
                    embedding,
                    confidence: 1.0,
                    half_life_days: half_life,
                    confirmation_count: conf_count,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or(now),
                    last_accessed_at: accessed_at.with_timezone(&Utc),
                };

                Some((fact, decayed_score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored)
    }

    /// Store a plain text fact (no embedding). For use by the interactive REPL
    /// when no embedding model is available.
    pub fn store_text(&self, content: &str, decay_class: &str, confidence: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let id = Uuid::new_v4();

        let half_life = match decay_class {
            "Core" => 365.0 * 100.0,
            "Semantic" => 365.0,
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

    /// Get the total number of facts in the vault.
    pub fn count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: usize = conn.query_row(
            "SELECT COUNT(*) FROM semantic_vault",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get the most recent N facts (for context injection).
    pub fn recent(&self, limit: usize) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT content FROM semantic_vault
             ORDER BY last_accessed_at DESC
             LIMIT ?1",
        )?;

        let results: Vec<String> = stmt
            .query_map(params![limit as i64], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    /// Dump the entire vault to a human-readable Markdown directory.
    pub async fn dump_to_markdown(&self, out_dir: impl AsRef<Path>) -> Result<()> {
        let mut groups: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        { // Scope the lock so it drops before async I/O
            let conn = self.conn.lock().unwrap();
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
                let md = groups.entry(dclass.clone()).or_insert_with(|| {
                    format!("# GZMO Memory Vault: {}\n\n", dclass)
                });
                
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

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| *x as f64 * *y as f64).sum();
    let mag_a: f64 = a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

fn bincode_embed(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

fn decode_embed(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
