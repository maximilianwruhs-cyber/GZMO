//! Workflow skills — progressive-disclosure engineering contracts (`SKILL.md`).
//!
//! Separate from Chaos pantheon slash skills (`crate::skills`). These are
//! model-invoked / operator-invoked discipline packs (grill, tdd, …).

mod activate;

pub use activate::ActivateWorkflowSkillTool;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

/// YAML frontmatter for a workflow `SKILL.md`.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub requires_evidence: bool,
}

/// Cheap index entry (frontmatter only).
#[derive(Debug, Clone)]
pub struct WorkflowIndexEntry {
    pub meta: WorkflowFrontmatter,
    pub path: PathBuf,
}

/// Loaded skill body ready to inject.
#[derive(Debug, Clone)]
pub struct WorkflowSkillBody {
    pub meta: WorkflowFrontmatter,
    pub body: String,
    pub path: PathBuf,
}

/// Session-scoped active workflow state.
#[derive(Debug, Default)]
pub struct WorkflowSessionState {
    active: Vec<String>,
    pub last_handoff: Option<PathBuf>,
}

impl WorkflowSessionState {
    pub fn active_names(&self) -> &[String] {
        &self.active
    }

    pub fn clear(&mut self) {
        self.active.clear();
    }
}

pub type SharedWorkflowSession = Arc<Mutex<WorkflowSessionState>>;

/// Indexed workflow skill pack.
#[derive(Debug, Clone)]
pub struct WorkflowSkillIndex {
    entries: HashMap<String, WorkflowIndexEntry>,
    dir: PathBuf,
    pub max_active: usize,
    pub model_can_activate: bool,
    pub handoff_dir: PathBuf,
}

impl WorkflowSkillIndex {
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
            dir: PathBuf::from("skills/workflows"),
            max_active: 2,
            model_can_activate: true,
            handoff_dir: PathBuf::from("data-next/handoffs"),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.entries.keys().map(|s| s.as_str()).collect();
        names.sort_unstable();
        names
    }

    pub fn get(&self, name: &str) -> Option<&WorkflowIndexEntry> {
        self.entries.get(name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// Scan `dir/*/SKILL.md` and parse frontmatter only.
    pub fn load_from_dir(
        dir: impl AsRef<Path>,
        max_active: usize,
        model_can_activate: bool,
        handoff_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        let mut entries = HashMap::new();
        if !dir.is_dir() {
            tracing::warn!(path = %dir.display(), "workflow skills dir missing");
            return Ok(Self {
                entries,
                dir,
                max_active,
                model_can_activate,
                handoff_dir: handoff_dir.as_ref().to_path_buf(),
            });
        }

        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("read workflow skills dir {}", dir.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let skill_path = entry.path().join("SKILL.md");
            if !skill_path.is_file() {
                continue;
            }
            let raw = std::fs::read_to_string(&skill_path)
                .with_context(|| format!("read {}", skill_path.display()))?;
            let meta = parse_frontmatter(&raw)
                .with_context(|| format!("parse frontmatter {}", skill_path.display()))?;
            let name = meta.name.clone();
            entries.insert(
                name,
                WorkflowIndexEntry {
                    meta,
                    path: skill_path,
                },
            );
        }

        tracing::info!(count = entries.len(), dir = %dir.display(), "Loaded workflow skills");
        Ok(Self {
            entries,
            dir,
            max_active,
            model_can_activate,
            handoff_dir: handoff_dir.as_ref().to_path_buf(),
        })
    }

    /// Load full body for injection.
    pub fn load_body(&self, name: &str) -> Result<WorkflowSkillBody> {
        let entry = self
            .entries
            .get(name)
            .with_context(|| format!("unknown workflow skill: {name}"))?;
        let raw = std::fs::read_to_string(&entry.path)
            .with_context(|| format!("read {}", entry.path.display()))?;
        let body = strip_frontmatter(&raw).to_string();
        Ok(WorkflowSkillBody {
            meta: entry.meta.clone(),
            body,
            path: entry.path.clone(),
        })
    }

    /// Compact index block for the system prompt (name + description only).
    pub fn prompt_index_block(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        let mut lines = vec![
            "\n\n## Workflow skills (engineering discipline)".to_string(),
            "Activate with slash (`/grill`, `/tdd`, …) or tool `activate_workflow_skill`."
                .to_string(),
            "Available:".to_string(),
        ];
        let mut names = self.names();
        names.sort_unstable();
        for name in names {
            if let Some(e) = self.entries.get(name) {
                lines.push(format!("- **{}**: {}", e.meta.name, e.meta.description));
            }
        }
        lines.join("\n")
    }

    /// Activate a skill into session state; returns inject content.
    pub fn activate(
        &self,
        session: &SharedWorkflowSession,
        name: &str,
        args: &str,
    ) -> Result<String> {
        if !self.has(name) {
            bail!("Unknown workflow skill: {name}");
        }
        let skill = self.load_body(name)?;
        {
            let mut state = session
                .lock()
                .map_err(|_| anyhow::anyhow!("workflow session lock poisoned"))?;
            if !state.active.iter().any(|n| n == name) {
                while state.active.len() >= self.max_active {
                    state.active.remove(0);
                }
                state.active.push(name.to_string());
            }
        }

        let mut out = format!(
            "[Workflow /{}]\n# {}\n\n{}\n",
            skill.meta.name, skill.meta.name, skill.body
        );
        if skill.meta.requires_evidence {
            out.push_str(
                "\n**Evidence gate:** Do not claim done without citing tool output (tests/lint/repro).\n",
            );
        }
        if !args.trim().is_empty() {
            out.push_str(&format!("\n**Operator topic / args:** {}\n", args.trim()));
        }
        out.push_str(
            "\nFollow this workflow now. Ask clarifying questions before coding when the skill requires it.\n",
        );
        Ok(out)
    }

    /// Write a handoff markdown file and record path on the session.
    pub fn write_handoff(
        &self,
        session: &SharedWorkflowSession,
        session_id: &str,
        content: &str,
    ) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.handoff_dir)
            .with_context(|| format!("create handoff dir {}", self.handoff_dir.display()))?;
        let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
        let path = self
            .handoff_dir
            .join(format!("handoff-{session_id}-{stamp}.md"));
        std::fs::write(&path, content)
            .with_context(|| format!("write handoff {}", path.display()))?;
        if let Ok(mut state) = session.lock() {
            state.last_handoff = Some(path.clone());
        }
        Ok(path)
    }

    /// Most recent handoff file in the configured directory (by mtime).
    pub fn latest_handoff(&self) -> Option<PathBuf> {
        let read = std::fs::read_dir(&self.handoff_dir).ok()?;
        let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let modified = entry.metadata().ok().and_then(|m| m.modified().ok())?;
            match &best {
                Some((t, _)) if modified <= *t => {}
                _ => best = Some((modified, path)),
            }
        }
        best.map(|(_, p)| p)
    }
}

/// Parse YAML frontmatter between leading `---` fences.
pub fn parse_frontmatter(raw: &str) -> Result<WorkflowFrontmatter> {
    let rest = raw.strip_prefix("---").context("missing opening ---")?;
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let end = rest.find("\n---").context("missing closing ---")?;
    let yaml = &rest[..end];
    let meta: WorkflowFrontmatter =
        serde_yaml::from_str(yaml).context("invalid workflow skill YAML")?;
    if meta.name.trim().is_empty() {
        bail!("workflow skill name is empty");
    }
    Ok(meta)
}

/// Body after frontmatter (or full raw if no fences).
pub fn strip_frontmatter(raw: &str) -> &str {
    let Some(rest) = raw.strip_prefix("---") else {
        return raw;
    };
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let Some(end) = rest.find("\n---") else {
        return raw;
    };
    let after = &rest[end + 4..]; // skip \n---
    after.strip_prefix('\n').unwrap_or(after)
}

/// Build index from config helpers.
pub fn load_from_config(cfg: &crate::config::WorkflowSkillsConfig) -> Result<WorkflowSkillIndex> {
    if !cfg.enabled {
        return Ok(WorkflowSkillIndex::empty());
    }
    WorkflowSkillIndex::load_from_dir(
        &cfg.dir,
        cfg.max_active,
        cfg.model_can_activate,
        &cfg.handoff_dir,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_frontmatter_and_body() {
        let raw = r#"---
name: grill
description: Pressure-test a plan
triggers:
  - grill
  - grill-me
requires_evidence: false
---

# Grill

Ask hard questions.
"#;
        let meta = parse_frontmatter(raw).unwrap();
        assert_eq!(meta.name, "grill");
        assert_eq!(meta.triggers.len(), 2);
        assert!(!meta.requires_evidence);
        let body = strip_frontmatter(raw);
        assert!(body.contains("# Grill"));
        assert!(!body.contains("name: grill"));
    }

    #[test]
    fn load_index_activate_and_cap() {
        let dir = tempfile_dir();
        write_skill(
            &dir,
            "grill",
            "---\nname: grill\ndescription: Grill\nrequires_evidence: false\n---\n\nBody grill\n",
        );
        write_skill(
            &dir,
            "tdd",
            "---\nname: tdd\ndescription: TDD\nrequires_evidence: true\n---\n\nBody tdd\n",
        );
        write_skill(
            &dir,
            "review",
            "---\nname: review\ndescription: Review\n---\n\nBody review\n",
        );

        let handoff = dir.join("handoffs");
        let idx = WorkflowSkillIndex::load_from_dir(&dir, 2, true, &handoff).unwrap();
        assert_eq!(idx.len(), 3);
        assert!(idx.has("grill"));
        let block = idx.prompt_index_block();
        assert!(block.contains("grill"));

        let session = Arc::new(Mutex::new(WorkflowSessionState::default()));
        let inject = idx.activate(&session, "grill", "ship auth").unwrap();
        assert!(inject.contains("[Workflow /grill]"));
        assert!(inject.contains("ship auth"));
        assert_eq!(
            session.lock().unwrap().active_names(),
            &["grill".to_string()]
        );

        let _ = idx.activate(&session, "tdd", "").unwrap();
        assert_eq!(session.lock().unwrap().active_names().len(), 2);
        let _ = idx.activate(&session, "review", "").unwrap();
        let active = session.lock().unwrap().active_names().to_vec();
        assert_eq!(active.len(), 2);
        assert!(!active.iter().any(|n| n == "grill"));
        assert!(active.iter().any(|n| n == "tdd"));
        assert!(active.iter().any(|n| n == "review"));

        let path = idx
            .write_handoff(&session, "sess1", "# Handoff\n\nNext: finish TDD.\n")
            .unwrap();
        assert!(path.is_file());
        assert_eq!(session.lock().unwrap().last_handoff.as_ref(), Some(&path));
        assert_eq!(idx.latest_handoff().as_ref(), Some(&path));
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gzmo-wf-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_skill(root: &Path, name: &str, content: &str) {
        let d = root.join(name);
        std::fs::create_dir_all(&d).unwrap();
        let mut f = std::fs::File::create(d.join("SKILL.md")).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }
}
