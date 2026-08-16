//! Cross-collection platform search: honeypot vault + Pi `knowledge` Qdrant.

use anyhow::Result;
use tracing::warn;

use crate::config::{
    EmbeddingsConfig, PlatformSearchConfig, QdrantConfig, RedisConfig, RerankConfig,
};
use crate::memory::embeddings::Embedder;
use crate::memory::qdrant_recall::QdrantRecall;
use crate::memory::rerank::Reranker;
use crate::memory::vault::SqliteVault;
use crate::platform_memory::{memory_search_core, MemoryHit};
use crate::types::SemanticFact;

const QUERY_INSTRUCTION: &str =
    "Instruct: Given a search query, retrieve relevant documentation passages that answer it\nQuery: ";

/// Unified search across honeypot vault and optional Pi knowledge collection.
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

    if !platform_cfg.include_knowledge_collection || !qdrant_cfg.enabled || !embed_cfg.enabled {
        return Ok((vault_text, items));
    }

    let knowledge_hits = match search_knowledge_collection(
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
            warn!(error = %e, "Knowledge collection search failed — honeypot-only");
            return Ok((vault_text, items));
        }
    };

    if knowledge_hits.is_empty() {
        return Ok((vault_text, items));
    }

    items.extend(knowledge_hits);
    let related: Vec<uuid::Uuid> = items.iter().filter_map(|h| h.fact_id).collect();
    let mut text = format_combined_output(query, &items, limit);
    text.push_str(&vault.format_failure_recall(query, &related)?);
    Ok((text, items))
}

async fn search_knowledge_collection(
    platform_cfg: &PlatformSearchConfig,
    qdrant_cfg: &QdrantConfig,
    embed_cfg: &EmbeddingsConfig,
    redis_cfg: &RedisConfig,
    rerank_cfg: &RerankConfig,
    query: &str,
    limit: usize,
) -> Result<Vec<MemoryHit>> {
    let embedder = Embedder::from_config(embed_cfg, redis_cfg)?;
    let qdrant = QdrantRecall::from_config(qdrant_cfg)?
        .with_collection(platform_cfg.knowledge_collection.clone());

    let query_text = format!("{QUERY_INSTRUCTION}{query}");
    let vector = embedder.embed(&query_text).await?;
    let prefetch = platform_cfg.knowledge_prefetch.max(limit);
    let raw = qdrant.search_with_payload(&vector, prefetch).await?;

    let mut candidates: Vec<(String, MemoryHit)> = raw
        .into_iter()
        .filter_map(|hit| {
            let path = hit.payload.get("path")?.as_str()?.to_string();
            let text = hit.payload.get("text")?.as_str()?.to_string();
            let chunk = hit
                .payload
                .get("chunk")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let content = format!("[knowledge:{path}#chunk{chunk}] {text}");
            Some((
                content.clone(),
                MemoryHit {
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
            let docs: Vec<String> = candidates.iter().map(|(d, _)| d.clone()).collect();
            if let Ok(order) = rr.rerank(query, &docs, Some(limit)).await {
                let mut reranked = Vec::new();
                for (idx, score) in order.into_iter().take(limit) {
                    if let Some((_, mut hit)) = candidates.get(idx).cloned() {
                        hit.score = score as f32;
                        reranked.push(hit);
                    }
                }
                if !reranked.is_empty() {
                    return Ok(reranked);
                }
            }
        }
    }

    candidates.sort_by(|a, b| {
        b.1.score
            .partial_cmp(&a.1.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(candidates.into_iter().take(limit).map(|(_, h)| h).collect())
}

fn vault_fact_to_hit(vault: &SqliteVault, fact: &SemanticFact, score: f64) -> MemoryHit {
    MemoryHit {
        content: fact.content.clone(),
        score: score as f32,
        source_file: vault.honeypot_source_file(fact.id).ok().flatten(),
        fact_id: Some(fact.id),
        evidence_text: vault.get_evidence_text(fact.id).ok().flatten(),
    }
}

fn format_combined_output(query: &str, items: &[MemoryHit], limit: usize) -> String {
    let mut vault_items: Vec<&MemoryHit> = items.iter().filter(|h| h.fact_id.is_some()).collect();
    let mut knowledge_items: Vec<&MemoryHit> =
        items.iter().filter(|h| h.fact_id.is_none()).collect();

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
        out.push_str("## Pi Knowledge (Qdrant)\n\n");
        for (i, hit) in knowledge_items.iter().take(limit).enumerate() {
            let src = hit.source_file.as_deref().unwrap_or("unknown");
            out.push_str(&format!(
                "- [{}] (Score: {:.2}, src: {src}) {}\n",
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
