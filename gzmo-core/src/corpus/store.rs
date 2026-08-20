//! SQLite FTS5-backed corpus passage store — separate from the promoted-fact
//! vault (`semantic_vault`/`honeypot`). See
//! `docs/superpowers/specs/2026-08-20-gzmo-demo-design.md`.

use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::memory::vault::SqliteVault;

/// One deterministically-chunked passage from a corpus source file.
///
/// `id` is `sha256:<file-content-sha256>:<zero-based-chunk-index>` so
/// re-ingesting unchanged content is idempotent and identifiable across runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorpusPassage {
    pub id: String,
    pub source_path: String,
    pub chunk_index: usize,
    pub content: String,
    pub content_sha256: String,
}

/// A single BM25-ranked full-text search result over the corpus store.
#[derive(Debug, Clone, PartialEq)]
pub struct CorpusSearchHit {
    pub passage: CorpusPassage,
    /// SQLite FTS5 `bm25()` rank — more negative is a better match.
    pub rank: f64,
}

/// SQLite FTS5-backed store for ingested corpus passages.
///
/// Reuses the vault's existing r2d2 connection pool (`SqliteVault::db_conn`)
/// rather than opening a second SQLite file, but keeps its own tables
/// (`corpus_passages` / `corpus_passages_fts`) fully separate from the
/// promoted-fact vault tables (`semantic_vault` / `honeypot`).
#[derive(Clone)]
pub struct CorpusStore {
    vault: SqliteVault,
}

impl CorpusStore {
    /// Wrap an already-open vault, creating the corpus tables if needed.
    pub fn new(vault: SqliteVault) -> Result<Self> {
        let conn = vault.db_conn()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS corpus_passages (
                id TEXT PRIMARY KEY,
                source_path TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                content_sha256 TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS corpus_passages_fts USING fts5(
                content,
                content='corpus_passages',
                content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS corpus_passages_ai AFTER INSERT ON corpus_passages BEGIN
                INSERT INTO corpus_passages_fts(rowid, content) VALUES (new.rowid, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS corpus_passages_ad AFTER DELETE ON corpus_passages BEGIN
                INSERT INTO corpus_passages_fts(corpus_passages_fts, rowid, content)
                    VALUES('delete', old.rowid, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS corpus_passages_au AFTER UPDATE ON corpus_passages BEGIN
                INSERT INTO corpus_passages_fts(corpus_passages_fts, rowid, content)
                    VALUES('delete', old.rowid, old.content);
                INSERT INTO corpus_passages_fts(rowid, content) VALUES (new.rowid, new.content);
            END;",
        )?;
        Ok(Self { vault })
    }

    /// Insert or update a passage. Idempotent: re-upserting an unchanged
    /// passage (same source_path/content_sha256) is a no-op that does not
    /// touch `updated_at` or fire the FTS sync triggers.
    pub fn upsert(&self, passage: &CorpusPassage) -> Result<()> {
        let conn = self.vault.db_conn()?;
        conn.execute(
            "INSERT INTO corpus_passages
                (id, source_path, chunk_index, content, content_sha256, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                source_path = excluded.source_path,
                chunk_index = excluded.chunk_index,
                content = excluded.content,
                content_sha256 = excluded.content_sha256,
                updated_at = excluded.updated_at
             WHERE corpus_passages.content_sha256 IS NOT excluded.content_sha256
                OR corpus_passages.source_path IS NOT excluded.source_path
                OR corpus_passages.chunk_index IS NOT excluded.chunk_index",
            params![
                passage.id,
                passage.source_path,
                passage.chunk_index as i64,
                passage.content,
                passage.content_sha256,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// BM25-ranked full-text search over ingested passages.
    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<CorpusSearchHit>> {
        let conn = self.vault.db_conn()?;
        let mut stmt = conn.prepare(
            "SELECT p.id, p.source_path, p.chunk_index, p.content, p.content_sha256,
                    bm25(corpus_passages_fts) AS rank
             FROM corpus_passages_fts
             JOIN corpus_passages p ON p.rowid = corpus_passages_fts.rowid
             WHERE corpus_passages_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Ok(CorpusSearchHit {
                passage: CorpusPassage {
                    id: row.get(0)?,
                    source_path: row.get(1)?,
                    chunk_index: row.get::<_, i64>(2)? as usize,
                    content: row.get(3)?,
                    content_sha256: row.get(4)?,
                },
                rank: row.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::vault::SqliteVault;

    #[test]
    fn fts_keeps_source_and_chunk_identity() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "gzmo-corpus-test-{}.db",
            uuid::Uuid::new_v4()
        ));
        let vault = SqliteVault::open(&path)?;
        let store = CorpusStore::new(vault)?;
        store.upsert(&CorpusPassage {
            id: "sha256:abc:0".into(),
            source_path: "orion-lantern.md".into(),
            chunk_index: 0,
            content: "The calibration phrase is cobalt finch 731.".into(),
            content_sha256: "abc".into(),
        })?;
        let hits = store.search_fts("calibration phrase", 5)?;
        assert_eq!(hits[0].passage.source_path, "orion-lantern.md");
        assert_eq!(hits[0].passage.chunk_index, 0);
        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    fn temp_store() -> Result<(CorpusStore, std::path::PathBuf)> {
        let path = std::env::temp_dir().join(format!(
            "gzmo-corpus-test-{}.db",
            uuid::Uuid::new_v4()
        ));
        let vault = SqliteVault::open(&path)?;
        Ok((CorpusStore::new(vault)?, path))
    }

    #[test]
    fn repeated_upsert_of_unchanged_passage_is_idempotent() -> Result<()> {
        let (store, path) = temp_store()?;
        let passage = CorpusPassage {
            id: "sha256:def:0".into(),
            source_path: "notes.md".into(),
            chunk_index: 0,
            content: "Repeat ingestion should not duplicate rows.".into(),
            content_sha256: "def".into(),
        };
        store.upsert(&passage)?;
        store.upsert(&passage)?;
        let hits = store.search_fts("duplicate", 10)?;
        assert_eq!(hits.len(), 1);
        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn upsert_with_changed_content_updates_and_stays_searchable() -> Result<()> {
        let (store, path) = temp_store()?;
        let id = "sha256:ghi:0".to_string();
        store.upsert(&CorpusPassage {
            id: id.clone(),
            source_path: "notes.md".into(),
            chunk_index: 0,
            content: "Original wording about zephyr.".into(),
            content_sha256: "ghi".into(),
        })?;
        store.upsert(&CorpusPassage {
            id: id.clone(),
            source_path: "notes.md".into(),
            chunk_index: 0,
            content: "Updated wording about marigold.".into(),
            content_sha256: "jkl".into(),
        })?;
        assert!(store.search_fts("zephyr", 10)?.is_empty());
        let hits = store.search_fts("marigold", 10)?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].passage.id, id);
        let _ = std::fs::remove_file(&path);
        Ok(())
    }
}
