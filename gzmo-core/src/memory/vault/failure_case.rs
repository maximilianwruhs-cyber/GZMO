//! Memento failure cases: record verify/gate refusals and recall them.

use super::{FailureCaseHit, SqliteVault, FAILURE_CASE_RECALL_LIMIT};
use crate::memory::recall_rrf::extract_entity_tokens;
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use uuid::Uuid;

impl SqliteVault {
    /// Memento: persist a verify/gate failure without polluting honeypot recall.
    pub fn record_failure_case(
        &self,
        kind: &str,
        content: &str,
        related_fact_id: Option<&str>,
    ) -> Result<()> {
        let conn = self.pool.get()?;
        Self::record_failure_case_conn(&conn, kind, content, related_fact_id)
    }

    pub(super) fn record_failure_case_conn(
        conn: &Connection,
        kind: &str,
        content: &str,
        related_fact_id: Option<&str>,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO failure_cases (id, kind, content, related_fact_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, kind, content, related_fact_id, now],
        )?;
        Ok(())
    }

    /// Memento retrieve: bounded `failure_cases` matching query tokens and/or
    /// related honeypot ids. Empty/short queries return nothing.
    pub fn search_failure_cases(
        &self,
        query: &str,
        related_fact_ids: &[Uuid],
        limit: usize,
    ) -> Result<Vec<FailureCaseHit>> {
        let cap = limit.min(FAILURE_CASE_RECALL_LIMIT);
        if cap == 0 {
            return Ok(Vec::new());
        }
        let tokens: Vec<String> = extract_entity_tokens(query)
            .into_iter()
            .map(|t| t.to_lowercase())
            .collect();
        let related: HashSet<String> = related_fact_ids.iter().map(|id| id.to_string()).collect();
        if tokens.is_empty() && related.is_empty() {
            return Ok(Vec::new());
        }
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT kind, content, related_fact_id FROM failure_cases
             ORDER BY created_at DESC LIMIT 64",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows.filter_map(|r| r.ok()) {
            let (kind, content, related_id) = row;
            let content_l = content.to_lowercase();
            let token_hit = tokens.iter().any(|t| content_l.contains(t.as_str()));
            let related_hit = related_id.as_ref().is_some_and(|id| related.contains(id));
            if token_hit || related_hit {
                out.push(FailureCaseHit {
                    kind,
                    content,
                    related_fact_id: related_id,
                });
                if out.len() >= cap {
                    break;
                }
            }
        }
        Ok(out)
    }

    /// Format a recall appendix (empty string when nothing matches).
    pub fn format_failure_recall(&self, query: &str, related_fact_ids: &[Uuid]) -> Result<String> {
        let hits = self.search_failure_cases(query, related_fact_ids, FAILURE_CASE_RECALL_LIMIT)?;
        if hits.is_empty() {
            return Ok(String::new());
        }
        let mut out = String::from("\nPrior refusals (not promoted):\n");
        for h in &hits {
            out.push_str(&format!("- [{}] {}\n", h.kind, h.content));
        }
        Ok(out)
    }

    /// Metacognitive guard: recall past `failure_cases` for a command/context.
    pub fn recall_failures(&self, description: &str) -> Result<Vec<String>> {
        Ok(self
            .search_failure_cases(description, &[], FAILURE_CASE_RECALL_LIMIT)?
            .into_iter()
            .map(|h| format!("[{}] {}", h.kind, h.content))
            .collect())
    }
}
