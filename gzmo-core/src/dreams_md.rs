//! Helpers for `DREAMS.md` — dream narrative vs appended spark sections.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

/// Split existing file content into (dream body, spark tail).
///
/// Spark sections start at a line `## Spark` (optionally `## Spark — date`).
/// Everything from the first such heading onward is preserved across dream rewrites.
pub fn split_dream_and_spark(content: &str) -> (String, String) {
    if let Some(pos) = content.find("\n## Spark") {
        let dream = content[..pos].trim_end().to_string();
        let spark = content[pos + 1..].to_string();
        return (dream, spark);
    }
    if content.starts_with("## Spark") {
        return (String::new(), content.to_string());
    }
    (content.to_string(), String::new())
}

/// Extract only the spark tail from `DREAMS.md` content.
pub fn extract_spark_sections(content: &str) -> String {
    split_dream_and_spark(content).1
}

/// Merge a fresh dream narrative with spark sections from an existing file.
pub fn merge_dream_narrative(existing: &str, narrative: &str) -> String {
    let sparks = extract_spark_sections(existing);
    let mut out = narrative.trim_end().to_string();
    if sparks.is_empty() {
        return out;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str(sparks.trim_start());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Write dream narrative to `DREAMS.md`, preserving any existing `## Spark` sections.
pub async fn write_dream_narrative(path: &Path, narrative: &str) -> Result<()> {
    let existing = if path.exists() {
        tokio::fs::read_to_string(path).await.unwrap_or_default()
    } else {
        String::new()
    };
    let merged = merge_dream_narrative(&existing, narrative);
    tokio::fs::write(path, merged).await?;
    Ok(())
}

/// Default size trigger for dream compaction (chars).
pub const DEFAULT_DREAMS_COMPACT_MAX_CHARS: usize = 100_000;

/// Result of compacting `DREAMS.md` (and optionally cold sessions).
#[derive(Debug, Clone)]
pub struct CompactReport {
    pub dreams_path: PathBuf,
    pub before_chars: usize,
    pub after_chars: usize,
    pub archived_dreams: Option<PathBuf>,
    pub compacted: bool,
    pub sessions_archived: usize,
    pub dry_run: bool,
}

/// Compact oversized `DREAMS.md`: archive full file, keep spark tail + truncated dream head/tail.
///
/// Never touches the vault/honeypot. Soft-fail safe for overnight GC.
pub fn compact_dreams_md(
    dreams_path: &Path,
    max_chars: usize,
    dry_run: bool,
) -> Result<CompactReport> {
    let content = if dreams_path.exists() {
        std::fs::read_to_string(dreams_path)
            .with_context(|| format!("read {}", dreams_path.display()))?
    } else {
        return Ok(CompactReport {
            dreams_path: dreams_path.to_path_buf(),
            before_chars: 0,
            after_chars: 0,
            archived_dreams: None,
            compacted: false,
            sessions_archived: 0,
            dry_run,
        });
    };

    let before = content.chars().count();
    if before <= max_chars {
        return Ok(CompactReport {
            dreams_path: dreams_path.to_path_buf(),
            before_chars: before,
            after_chars: before,
            archived_dreams: None,
            compacted: false,
            sessions_archived: 0,
            dry_run,
        });
    }

    let (dream_body, spark_tail) = split_dream_and_spark(&content);
    let compact_body = truncate_dream_body(&dream_body, max_chars.saturating_sub(spark_tail.len()));
    let mut out = compact_body.trim_end().to_string();
    if !spark_tail.is_empty() {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(spark_tail.trim_start());
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    let after = out.chars().count();

    let archive_path = dreams_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("dreams-archive")
        .join(format!(
            "DREAMS-{}.md",
            Utc::now().format("%Y%m%dT%H%M%SZ")
        ));

    if dry_run {
        return Ok(CompactReport {
            dreams_path: dreams_path.to_path_buf(),
            before_chars: before,
            after_chars: after,
            archived_dreams: Some(archive_path),
            compacted: true,
            sessions_archived: 0,
            dry_run: true,
        });
    }

    if let Some(parent) = archive_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&archive_path, &content)
        .with_context(|| format!("archive {}", archive_path.display()))?;
    std::fs::write(dreams_path, &out).with_context(|| format!("write {}", dreams_path.display()))?;

    Ok(CompactReport {
        dreams_path: dreams_path.to_path_buf(),
        before_chars: before,
        after_chars: after,
        archived_dreams: Some(archive_path),
        compacted: true,
        sessions_archived: 0,
        dry_run: false,
    })
}

/// Keep head + tail of dream body when over budget (middle dropped with marker).
fn truncate_dream_body(body: &str, budget: usize) -> String {
    if budget == 0 {
        return String::new();
    }
    let chars: Vec<char> = body.chars().collect();
    if chars.len() <= budget {
        return body.to_string();
    }
    let marker = "\n\n… [compacted — middle archived] …\n\n";
    let marker_len = marker.chars().count();
    if budget <= marker_len + 64 {
        return chars
            .into_iter()
            .take(budget)
            .collect::<String>()
            + "\n";
    }
    let keep = budget - marker_len;
    let head_n = keep / 2;
    let tail_n = keep - head_n;
    let head: String = chars.iter().take(head_n).collect();
    let tail: String = chars[chars.len() - tail_n..].iter().collect();
    format!("{head}{marker}{tail}")
}

/// Move session JSON files older than `archive_after_days` into `sessions-archive/`.
pub fn archive_cold_sessions(
    sessions_dir: &Path,
    archive_after_days: i64,
    dry_run: bool,
) -> Result<usize> {
    if archive_after_days <= 0 || !sessions_dir.is_dir() {
        return Ok(0);
    }
    let cutoff = Utc::now() - chrono::Duration::days(archive_after_days);
    let archive_dir = sessions_dir
        .parent()
        .unwrap_or(sessions_dir)
        .join("sessions-archive");
    let mut n = 0usize;
    for entry in std::fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let meta = entry.metadata()?;
        let modified = meta.modified().ok();
        let Some(modified) = modified else {
            continue;
        };
        let modified = DateTime::<Utc>::from(modified);
        if modified >= cutoff {
            continue;
        }
        let dest = archive_dir.join(path.file_name().unwrap_or_default());
        if dry_run {
            n += 1;
            continue;
        }
        std::fs::create_dir_all(&archive_dir)?;
        std::fs::rename(&path, &dest)
            .or_else(|_| {
                std::fs::copy(&path, &dest)?;
                std::fs::remove_file(&path)
            })
            .with_context(|| format!("archive session {}", path.display()))?;
        n += 1;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn preserves_spark_after_dream_rewrite() {
        let existing = "# Dream Consolidation — 2026-05-30\n\nEntities: 2\n\n\
            ## Spark — 2026-05-30\n\nConnection here.\n";
        let new_dream = "# Dream Consolidation — 2026-05-31\n\nEntities: 5\n";
        let merged = merge_dream_narrative(existing, new_dream);
        assert!(merged.starts_with("# Dream Consolidation — 2026-05-31"));
        assert!(merged.contains("## Spark — 2026-05-30"));
        assert!(merged.contains("Connection here."));
        assert!(!merged.contains("Entities: 2"));
    }

    #[test]
    fn no_sparks_passthrough() {
        let merged = merge_dream_narrative("old\n", "new\n");
        assert_eq!(merged.trim(), "new");
    }

    #[test]
    fn compact_archives_and_truncates() {
        let root = std::env::temp_dir().join(format!(
            "gzmo-dreams-compact-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("DREAMS.md");
        let mut body = "# Dream\n\n".to_string();
        body.push_str(&"x".repeat(5000));
        body.push_str("\n\n## Spark — 2026-07-01\n\nKeep me.\n");
        fs::write(&path, &body).unwrap();

        let report = compact_dreams_md(&path, 800, false).unwrap();
        assert!(report.compacted);
        assert!(report.before_chars > report.after_chars);
        assert!(report.archived_dreams.unwrap().exists());
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("## Spark — 2026-07-01"));
        assert!(after.contains("Keep me."));
        assert!(after.contains("compacted"));

        let _ = fs::remove_dir_all(&root);
    }
}
