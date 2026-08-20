//! Cross-collection platform search: honeypot vault + native corpus passages
//! (separately indexed via `crate::corpus`: SQLite FTS5 + Qdrant vectors).

use std::collections::HashMap;

use anyhow::Result;
use tracing::warn;

use crate::config::{
    EmbeddingsConfig, PlatformSearchConfig, QdrantConfig, RedisConfig, RerankConfig,
};
use crate::corpus::store::CorpusStore;
use crate::memory::embeddings::Embedder;
use crate::memory::qdrant_recall::QdrantRecall;
use crate::memory::rerank::Reranker;
use crate::memory::vault::SqliteVault;
use crate::platform_memory::{memory_search_core, MemoryHit, MemoryHitKind, RetrievalChannel};
use crate::types::SemanticFact;

const QUERY_INSTRUCTION: &str =
    "Instruct: Given a search query, retrieve relevant documentation passages that answer it\nQuery: ";

/// Reciprocal rank fusion constant. 60 is the standard RRF default (Cormack
/// et al. 2009) — large enough that a single channel's rank ordering isn't
/// swamped by ties, small enough that top ranks still dominate the score.
const RRF_K: f64 = 60.0;

/// Unified search across the honeypot/vault (promoted facts) and the native
/// corpus passage store (FTS5 + Qdrant vectors, fused by `passage_id`).
pub async fn platform_cross_search(
    vault: &SqliteVault,
    platform_cfg: &PlatformSearchConfig,
    qdrant_cfg: &QdrantConfig,
    embed_cfg: &EmbeddingsConfig,
    redis_cfg: &RedisConfig,
    rerank_cfg: &RerankConfig,
    query: &str,
    limit: usize,
) -> Result<(String, Vec<MemoryHit>)> {
    let (vault_text, vault_results) = memory_search_core(vault, query, limit).await?;
    let mut items: Vec<MemoryHit> = vault_results
        .iter()
        .map(|(fact, score)| vault_fact_to_hit(vault, fact, *score))
        .collect();

    if !platform_cfg.include_knowledge_collection {
        return Ok((vault_text, items));
    }

    let corpus_hits = fetch_and_fuse_corpus_hits(
        vault,
        platform_cfg,
        qdrant_cfg,
        embed_cfg,
        redis_cfg,
        rerank_cfg,
        query,
        limit,
    )
    .await;

    if corpus_hits.is_empty() {
        return Ok((vault_text, items));
    }

    items.extend(corpus_hits);
    let related: Vec<uuid::Uuid> = items.iter().filter_map(|h| h.fact_id).collect();
    let mut text = format_combined_output(query, &items, limit);
    text.push_str(&vault.format_failure_recall(query, &related)?);
    Ok((text, items))
}

/// Query corpus FTS and Qdrant vector recall in parallel-equivalent fashion
/// (FTS is a fast local SQLite call, Qdrant is a network call) and fuse the
/// results keyed by `passage_id` via reciprocal rank fusion. Either channel
/// failing/being disabled degrades to single-channel corpus recall rather
/// than failing the whole search.
async fn fetch_and_fuse_corpus_hits(
    vault: &SqliteVault,
    platform_cfg: &PlatformSearchConfig,
    qdrant_cfg: &QdrantConfig,
    embed_cfg: &EmbeddingsConfig,
    redis_cfg: &RedisConfig,
    rerank_cfg: &RerankConfig,
    query: &str,
    limit: usize,
) -> Vec<MemoryHit> {
    let fts_hits = search_corpus_fts(vault, query, limit);

    let vector_hits = if qdrant_cfg.enabled && embed_cfg.enabled {
        match search_knowledge_collection(
            platform_cfg,
            qdrant_cfg,
            embed_cfg,
            redis_cfg,
            rerank_cfg,
            query,
            limit,
        )
        .await
        {
            Ok(h) => h,
            Err(e) => {
                warn!(error = %e, "Knowledge collection search failed — FTS-only corpus recall");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    if fts_hits.is_empty() && vector_hits.is_empty() {
        return Vec::new();
    }

    reciprocal_rank_fuse(fts_hits, vector_hits, limit)
}

/// BM25-ranked corpus FTS recall, keyed by `passage_id` for fusion.
fn search_corpus_fts(vault: &SqliteVault, query: &str, limit: usize) -> Vec<(String, MemoryHit)> {
    let store = match CorpusStore::new(vault.clone()) {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "Corpus FTS store unavailable — vector-only corpus recall");
            return Vec::new();
        }
    };
    match store.search_fts(query, limit) {
        Ok(hits) => hits
            .into_iter()
            .map(|hit| {
                let passage_id = hit.passage.id.clone();
                let content = format!(
                    "[corpus:{}#chunk{}] {}",
                    hit.passage.source_path, hit.passage.chunk_index, hit.passage.content
                );
                (
                    passage_id,
                    MemoryHit {
                        kind: MemoryHitKind::CorpusPassage,
                        retrieval_channels: vec![RetrievalChannel::Fts],
                        content,
                        // FTS5 bm25() is smaller-is-better; flip sign so a
                        // higher MemoryHit::score is always the better match.
                        score: -(hit.rank as f32),
                        source_file: Some(hit.passage.source_path),
                        fact_id: None,
                        evidence_text: None,
                    },
                )
            })
            .collect(),
        Err(e) => {
            warn!(error = %e, "Corpus FTS search failed — vector-only corpus recall");
            Vec::new()
        }
    }
}

/// Fuse FTS and vector corpus hits keyed by `passage_id`. A `passage_id`
/// present in both channels merges into a single hit whose
/// `retrieval_channels` lists both, always in `RetrievalChannel` declaration
/// order (`[fts, vector]`) for deterministic JSON. Combined score is the
/// reciprocal-rank-fusion sum across the channel(s) that matched.
fn reciprocal_rank_fuse(
    fts_hits: Vec<(String, MemoryHit)>,
    vector_hits: Vec<(String, MemoryHit)>,
    limit: usize,
) -> Vec<MemoryHit> {
    struct Fused {
        hit: MemoryHit,
        channels: Vec<RetrievalChannel>,
        rrf_score: f64,
    }

    let mut fused: HashMap<String, Fused> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    let mut accumulate = |passage_id: String, hit: MemoryHit, channel: RetrievalChannel, rank: usize| {
        let rrf = 1.0 / (RRF_K + rank as f64 + 1.0);
        let entry = fused.entry(passage_id.clone()).or_insert_with(|| {
            order.push(passage_id);
            Fused {
                hit,
                channels: Vec::new(),
                rrf_score: 0.0,
            }
        });
        entry.rrf_score += rrf;
        if !entry.channels.contains(&channel) {
            entry.channels.push(channel);
        }
    };

    for (rank, (passage_id, hit)) in fts_hits.into_iter().enumerate() {
        accumulate(passage_id, hit, RetrievalChannel::Fts, rank);
    }
    for (rank, (passage_id, hit)) in vector_hits.into_iter().enumerate() {
        accumulate(passage_id, hit, RetrievalChannel::Vector, rank);
    }

    let mut results: Vec<MemoryHit> = order
        .into_iter()
        .filter_map(|id| fused.remove(&id))
        .map(|mut f| {
            f.channels.sort_by_key(channel_rank);
            f.hit.retrieval_channels = f.channels;
            f.hit.score = f.rrf_score as f32;
            f.hit
        })
        .collect();

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

/// Stable sort key so fused `retrieval_channels` always serialize as
/// `[fts, vector]` regardless of which channel matched first.
fn channel_rank(channel: &RetrievalChannel) -> u8 {
    match channel {
        RetrievalChannel::Fts => 0,
        RetrievalChannel::Vector => 1,
    }
}

/// Vector recall over the corpus passage Qdrant collection, keyed by
/// `passage_id` for fusion with FTS. Falls back to a `path#chunk` synthetic
/// key for any legacy point without a `passage_id` payload field, so it still
/// gets its own single-channel hit instead of being dropped.
async fn search_knowledge_collection(
    platform_cfg: &PlatformSearchConfig,
    qdrant_cfg: &QdrantConfig,
    embed_cfg: &EmbeddingsConfig,
    redis_cfg: &RedisConfig,
    rerank_cfg: &RerankConfig,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, MemoryHit)>> {
    let embedder = Embedder::from_config(embed_cfg, redis_cfg)?;
    let qdrant = QdrantRecall::from_config(qdrant_cfg)?
        .with_collection(platform_cfg.knowledge_collection.clone());

    let query_text = format!("{QUERY_INSTRUCTION}{query}");
    let vector = embedder.embed(&query_text).await?;
    let prefetch = platform_cfg.knowledge_prefetch.max(limit);
    let raw = qdrant.search_with_payload(&vector, prefetch).await?;

    // (rerank doc text, passage_id, hit)
    let mut candidates: Vec<(String, String, MemoryHit)> = raw
        .into_iter()
        .filter_map(|hit| {
            let path = hit.payload.get("path")?.as_str()?.to_string();
            let text = hit.payload.get("text")?.as_str()?.to_string();
            let chunk = hit
                .payload
                .get("chunk")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let passage_id = hit
                .payload
                .get("passage_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{path}#chunk{chunk}"));
            let content = format!("[corpus:{path}#chunk{chunk}] {text}");
            Some((
                content.clone(),
                passage_id,
                MemoryHit {
                    kind: MemoryHitKind::CorpusPassage,
                    retrieval_channels: vec![RetrievalChannel::Vector],
                    content,
                    score: hit.score as f32,
                    source_file: Some(path),
                    fact_id: None,
                    evidence_text: None,
                },
            ))
        })
        .collect();

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    if rerank_cfg.enabled {
        if let Ok(rr) = Reranker::from_config(rerank_cfg) {
            let docs: Vec<String> = candidates.iter().map(|(d, _, _)| d.clone()).collect();
            if let Ok(order) = rr.rerank(query, &docs, Some(limit)).await {
                let mut reranked = Vec::new();
                for (idx, score) in order.into_iter().take(limit) {
                    if let Some((_, passage_id, mut hit)) = candidates.get(idx).cloned() {
                        hit.score = score as f32;
                        reranked.push((passage_id, hit));
                    }
                }
                if !reranked.is_empty() {
                    return Ok(reranked);
                }
            }
        }
    }

    candidates.sort_by(|a, b| {
        b.2.score
            .partial_cmp(&a.2.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(candidates
        .into_iter()
        .take(limit)
        .map(|(_, passage_id, hit)| (passage_id, hit))
        .collect())
}

fn vault_fact_to_hit(vault: &SqliteVault, fact: &SemanticFact, score: f64) -> MemoryHit {
    MemoryHit {
        kind: MemoryHitKind::PromotedFact,
        retrieval_channels: Vec::new(),
        content: fact.content.clone(),
        score: score as f32,
        source_file: vault.honeypot_source_file(fact.id).ok().flatten(),
        fact_id: Some(fact.id),
        evidence_text: vault.get_evidence_text(fact.id).ok().flatten(),
    }
}

fn format_combined_output(query: &str, items: &[MemoryHit], limit: usize) -> String {
    let mut vault_items: Vec<&MemoryHit> = items
        .iter()
        .filter(|h| h.kind == MemoryHitKind::PromotedFact)
        .collect();
    let mut knowledge_items: Vec<&MemoryHit> = items
        .iter()
        .filter(|h| h.kind == MemoryHitKind::CorpusPassage)
        .collect();

    vault_items.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    knowledge_items.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut out = String::new();
    out.push_str(&format!("Platform recall for '{query}':\n\n"));

    if !vault_items.is_empty() {
        out.push_str("## Honeypot / Vault\n\n");
        for (i, hit) in vault_items.iter().take(limit).enumerate() {
            out.push_str(&format!(
                "- [{}] (Score: {:.2}) {}\n",
                i + 1,
                hit.score,
                hit.content
            ));
        }
        out.push('\n');
    }

    if !knowledge_items.is_empty() {
        out.push_str("## Corpus Passages (FTS + Vector)\n\n");
        for (i, hit) in knowledge_items.iter().take(limit).enumerate() {
            let src = hit.source_file.as_deref().unwrap_or("unknown");
            let channels = if hit.retrieval_channels.is_empty() {
                "unlabeled".to_string()
            } else {
                hit.retrieval_channels
                    .iter()
                    .map(|c| match c {
                        RetrievalChannel::Fts => "fts",
                        RetrievalChannel::Vector => "vector",
                    })
                    .collect::<Vec<_>>()
                    .join("+")
            };
            out.push_str(&format!(
                "- [{}] (Score: {:.2}, src: {src}, channels: {channels}) {}\n",
                i + 1,
                hit.score,
                hit.content
            ));
        }
    }

    if vault_items.is_empty() && knowledge_items.is_empty() {
        out.push_str(&format!("No relevant memories found for query: '{query}'"));
    }

    out
}

#[cfg(test)]
mod corpus_fusion_tests {
    use super::*;
    use crate::platform_memory::{MemoryHitKind, RetrievalChannel};

    fn corpus_hit(passage_id: &str, channel: RetrievalChannel, score: f32) -> (String, MemoryHit) {
        (
            passage_id.to_string(),
            MemoryHit {
                kind: MemoryHitKind::CorpusPassage,
                retrieval_channels: vec![channel],
                content: format!("[corpus:{passage_id}] passage body"),
                score,
                source_file: Some("notes.md".into()),
                fact_id: None,
                evidence_text: None,
            },
        )
    }

    #[test]
    fn matching_passage_ids_fuse_into_one_hit_with_both_channels_in_order() {
        let fts = vec![corpus_hit("sha256:abc:0", RetrievalChannel::Fts, 5.0)];
        let vector = vec![corpus_hit("sha256:abc:0", RetrievalChannel::Vector, 0.8)];

        let fused = reciprocal_rank_fuse(fts, vector, 10);

        assert_eq!(fused.len(), 1, "matching passage_id must fuse to one hit");
        assert_eq!(fused[0].kind, MemoryHitKind::CorpusPassage);
        assert_eq!(
            fused[0].retrieval_channels,
            vec![RetrievalChannel::Fts, RetrievalChannel::Vector],
            "fused channels must be in declaration order [fts, vector]"
        );
    }

    #[test]
    fn non_matching_passage_ids_stay_distinct_single_channel_hits() {
        let fts = vec![corpus_hit("sha256:aaa:0", RetrievalChannel::Fts, 5.0)];
        let vector = vec![corpus_hit("sha256:bbb:0", RetrievalChannel::Vector, 0.8)];

        let fused = reciprocal_rank_fuse(fts, vector, 10);

        assert_eq!(fused.len(), 2);
        let fts_only = fused
            .iter()
            .find(|h| h.content.contains("sha256:aaa:0"))
            .expect("fts-only hit present");
        let vector_only = fused
            .iter()
            .find(|h| h.content.contains("sha256:bbb:0"))
            .expect("vector-only hit present");
        assert_eq!(fts_only.retrieval_channels, vec![RetrievalChannel::Fts]);
        assert_eq!(vector_only.retrieval_channels, vec![RetrievalChannel::Vector]);
    }

    #[test]
    fn fusion_respects_limit() {
        let fts = vec![
            corpus_hit("sha256:a:0", RetrievalChannel::Fts, 5.0),
            corpus_hit("sha256:b:0", RetrievalChannel::Fts, 4.0),
            corpus_hit("sha256:c:0", RetrievalChannel::Fts, 3.0),
        ];
        let fused = reciprocal_rank_fuse(fts, Vec::new(), 2);
        assert_eq!(fused.len(), 2);
    }
}
