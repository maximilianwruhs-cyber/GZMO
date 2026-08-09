//! GZMO Long-Term Memory Production Readiness Integration Suite
//!
//! Tests vault storage, honeypot promotion, contradiction resolution (`is_latest`),
//! utility-based eviction, evidence localization, and profile context generation.

use chrono::NaiveDate;
use uuid::Uuid;

use gzmo_core::memory::lifecycle::{classify_truth_pair, LifecycleKind};
use gzmo_core::memory::profile::ProfileOptions;
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
async fn test_vault_honeypot_promotion_and_contradiction_flow() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_vault_{}.db", Uuid::new_v4()));
    let vault = SqliteVault::open(&db_path)?;

    // Store initial fact
    let truth_v1 = sample_truth(
        "[SERVICE:Prime] runs on port 8000 on CT101",
        0.95,
        Some("wave_01_config.md"),
    );

    vault.promote_truths(&[truth_v1.clone()]).await?;
    let v1_id = truth_v1.id.to_string();

    let conn = vault.db_conn()?;
    let is_latest_v1_initial: i64 = conn.query_row(
        "SELECT is_latest FROM honeypot WHERE id = ?1 OR vault_id = ?1",
        rusqlite::params![v1_id],
        |r| r.get(0),
    )?;
    assert_eq!(is_latest_v1_initial, 1);

    // Store contradicting fact
    let truth_v2 = sample_truth(
        "[SERVICE:Prime] is deprecated and replaced by port 8081",
        0.95,
        Some("wave_01_config.md"),
    );

    assert_eq!(
        classify_truth_pair(&truth_v1.content, &truth_v2.content),
        LifecycleKind::Contradicts
    );

    vault.promote_truths(&[truth_v2.clone()]).await?;
    let v2_id = truth_v2.id.to_string();

    // Verify contradiction resolution: v1 superseded (is_latest = 0), v2 active (is_latest = 1)
    let is_latest_v1_after: i64 = conn.query_row(
        "SELECT is_latest FROM honeypot WHERE id = ?1 OR vault_id = ?1",
        rusqlite::params![v1_id],
        |r| r.get(0),
    )?;
    assert_eq!(is_latest_v1_after, 0);

    let is_latest_v2: i64 = conn.query_row(
        "SELECT is_latest FROM honeypot WHERE id = ?1 OR vault_id = ?1",
        rusqlite::params![v2_id],
        |r| r.get(0),
    )?;
    assert_eq!(is_latest_v2, 1);

    // Verify Merkle ledger automatically captured both promotions
    let merkle_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM merkle_ledger",
        [],
        |r| r.get(0),
    )?;
    assert!(merkle_count >= 2);
    assert!(vault.verify_merkle_ledger()?);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[tokio::test]
async fn test_utility_feedback_and_eviction() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_vault_{}.db", Uuid::new_v4()));
    let vault = SqliteVault::open(&db_path)?;

    let truth = sample_truth(
        "[SYSTEM:Backup] temporary scratch note",
        0.90,
        Some("doc_scratch.md"),
    );
    vault.promote_truths(&[truth.clone()]).await?;

    let vault_id = truth.id.to_string();

    // Record 5 unutilized recall events (should decay utility_score)
    for _ in 0..5 {
        vault.record_memory_utilization(&vault_id, false)?;
    }

    // Evict low-utility honeypot facts
    let evicted = vault.evict_low_utility_honeypot()?;
    assert_eq!(evicted, 1);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}

#[tokio::test]
async fn test_profile_generation() -> anyhow::Result<()> {
    let db_path = std::env::temp_dir().join(format!("test_vault_{}.db", Uuid::new_v4()));
    let vault = SqliteVault::open(&db_path)?;

    let static_fact = sample_truth(
        "[CONCEPT:GZMO] Memory Architecture v0.2",
        0.98,
        Some("spec.md"),
    );
    vault.promote_truths(&[static_fact]).await?;

    let profile = vault.build_profile(ProfileOptions::default())?;
    assert_eq!(profile.container_tag, "obolus");
    assert!(profile.token_estimate > 0);

    let _ = std::fs::remove_file(&db_path);
    Ok(())
}
