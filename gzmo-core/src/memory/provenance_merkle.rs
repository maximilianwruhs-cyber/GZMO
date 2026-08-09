//! Merkle Cryptographic Provenance Ledger for Zero-Trust Memory Integrity.
//!
//! Maintains an append-only SHA-256 hash chain of all honeypot fact additions and updates.
//! $H_i = \text{SHA256}(H_{i-1} \parallel \text{FactID} \parallel \text{ContentSHA256} \parallel \text{Timestamp})$

use anyhow::{bail, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Ensure merkle ledger table exists.
pub fn init_merkle_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS merkle_ledger (
            block_index INTEGER PRIMARY KEY AUTOINCREMENT,
            fact_id TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            prev_block_hash TEXT NOT NULL,
            block_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_merkle_fact ON merkle_ledger(fact_id);
        CREATE INDEX IF NOT EXISTS idx_merkle_block ON merkle_ledger(block_index);",
    )?;
    Ok(())
}

/// Compute SHA-256 hex string.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Compute block hash $H_i = \text{SHA256}(H_{i-1} \parallel \text{FactID} \parallel \text{ContentHash} \parallel \text{Timestamp})$.
pub fn compute_block_hash(
    prev_hash: &str,
    fact_id: &str,
    content_hash: &str,
    timestamp: &str,
) -> String {
    let payload = format!("{prev_hash}:{fact_id}:{content_hash}:{timestamp}");
    sha256_hex(&payload)
}

/// Append a new block to the cryptographic Merkle chain.
pub fn append_merkle_block(
    conn: &Connection,
    fact_id: &str,
    content: &str,
) -> Result<String> {
    init_merkle_schema(conn)?;

    let now = Utc::now().to_rfc3339();
    let content_hash = sha256_hex(content);

    let prev_hash: String = conn
        .query_row(
            "SELECT block_hash FROM merkle_ledger ORDER BY block_index DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| GENESIS_HASH.to_string());

    let block_hash = compute_block_hash(&prev_hash, fact_id, &content_hash, &now);

    conn.execute(
        "INSERT INTO merkle_ledger (
            fact_id, content_hash, prev_block_hash, block_hash, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![fact_id, content_hash, prev_hash, block_hash, now],
    )?;

    Ok(block_hash)
}

/// Audit the entire Merkle ledger chain to guarantee zero tamperability.
pub fn verify_merkle_integrity(conn: &Connection) -> Result<bool> {
    init_merkle_schema(conn)?;

    let mut stmt = conn.prepare(
        "SELECT block_index, fact_id, content_hash, prev_block_hash, block_hash, created_at
         FROM merkle_ledger
         ORDER BY block_index ASC",
    )?;

    let mut expected_prev = GENESIS_HASH.to_string();

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    for row in rows {
        let (idx, fact_id, content_hash, prev_hash, block_hash, timestamp) = row?;

        if prev_hash != expected_prev {
            bail!("Merkle chain broken at block #{idx}: prev_hash mismatch (expected {expected_prev}, got {prev_hash})");
        }

        let recalculated = compute_block_hash(&prev_hash, &fact_id, &content_hash, &timestamp);
        if recalculated != block_hash {
            bail!("Merkle chain corrupted at block #{idx}: block_hash mismatch");
        }

        expected_prev = block_hash;
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_merkle_chain_creation_and_verification() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        init_merkle_schema(&conn)?;

        let hash1 = append_merkle_block(&conn, "fact_1", "[CONCEPT:GZMO] Test fact 1")?;
        let hash2 = append_merkle_block(&conn, "fact_2", "[CONCEPT:GZMO] Test fact 2")?;

        assert!(!hash1.is_empty());
        assert!(!hash2.is_empty());
        assert_ne!(hash1, hash2);

        assert!(verify_merkle_integrity(&conn)?);

        Ok(())
    }

    #[test]
    fn test_merkle_detects_tampering() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        append_merkle_block(&conn, "fact_1", "[CONCEPT:GZMO] Test fact 1")?;
        append_merkle_block(&conn, "fact_2", "[CONCEPT:GZMO] Test fact 2")?;

        // Tamper with block 1 hash in SQLite directly
        conn.execute(
            "UPDATE merkle_ledger SET block_hash = 'tampered_hash' WHERE block_index = 1",
            [],
        )?;

        assert!(verify_merkle_integrity(&conn).is_err());
        Ok(())
    }
}
