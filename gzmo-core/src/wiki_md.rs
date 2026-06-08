//! Markdown helpers for the `wiki/` layer.
//!
//! Mirrors the read-merge-write pattern in `dreams_md.rs`, but for the
//! structured wiki pages (entities / concepts / sources) plus `index.md` and
//! `log.md`. Pure file + string helpers — no LLM, no DB. The `WikiEngine`
//! (`wiki.rs`) orchestrates these.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// YAML frontmatter on every wiki page (consumed by Obsidian Dataview).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageFrontmatter {
    #[serde(rename = "type")]
    pub page_type: String, // entity | concept | source
    pub title: String,
    pub created: String, // YYYY-MM-DD
    pub updated: String, // YYYY-MM-DD
    #[serde(default)]
    pub sources: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_status")]
    pub status: String, // draft | stable | stale
    /// Marks pages emitted by the engine — the ingest guard refuses to ingest
    /// any file carrying this flag, preventing derived-fact feedback loops.
    #[serde(default)]
    pub gzmo_synthetic: bool,
}

fn default_status() -> String {
    "draft".to_string()
}

impl PageFrontmatter {
    pub fn new(page_type: &str, title: &str, date: &str) -> Self {
        Self {
            page_type: page_type.to_string(),
            title: title.to_string(),
            created: date.to_string(),
            updated: date.to_string(),
            sources: 0,
            tags: Vec::new(),
            status: default_status(),
            gzmo_synthetic: true,
        }
    }
}

/// Convert a title into a filesystem-safe, link-friendly slug.
pub fn slugify(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut prev_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("untitled");
    }
    out
}

/// Split a page into (frontmatter, body). Returns `None` frontmatter when the
/// file does not start with a `---` YAML block.
pub fn split_frontmatter(content: &str) -> (Option<PageFrontmatter>, String) {
    let trimmed = content.trim_start_matches('\u{feff}');
    if let Some(rest) = trimmed.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---") {
            let yaml = &rest[..end];
            let after = &rest[end + 4..];
            let body = after.strip_prefix('\n').unwrap_or(after).to_string();
            match serde_yaml::from_str::<PageFrontmatter>(yaml) {
                Ok(fm) => return (Some(fm), body),
                Err(_) => return (None, content.to_string()),
            }
        }
    }
    (None, content.to_string())
}

/// Render a full page from frontmatter + body.
pub fn render_page(fm: &PageFrontmatter, body: &str) -> String {
    let yaml = serde_yaml::to_string(fm).unwrap_or_default();
    format!("---\n{}---\n\n{}\n", yaml, body.trim_end())
}

/// Read an existing page if present.
pub async fn read_page(path: &Path) -> Option<(PageFrontmatter, String)> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    let (fm, body) = split_frontmatter(&content);
    fm.map(|fm| (fm, body))
}

/// Write a page, preserving the original `created` date and bumping `updated`.
pub async fn upsert_page(path: &Path, mut fm: PageFrontmatter, body: &str) -> Result<()> {
    if let Some((existing, _)) = read_page(path).await {
        fm.created = existing.created;
        // Keep the higher source count if the caller did not supply one.
        if fm.sources == 0 {
            fm.sources = existing.sources;
        }
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    tokio::fs::write(path, render_page(&fm, body)).await?;
    Ok(())
}

/// Insert or repoint a catalog entry under a `## <Category>` heading in
/// `index.md`. Idempotent: an existing bullet for the same `title` is replaced,
/// and the placeholder italic line for an empty category is removed.
pub fn upsert_index_entry(index: &str, category: &str, title: &str, link: &str, summary: &str) -> String {
    let heading = format!("## {category}");
    let bullet = format!("- [{title}]({link}) — {summary}");
    let mut lines: Vec<String> = index.lines().map(|l| l.to_string()).collect();

    let Some(h_idx) = lines.iter().position(|l| l.trim() == heading) else {
        // Category missing — append a new section.
        let mut out = index.trim_end().to_string();
        out.push_str(&format!("\n\n{heading}\n\n{bullet}\n"));
        return out;
    };

    // Find the end of this section (next "## " heading or EOF).
    let sec_end = lines
        .iter()
        .enumerate()
        .skip(h_idx + 1)
        .find(|(_, l)| l.trim_start().starts_with("## "))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    // Drop placeholder italic lines within the section.
    let mut i = h_idx + 1;
    let mut end = sec_end;
    while i < end {
        let t = lines[i].trim();
        if t.starts_with('_') && t.ends_with('_') {
            lines.remove(i);
            end -= 1;
        } else {
            i += 1;
        }
    }

    // Replace an existing bullet for the same title.
    let needle = format!("- [{title}]");
    if let Some(pos) = lines[h_idx + 1..end]
        .iter()
        .position(|l| l.trim_start().starts_with(&needle))
    {
        lines[h_idx + 1 + pos] = bullet;
    } else {
        lines.insert(end, bullet);
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Append a chronological log line, ensuring a single trailing newline.
pub async fn append_log(log_path: &Path, line: &str) -> Result<()> {
    let mut content = if log_path.exists() {
        tokio::fs::read_to_string(log_path).await.unwrap_or_default()
    } else {
        String::from("# Wiki Log\n")
    };
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(line.trim_end());
    content.push('\n');
    tokio::fs::write(log_path, content).await?;
    Ok(())
}

/// A wiki search hit (emit-only retrieval — no Qdrant).
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub path: String,
    pub title: String,
    pub score: u32,
    pub snippet: String,
}

/// Naive lexical search over `wiki/**/*.md`. Title/heading matches weigh more.
/// This is deliberately simple: the wiki is small enough that grep-class
/// scoring is sufficient, and it avoids any honeypot/Qdrant feedback.
pub fn search(dir: &Path, query: &str, limit: usize) -> Vec<SearchHit> {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|t| t.to_ascii_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    if terms.is_empty() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_md_files(dir, &mut files);

    let mut hits: Vec<SearchHit> = Vec::new();
    for path in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (fm, body) = split_frontmatter(&content);
        let title = fm
            .as_ref()
            .map(|f| f.title.clone())
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default()
            });

        let title_lc = title.to_ascii_lowercase();
        let body_lc = body.to_ascii_lowercase();
        let mut score = 0u32;
        for term in &terms {
            if title_lc.contains(term) {
                score += 5;
            }
            // Heading lines (start with #) weigh more than body.
            for line in body_lc.lines() {
                let occ = line.matches(term.as_str()).count() as u32;
                if occ == 0 {
                    continue;
                }
                if line.trim_start().starts_with('#') {
                    score += occ * 3;
                } else {
                    score += occ;
                }
            }
        }
        if score > 0 {
            hits.push(SearchHit {
                path: path.display().to_string(),
                title,
                score,
                snippet: first_snippet(&body, &terms),
            });
        }
    }

    hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.title.cmp(&b.title)));
    hits.truncate(limit.max(1));
    hits
}

fn collect_md_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

fn first_snippet(body: &str, terms: &[String]) -> String {
    for line in body.lines() {
        let line_lc = line.to_ascii_lowercase();
        if terms.iter().any(|t| line_lc.contains(t)) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let snippet: String = trimmed.chars().take(160).collect();
            return snippet;
        }
    }
    body.lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().chars().take(160).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("GZMO Platform!"), "gzmo-platform");
        assert_eq!(slugify("  Multi   Word  "), "multi-word");
        assert_eq!(slugify("***"), "untitled");
    }

    #[test]
    fn frontmatter_roundtrip() {
        let fm = PageFrontmatter::new("entity", "GZMO", "2026-06-07");
        let rendered = render_page(&fm, "Body text.");
        let (parsed, body) = split_frontmatter(&rendered);
        let parsed = parsed.expect("frontmatter parses");
        assert_eq!(parsed.title, "GZMO");
        assert_eq!(parsed.page_type, "entity");
        assert!(parsed.gzmo_synthetic);
        assert_eq!(body.trim(), "Body text.");
    }

    #[test]
    fn index_upsert_replaces_placeholder_and_dedupes() {
        let index = "# Wiki Index\n\n## Entities\n\n_No entries yet._\n\n## Concepts\n\n_No entries yet._\n";
        let once = upsert_index_entry(index, "Entities", "GZMO", "entities/gzmo.md", "the daemon");
        assert!(once.contains("- [GZMO](entities/gzmo.md) — the daemon"));
        // The Concepts section (and its placeholder) is untouched.
        assert!(once.contains("## Concepts"));
        // The Entities placeholder is removed (its bullet replaced it).
        let entities_section = once.split("## Concepts").next().unwrap();
        assert!(!entities_section.contains("_No entries yet._"));
        // Re-point the same title — no duplicate.
        let twice = upsert_index_entry(&once, "Entities", "GZMO", "entities/gzmo.md", "updated summary");
        assert_eq!(twice.matches("- [GZMO]").count(), 1);
        assert!(twice.contains("updated summary"));
    }

    #[test]
    fn search_ranks_title_matches_first() {
        let dir = std::env::temp_dir().join(format!("wiki_md_test_{}", std::process::id()));
        let ent = dir.join("entities");
        std::fs::create_dir_all(&ent).unwrap();
        std::fs::write(
            ent.join("gzmo.md"),
            "---\ntype: entity\ntitle: GZMO\ncreated: 2026-06-07\nupdated: 2026-06-07\nstatus: stable\n---\n\n# GZMO\n\nSovereign daemon.\n",
        )
        .unwrap();
        std::fs::write(
            ent.join("other.md"),
            "---\ntype: entity\ntitle: Other\ncreated: 2026-06-07\nupdated: 2026-06-07\nstatus: stable\n---\n\nMentions gzmo once.\n",
        )
        .unwrap();
        let hits = search(&dir, "gzmo", 10);
        std::fs::remove_dir_all(&dir).ok();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].title, "GZMO");
    }
}
