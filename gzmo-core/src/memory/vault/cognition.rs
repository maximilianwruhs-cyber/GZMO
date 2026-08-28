//! Cognition layer selection (vault vs curated honeypot) and association reads.

use super::SqliteVault;
use crate::types::SemanticFact;
use anyhow::Result;
use rusqlite::Connection;

impl SqliteVault {
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
}
