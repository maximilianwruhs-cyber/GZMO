//! Shared helpers for writing verified knowledge to the Neo4j graph.

use chrono::NaiveDate;

/// Neo4j relationship type for serendipitous / unverified links (spark only).
pub const HYPOTHESIZED_LINK: &str = "HYPOTHESIZED_LINK";

/// Minimum length for a quotable verification span (bulletproof gate).
pub const MIN_EVIDENCE_CHARS: usize = 12;

/// Neo4j batch size for MCP create_entities / create_relations calls.
pub const KG_BATCH_SIZE: usize = 20;

/// Normalize a free-text relation predicate into a Neo4j-safe relationship token.
pub fn sanitize_relation_type(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_underscore = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "RELATED_TO".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Map synonym predicates to one canonical token (prevents AUTHOR/AUTHORED/WROTE triplication).
pub fn canonicalize_relation_type(raw: &str) -> String {
    let base = sanitize_relation_type(raw);
    match base.as_str() {
        "AUTHOR" | "AUTHORED" | "WROTE" | "WRITTEN_BY" | "BY" | "CREATED_BY" => {
            "AUTHORED_BY".to_string()
        }
        "COVERS" | "ABOUT" | "DISCUSSES" | "MENTIONS" | "REFERENCES" => "RELATED_TO".to_string(),
        // L3 only — never promote via dream/ingest pipelines
        "HYPOTHESIZED_LINK" | "HYPOTHESIS" => String::new(),
        _ => base,
    }
}

/// Entity names must be concrete labels, not empty or single-character noise.
pub fn is_valid_entity_name(name: &str) -> bool {
    let t = name.trim();
    t.len() >= 2 && t.chars().any(|c| c.is_alphabetic())
}

/// Relations promoted to permanent memory must have real endpoints and allowed types.
pub fn is_valid_relation_endpoints(from: &str, to: &str, relation_type: &str) -> bool {
    if relation_type.is_empty() {
        return false;
    }
    if !is_valid_entity_name(from) || !is_valid_entity_name(to) {
        return false;
    }
    normalize_entity_key(from) != normalize_entity_key(to)
}

/// Normalize entity names for deduplication (lowercase alphanumeric tokens).
pub fn normalize_entity_key(name: &str) -> String {
    let mut tokens: Vec<String> = name
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();
    tokens.sort();
    tokens.join(" ")
}

/// Provenance observation appended to dream-promoted entities.
pub fn provenance_note(date: NaiveDate, confidence: f64, evidence: &str) -> String {
    if evidence.is_empty() {
        format!("[provenance] consolidated {date}; verified confidence {confidence:.2}")
    } else {
        format!(
            "[provenance] consolidated {date}; verified confidence {confidence:.2}; evidence: \"{evidence}\""
        )
    }
}

/// Names that may be written as KG entities. Empty / all-invalid input is a no-op.
pub fn promotable_entities<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    names
        .into_iter()
        .filter(|n| is_valid_entity_name(n))
        .map(|n| n.trim().to_string())
        .collect()
}

/// `(from, to, type)` triples that may be written as KG relations.
/// Empty input and leftover-invalid sets are a no-op (no invented edges).
pub fn promotable_relations<'a>(
    triples: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>,
) -> Vec<(String, String, String)> {
    triples
        .into_iter()
        .filter_map(|(from, to, rel)| {
            let relation_type = canonicalize_relation_type(rel);
            if !is_valid_relation_endpoints(from, to, &relation_type) {
                None
            } else {
                Some((from.trim().to_string(), to.trim().to_string(), relation_type))
            }
        })
        .collect()
}

/// Try to extract an entity name from vault content shaped like `[Type:Name] ...`.
pub fn entity_label_from_fact(content: &str) -> String {
    if let Some(rest) = content.strip_prefix('[') {
        if let Some(idx) = rest.find(']') {
            let inside = &rest[..idx];
            if let Some((_, name)) = inside.split_once(':') {
                let name = name.trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    content
        .chars()
        .take(48)
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_author_synonyms() {
        assert_eq!(canonicalize_relation_type("WROTE"), "AUTHORED_BY");
        assert_eq!(canonicalize_relation_type("author"), "AUTHORED_BY");
        assert_eq!(canonicalize_relation_type("HYPOTHESIZED_LINK"), "");
    }

    #[test]
    fn empty_input_is_noop() {
        assert!(
            promotable_entities(std::iter::empty::<&str>()).is_empty(),
            "empty entity input must not invent names"
        );
        assert!(
            promotable_relations(std::iter::empty::<(&str, &str, &str)>()).is_empty(),
            "empty relation input must not invent edges"
        );
    }

    #[test]
    fn no_valid_candidates_is_noop() {
        assert!(
            promotable_entities(["", " ", "x", "42"]).is_empty(),
            "noise-only names are not candidates"
        );
        assert!(
            promotable_relations([
                ("Alice", "Bob", "HYPOTHESIZED_LINK"),
                ("", "Bob", "KNOWS"),
                ("Alice", "Alice", "KNOWS"),
            ])
            .is_empty(),
            "no valid leftover must be a no-op — do not write"
        );
    }

    #[test]
    fn valid_candidates_survive() {
        assert_eq!(promotable_entities(["  Alice  ", "x"]), vec!["Alice"]);
        let rels = promotable_relations([("Alice", "Book", "WROTE")]);
        assert_eq!(rels, vec![("Alice".into(), "Book".into(), "AUTHORED_BY".into())]);
    }
}
