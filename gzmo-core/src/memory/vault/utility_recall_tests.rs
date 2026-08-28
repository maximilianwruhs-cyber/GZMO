//! Cross-cutting recall/utility (MemRL) integration tests for the vault.

use super::*;
use crate::memory::honeypot::insert_honeypot_lifecycle;
use crate::types::{DecayClass, ExtractedTruth};
use rusqlite::params;
use std::env;

use crate::memory::felt_use::{self, FeltUseKind};
use chrono::Utc;
use uuid::Uuid;
fn tempfile_db() -> std::path::PathBuf {
    let mut p = env::temp_dir();
    p.push(format!(
        "gzmo_utility_recall_{}.db",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    p
}

fn insert_fact(vault: &SqliteVault, content: &str, source: &str, utility: f64) -> Uuid {
    let id = Uuid::new_v4();
    let now = Utc::now().to_rfc3339();
    let truth = ExtractedTruth {
        id,
        content: content.to_string(),
        confidence: 0.95,
        mmr_score: 0.0,
        source_date: Utc::now().date_naive(),
        decay_class: DecayClass::CuratedVault,
        source_file: Some(source.to_string()),
        evidence: None,
    };
    let conn = vault.db_conn().expect("conn");
    conn.execute(
        "INSERT INTO semantic_vault
            (id, content, embedding, half_life_days, confidence, confirmation_count,
             decay_class, created_at, last_accessed_at, source_file, content_norm)
         VALUES (?1, ?2, ?3, 60.0, 0.95, 1, 'CuratedVault', ?4, ?4, ?5, ?6)",
        params![
            id.to_string(),
            content,
            Vec::<u8>::new(),
            now,
            source,
            normalize_truth_content(content),
        ],
    )
    .expect("insert vault");
    insert_honeypot_lifecycle(
        &conn,
        &id.to_string(),
        &truth,
        &[],
        &normalize_truth_content(content),
        "honeypot",
        None,
        None,
    )
    .expect("insert honeypot");
    conn.execute(
        "UPDATE honeypot SET utility_score = ?1 WHERE id = ?2",
        params![utility, id.to_string()],
    )
    .expect("set utility");
    id
}

#[tokio::test]
async fn search_recall_orders_by_utility_inside_fts_pool() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let low = insert_fact(
        &vault,
        "alpha widget sits in the low-utility drawer",
        "a.md",
        0.0,
    );
    let high = insert_fact(
        &vault,
        "alpha gadget sits in the high-utility drawer",
        "b.md",
        20.0,
    );

    let hits = vault.search_recall("alpha", 5).await.expect("search");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));

    assert!(
        hits.len() >= 2,
        "both fixture facts must be in the FTS pool, got {}",
        hits.len()
    );
    assert_eq!(
        hits[0].0.id,
        high,
        "high utility_score must rank first, got {:?}",
        hits.iter().map(|(f, s)| (f.id, s)).collect::<Vec<_>>()
    );
    assert!(hits.iter().any(|(f, _)| f.id == low));
}

#[tokio::test]
async fn empty_query_returns_no_hits() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    insert_fact(&vault, "alpha widget", "a.md", 9.0);
    let hits = vault.search_recall("   ", 5).await.expect("search");
    let _ = std::fs::remove_file(&path);
    assert!(hits.is_empty(), "empty query must not invent recall");
}

#[test]
fn reinforce_by_bumps_utility_score() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let id = insert_fact(&vault, "bonded scar about felt use", "c.md", 1.0);
    vault.reinforce_by(id, 5).expect("reinforce");
    let conn = vault.db_conn().expect("conn");
    let (recall, utility): (i64, f64) = conn
        .query_row(
            "SELECT recall_count, utility_score FROM honeypot WHERE id = ?1",
            params![id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("read");
    let _ = std::fs::remove_file(&path);
    assert_eq!(recall, 5);
    assert!((utility - 6.0).abs() < 1e-9, "utility was {utility}");
}

fn read_ru(vault: &SqliteVault, id: Uuid) -> (i64, f64) {
    let conn = vault.db_conn().expect("conn");
    conn.query_row(
        "SELECT recall_count, utility_score FROM honeypot WHERE id = ?1",
        params![id.to_string()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .expect("read")
}

#[test]
fn glance_bumps_recall_not_utility() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let id = insert_fact(&vault, "[AGENT:Foo] sits in the drawer", "g.md", 4.0);
    felt_use::touch(&vault, id, FeltUseKind::Glance).expect("glance");
    let (recall, utility) = read_ru(&vault, id);
    let _ = std::fs::remove_file(&path);
    assert_eq!(recall, 1);
    assert!(
        (utility - 4.0).abs() < 1e-9,
        "glance must not mint Q, got {utility}"
    );
}

#[test]
fn outcome_bumps_q_on_previously_recalled_entity() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let recalled = insert_fact(&vault, "[AGENT:Foo] was recalled in session", "r.md", 1.0);
    felt_use::touch(&vault, recalled, FeltUseKind::Glance).expect("recall");
    let takeaway = ExtractedTruth {
        id: Uuid::new_v4(),
        content: "[AGENT:Foo] later takeaway cites the scar".into(),
        confidence: 0.95,
        mmr_score: 0.0,
        source_date: Utc::now().date_naive(),
        decay_class: DecayClass::SessionDistill,
        source_file: Some("session.md".into()),
        evidence: None,
    };
    let n = vault
        .reinforce_outcome_from_new_truths(&[takeaway])
        .expect("outcome");
    let (_, utility) = read_ru(&vault, recalled);
    let _ = std::fs::remove_file(&path);
    assert_eq!(n, 1);
    assert!(
        (utility - 9.0).abs() < 1e-9,
        "outcome +8 on base 1, got {utility}"
    );
}

#[test]
fn failure_case_is_stored_and_recalled_when_query_matches() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    vault
        .record_failure_case("verify_fail", "bad quote about CT101 vault lock", None)
        .expect("fail");
    let hit = vault
        .search_failure_cases("CT101 vault lock", &[], 3)
        .expect("search");
    let miss = vault.search_failure_cases("the", &[], 3).expect("short");
    let unrelated = vault
        .search_failure_cases("alpha beta gamma delta", &[], 3)
        .expect("unrelated");
    let via_guard = vault.recall_failures("CT101 vault lock").expect("guard");
    let _ = std::fs::remove_file(&path);
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].kind, "verify_fail");
    assert!(miss.is_empty(), "stopword query must not dump failures");
    assert!(
        unrelated.is_empty(),
        "unrelated tokens must not dump failures"
    );
    assert_eq!(via_guard.len(), 1);
    assert!(via_guard[0].contains("verify_fail"));
}

#[test]
fn failure_case_recalls_when_related_fact_in_pool() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let fact = insert_fact(&vault, "[AGENT:Qux] living mutex is ct101", "q.md", 1.0);
    vault
        .record_failure_case(
            "gate_refuse",
            "unverified derived Qux note",
            Some(&fact.to_string()),
        )
        .expect("fail");
    let hit = vault
        .search_failure_cases("zzzz", &[fact], 3)
        .expect("related");
    let miss = vault
        .search_failure_cases("zzzz", &[], 3)
        .expect("no related");
    let _ = std::fs::remove_file(&path);
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].kind, "gate_refuse");
    assert!(miss.is_empty());
}

#[test]
fn as_of_includes_superseded_before_valid_to() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let old = insert_fact(&vault, "[AGENT:Bar] old value", "old.md", 1.0);
    let conn = vault.db_conn().expect("conn");
    conn.execute(
        "UPDATE honeypot SET valid_from = '2020-01-01T00:00:00+00:00' WHERE id = ?1",
        params![old.to_string()],
    )
    .expect("valid_from");
    crate::memory::lifecycle::supersede_honeypot(&conn, &old.to_string()).expect("supersede");
    drop(conn);
    let historic = vault
        .honeypot_as_of("2021-06-01T00:00:00+00:00", 10)
        .expect("as_of historic");
    let future = vault
        .honeypot_as_of("2099-01-01T00:00:00+00:00", 10)
        .expect("as_of future");
    let _ = std::fs::remove_file(&path);
    assert!(
        historic.iter().any(|(id, _, _)| id == &old.to_string()),
        "as_of 2021 must still see the 2020–now fact"
    );
    assert!(
        future.iter().all(|(id, _, _)| id != &old.to_string()),
        "as_of after valid_to must not see the superseded fact"
    );
}

#[test]
fn filter_assertable_drops_superseded_preserves_latest_order() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let old = insert_fact(&vault, "[AGENT:Gpm] stale city", "old.md", 1.0);
    let new = insert_fact(&vault, "[AGENT:Gpm] current city", "new.md", 1.0);
    let conn = vault.db_conn().expect("conn");
    crate::memory::lifecycle::supersede_honeypot(&conn, &old.to_string()).expect("supersede");
    drop(conn);
    let filtered = vault
        .filter_assertable_honeypot_ids(&[old, new, old])
        .expect("filter");
    let _ = std::fs::remove_file(&path);
    assert_eq!(filtered, vec![new]);
}

#[test]
fn take_assertable_prefetch_refills_cap_after_dropping_stale() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let stale_a = insert_fact(&vault, "[AGENT:Overfetch] stale a", "a.md", 1.0);
    let latest_a = insert_fact(&vault, "[AGENT:Overfetch] current a", "a2.md", 1.0);
    let stale_b = insert_fact(&vault, "[AGENT:Overfetch] stale b", "b.md", 1.0);
    let latest_b = insert_fact(&vault, "[AGENT:Overfetch] current b", "b2.md", 1.0);
    let latest_c = insert_fact(&vault, "[AGENT:Overfetch] current c", "c.md", 1.0);
    let conn = vault.db_conn().expect("conn");
    crate::memory::lifecycle::supersede_honeypot(&conn, &stale_a.to_string()).expect("supersede a");
    crate::memory::lifecycle::supersede_honeypot(&conn, &stale_b.to_string()).expect("supersede b");
    drop(conn);
    // Without overfetch, the first two hits would be stale_a + latest_a → cap 2
    // after filter would be only latest_a. Overfetch keeps later latest ids.
    let filled = vault
        .take_assertable_prefetch(&[stale_a, latest_a, stale_b, latest_b, latest_c], 2)
        .expect("prefetch");
    let starved = vault
        .take_assertable_prefetch(&[stale_a, latest_a], 2)
        .expect("starved");
    let _ = std::fs::remove_file(&path);
    assert_eq!(filled, vec![latest_a, latest_b]);
    assert_eq!(starved, vec![latest_a]);
}

#[test]
fn region_rewrite_supersedes_other_latest_for_entity() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let a = insert_fact(&vault, "[AGENT:Baz] first working set", "a.md", 1.0);
    let b = insert_fact(&vault, "[AGENT:Baz] compact replacement", "b.md", 1.0);
    let conn = vault.db_conn().expect("conn");
    let n = SqliteVault::region_rewrite_entity(&conn, "Baz", &b.to_string()).expect("rewrite");
    let a_latest: i32 = conn
        .query_row(
            "SELECT is_latest FROM honeypot WHERE id = ?1",
            params![a.to_string()],
            |r| r.get(0),
        )
        .expect("a");
    let event: String = conn
        .query_row(
            "SELECT gate_event FROM honeypot WHERE id = ?1",
            params![a.to_string()],
            |r| r.get(0),
        )
        .expect("event");
    let _ = std::fs::remove_file(&path);
    assert!(n >= 1);
    assert_eq!(a_latest, 0);
    assert_eq!(event, "region_rewrite");
}

#[tokio::test]
async fn low_confidence_promote_records_failure_and_quarantine() {
    let path = tempfile_db();
    let vault = SqliteVault::open(&path).expect("open");
    let truth = ExtractedTruth {
        id: Uuid::new_v4(),
        content: "[AGENT:Q] weak unverified guess".into(),
        confidence: 0.4,
        mmr_score: 0.0,
        source_date: Utc::now().date_naive(),
        decay_class: DecayClass::CuratedVault,
        source_file: Some("weak.md".into()),
        evidence: None,
    };
    vault
        .promote_truths_with_origin(&[truth], "ingest")
        .await
        .expect("promote");
    let conn = vault.db_conn().expect("conn");
    let fails: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM failure_cases WHERE kind = 'verify_fail'",
            [],
            |r| r.get(0),
        )
        .expect("fails");
    let q: i64 = conn
        .query_row("SELECT COUNT(*) FROM quarantine_vault", [], |r| r.get(0))
        .expect("q");
    let _ = std::fs::remove_file(&path);
    assert_eq!(fails, 1);
    assert_eq!(q, 1);
}
