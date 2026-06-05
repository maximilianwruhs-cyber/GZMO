//! Honeypot hybrid recall helpers: RRF fusion and diversification.

use std::collections::HashMap;

use uuid::Uuid;

use crate::types::SemanticFact;

const RRF_K: f64 = 60.0;
pub const PREFETCH_K: usize = 50;
pub const MAX_PER_SOURCE_FILE: usize = 5;
pub const RERANK_PREFETCH: usize = 40;

/// Candidate with honeypot metadata for diversification.
#[derive(Clone)]
pub struct RecallCandidate {
    pub fact: SemanticFact,
    pub source_file: Option<String>,
}

/// Interleave two ranked lists (deduped) into one — avoids double-counting correlated vector streams in RRF.
pub fn merge_interleaved_rank(a: &[Uuid], b: &[Uuid], cap: usize) -> Vec<Uuid> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(cap.min(a.len() + b.len()));
    let mut ia = 0usize;
    let mut ib = 0usize;
    while out.len() < cap && (ia < a.len() || ib < b.len()) {
        if ia < a.len() {
            let id = a[ia];
            ia += 1;
            if seen.insert(id) {
                out.push(id);
            }
        }
        if out.len() >= cap {
            break;
        }
        if ib < b.len() {
            let id = b[ib];
            ib += 1;
            if seen.insert(id) {
                out.push(id);
            }
        }
    }
    out
}

/// Reciprocal rank fusion over ranked id lists (1-indexed ranks).
pub fn rrf_fuse(rank_lists: &[Vec<Uuid>]) -> HashMap<Uuid, f64> {
    let mut scores: HashMap<Uuid, f64> = HashMap::new();
    for list in rank_lists {
        for (idx, id) in list.iter().enumerate() {
            let rank = (idx + 1) as f64;
            let contrib = 1.0 / (RRF_K + rank);
            *scores.entry(*id).or_insert(0.0) += contrib;
        }
    }
    scores
}

/// Max `limit` items with at most `max_per_file` per `source_file` key.
pub fn diversify_by_source_file(
    mut ranked: Vec<(RecallCandidate, f64)>,
    limit: usize,
    max_per_file: usize,
) -> Vec<(RecallCandidate, f64)> {
    let mut per_file: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(limit);
    for (cand, score) in ranked.drain(..) {
        let key = cand
            .source_file
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("__none__")
            .to_string();
        let n = per_file.entry(key).or_insert(0);
        if *n >= max_per_file {
            continue;
        }
        *n += 1;
        out.push((cand, score));
        if out.len() >= limit {
            break;
        }
    }
    out
}

/// Tokens for graph / entity-aligned honeypot matching.
pub fn extract_entity_tokens(query: &str) -> Vec<String> {
    let mut out = Vec::new();
    let q = query.trim();
    if q.is_empty() {
        return out;
    }
    let mut rest = q;
    while let Some(start) = rest.find('[') {
        let Some(end_rel) = rest[start..].find(']') else {
            break;
        };
        let end = start + end_rel;
        let inner = &rest[start + 1..end];
        if let Some((_tag, name)) = inner.split_once(':') {
            let s = name.trim();
            if !s.is_empty() {
                out.push(s.to_string());
            }
        }
        rest = &rest[end + 1..];
    }
    const STOP: &[&str] = &[
        "what", "which", "does", "the", "and", "for", "how", "when", "where", "that", "this",
        "with", "from", "into", "about",
    ];
    for word in q.split_whitespace() {
        let w: String = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        if w.len() < 4 {
            continue;
        }
        let lower = w.to_lowercase();
        if STOP.contains(&lower.as_str()) {
            continue;
        }
        out.push(w);
    }
    let mut seen = std::collections::HashSet::new();
    out.retain(|t| seen.insert(t.to_lowercase()));
    out.truncate(8);
    out
}

pub fn fts_match_query(query: &str) -> String {
    fts_match_query_mode(query, false)
}

/// Broader FTS query (OR across tokens) for recall — reduces zero-hit lexical streams.
pub fn fts_match_query_broad(query: &str) -> String {
    fts_match_query_mode(query, true)
}

fn fts_match_query_mode(query: &str, broad: bool) -> String {
    let words: Vec<String> = query
        .split_whitespace()
        .filter(|w| w.len() >= 2)
        .map(|w| format!("\"{}\"", w.replace('"', "")))
        .collect();
    if words.is_empty() {
        return String::new();
    }
    if broad || words.len() <= 2 {
        words.join(" OR ")
    } else {
        // Top terms OR + remaining as optional OR (still one FTS clause)
        words.join(" OR ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn dummy_fact(id: Uuid, content: &str) -> RecallCandidate {
        let now = Utc::now();
        RecallCandidate {
            fact: SemanticFact {
                id,
                content: content.to_string(),
                embedding: Vec::new(),
                half_life_days: 60.0,
                confidence: 1.0,
                confirmation_count: 0,
                decay_class: "CuratedVault".to_string(),
                created_at: now,
                last_accessed_at: now,
            },
            source_file: Some("wave_01_same.md".to_string()),
        }
    }

    #[test]
    fn test_merge_interleaved_rank_dedupes() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let merged = merge_interleaved_rank(&[a, b], &[b, a], 4);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], a);
        assert_eq!(merged[1], b);
    }

    #[test]
    fn test_rrf_empty_lists() {
        let scores = rrf_fuse(&[]);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_rrf_scoring_order() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let scores = rrf_fuse(&[vec![a, b, c], vec![a, c, b]]);
        assert!(scores[&a] > scores[&b]);
        assert!(scores[&a] > scores[&c]);
    }

    #[test]
    fn test_extract_entity_tokens_bracket_and_words() {
        let t = extract_entity_tokens("What does [AGENT:Backup-Custodian] monitor for GZMO?");
        assert!(t.iter().any(|x| x.contains("Backup")));
        assert!(t.iter().any(|x| x.to_lowercase().contains("gzmo")));
    }

    #[test]
    fn test_diversification_limit() {
        let id = Uuid::new_v4();
        let ranked: Vec<(RecallCandidate, f64)> = (0..5)
            .map(|i| (dummy_fact(id, &format!("fact {i}")), 1.0 / (i as f64 + 1.0)))
            .collect();
        let out = diversify_by_source_file(ranked, 10, 3);
        assert_eq!(out.len(), 3);
    }
}
