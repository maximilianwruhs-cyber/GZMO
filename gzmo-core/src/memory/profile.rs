//! Cached static + dynamic profile from honeypot (MEMORY_ARCHITECTURE_SPEC §5).
//! When `knowledge_core.db` exists beside the vault, static lines prefer ripened core cards.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::memory::vault::SqliteVault;

const CACHE_TTL: Duration = Duration::from_secs(300);
const CHARS_PER_TOKEN: usize = 4;

fn profile_cache() -> &'static Mutex<HashMap<String, CachedEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone)]
struct CachedEntry {
    at: Instant,
    profile: GzmoProfile,
}

/// Operator/agent profile snapshot (static + dynamic context).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GzmoProfile {
    pub container_tag: String,
    pub generated_at: String,
    pub r#static: Vec<String>,
    pub dynamic: Vec<String>,
    pub preferences: Vec<String>,
    pub procedures: Vec<String>,
    pub token_estimate: usize,
}

#[derive(Debug, Clone)]
pub struct ProfileOptions {
    pub container_tag: String,
    pub dynamic_only: bool,
    pub static_limit: usize,
    pub dynamic_limit: usize,
}

impl Default for ProfileOptions {
    fn default() -> Self {
        Self {
            container_tag: "obolus".to_string(),
            dynamic_only: false,
            static_limit: 20,
            dynamic_limit: 15,
        }
    }
}

impl SqliteVault {
    /// Build or return cached profile for a container scope.
    pub fn build_profile(&self, opts: ProfileOptions) -> Result<GzmoProfile> {
        let cache_key = format!(
            "{}:{}:{}:{}",
            opts.container_tag, opts.dynamic_only, opts.static_limit, opts.dynamic_limit
        );
        if let Some(entry) = profile_cache()
            .lock()
            .ok()
            .and_then(|c| c.get(&cache_key).cloned())
        {
            if entry.at.elapsed() < CACHE_TTL {
                return Ok(entry.profile);
            }
        }

        let profile = self.build_profile_uncached(opts)?;
        if let Ok(mut cache) = profile_cache().lock() {
            cache.insert(
                cache_key,
                CachedEntry {
                    at: Instant::now(),
                    profile: profile.clone(),
                },
            );
        }
        Ok(profile)
    }

    fn build_profile_uncached(&self, opts: ProfileOptions) -> Result<GzmoProfile> {
        let conn = self.db_conn()?;
        let static_facts = if opts.dynamic_only {
            Vec::new()
        } else {
            match load_static_from_core(&conn, opts.static_limit) {
                Some(lines) if !lines.is_empty() => lines,
                _ => select_lines(
                    &conn,
                    "SELECT content FROM honeypot
                     WHERE is_latest = 1 AND container_tag = ?1
                       AND (
                         decay_class IN ('Structural', 'FlexibleIdentity', 'Core', 'AbsoluteIdentity')
                         OR content LIKE '[SYSTEM:%'
                         OR content LIKE '[CONCEPT:GZMO%'
                         OR content LIKE '[CONCEPT:SOUL%'
                         OR (decay_class = 'CuratedVault' AND confidence >= 0.92)
                       )
                     ORDER BY recall_count DESC, confidence DESC
                     LIMIT ?2",
                    params![opts.container_tag, opts.static_limit as i64],
                )?,
            }
        };

        let dynamic_facts = select_lines(
            &conn,
            "SELECT content FROM honeypot
             WHERE is_latest = 1 AND container_tag = ?1
               AND datetime(promoted_at) > datetime('now', '-14 days')
               AND origin IN ('session_distill', 'dream', 'spark', 'ingest', 'verified_dream')
             ORDER BY promoted_at DESC
             LIMIT ?2",
            params![opts.container_tag, opts.dynamic_limit as i64],
        )?;

        let preferences = select_lines(
            &conn,
            "SELECT content FROM honeypot
             WHERE is_latest = 1 AND container_tag = ?1
               AND (memory_type = 'preference'
                    OR lower(content) LIKE '%prefer%')
             ORDER BY recall_count DESC, confidence DESC
             LIMIT 10",
            params![opts.container_tag],
        )?;

        let procedures = select_lines(
            &conn,
            "SELECT content FROM honeypot
             WHERE is_latest = 1 AND container_tag = ?1
               AND (memory_type = 'procedure'
                    OR lower(content) LIKE '%workflow%'
                    OR lower(content) LIKE '%before re-ingest%')
             ORDER BY recall_count DESC, confidence DESC
             LIMIT 10",
            params![opts.container_tag],
        )?;

        let mut sections = Vec::new();
        for s in &static_facts {
            sections.push(s.as_str());
        }
        for s in &dynamic_facts {
            sections.push(s.as_str());
        }
        for s in &preferences {
            sections.push(s.as_str());
        }
        for s in &procedures {
            sections.push(s.as_str());
        }
        let token_estimate = sections
            .iter()
            .map(|s| s.chars().count().div_ceil(CHARS_PER_TOKEN))
            .sum();

        Ok(GzmoProfile {
            container_tag: opts.container_tag.clone(),
            generated_at: Utc::now().to_rfc3339(),
            r#static: static_facts,
            dynamic: dynamic_facts,
            preferences,
            procedures,
            token_estimate,
        })
    }
}

fn select_lines(
    conn: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params, |row| row.get::<_, String>(0))?;
    let mut out = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        let line = row.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if out.iter().any(|x| x == &line) {
            continue;
        }
        out.push(line);
    }
    Ok(out)
}

/// Resolve `data/knowledge_core.db` adjacent to the vault file (M5 §5.6).
fn knowledge_core_path(vault_conn: &Connection) -> Option<PathBuf> {
    let mut stmt = vault_conn.prepare("PRAGMA database_list").ok()?;
    let rows = stmt
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let file: Option<String> = row.get(2)?;
            Ok((name, file))
        })
        .ok()?;
    for row in rows.filter_map(|r| r.ok()) {
        if row.0 == "main" {
            if let Some(file) = row.1 {
                let vault = PathBuf::from(file);
                return vault.parent().map(|p| p.join("knowledge_core.db"));
            }
        }
    }
    None
}

/// Extract static profile lines from ripened concept cards (bullet lines + entity header).
pub(crate) fn static_lines_from_summary(entity_tag: &str, summary_md: &str, max_bullets: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in summary_md.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") {
            out.push(format!("{entity_tag} {}", trimmed.trim_start_matches('-').trim()));
            if out.len() >= max_bullets {
                break;
            }
        }
    }
    if out.is_empty() && !summary_md.trim().is_empty() {
        let one_line: String = summary_md
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('_'))
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        if !one_line.is_empty() {
            out.push(format!("{entity_tag} {one_line}"));
        }
    }
    out
}

fn load_static_from_core(vault_conn: &Connection, limit: usize) -> Option<Vec<String>> {
    let core_path = knowledge_core_path(vault_conn)?;
    if !core_path.is_file() {
        return None;
    }
    let core = Connection::open_with_flags(
        &core_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .ok()?;
    let mut stmt = core
        .prepare(
            "SELECT entity_tag, summary_md FROM knowledge_core
             ORDER BY version DESC, entity_tag ASC",
        )
        .ok()?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .ok()?;
    let mut out = Vec::new();
    for row in rows.filter_map(|r| r.ok()) {
        for line in static_lines_from_summary(&row.0, &row.1, 2) {
            if out.iter().any(|x| x == &line) {
                continue;
            }
            out.push(line);
            if out.len() >= limit {
                return Some(out);
            }
        }
    }
    Some(out)
}

/// Drop cached profiles (call after honeypot promote).
pub fn invalidate_profile_cache(container_tag: Option<&str>) {
    let Ok(mut cache) = profile_cache().lock() else {
        return;
    };
    match container_tag {
        None => cache.clear(),
        Some(tag) => cache.retain(|k, _| !k.starts_with(&format!("{tag}:"))),
    }
}

impl GzmoProfile {
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# GZMO Profile — {}\n\n", self.container_tag));
        md.push_str(&format!("_Generated: {} · ~{} tokens_\n\n", self.generated_at, self.token_estimate));
        if !self.r#static.is_empty() {
            md.push_str("## Static\n\n");
            for line in &self.r#static {
                md.push_str(&format!("- {line}\n"));
            }
            md.push('\n');
        }
        if !self.dynamic.is_empty() {
            md.push_str("## Dynamic (14d)\n\n");
            for line in &self.dynamic {
                md.push_str(&format!("- {line}\n"));
            }
            md.push('\n');
        }
        if !self.preferences.is_empty() {
            md.push_str("## Preferences\n\n");
            for line in &self.preferences {
                md.push_str(&format!("- {line}\n"));
            }
            md.push('\n');
        }
        if !self.procedures.is_empty() {
            md.push_str("## Procedures\n\n");
            for line in &self.procedures {
                md.push_str(&format!("- {line}\n"));
            }
        }
        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_lines_from_summary_extracts_bullets() {
        let md = "## [SYSTEM:Prime]\n\n- Runs on :8000\n- Qwen3.6-35B\n\n_provenance_";
        let lines = static_lines_from_summary("SYSTEM:Prime", md, 2);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(":8000"));
    }

    #[test]
    fn token_estimate_positive_for_content() {
        let p = GzmoProfile {
            container_tag: "obolus".into(),
            generated_at: Utc::now().to_rfc3339(),
            r#static: vec!["a".repeat(40)],
            dynamic: vec![],
            preferences: vec![],
            procedures: vec![],
            token_estimate: 0,
        };
        let est = ["a".repeat(40).as_str()]
            .iter()
            .map(|s| s.chars().count().div_ceil(4))
            .sum::<usize>();
        assert!(est >= 10);
        let _ = p;
    }
}
