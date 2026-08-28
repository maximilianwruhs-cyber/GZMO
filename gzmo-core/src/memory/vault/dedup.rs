//! Idempotence: distill/ingest dedup ledgers and temporal-validity prefetch filters.

use super::SqliteVault;
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, params_from_iter};
use std::collections::HashSet;
use uuid::Uuid;

impl SqliteVault {
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

    /// GPM / Temporal Validity: current-time rank lists may only contain
    /// `is_latest = 1` ids. Superseded Qdrant hits must not occupy prefetch.
    pub fn filter_assertable_honeypot_ids(&self, ids: &[Uuid]) -> Result<Vec<Uuid>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        let id_strs: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let placeholders = id_strs.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("SELECT id FROM honeypot WHERE is_latest = 1 AND id IN ({placeholders})");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(id_strs.iter()), |row| {
            row.get::<_, String>(0)
        })?;
        let mut latest = HashSet::new();
        for id in rows.flatten() {
            latest.insert(id);
        }
        Ok(ids
            .iter()
            .copied()
            .filter(|id| latest.contains(&id.to_string()))
            .collect())
    }

    /// Drop superseded Qdrant hits then cap. Callers overfetch (`QDRANT_PREFETCH_K`)
    /// so stale points do not shrink the vector list below `cap`.
    pub fn take_assertable_prefetch(&self, ids: &[Uuid], cap: usize) -> Result<Vec<Uuid>> {
        let filtered = self.filter_assertable_honeypot_ids(ids)?;
        Ok(filtered.into_iter().take(cap).collect())
    }
}
