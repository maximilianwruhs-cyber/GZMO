//! Helpers for `DREAMS.md` — dream narrative vs appended spark sections.

use std::path::Path;

use anyhow::Result;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
