//! GZMO vault / wiki facts → OKF Concept markdown for OKForge OKCP.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::config::{WikiConfig, WikiOkforgeConfig};
use crate::okforge_client::OkforgeClient;
use crate::wiki_md;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkfConceptDraft {
    pub slug: String,
    pub title: String,
    pub body_md: String,
    pub tags: Vec<String>,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WikiPushReport {
    pub mode: String,
    pub origin: String,
    pub concepts_written: usize,
    pub commit_sha: String,
    pub branch: String,
    pub skipped_reason: String,
    pub paths: Vec<String>,
    /// Operator/Observatory signal: false when live push failed or was skipped unexpectedly.
    #[serde(default)]
    pub healthy: bool,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub error: String,
}

/// Convert `[[wikilink]]` / `[[slug|Title]]` to absolute OKF markdown links.
pub fn convert_wikilinks(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("]]") {
            let inner = &after[..end];
            let (target, title) = match inner.split_once('|') {
                Some((t, title)) => (t.trim(), title.trim()),
                None => (inner.trim(), inner.trim()),
            };
            let slug = wiki_md::slugify(target);
            out.push_str(&format!("[{title}](/concepts/{slug}.md)"));
            rest = &after[end + 2..];
        } else {
            out.push_str("[[");
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Quote a scalar for YAML double-quoted frontmatter values.
fn yaml_quote(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', "");
    format!("\"{escaped}\"")
}

/// Build OKF Concept file content (frontmatter + body).
pub fn render_concept(draft: &OkfConceptDraft) -> String {
    let ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let tags = draft
        .tags
        .iter()
        .filter(|t| !t.is_empty())
        .map(|t| yaml_quote(t))
        .collect::<Vec<_>>()
        .join(", ");
    let body = convert_wikilinks(&draft.body_md);
    let desc: String = draft
        .body_md
        .lines()
        .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .unwrap_or("GZMO-next knowledge concept")
        .chars()
        .take(160)
        .collect::<String>()
        .replace('\n', " ");
    format!(
        "---\ntype: Concept\ntitle: {}\ndescription: {}\ntags: [{tags}]\ntimestamp: {ts}\n---\n\n{body}\n",
        yaml_quote(&draft.title),
        yaml_quote(&desc),
    )
}

/// Load recent vault facts as concept drafts (verified semantic_vault rows).
pub fn drafts_from_vault(
    vault_db: &Path,
    origin: &str,
    limit: usize,
) -> Result<Vec<OkfConceptDraft>> {
    let conn = rusqlite::Connection::open(vault_db)
        .with_context(|| format!("open vault {}", vault_db.display()))?;
    let mut drafts = Vec::new();

    // Prefer honeypot (promoted); fall back to semantic_vault.
    let sql_honeypot =
        "SELECT id, content FROM honeypot WHERE is_latest = 1 ORDER BY rowid DESC LIMIT ?1";
    let sql_vault = "SELECT id, content FROM semantic_vault ORDER BY created_at DESC LIMIT ?1";

    let mut rows_ok = false;
    if let Ok(mut stmt) = conn.prepare(sql_honeypot) {
        if let Ok(mapped) = stmt.query_map([limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            for row in mapped.flatten() {
                drafts.push(fact_to_draft(&row.0, &row.1, origin));
            }
            rows_ok = !drafts.is_empty();
        }
    }
    if !rows_ok {
        let mut stmt = conn.prepare(sql_vault).context("prepare semantic_vault")?;
        let mapped = stmt.query_map([limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in mapped.flatten() {
            drafts.push(fact_to_draft(&row.0, &row.1, origin));
        }
    }
    Ok(drafts)
}

fn fact_to_draft(id: &str, content: &str, origin: &str) -> OkfConceptDraft {
    let content = content.trim();
    let title: String = content
        .chars()
        .take(72)
        .collect::<String>()
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ");
    let title = if title.is_empty() {
        format!("fact-{id}")
    } else {
        title
    };
    let slug_base = if id.len() >= 8 {
        format!("{}-{}", wiki_md::slugify(&title), &id[..8])
    } else {
        wiki_md::slugify(&title)
    };
    OkfConceptDraft {
        slug: slug_base,
        title: title.clone(),
        body_md: format!("# {title}\n\n{content}\n\nOrigin: `{origin}` (GZMO-next).\n"),
        tags: vec!["gzmo-next".into(), origin.to_string(), "verified".into()],
        origin: origin.to_string(),
    }
}

fn stamp(report: &mut WikiPushReport) {
    report.timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
}

/// Push drafts to OKForge via OKCP. Returns report (also suitable for JSON meta).
pub async fn push_concepts(
    wiki: &WikiConfig,
    drafts: &[OkfConceptDraft],
    origin: &str,
    dry_run: bool,
) -> Result<WikiPushReport> {
    let mut report = WikiPushReport {
        mode: if dry_run {
            "dry_run".into()
        } else {
            "live".into()
        },
        origin: origin.to_string(),
        ..Default::default()
    };
    stamp(&mut report);

    if !wiki.enabled {
        report.skipped_reason = "wiki.enabled=false".into();
        report.healthy = true; // intentional off
        return Ok(report);
    }
    if wiki.backend != "okforge" {
        report.skipped_reason = format!("wiki.backend={} (need okforge)", wiki.backend);
        report.healthy = false;
        return Ok(report);
    }
    let Some(okf_cfg) = wiki.okforge.as_ref() else {
        report.skipped_reason = "wiki.okforge missing".into();
        report.healthy = false;
        return Ok(report);
    };
    if drafts.is_empty() {
        report.skipped_reason = "no concepts to push".into();
        report.healthy = true; // empty vault is not a forge outage
        return Ok(report);
    }

    if dry_run {
        for d in drafts {
            report.paths.push(format!("concepts/{}.md", d.slug));
        }
        report.concepts_written = drafts.len();
        report.healthy = true;
        return Ok(report);
    }

    let client = OkforgeClient::from_config(okf_cfg)?;
    let session = client
        .session_start(okf_cfg, Some(&format!("gzmo-{origin}")))
        .await?;

    for d in drafts {
        let rel = format!("concepts/{}.md", d.slug);
        let content = render_concept(d);
        client
            .concept_write(
                &okf_cfg.owner,
                &okf_cfg.repo,
                &rel,
                &content,
                &session.session_id,
            )
            .await?;
        report.paths.push(rel);
    }

    if okf_cfg.auto_commit {
        let msg = format!("okf: gzmo-next {} push ({} concepts)", origin, drafts.len());
        let committed = client
            .session_commit(okf_cfg, &session.session_id, &msg)
            .await?;
        report.commit_sha = committed.commit_sha;
        report.branch = committed.branch;
        report.concepts_written = committed.files.max(drafts.len());
        report.healthy = !report.commit_sha.is_empty();
    } else {
        report.concepts_written = drafts.len();
        report.branch = session.branch;
        report.skipped_reason = "auto_commit=false — session left open".into();
        report.healthy = true;
    }

    info!(
        concepts = report.concepts_written,
        sha = %report.commit_sha,
        "wiki OKForge push complete"
    );
    Ok(report)
}

/// Convenience: vault → push using `[wiki.okforge]`.
pub async fn push_from_vault(
    wiki: &WikiConfig,
    vault_db: &Path,
    origin: &str,
    limit: usize,
    dry_run: bool,
) -> Result<WikiPushReport> {
    let drafts = drafts_from_vault(vault_db, origin, limit)?;
    push_concepts(wiki, &drafts, origin, dry_run).await
}

/// Load concept drafts from a JSON array (`[{id, content}]` or full draft objects).
pub fn drafts_from_json(raw: &str, origin: &str) -> Result<Vec<OkfConceptDraft>> {
    let v: serde_json::Value = serde_json::from_str(raw).context("parse wiki draft JSON")?;
    let arr = v
        .as_array()
        .or_else(|| v.get("facts").and_then(|x| x.as_array()))
        .or_else(|| v.get("drafts").and_then(|x| x.as_array()))
        .context("wiki draft JSON must be an array or {facts:[]} / {drafts:[]}")?;
    let mut drafts = Vec::with_capacity(arr.len());
    for (i, item) in arr.iter().enumerate() {
        if let Some(content) = item.get("content").and_then(|x| x.as_str()) {
            let id = item
                .get("id")
                .and_then(|x| x.as_str())
                .unwrap_or("json")
                .to_string();
            drafts.push(fact_to_draft(&id, content, origin));
            continue;
        }
        if let Some(body) = item.get("body_md").and_then(|x| x.as_str()) {
            let title = item
                .get("title")
                .and_then(|x| x.as_str())
                .unwrap_or("untitled")
                .to_string();
            let slug = item
                .get("slug")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| wiki_md::slugify(&title));
            let tags = item
                .get("tags")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_else(|| vec!["gzmo-next".into(), origin.to_string()]);
            drafts.push(OkfConceptDraft {
                slug,
                title,
                body_md: body.to_string(),
                tags,
                origin: origin.to_string(),
            });
            continue;
        }
        bail!("draft[{i}] needs content or body_md");
    }
    Ok(drafts)
}

/// Write `wiki-push-latest.json` even when the live push fails (Observatory honesty).
pub fn record_push_result(
    path: &Path,
    origin: &str,
    result: Result<WikiPushReport>,
) -> Result<WikiPushReport> {
    match result {
        Ok(report) => {
            write_push_report(path, &report)?;
            Ok(report)
        }
        Err(e) => {
            let mut report = WikiPushReport {
                mode: "live".into(),
                origin: origin.to_string(),
                error: e.to_string(),
                skipped_reason: e.to_string(),
                healthy: false,
                ..Default::default()
            };
            stamp(&mut report);
            write_push_report(path, &report)?;
            Err(e)
        }
    }
}

/// Write push report JSON for Observatory Body / scheduler meta.
pub fn write_push_report(path: &Path, report: &WikiPushReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json + "\n")?;
    Ok(())
}

/// Path to nightburst concept-gate artifact next to the vault DB.
pub fn concept_gate_path(vault_db: &Path) -> std::path::PathBuf {
    vault_db
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("concept-gate")
        .join("latest.json")
}

/// Whether concept-gate soft hold is enabled (`GZMO_CONCEPT_GATE`, default on).
pub fn concept_gate_enforced() -> bool {
    match std::env::var("GZMO_CONCEPT_GATE") {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            !(t.is_empty() || t == "0" || t == "false" || t == "off" || t == "no")
        }
        Err(_) => true,
    }
}

/// If the latest concept-gate verdict is HOLD, return a short reason.
/// Missing gate file → no hold (gate is advisory until first nightburst run).
pub fn concept_gate_hold_reason(vault_db: &Path) -> Option<String> {
    if !concept_gate_enforced() {
        return None;
    }
    let path = concept_gate_path(vault_db);
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let verdict = v.get("verdict")?.as_str()?;
    if !verdict.eq_ignore_ascii_case("HOLD") {
        return None;
    }
    let hold = v.get("hold").and_then(|x| x.as_u64()).unwrap_or(0);
    let checked = v.get("checked").and_then(|x| x.as_u64()).unwrap_or(0);
    Some(format!(
        "concept-gate HOLD ({hold}/{checked} concepts lack vault evidence); see {}",
        path.display()
    ))
}

#[allow(dead_code)]
pub fn okforge_cfg(wiki: &WikiConfig) -> Option<&WikiOkforgeConfig> {
    wiki.okforge.as_ref()
}

#[cfg(test)]
mod concept_gate_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn hold_reason_reads_verdict() {
        let dir =
            std::env::temp_dir().join(format!("gzmo-concept-gate-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let vault = dir.join("vault.db");
        std::fs::write(&vault, b"").unwrap();
        let gate_dir = dir.join("concept-gate");
        std::fs::create_dir_all(&gate_dir).unwrap();
        let mut f = std::fs::File::create(gate_dir.join("latest.json")).unwrap();
        write!(f, r#"{{"verdict":"HOLD","hold":2,"checked":3}}"#).unwrap();
        std::env::set_var("GZMO_CONCEPT_GATE", "1");
        let reason = concept_gate_hold_reason(&vault).expect("hold");
        assert!(reason.contains("HOLD"));
        std::env::set_var("GZMO_CONCEPT_GATE", "0");
        assert!(concept_gate_hold_reason(&vault).is_none());
        std::env::remove_var("GZMO_CONCEPT_GATE");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn convert_wikilinks_to_okf_paths() {
        let out = convert_wikilinks("see [[Foo Bar|Title]] and [[baz]]");
        assert!(out.contains("[Title](/concepts/foo-bar.md)"));
        assert!(out.contains("[baz](/concepts/baz.md)"));
    }

    #[test]
    fn drafts_from_json_facts() {
        let raw = r#"[{"id":"abc12345deadbeef","content":"Honeypot fact about Lint"}]"#;
        let drafts = drafts_from_json(raw, "living").unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].origin, "living");
        assert!(drafts[0].body_md.contains("Honeypot fact"));
    }

    #[test]
    fn record_push_writes_unhealthy_on_err() {
        let dir = std::env::temp_dir().join(format!("gzmo-wiki-push-err-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let meta = dir.join("wiki-push-latest.json");
        let err = anyhow::anyhow!("OKCP session.start failed (503): down");
        let r = record_push_result(&meta, "manual", Err(err));
        assert!(r.is_err());
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&meta).unwrap()).unwrap();
        assert_eq!(v["healthy"], false);
        assert!(v["error"].as_str().unwrap().contains("503"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
