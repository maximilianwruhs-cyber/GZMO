//! Pre-LLM document preparation: frontmatter, doc class, body cleanup.

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Frontmatter {
    pub migration_id: Option<String>,
    pub source: Option<String>,
    pub notebook: Option<String>,
    pub original_path: Option<String>,
    pub wave: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocClass {
    AgentSpec,
    Reference,
    ChatExport,
    Narrative,
}

/// Split YAML frontmatter from body. Returns empty frontmatter if none.
pub fn split_frontmatter(raw: &str) -> (Frontmatter, String) {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return (Frontmatter::default(), raw.to_string());
    }
    let after_first = trimmed.strip_prefix("---").unwrap_or(trimmed).trim_start();
    let rest = after_first
        .strip_prefix('\n')
        .or_else(|| after_first.strip_prefix("\r\n"))
        .unwrap_or(after_first);
    let end = rest.find("\n---").or_else(|| rest.find("\r\n---"));
    let Some(end) = end else {
        return (Frontmatter::default(), raw.to_string());
    };
    let yaml = &rest[..end];
    let body_start = end + 1;
    let body = if rest[body_start..].starts_with('\n') {
        &rest[body_start + 1..]
    } else if rest[body_start..].starts_with("\r\n") {
        &rest[body_start + 2..]
    } else if rest[body_start..].starts_with("---") {
        rest[body_start..]
            .strip_prefix("---")
            .unwrap_or(&rest[body_start..])
            .trim_start_matches(['\n', '\r'])
    } else {
        &rest[body_start..]
    };
    let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap_or_default();
    (fm, body.to_string())
}

pub fn classify_document(file_name: &str, frontmatter: &Frontmatter, body: &str) -> DocClass {
    let path_key = frontmatter
        .original_path
        .as_deref()
        .unwrap_or(file_name)
        .to_lowercase();
    let name_key = file_name.to_lowercase();

    if (path_key.contains("chat_history") || name_key.contains("chat_history"))
        && body.contains("USER:")
    {
        return DocClass::ChatExport;
    }
    // NotebookLM chat exports often have MODEL-only turns (no USER: line).
    if (path_key.contains("chat_history") || name_key.contains("chat_history"))
        && body.contains("MODEL:")
    {
        return DocClass::ChatExport;
    }
    if body.contains("MODEL:") && body.contains("USER:") {
        return DocClass::ChatExport;
    }
    if path_key.contains("/agents/") || (name_key.contains("agents") && name_key.ends_with("md.md"))
    {
        return DocClass::AgentSpec;
    }
    if path_key.contains("readme")
        || path_key.contains("roadmap")
        || path_key.contains("concept")
        || path_key.contains("/docs/")
        || path_key.contains("systemkonzept")
        || path_key.contains("scientific_foundations")
        || path_key.contains("visual_identity")
        || path_key.contains("judge_dna")
        || name_key.contains("readmemd")
        || name_key.contains("roadmapmd")
        || name_key.contains("conceptmd")
        || name_key.contains("docs")
        || name_key.contains("systemkonzept")
        || name_key.contains("scientific_foundationsmd")
        || name_key.contains("visual_identitymd")
        || name_key.contains("judge_dnamd")
    {
        return DocClass::Reference;
    }
    // NotebookLM source scrapes (html exports) — facts over relations.
    if (path_key.contains("sources") || name_key.contains("sources"))
        && (name_key.contains("html") || name_key.contains("quelltext"))
    {
        return DocClass::Reference;
    }
    DocClass::Narrative
}

pub fn clean_body(doc_class: DocClass, body: &str) -> String {
    let mut lines: Vec<String> = body
        .lines()
        .filter(|line| {
            let t = line.trim();
            if t.is_empty() {
                return true;
            }
            let lower = t.to_lowercase();
            if doc_class == DocClass::ChatExport {
                if lower.contains("sources do not contain")
                    || lower.contains("i can search")
                    || lower.contains("would you like me to")
                {
                    return false;
                }
            }
            if lower.contains("skip to content")
                || lower.starts_with("feat: feat:")
                || lower == "github"
                || lower == "navigation"
            {
                return false;
            }
            true
        })
        .map(|s| s.to_string())
        .collect();
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

/// Normalize NotebookLM JSON mind-map exports into a flat outline for extraction.
pub fn json_tree_to_outline(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if !trimmed.starts_with('{') || !trimmed.contains("\"children\"") {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let mut lines = Vec::new();
    flatten_json_tree(&value, 0, &mut lines);
    if lines.is_empty() {
        return None;
    }
    Some(lines.join("\n"))
}

fn flatten_json_tree(node: &serde_json::Value, depth: usize, out: &mut Vec<String>) {
    if let Some(name) = node.get("name").and_then(|v| v.as_str()) {
        let indent = "  ".repeat(depth);
        out.push(format!("{indent}- {name}"));
    }
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            flatten_json_tree(child, depth + 1, out);
        }
    }
}

/// Clean body and convert JSON tree exports when detected.
pub fn prepare_body(doc_class: DocClass, body: &str) -> String {
    let cleaned = clean_body(doc_class, body);
    json_tree_to_outline(&cleaned).unwrap_or(cleaned)
}

/// Derive primary agent display name from path or wave filename (AgentSpec).
pub fn infer_agent_name(file_name: &str, frontmatter: &Frontmatter) -> Option<String> {
    if let Some(path) = &frontmatter.original_path {
        if let Some(agent) = agent_slug_from_path(path) {
            return Some(slug_to_agent_name(&agent));
        }
    }
    let name = file_name.to_lowercase();
    let Some(start) = name.find("agents") else {
        return None;
    };
    let rest = &name[start + 6..];
    let slug = wave_agent_slug(rest);
    if slug.is_empty() {
        return None;
    }
    Some(slug_to_agent_name(slug))
}

fn wave_agent_slug(file_tail: &str) -> &str {
    let s = file_tail.trim();
    for suffix in ["_agentmd.md", "_agent.md", "md.md", ".md"] {
        if let Some(stem) = s.strip_suffix(suffix) {
            return stem.trim_end_matches('_');
        }
    }
    s.trim_end_matches('_')
}

fn agent_slug_from_path(path: &str) -> Option<String> {
    let path = path.replace('\\', "/");
    for segment in path.split('/') {
        if segment.ends_with(".md") || segment.ends_with(".html") {
            if path.contains("/agents/") || path.contains("agents/") {
                let slug = segment.trim_end_matches(".md").trim_end_matches(".html");
                if !slug.is_empty() {
                    return Some(slug.to_string());
                }
            }
        }
    }
    None
}

fn slug_to_agent_name(slug: &str) -> String {
    slug.split(['_', '-', '/'])
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

pub fn extract_system_prompt(doc_class: DocClass) -> &'static str {
    match doc_class {
        DocClass::AgentSpec => INGEST_EXTRACT_AGENT_SPEC,
        DocClass::Reference => INGEST_EXTRACT_REFERENCE,
        DocClass::ChatExport => INGEST_EXTRACT_CHAT,
        DocClass::Narrative => INGEST_EXTRACT_NARRATIVE,
    }
}

pub fn build_provenance(file_name: &str, frontmatter: &Frontmatter) -> String {
    let mut s = format!("[ingest] source={file_name}");
    if let Some(nb) = &frontmatter.notebook {
        s.push_str(&format!(" notebook={nb}"));
    }
    if let Some(op) = &frontmatter.original_path {
        s.push_str(&format!(" path={op}"));
    }
    s
}

const INGEST_EXTRACT_NARRATIVE: &str = concat!(
    "You are a document knowledge extractor. Extract ONLY what is explicitly in the SOURCE.\n\n",
    "Rules:\n",
    "1. Use internal_analysis to reason first.\n",
    "2. Entities must be NAMED items: people, systems, concepts, tools, projects, organizations.\n",
    "3. entity type: BOOK, AUTHOR, PERSON, SYSTEM, CONCEPT, ORGANIZATION, TOOL, PROJECT.\n",
    "4. observations: 1-3 short factual bullets from the SOURCE only (required per entity).\n",
    "5. Relations: AUTHORED_BY, USES, PART_OF, RELATED_TO — only when SOURCE states an explicit link.\n",
    "6. Relation endpoints MUST exactly match entity names you listed.\n",
    "7. Do NOT extract migration metadata (takeout, wave names, migration_id) as entities.\n",
    "8. If nothing extractable, return empty arrays."
);

const INGEST_EXTRACT_AGENT_SPEC: &str = concat!(
    "You are extracting knowledge from an AGENT SPEC document (role, capabilities, integrations).\n\n",
    "Rules:\n",
    "1. The primary agent name is the title or first heading — extract as SYSTEM or AGENT type.\n",
    "2. observations: concrete capabilities, tools, integrations, responsibilities from SOURCE.\n",
    "3. Emit relations ONLY when SOURCE explicitly states dependency (e.g. 'uses Neo4j', 'part of Obolus').\n",
    "4. Do NOT invent relations from bullet lists without explicit linkage words.\n",
    "5. entity types: SYSTEM, TOOL, PROJECT, PERSON, CONCEPT, ORGANIZATION.\n",
    "6. Do NOT extract migration metadata as entities.\n",
    "7. If nothing extractable, return empty arrays."
);

const INGEST_EXTRACT_REFERENCE: &str = concat!(
    "You are extracting knowledge from a reference document (README, roadmap, concept).\n\n",
    "Rules:\n",
    "1. Focus on TOOL, PROJECT, METRIC, SYSTEM entities named in the document.\n",
    "2. observations: 1-3 factual bullets with numbers, commands, or definitions when present.\n",
    "3. Relations only for explicit links (USES, PART_OF).\n",
    "4. Do NOT extract migration metadata as entities.\n",
    "5. If nothing extractable, return empty arrays."
);

const INGEST_EXTRACT_CHAT: &str = concat!(
    "You are extracting knowledge from a chat export.\n\n",
    "Rules:\n",
    "1. Extract decisions, PROJECT names, SYSTEM names, and concrete facts from dialog.\n",
    "2. Skip meta-refusals ('sources do not contain') — do not extract as entities.\n",
    "3. Prefer fewer high-confidence entities over noisy extraction.\n",
    "4. Relations only when explicitly stated in dialog.\n",
    "5. If nothing extractable, return empty arrays."
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_frontmatter_and_body() {
        let raw = "---\nmigration_id: test\nnotebook: My NB\n---\n\n# Title\n\nBody text.\n";
        let (fm, body) = split_frontmatter(raw);
        assert_eq!(fm.migration_id.as_deref(), Some("test"));
        assert_eq!(fm.notebook.as_deref(), Some("My NB"));
        assert!(body.contains("# Title"));
        assert!(!body.contains("migration_id"));
    }

    #[test]
    fn infers_agent_from_wave_filename() {
        let fm = Frontmatter::default();
        assert_eq!(
            infer_agent_name("wave_01_agentsarchitectural_scoutmd.md", &fm),
            Some("Architectural-Scout".to_string())
        );
    }

    #[test]
    fn json_tree_becomes_outline() {
        let body = r#"{"name":"Root","children":[{"name":"Child A"}]}"#;
        let out = json_tree_to_outline(body).expect("outline");
        assert!(out.contains("Root"));
        assert!(out.contains("Child A"));
    }

    #[test]
    fn classifies_agent_from_filename() {
        let fm = Frontmatter {
            original_path: Some("drive/Obolus/Obolus-master/agents/scout.md".into()),
            ..Default::default()
        };
        assert_eq!(
            classify_document("wave_01_agentsarchitectural_scoutmd.md", &fm, ""),
            DocClass::AgentSpec
        );
    }

    #[test]
    fn classifies_notebooklm_chat_model_only_as_chat_export() {
        let fm = Frontmatter {
            original_path: Some(
                "notebooklm/Takeout/NotebookLM/GZMO/Chat History/Chat Session - 360db267.html"
                    .into(),
            ),
            ..Default::default()
        };
        let body = "MODEL:\nGZMO acts as Chief of Staff.\n";
        assert_eq!(
            classify_document(
                "wave_01_notebooklmChat_HistoryChat_Session_-_360db267html.md",
                &fm,
                body,
            ),
            DocClass::ChatExport
        );
    }

    #[test]
    fn classifies_systemkonzept_as_reference() {
        let fm = Frontmatter::default();
        assert_eq!(
            classify_document(
                "wave_01_notebooklmArtifactsTechnisches_Systemkonzept__Das_evolutionre_Agenmd.md",
                &fm,
                "Ein evolutionäres Systemkonzept für Agenten.",
            ),
            DocClass::Reference
        );
    }

    #[test]
    fn classifies_chat_history_with_user_as_chat_export() {
        let fm = Frontmatter {
            original_path: Some(
                "notebooklm/Takeout/NotebookLM/Obolus/Chat History/Chat Session - 9153c22f.html"
                    .into(),
            ),
            ..Default::default()
        };
        let body = "MODEL:\nAnswer.\n\nUSER:\nQuestion?\n";
        assert_eq!(
            classify_document(
                "wave_01_notebooklmChat_HistoryChat_Session_-_9153c22fhtml.md",
                &fm,
                body,
            ),
            DocClass::ChatExport
        );
    }
}
