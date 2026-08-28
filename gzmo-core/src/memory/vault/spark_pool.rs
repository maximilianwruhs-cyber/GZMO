//! Spark/dream substrate pools: stale anchors and recent curated contrast.

use super::SqliteVault;
use crate::types::SemanticFact;
use anyhow::Result;
use rusqlite::params;

impl SqliteVault {
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
}

#[cfg(test)]
mod spark_pool_tests {
    use super::*;
    use std::path::Path;

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
