//! WikiEngine — deterministic maintenance of the git-tracked `wiki/` layer.
//!
//! Sits between raw RAG retrieval and `DREAMS.md`. Pages are derived from
//! already-verified vault facts (no new LLM extraction), so retrieval is
//! **emit-only**: [`WikiEngine::search`] greps over `wiki/*.md` and emitted
//! pages are never re-ingested into the honeypot (which would create circular
//! facts — see the `gzmo_synthetic` guard in `ingest.rs`).
//!
//! Operations map to the "Knowledge Gardener" duties in `WIKI.md`:
//! - [`WikiEngine::emit_source_page`] — called from `IngestEngine` on promotion.
//! - [`WikiEngine::sync`] — rebuild `index.md` from pages on disk (gardening).
//! - [`WikiEngine::lint`] — structural health report (orphans, missing pages).
//! - [`WikiEngine::search`] / [`WikiEngine::file_back`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::NaiveDate;
use tracing::info;

use crate::config::WikiConfig;
use crate::memory::kg_extract::{VerifiedEntity, VerifiedRelation};
use crate::wiki_md::{
    self, append_log, read_page, upsert_index_entry, upsert_page, PageFrontmatter, SearchHit,
};

pub struct WikiEngine {
    config: WikiConfig,
}

#[derive(Debug, Default)]
pub struct EmitReport {
    pub source_page: String,
    pub entity_pages: usize,
}

#[derive(Debug, Default)]
pub struct LintReport {
    pub pages: usize,
    pub orphans: Vec<String>,
    pub missing_frontmatter: Vec<String>,
    pub broken_links: Vec<String>,
    pub stale: Vec<String>,
    pub report_path: String,
}

#[derive(Debug, Default)]
pub struct SyncReport {
    pub pages: usize,
    pub index_entries: usize,
}

impl WikiEngine {
    pub fn new(config: WikiConfig) -> Self {
        Self { config }
    }

    fn dir(&self) -> PathBuf {
        PathBuf::from(&self.config.directory)
    }
    fn entities_dir(&self) -> PathBuf {
        self.dir().join("entities")
    }
    fn concepts_dir(&self) -> PathBuf {
        self.dir().join("concepts")
    }
    fn sources_dir(&self) -> PathBuf {
        self.dir().join("sources")
    }
    fn index_path(&self) -> PathBuf {
        PathBuf::from(&self.config.index_path)
    }
    fn log_path(&self) -> PathBuf {
        PathBuf::from(&self.config.log_path)
    }

    /// Emit / update a `wiki/sources` page and the entity pages it touches,
    /// using only already-verified facts. Called from `IngestEngine`.
    pub async fn emit_source_page(
        &self,
        source_file: &str,
        entities: &[VerifiedEntity],
        relations: &[VerifiedRelation],
        date: NaiveDate,
    ) -> Result<EmitReport> {
        if !self.config.enabled {
            return Ok(EmitReport::default());
        }
        let date_str = date.format("%Y-%m-%d").to_string();
        let stem = source_file
            .rsplit_once('.')
            .map(|(s, _)| s)
            .unwrap_or(source_file);
        let source_title = stem.to_string();
        let source_slug = wiki_md::slugify(stem);

        // --- entity pages -------------------------------------------------
        let mut entity_links: Vec<String> = Vec::new();
        for ve in entities {
            let name = ve.entity.name.trim();
            if name.is_empty() {
                continue;
            }
            let slug = wiki_md::slugify(name);
            let path = self.entities_dir().join(format!("{slug}.md"));
            let (fm, body) = match read_page(&path).await {
                Some((fm, body)) => (fm, body),
                None => (
                    PageFrontmatter::new("entity", name, &date_str),
                    format!("# {name}\n\nType: {}\n", ve.entity.entity_type),
                ),
            };
            let section_marker = format!("## From [[{source_slug}|{source_title}]]");
            let mut new_body = body.clone();
            if !new_body.contains(&section_marker) {
                let mut section = format!("\n{section_marker} ({date_str})\n");
                for obs in &ve.entity.observations {
                    let obs = obs.trim();
                    if !obs.is_empty() {
                        section.push_str(&format!("- {obs}\n"));
                    }
                }
                new_body = format!("{}\n{}", new_body.trim_end(), section);
            }
            let source_count = new_body.matches("## From [[").count() as u32;
            let mut fm = fm;
            fm.updated = date_str.clone();
            fm.sources = source_count;
            upsert_page(&path, fm, &new_body).await?;

            // index catalog line
            let index = read_string(&self.index_path()).await;
            let updated = upsert_index_entry(
                &index,
                "Entities",
                name,
                &format!("entities/{slug}.md"),
                &format!("{} ({} source(s))", ve.entity.entity_type, source_count),
            );
            write_string(&self.index_path(), &updated).await?;

            entity_links.push(format!("- [[{slug}|{name}]] ({})", ve.entity.entity_type));
        }

        // --- source summary page -----------------------------------------
        let mut body = format!("# {source_title}\n\nIngested source summary ({date_str}).\n");
        if !entity_links.is_empty() {
            body.push_str("\n## Entities\n");
            for link in &entity_links {
                body.push_str(link);
                body.push('\n');
            }
        }
        if !relations.is_empty() {
            body.push_str("\n## Relations\n");
            for vr in relations {
                body.push_str(&format!(
                    "- {} → {} → {}\n",
                    vr.relation.from, vr.relation.relation_type, vr.relation.to
                ));
            }
        }
        let mut fm = PageFrontmatter::new("source", &source_title, &date_str);
        fm.sources = 1;
        fm.status = "stable".to_string();
        let source_path = self.sources_dir().join(format!("{source_slug}.md"));
        upsert_page(&source_path, fm, &body).await?;

        // source catalog line + log
        let index = read_string(&self.index_path()).await;
        let updated = upsert_index_entry(
            &index,
            "Sources",
            &source_title,
            &format!("sources/{source_slug}.md"),
            &format!(
                "{} entities, {} relations",
                entity_links.len(),
                relations.len()
            ),
        );
        write_string(&self.index_path(), &updated).await?;
        append_log(
            &self.log_path(),
            &format!("## [{date_str}] ingest | {source_title}"),
        )
        .await?;

        let report = EmitReport {
            source_page: source_path.display().to_string(),
            entity_pages: entity_links.len(),
        };
        info!(
            source = %report.source_page,
            entities = report.entity_pages,
            "WikiEngine emitted source page"
        );
        Ok(report)
    }

    /// Rebuild `index.md` from the pages currently on disk (gardening pass).
    /// Deterministic and DB-free: the canonical fact→page path is the ingest
    /// emit hook; `sync` keeps the catalog consistent with what is on disk.
    pub async fn sync(&self) -> Result<SyncReport> {
        let mut by_category: BTreeMap<&str, Vec<(String, String, String)>> = BTreeMap::new();
        let mut total = 0usize;

        for (category, dir) in [
            ("Entities", self.entities_dir()),
            ("Concepts", self.concepts_dir()),
            ("Sources", self.sources_dir()),
        ] {
            let mut rows = Vec::new();
            for path in list_md(&dir) {
                let Some((fm, _body)) = read_page(&path).await else {
                    continue;
                };
                let fname = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let sub = dir
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let link = format!("{sub}/{fname}");
                let summary = format!("{} — {} source(s)", fm.status, fm.sources);
                rows.push((fm.title, link, summary));
                total += 1;
            }
            rows.sort();
            by_category.insert(category, rows);
        }

        let mut index = read_string(&self.index_path()).await;
        if index.trim().is_empty() {
            index = "---\ntitle: Wiki Index\ntype: index\n---\n\n# Wiki Index\n".to_string();
        }
        let mut entries = 0usize;
        for (category, rows) in &by_category {
            for (title, link, summary) in rows {
                index = upsert_index_entry(&index, category, title, link, summary);
                entries += 1;
            }
        }
        write_string(&self.index_path(), &index).await?;

        let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
        append_log(
            &self.log_path(),
            &format!("## [{date_str}] sync | {entries} index entries over {total} pages"),
        )
        .await?;

        Ok(SyncReport {
            pages: total,
            index_entries: entries,
        })
    }

    /// Structural health check (report-only). Writes `sources/_lint-DATE.md`.
    pub async fn lint(&self) -> Result<LintReport> {
        let pages = list_md_recursive(&self.dir());
        let mut report = LintReport {
            pages: pages.len(),
            ..Default::default()
        };

        // Build link graph: which page basenames are referenced via [[link]].
        let mut referenced: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut page_slugs: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut contents: Vec<(PathBuf, String)> = Vec::new();
        for path in &pages {
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                page_slugs.insert(stem.to_string());
            }
            for target in extract_wikilinks(&content) {
                referenced.insert(target);
            }
            contents.push((path.clone(), content));
        }

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        for (path, content) in &contents {
            let name = rel_display(path, &self.dir());
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            // Skip index/log/lint reports themselves.
            if stem == "index" || stem == "log" || stem.starts_with("_lint-") {
                continue;
            }
            let (fm, _body) = wiki_md::split_frontmatter(content);
            match &fm {
                None => report.missing_frontmatter.push(name.clone()),
                Some(f) => {
                    if f.status == "stale" {
                        report.stale.push(name.clone());
                    }
                }
            }
            // Orphan: never referenced by any other page.
            if !referenced.contains(&stem) {
                report.orphans.push(name.clone());
            }
            // Broken links: [[target]] with no matching page slug.
            for target in extract_wikilinks(content) {
                if !page_slugs.contains(&target) {
                    report
                        .broken_links
                        .push(format!("{name} -> [[{target}]]"));
                }
            }
        }
        report.orphans.sort();
        report.orphans.dedup();
        report.broken_links.sort();
        report.broken_links.dedup();

        let body = render_lint(&report, &today);
        let report_path = self.sources_dir().join(format!("_lint-{today}.md"));
        if let Some(parent) = report_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        tokio::fs::write(&report_path, body).await?;
        report.report_path = report_path.display().to_string();
        append_log(
            &self.log_path(),
            &format!(
                "## [{today}] lint | {} orphans, {} broken links, {} missing frontmatter",
                report.orphans.len(),
                report.broken_links.len(),
                report.missing_frontmatter.len()
            ),
        )
        .await?;
        Ok(report)
    }

    /// Emit-only lexical search over `wiki/*.md`.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        wiki_md::search(&self.dir(), query, limit)
    }

    /// File a query answer back into the wiki as a `concepts/` page.
    pub async fn file_back(&self, title: &str, body: &str) -> Result<String> {
        let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let slug = wiki_md::slugify(title);
        let path = self.concepts_dir().join(format!("{slug}.md"));
        let mut fm = PageFrontmatter::new("concept", title, &date_str);
        fm.updated = date_str.clone();
        let page_body = format!("# {title}\n\n{}\n", body.trim());
        upsert_page(&path, fm, &page_body).await?;

        let index = read_string(&self.index_path()).await;
        let updated = upsert_index_entry(
            &index,
            "Concepts",
            title,
            &format!("concepts/{slug}.md"),
            "filed-back query answer",
        );
        write_string(&self.index_path(), &updated).await?;
        append_log(&self.log_path(), &format!("## [{date_str}] query | {title}")).await?;
        Ok(path.display().to_string())
    }
}

// ─── helpers ────────────────────────────────────────────────────────────────

async fn read_string(path: &Path) -> String {
    if path.exists() {
        tokio::fs::read_to_string(path).await.unwrap_or_default()
    } else {
        String::new()
    }
}

async fn write_string(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    tokio::fs::write(path, content).await?;
    Ok(())
}

fn list_md(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("md") {
                out.push(p);
            }
        }
    }
    out
}

fn list_md_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(list_md_recursive(&p));
            } else if p.extension().and_then(|x| x.to_str()) == Some("md") {
                out.push(p);
            }
        }
    }
    out
}

fn rel_display(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Extract `[[target]]` and `[[target|alias]]` link targets (basenames).
fn extract_wikilinks(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = content;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find("]]") {
            let inner = &after[..end];
            let target = inner.split('|').next().unwrap_or(inner).trim();
            // Normalize to basename slug (strip any path / extension).
            let base = target
                .rsplit('/')
                .next()
                .unwrap_or(target)
                .trim_end_matches(".md");
            if !base.is_empty() {
                out.push(base.to_string());
            }
            rest = &after[end + 2..];
        } else {
            break;
        }
    }
    out
}

fn render_lint(report: &LintReport, date: &str) -> String {
    let mut s = format!(
        "---\ntype: source\ntitle: \"Lint {date}\"\ncreated: {date}\nupdated: {date}\nstatus: stable\ngzmo_synthetic: true\n---\n\n# Wiki Lint — {date}\n\nPages scanned: {}\n",
        report.pages
    );
    let section = |s: &mut String, title: &str, items: &[String]| {
        s.push_str(&format!("\n## {title} ({})\n", items.len()));
        if items.is_empty() {
            s.push_str("_none_\n");
        } else {
            for it in items {
                s.push_str(&format!("- {it}\n"));
            }
        }
    };
    section(&mut s, "Orphan pages", &report.orphans);
    section(&mut s, "Broken wikilinks", &report.broken_links);
    section(&mut s, "Missing frontmatter", &report.missing_frontmatter);
    section(&mut s, "Stale pages", &report.stale);
    s.push_str("\n_Report-only: fixes stay human-directed (see WIKI.md)._\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::kg_extract::{KgEntity, KgRelation, VerifiedEntity, VerifiedRelation};

    fn ve(name: &str, ty: &str, obs: &[&str]) -> VerifiedEntity {
        VerifiedEntity {
            entity: KgEntity {
                name: name.to_string(),
                entity_type: ty.to_string(),
                observations: obs.iter().map(|s| s.to_string()).collect(),
            },
            confidence: 0.9,
            evidence: "evidence span".to_string(),
        }
    }

    fn engine_in(dir: &Path) -> WikiEngine {
        let cfg = WikiConfig {
            directory: dir.display().to_string(),
            index_path: dir.join("index.md").display().to_string(),
            log_path: dir.join("log.md").display().to_string(),
            ..WikiConfig::default()
        };
        WikiEngine::new(cfg)
    }

    #[tokio::test]
    async fn emit_creates_source_and_entity_pages() {
        let dir = std::env::temp_dir().join(format!("wiki_engine_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let engine = engine_in(&dir);
        let date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        let entities = vec![ve("GZMO", "System", &["Sovereign daemon", "Runs on llama.cpp"])];
        let relations = vec![VerifiedRelation {
            relation: KgRelation {
                from: "GZMO".to_string(),
                to: "llama.cpp".to_string(),
                relation_type: "RUNS_ON".to_string(),
            },
            confidence: 0.9,
            evidence: "evidence".to_string(),
        }];

        let report = engine
            .emit_source_page("Architecture.md", &entities, &relations, date)
            .await
            .unwrap();
        assert_eq!(report.entity_pages, 1);
        assert!(dir.join("entities/gzmo.md").exists());
        assert!(dir.join("sources/architecture.md").exists());

        let index = std::fs::read_to_string(dir.join("index.md")).unwrap();
        assert!(index.contains("- [GZMO](entities/gzmo.md)"));
        assert!(index.contains("- [Architecture](sources/architecture.md)"));

        let log = std::fs::read_to_string(dir.join("log.md")).unwrap();
        assert!(log.contains("## [2026-06-07] ingest | Architecture"));

        // Re-emit same source — no duplicate "## From" section.
        engine
            .emit_source_page("Architecture.md", &entities, &relations, date)
            .await
            .unwrap();
        let ent = std::fs::read_to_string(dir.join("entities/gzmo.md")).unwrap();
        assert_eq!(ent.matches("## From [[architecture|Architecture]]").count(), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn lint_flags_orphans_and_broken_links() {
        let dir = std::env::temp_dir().join(format!("wiki_lint_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("entities")).unwrap();
        std::fs::write(
            dir.join("entities/lonely.md"),
            "---\ntype: entity\ntitle: Lonely\ncreated: 2026-06-07\nupdated: 2026-06-07\nstatus: draft\n---\n\n# Lonely\n\nLinks to [[ghost]].\n",
        )
        .unwrap();
        let engine = engine_in(&dir);
        let report = engine.lint().await.unwrap();
        assert!(report.orphans.iter().any(|o| o.contains("lonely")));
        assert!(report.broken_links.iter().any(|b| b.contains("ghost")));
        assert!(Path::new(&report.report_path).exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
