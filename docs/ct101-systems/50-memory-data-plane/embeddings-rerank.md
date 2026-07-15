# Embeddings & Rerank — VM200 Retrieval Stack

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Sources:** `gzmo-core/src/memory/embeddings.rs`, `gzmo-core/src/memory/rerank.rs`

---

## Capability

Embeddings and reranking delegate to **VM200** (`.110`): OpenAI-compatible `/embeddings` and `/rerank` endpoints. Redis caches embedding vectors (`gzmo:embed:{model}:{sha256}`). Boot probe attaches embedder/reranker to `SqliteVault`; failures degrade gracefully to keyword-only recall.

---

## How it works

### Embedder + Redis cache

```154:198:gzmo-core/src/memory/embeddings.rs
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(cache) = &self.cache {
            if let Some(vec) = cache.get(text).await {
                return Ok(vec);
            }
        }
        let embedding = self.embed_remote(text).await?;
        // cache.set on success
    }
```

Cache keys include model name; TTL from `[embeddings] cache_ttl_secs`.

### Vault open helper

```219:272:gzmo-core/src/memory/embeddings.rs
pub async fn open_vault_with_embeddings(/* db, embed, redis, rerank, qdrant */) -> Result<SqliteVault> {
    let vault = SqliteVault::open(db_path)?;
    // probe embed("gzmo vault probe") → with_embedder
    let vault = attach_reranker(vault, rerank_cfg).await;
    // QdrantRecall::from_config if enabled
}
```

### Reranker

```48:94:gzmo-core/src/memory/rerank.rs
    pub async fn rerank(&self, query: &str, documents: &[String], top_n: Option<usize>) -> Result<Vec<(usize, f64)>> {
        let endpoint = format!("{}/rerank", self.url);
        // POST model, query, documents → sorted by relevance_score
    }

    pub fn prefetch_limit(&self, limit: usize) -> usize {
        (limit * self.prefetch_multiplier).clamp(limit, 50)
    }
```

`attach_reranker` probes with `"vault probe"` before enabling; vault uses rerank in `search_with_decay_reranked` and `recall_rrf` final stage.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config `[embeddings]` | `enabled`, `url` (VM200), `model`, `cache_enabled`, `cache_ttl_secs` |
| Config `[rerank]` | `enabled`, `url`, `model`, `prefetch_multiplier` |
| Config `[redis]` | Required for embed cache when `cache_enabled` |
| Vector dim | 1024 (Qwen3-Embedding on VM200) |
| External | [110-external-nodes/vm200-retrieval](../110-external-nodes/vm200-retrieval.md) |

---

## THINKING nodes

> **THINKING — embeddings.rs:EmbeddingCache**
> - *Reviewed:* Lazy ConnectionManager, 3s connect timeout, invalidate conn on GET/SET errors.
> - *Insight:* Cache reduces VM200 load during RRF prefetch (50 candidates × query).
> - *Risk / limitation:* Redis outage → every query hits VM200; no in-memory embed fallback.
> - *Enhancement:* Short-lived process-local LRU in addition to Redis [CT101-safe].

> **THINKING — embeddings.rs:open_vault_with_embeddings**
> - *Reviewed:* Probe embed at boot; warn and continue without vectors on failure.
> - *Insight:* Daemon stays up on VM200 blip — CT101 resilience pattern.
> - *Risk / limitation:* Silent vector-less mode until restart if VM200 recovers mid-run.
> - *Enhancement:* Periodic embed health re-probe in orchestrator [CT101-safe].

> **THINKING — rerank.rs:prefetch_limit**
> - *Reviewed:* Caps rerank batch at 50 docs regardless of multiplier.
> - *Insight:* Protects VM200 from 200-doc batches during wide RRF prefetch.
> - *Risk / limitation:* Hard cap may drop good tail candidates in dense corpora (37k honeypot).
> - *Enhancement:* Configurable rerank cap per engine profile [GZMO-next].

---

## Advancement

- **CT101:** Monitor VM200 latency in health tick; alert when rerank disabled >1h.
- **GZMO-next:** Co-locate embed/rerank on CT101 sidecar only if beat-gates prove VM200 SPOF.

---

## Enhancement backlog

1. **[CT101-safe]** Orchestrator job: re-probe embed/rerank and hot-attach to vault.
2. **[CT101-safe]** Log cache hit rate metric per daemon day.
3. **[CT101-safe]** Timeout tuning for 1024-dim batches under load.
4. **[GZMO-next]** Unified retrieval gateway abstraction (embed + rerank + sparse).
5. **[GZMO-next]** Local GGUF fallback when VM200 unreachable (energy/cost tradeoff).
