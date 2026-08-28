//! Promotion pipeline: extracted truths into vault rows and the curated honeypot.

use super::embed::bincode_embed;
use super::{
    decode_embed, normalize_truth_content, parse_decay_class, PromoteMatureReport, SqliteVault,
};
use crate::memory::core_pin;
use crate::memory::felt_use::{self, FeltUseKind};
use crate::memory::honeypot::{self, qualifies_for_honeypot};
use crate::memory::lifecycle::{
    classify_truth_pair, extract_primary_entity, find_latest_honeypot_by_entity,
    is_unverified_derived, supersede_honeypot, LifecycleKind,
};
use crate::memory::recall_rrf::RecallCandidate;
use crate::types::{ExtractedTruth, SemanticFact};
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use tracing::info;
use uuid::Uuid;

impl SqliteVault {
    pub(super) fn load_honeypot_candidate(&self, id: Uuid) -> Result<Option<RecallCandidate>> {
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
                            Self::maybe_region_rewrite(conn, origin, truth);
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
                            Self::maybe_region_rewrite(conn, origin, truth);
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
            Self::maybe_region_rewrite(conn, origin, truth);
        }
        Ok(())
    }

    fn maybe_region_rewrite(conn: &Connection, origin: &str, truth: &ExtractedTruth) {
        if origin != "verified_dream" {
            return;
        }
        let Some(entity) = extract_primary_entity(&truth.content) else {
            return;
        };
        if let Err(e) = Self::region_rewrite_entity(conn, &entity, &truth.id.to_string()) {
            tracing::debug!(error = %e, "region rewrite skipped");
        }
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
                conn.execute_batch("SAVEPOINT promote_one")?;
                let one = (|| -> Result<()> {
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
                        Self::record_failure_case_conn(
                            &conn,
                            "verify_fail",
                            &truth.content,
                            Some(&truth.id.to_string()),
                        )?;
                        tracing::warn!(
                            id = %truth.id,
                            confidence,
                            "Ingest truth quarantined due to low confidence"
                        );
                        return Ok(());
                    }

                    if is_unverified_derived(truth, origin) && qualifies_for_honeypot(truth) {
                        Self::record_failure_case_conn(
                            &conn,
                            "gate_refuse",
                            &truth.content,
                            Some(&truth.id.to_string()),
                        )?;
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
                    Ok(())
                })();
                match one {
                    Ok(()) => conn.execute_batch("RELEASE promote_one")?,
                    Err(e) => {
                        let _ = conn.execute_batch("ROLLBACK TO promote_one");
                        let _ = conn.execute_batch("RELEASE promote_one");
                        Self::record_failure_case_conn(
                            &conn,
                            "promote_rollback",
                            &format!("{e}; {}", truth.content),
                            Some(&truth.id.to_string()),
                        )?;
                        tracing::warn!(id = %truth.id, error = %e, "Promote rolled back (savepoint)");
                    }
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
                if origin == "session_distill" {
                    if let Err(e) = self.reinforce_outcome_from_new_truths(truths) {
                        tracing::debug!(error = %e, "outcome-link skipped");
                    }
                }
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
}
