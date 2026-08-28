//! Read-only introspection: quarantine, census, chains, and Markdown dumps.

use super::SqliteVault;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

impl SqliteVault {
    /// Retrieve items placed in quarantine awaiting HITL validation
    pub fn list_quarantine(&self) -> Result<Vec<(String, String, f64, String)>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, content, confidence, created_at FROM quarantine_vault ORDER BY created_at DESC")?;
        let results = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    /// Census for M5 export gates (`gzmo ripen status` / overnight honesty).
    pub fn ripen_gate_census(
        &self,
        min_confidence: f64,
        min_recall: i64,
    ) -> Result<crate::memory::ripen::RipenGateCensus> {
        let conn = self.pool.get()?;
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='honeypot'",
            [],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Ok(crate::memory::ripen::RipenGateCensus {
                latest: 0,
                nonzero_recall: 0,
                dual: 0,
                dual_origin: 0,
            });
        }
        let latest: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1",
            [],
            |r| r.get(0),
        )?;
        let nonzero_recall: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1 AND recall_count > 0",
            [],
            |r| r.get(0),
        )?;
        let dual: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot
             WHERE is_latest = 1 AND confidence >= ?1 AND recall_count >= ?2",
            params![min_confidence, min_recall],
            |r| r.get(0),
        )?;
        let dual_origin: i64 = conn.query_row(
            "SELECT COUNT(*) FROM honeypot
             WHERE is_latest = 1 AND confidence >= ?1 AND recall_count >= ?2
               AND origin IN ('ingest','verified_dream','session_distill')",
            params![min_confidence, min_recall],
            |r| r.get(0),
        )?;
        Ok(crate::memory::ripen::RipenGateCensus {
            latest,
            nonzero_recall,
            dual,
            dual_origin,
        })
    }

    /// Row count in a sibling `knowledge_core.db` (separate file).
    pub fn knowledge_core_row_count(&self, core_path: &Path) -> Result<i64> {
        if !core_path.exists() {
            anyhow::bail!("knowledge_core missing: {}", core_path.display());
        }
        let conn = Connection::open(core_path)?;
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM knowledge_core", [], |r| r.get(0))?;
        Ok(n)
    }

    /// Latest honeypot rows matching SQL LIKE pattern (immune patrol / ops).
    pub fn honeypot_latest_matching(
        &self,
        like_pattern: &str,
        limit: usize,
    ) -> Result<Vec<(Uuid, String, f64)>> {
        let conn = self.pool.get()?;
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='honeypot'",
            [],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Ok(Vec::new());
        }
        let mut stmt = conn.prepare(
            "SELECT id, content, confidence FROM honeypot
             WHERE is_latest = 1 AND content LIKE ?1
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![like_pattern, limit as i64], |row| {
                let id_s: String = row.get(0)?;
                let content: String = row.get(1)?;
                let confidence: f64 = row.get(2)?;
                Ok((id_s, content, confidence))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(id_s, content, confidence)| {
                Uuid::parse_str(&id_s)
                    .ok()
                    .map(|id| (id, content, confidence))
            })
            .collect();
        Ok(rows)
    }

    pub fn honeypot_source_file(&self, id: Uuid) -> Result<Option<String>> {
        let conn = self.pool.get()?;
        conn.query_row(
            "SELECT source_file FROM honeypot WHERE id = ?1",
            params![id.to_string()],
            |r| r.get(0),
        )
        .map_err(Into::into)
    }

    pub fn get_evidence_text(&self, fact_id: Uuid) -> Result<Option<String>> {
        let conn = self.pool.get()?;
        let res = conn.query_row(
            "SELECT evidence_text FROM evidence WHERE fact_id = ?1 LIMIT 1",
            params![fact_id.to_string()],
            |row| row.get::<_, String>(0),
        );
        match res {
            Ok(text) => Ok(Some(text)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Chain of facts for one honeypot id (latest first, includes superseded).
    pub fn get_memory_chain(&self, fact_id: &str) -> Result<Vec<(String, bool, Option<String>)>> {
        let conn = self.db_conn()?;
        let mut chain = Vec::new();
        let mut cursor = fact_id.to_string();
        let mut seen = std::collections::HashSet::new();
        while !cursor.is_empty() && seen.insert(cursor.clone()) {
            let row = conn.query_row(
                "SELECT content, is_latest, graph_rel, supersedes_id
                 FROM honeypot WHERE id = ?1 OR vault_id = ?1",
                params![cursor],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i32>(1)? != 0,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            );
            let Ok((content, is_latest, graph_rel, supersedes_id)) = row else {
                break;
            };
            chain.push((content, is_latest, graph_rel));
            cursor = supersedes_id.unwrap_or_default();
        }
        Ok(chain)
    }

    /// Dump the entire vault to a human-readable Markdown directory.
    pub async fn dump_to_markdown(&self, out_dir: impl AsRef<Path>) -> Result<()> {
        let mut groups: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        {
            // Scope the lock so it drops before async I/O
            let conn = self.pool.get()?;
            let mut stmt = conn.prepare(
                "SELECT id, content, half_life_days, confirmation_count, decay_class, created_at
                 FROM semantic_vault
                 ORDER BY decay_class ASC, created_at DESC",
            )?;

            let rows = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let content: String = row.get(1)?;
                let hld: f64 = row.get(2)?;
                let conf: u32 = row.get(3)?;
                let dclass: String = row.get(4)?;
                let created: String = row.get(5)?;
                Ok((id, content, hld, conf, dclass, created))
            })?;

            for r in rows.flatten() {
                let (id, content, hld, conf, dclass, created) = r;
                let md = groups
                    .entry(dclass.clone())
                    .or_insert_with(|| format!("# GZMO Memory Vault: {}\n\n", dclass));

                md.push_str(&format!(
                    "## Entry: {}\n- **Created:** {}\n- **Confirmations:** {}\n- **Half-life:** {} days\n\n> {}\n\n---\n",
                    id, created, conf, hld, content.replace('\n', "\n> ")
                ));
            }
        } // Lock drops here

        let dir = out_dir.as_ref();
        tokio::fs::create_dir_all(dir).await?;

        for (dclass, markdown) in groups {
            let file_name = format!("Vault_{}.md", dclass);
            let target = dir.join(file_name);
            tokio::fs::write(&target, markdown).await?;
            println!("Exported memory partition: {:?}", target);
        }

        Ok(())
    }
}
