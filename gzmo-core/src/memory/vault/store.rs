//! Vault writes: store, reinforce (Felt Use), and point-in-time reads.

use super::embed::bincode_embed;
use super::SqliteVault;
use crate::memory::felt_use::{self, FeltUseKind};
use crate::memory::lifecycle::extract_primary_entity;
use crate::types::SemanticFact;
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use tracing::info;
use uuid::Uuid;

impl SqliteVault {
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

    /// Reinforce a fact: increment confirmation_count and reset decay clock.
    pub fn reinforce(&self, fact_id: Uuid) -> Result<()> {
        self.reinforce_by(fact_id, 1)
    }

    /// Graded Felt Use: bump vault confirmation + honeypot `recall_count` by `delta`.
    /// Utility is bumped by the same delta (Bonded-style). Prefer `reinforce_felt` for Glance.
    pub fn reinforce_by(&self, fact_id: Uuid, delta: i64) -> Result<()> {
        self.reinforce_felt(fact_id, delta, delta)
    }

    /// Split Felt Use: recall (ripen) vs utility (MemRL Q). Either delta may be 0.
    pub fn reinforce_felt(
        &self,
        fact_id: Uuid,
        recall_delta: i64,
        utility_delta: i64,
    ) -> Result<()> {
        if recall_delta <= 0 && utility_delta <= 0 {
            return Ok(());
        }
        let conn = self.pool.get()?;
        let now = Utc::now().to_rfc3339();
        let id = fact_id.to_string();
        if recall_delta > 0 {
            conn.execute(
                "UPDATE semantic_vault
                 SET confirmation_count = confirmation_count + ?1,
                     last_accessed_at = ?2
                 WHERE id = ?3",
                params![recall_delta, now, id],
            )?;
        }
        let _ = conn.execute(
            "UPDATE honeypot
             SET recall_count = recall_count + ?1,
                 last_recalled_at = CASE WHEN ?1 > 0 THEN ?2 ELSE last_recalled_at END,
                 utility_score = utility_score + CAST(?3 AS REAL)
             WHERE id = ?4",
            params![recall_delta.max(0), now, utility_delta.max(0), id],
        );
        info!(
            fact_id = %fact_id,
            recall_delta,
            utility_delta,
            "Reinforced semantic fact (felt use + utility)"
        );
        Ok(())
    }

    /// SuperLocalMemory steal: facts valid at `as_of` (RFC3339), including superseded.
    pub fn honeypot_as_of(&self, as_of: &str, limit: usize) -> Result<Vec<(String, String, bool)>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, content, is_latest FROM honeypot
             WHERE datetime(COALESCE(valid_from, promoted_at)) <= datetime(?1)
               AND (valid_to IS NULL OR datetime(valid_to) > datetime(?1))
             ORDER BY COALESCE(valid_from, promoted_at) DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![as_of, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)? != 0,
            ))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// MemRL outcome: bump Q on previously recalled facts whose entity the new takeaway cites.
    pub fn reinforce_outcome_from_new_truths(
        &self,
        truths: &[crate::types::ExtractedTruth],
    ) -> Result<usize> {
        let mut n = 0usize;
        let conn = self.pool.get()?;
        for truth in truths {
            let Some(entity) = extract_primary_entity(&truth.content) else {
                continue;
            };
            let pattern = format!("%{}%", entity.replace('%', ""));
            let mut stmt = conn.prepare(
                "SELECT id FROM honeypot
                 WHERE is_latest = 1
                   AND last_recalled_at IS NOT NULL
                   AND id != ?1
                   AND content LIKE ?2
                 LIMIT 8",
            )?;
            let ids: Vec<String> = stmt
                .query_map(params![truth.id.to_string(), pattern], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            for id_str in ids {
                let Ok(uuid) = Uuid::parse_str(&id_str) else {
                    continue;
                };
                felt_use::touch(self, uuid, FeltUseKind::Outcome)?;
                n += 1;
            }
        }
        Ok(n)
    }

    /// Auto-Dreamer: compact replacement supersedes other latest rows for the same entity.
    pub(super) fn region_rewrite_entity(
        conn: &Connection,
        entity: &str,
        keep_id: &str,
    ) -> Result<usize> {
        let now = Utc::now().to_rfc3339();
        let pattern = format!("%{}%", entity.replace('%', ""));
        let n = conn.execute(
            "UPDATE honeypot
             SET is_latest = 0,
                 valid_to = COALESCE(valid_to, ?1),
                 gate_event = 'region_rewrite'
             WHERE is_latest = 1
               AND id != ?2
               AND content LIKE ?3",
            params![now, keep_id, pattern],
        )?;
        Ok(n)
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
}
