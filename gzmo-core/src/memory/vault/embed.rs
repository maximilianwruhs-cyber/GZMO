//! Embedding storage: backfill, blob codec, and vector math.

use super::{EmbedBackfillReport, SqliteVault};
use anyhow::{Context, Result};
use rusqlite::params;
use tracing::info;

impl SqliteVault {
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
}

// ---------------------------------------------------------------------------
// Math utilities
// ---------------------------------------------------------------------------

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

pub(super) fn bincode_embed(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub(crate) fn decode_embed(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
