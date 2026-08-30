//! Honeypot hybrid recall helpers: RRF fusion and diversification.

use std::collections::HashMap;

use uuid::Uuid;

use crate::types::SemanticFact;

const RRF_K: f64 = 60.0;
pub const PREFETCH_K: usize = 50;
/// Overfetch Qdrant so `filter_assertable_honeypot_ids` can still fill `PREFETCH_K`
/// after dropping superseded points (GPM / Temporal Validity).
pub const QDRANT_PREFETCH_K: usize = PREFETCH_K * 2;
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

/// Reciprocal rank fusion over ranked id lists (1-indexed ranks) with stream weights.
pub fn rrf_fuse_weighted(weighted_lists: &[(Vec<Uuid>, f64)]) -> HashMap<Uuid, f64> {
    let mut scores: HashMap<Uuid, f64> = HashMap::new();
    for (list, weight) in weighted_lists {
        for (idx, id) in list.iter().enumerate() {
            let rank = (idx + 1) as f64;
            let contrib = weight / (RRF_K + rank);
            *scores.entry(*id).or_insert(0.0) += contrib;
        }
    }
    scores
}

/// Reciprocal rank fusion over ranked id lists (1-indexed ranks).
pub fn rrf_fuse(rank_lists: &[Vec<Uuid>]) -> HashMap<Uuid, f64> {
    rrf_fuse_weighted(
        &rank_lists
            .iter()
            .map(|l| (l.clone(), 1.0))
            .collect::<Vec<_>>(),
    )
}

/// Normalize positive RRF scores into the [0.0, 1.0] range by dividing by the maximum score.
pub fn normalize_rrf_scores(scores: &HashMap<Uuid, f64>) -> HashMap<Uuid, f64> {
    if scores.is_empty() {
        return HashMap::new();
    }
    let max_score = scores.values().copied().fold(0.0f64, f64::max);
    if max_score <= 0.0 {
        return HashMap::new();
    }
    scores
        .iter()
        .map(|(&id, &score)| (id, score / max_score))
        .collect()
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

/// Pool-relative utility weight. RRF adjacent ranks differ by ~3e-4; 0.05 lets
/// max-Q in the pool outrank min-Q without inventing hits. Cross-encoder gaps
/// (~0.2–0.9) still dominate, so relevance is not discarded.
pub const UTILITY_POOL_LAMBDA: f64 = 0.05;

/// MemRL phase B: boost relevance scores by in-pool `utility_score` (Q), then
/// re-sort. Does not add hits. Equal utility leaves relative relevance order.
pub fn apply_utility_boost(scored: &mut Vec<(SemanticFact, f64)>, utility: &HashMap<Uuid, f64>) {
    if scored.len() <= 1 {
        return;
    }
    let us: Vec<f64> = scored
        .iter()
        .map(|(f, _)| utility.get(&f.id).copied().unwrap_or(0.0).max(0.0))
        .collect();
    let u_min = us.iter().copied().fold(f64::INFINITY, f64::min);
    let u_max = us.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = u_max - u_min;
    let orig: Vec<f64> = scored.iter().map(|(_, s)| *s).collect();
    if span > 1e-12 {
        for (i, (_, score)) in scored.iter_mut().enumerate() {
            *score += UTILITY_POOL_LAMBDA * ((us[i] - u_min) / span);
        }
    }
    let mut order: Vec<usize> = (0..scored.len()).collect();
    order.sort_by(|&i, &j| {
        scored[j]
            .1
            .partial_cmp(&scored[i].1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                orig[j]
                    .partial_cmp(&orig[i])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| i.cmp(&j))
    });
    *scored = order.into_iter().map(|i| scored[i].clone()).collect();
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

fn fts_match_query_mode(query: &str, _broad: bool) -> String {
    // Porter stems `Use`→`us` and hyphenates `token` out of junk queries.
    // Drop those query tokens so MATCH cannot form the 407-row `us` pool.
    const STOP: &[&str] = &[
        "what", "which", "does", "the", "and", "for", "how", "when", "where", "that", "this",
        "with", "from", "into", "about", "use", "used", "using", "user", "useful", "usage",
        "token", "tokens",
    ];
    let mut words: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for w in query.split(|c: char| !c.is_alphanumeric()) {
        if w.len() < 2 {
            continue;
        }
        let lower = w.to_lowercase();
        if STOP.contains(&lower.as_str()) {
            continue;
        }
        if !seen.insert(lower) {
            continue;
        }
        words.push(format!("\"{}\"", w.replace('"', "")));
    }
    words.join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

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
    fn fts_drops_use_so_felt_use_is_not_or_use() {
        let q = fts_match_query("Felt Use");
        assert!(q.contains("Felt"), "{q}");
        assert!(!q.to_lowercase().contains("use"), "{q}");
    }

    #[test]
    fn fts_drops_token_in_hyphenated_abstention() {
        let q = fts_match_query("zzzz-nonexistent-token-9f3a2");
        assert!(!q.to_lowercase().contains("token"), "{q}");
        assert!(
            q.is_empty() || q.contains("zzzz") || q.contains("nonexistent"),
            "{q}"
        );
    }

    #[test]
    fn fts_use_alone_is_empty() {
        assert_eq!(fts_match_query("use"), "");
        assert_eq!(fts_match_query("token"), "");
    }

    #[test]
    fn fts_keeps_prometheus_promql() {
        let q = fts_match_query("Prometheus PromQL");
        assert!(q.contains("Prometheus"), "{q}");
        assert!(q.contains("PromQL"), "{q}");
    }

    #[test]
    fn qdrant_prefetch_overfetches_assertable_filter() {
        assert_eq!(QDRANT_PREFETCH_K, PREFETCH_K * 2);
        assert!(QDRANT_PREFETCH_K > PREFETCH_K);
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

    #[test]
    fn utility_boost_promotes_high_q_inside_pool() {
        let low = Uuid::new_v4();
        let high = Uuid::new_v4();
        let mut scored = vec![
            (dummy_fact(low, "low q").fact, 0.0164),
            (dummy_fact(high, "high q").fact, 0.0161),
        ];
        let mut utility = HashMap::new();
        utility.insert(low, 0.0);
        utility.insert(high, 12.0);
        apply_utility_boost(&mut scored, &utility);
        assert_eq!(scored[0].0.id, high, "phase B must prefer higher utility");
        assert_eq!(scored.len(), 2, "must not invent or drop hits");
    }

    #[test]
    fn utility_boost_equal_q_keeps_relevance_order() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut scored = vec![
            (dummy_fact(a, "first").fact, 0.9),
            (dummy_fact(b, "second").fact, 0.2),
        ];
        let mut utility = HashMap::new();
        utility.insert(a, 3.0);
        utility.insert(b, 3.0);
        apply_utility_boost(&mut scored, &utility);
        assert_eq!(scored[0].0.id, a);
        assert_eq!(scored[1].0.id, b);
    }

    #[test]
    fn utility_boost_empty_and_singleton_are_nops() {
        let mut empty: Vec<(SemanticFact, f64)> = Vec::new();
        apply_utility_boost(&mut empty, &HashMap::new());
        assert!(empty.is_empty());

        let id = Uuid::new_v4();
        let mut one = vec![(dummy_fact(id, "only").fact, 0.5)];
        apply_utility_boost(&mut one, &HashMap::new());
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].0.id, id);
        assert!((one[0].1 - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_rrf_fuse_weighted_unequal_weights() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let scores = rrf_fuse_weighted(&[(vec![id1], 2.0), (vec![id2], 0.5)]);
        assert_eq!(scores[&id1], 2.0 / 61.0);
        assert_eq!(scores[&id2], 0.5 / 61.0);
        assert!(scores[&id1] > scores[&id2]);
    }

    #[test]
    fn test_normalize_rrf_scores_empty() {
        let scores: HashMap<Uuid, f64> = HashMap::new();
        assert!(normalize_rrf_scores(&scores).is_empty());
    }

    #[test]
    fn test_normalize_rrf_scores_single() {
        let id = Uuid::new_v4();
        let scores = HashMap::from([(id, 0.5)]);
        let normalized = normalize_rrf_scores(&scores);
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[&id], 1.0);
    }

    #[test]
    fn test_normalize_rrf_scores_multi() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let scores = HashMap::from([(id1, 2.0), (id2, 1.0), (id3, 0.5)]);
        let normalized = normalize_rrf_scores(&scores);
        assert_eq!(normalized[&id1], 1.0);
        assert_eq!(normalized[&id2], 0.5);
        assert_eq!(normalized[&id3], 0.25);
    }

    #[test]
    fn test_rrf_fuse_backward_compatibility() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let list1 = vec![a, b];
        let list2 = vec![b, a];
        let scores1 = rrf_fuse(&[list1.clone(), list2.clone()]);
        let scores2 = rrf_fuse_weighted(&[(list1, 1.0), (list2, 1.0)]);
        assert_eq!(scores1.len(), scores2.len());
        for (id, score) in scores1 {
            assert_eq!(score, scores2[&id]);
        }
    }
}
