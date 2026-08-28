//! Honeypot recall streams: FTS sync/queries, evidence vectors, graph hints.

use super::{decode_embed, embedding_cosine_similarity, SqliteVault};
use crate::memory::recall_rrf::{extract_entity_tokens, fts_match_query, fts_match_query_broad};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use std::process::Command as StdCommand;
use uuid::Uuid;

impl SqliteVault {
    pub(super) fn ensure_honeypot_fts_synced(&self, conn: &Connection) -> Result<()> {
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

    pub(super) fn ensure_evidence_fts_synced(&self, conn: &Connection) -> Result<()> {
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

    pub(super) fn honeypot_fts_stream(
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

    pub(super) fn honeypot_evidence_fts_stream(
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

    pub(super) fn honeypot_evidence_vector_stream(
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

    pub(super) fn honeypot_keyword_stream(&self, query: &str, limit: usize) -> Result<Vec<Uuid>> {
        let scored = self.keyword_search(query, limit)?;
        Ok(scored.into_iter().map(|(f, _)| f.id).collect())
    }

    /// Graph stream: Neo4j hints (optional) mapped to honeypot rows; SQLite entity overlap fallback.
    pub(super) fn honeypot_graph_stream(&self, query: &str, limit: usize) -> Result<Vec<Uuid>> {
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
}
