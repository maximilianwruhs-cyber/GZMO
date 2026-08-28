//! Hybrid recall: decayed vector/keyword search, RRF fusion, rerank, MemRL Q-select.

use super::{decode_embed, embedding_cosine_similarity, SqliteVault};
use crate::memory::recall_rrf::{
    apply_utility_boost, diversify_by_source_file, merge_interleaved_rank, rrf_fuse,
    RecallCandidate, PREFETCH_K, QDRANT_PREFETCH_K, RERANK_PREFETCH,
};
use crate::types::SemanticFact;
use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use std::collections::HashMap;
use uuid::Uuid;

impl SqliteVault {
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
                        if let Ok(ids) = qdrant.search_ids(&emb, QDRANT_PREFETCH_K).await {
                            qdrant_ids = self.take_assertable_prefetch(&ids, PREFETCH_K)?;
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

    pub(super) fn query_semantic_facts<P: rusqlite::Params>(
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
}
