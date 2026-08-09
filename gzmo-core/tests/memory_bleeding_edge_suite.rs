//! GZMO Next-Gen Bleeding-Edge Memory Integration Suite
//!
//! Tests:
//! 1. Intent-Contextual Utility Matrix (IEU Matrix).
//! 2. 2-Hop Graph-RAG Subgraph Expansion.
//! 3. Zero-Trust Merkle Cryptographic Provenance Ledger & Audit.

use chrono::NaiveDate;
use uuid::Uuid;

use gzmo_core::memory::graph_rag::traverse_2hop_subgraph;
use gzmo_core::memory::provenance_merkle::append_merkle_block;
use gzmo_core::memory::vault::SqliteVault;
use gzmo_core::types::{DecayClass, ExtractedTruth};

fn sample_truth(content: &str, confidence: f32, source_file: Option<&str>) -> ExtractedTruth {
    ExtractedTruth {
        id: Uuid::new_v4(),
        content: content.to_string(),
        confidence,
        mmr_score: 0.0,
        source_date: NaiveDate::from_ymd_opt(2026, 8, 9).unwrap(),
        decay_class: DecayClass::CuratedVault,
        source_file: source_file.map(str::to_string),
        evidence: None,
    }
}

#[tokio::test]
async fn test_ieu_matrix_contextual_utility() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_bleeding_edge_{}.db", Uuid::new_v4()));
    let vault = SqliteVault::open(&db_path)?;

    let truth = sample_truth(
        "[SERVICE:vLLM] runs Blackwell backend on port 8000",
        0.95,
        Some("vllm_config.md"),
    );
    vault.promote_truths(&[truth.clone()]).await?;
    let vault_id = truth.id.to_string();

    // Record contextual feedback under 'infrastructure' domain
    vault.record_contextual_utility(&vault_id, "infrastructure", true)?;

    let conn = vault.db_conn()?;
    let domain: String = conn.query_row(
        "SELECT domain_tag FROM honeypot WHERE id = ?1 OR vault_id = ?1",
        rusqlite::params![vault_id],
        |r| r.get(0),
    )?;

    assert_eq!(domain, "infrastructure");

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[tokio::test]
async fn test_merkle_provenance_ledger_and_audit() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_bleeding_edge_{}.db", Uuid::new_v4()));
    let vault = SqliteVault::open(&db_path)?;

    let conn = vault.db_conn()?;

    // Append 3 blocks to Merkle ledger
    let block1 = append_merkle_block(&conn, "fact_1", "[CONCEPT:GZMO] Memory Spec v0.2")?;
    let block2 = append_merkle_block(&conn, "fact_2", "[SERVICE:Prime] Port 8000")?;
    let block3 = append_merkle_block(&conn, "fact_3", "[HOST:CT101] Proxmox LXC")?;

    assert!(!block1.is_empty());
    assert!(!block2.is_empty());
    assert!(!block3.is_empty());

    // Audit Merkle integrity -> must pass on untampered ledger
    assert!(vault.verify_merkle_ledger()?);

    // Tamper with block 1 in SQLite directly -> audit must fail closed
    conn.execute(
        "UPDATE merkle_ledger SET block_hash = 'tampered_hash_value' WHERE block_index = 1",
        [],
    )?;

    assert!(vault.verify_merkle_ledger().is_err());

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[tokio::test]
async fn test_2hop_graph_rag_subgraph_traversal() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_bleeding_edge_{}.db", Uuid::new_v4()));
    let vault = SqliteVault::open(&db_path)?;

    // Hop 1: [CONCEPT:GZMO] uses [SERVICE:Prime]
    let t1 = sample_truth(
        "[CONCEPT:GZMO] uses [SERVICE:Prime] for inference",
        0.95,
        Some("spec.md"),
    );
    // Hop 2: [SERVICE:Prime] runs on [HOST:CT101]
    let t2 = sample_truth(
        "[SERVICE:Prime] runs on [HOST:CT101] LXC container",
        0.95,
        Some("spec.md"),
    );

    vault.promote_truths(&[t1, t2]).await?;

    let conn = vault.db_conn()?;
    let chains = traverse_2hop_subgraph(&conn, &["GZMO".to_string()], 5)?;

    assert!(!chains.is_empty());
    assert_eq!(chains[0].seed_entity, "GZMO");
    assert!(!chains[0].hops.is_empty());

    // Assert Hop 1 target entity is 'Prime'
    assert_eq!(chains[0].hops[0].source_entity, "GZMO");
    assert_eq!(chains[0].hops[0].target_entity, "Prime");

    // Assert Hop 2 target entity is 'CT101'
    assert!(chains[0].hops.len() >= 2);
    assert_eq!(chains[0].hops[1].source_entity, "Prime");
    assert_eq!(chains[0].hops[1].target_entity, "CT101");

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}
