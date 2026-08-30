# Why abstention still returns “token” facts after FTS stop-stems

**Date:** 2026-08-30  
**Status:** Research only — do not ship from this note  
**Living evidence:** after [#230](https://github.com/maximilianwruhs-cyber/GZMO/pull/230) + daemon restart, `felt-use-findable-prove.sh` PASS; `zzzz-nonexistent-token-9f3a2` still returns 5 hits (mint token / gemini token consumption), rerank scores ≈ **−6.8**.

## Verdict

Embeddings always return a nearest neighbor. FTS for that query is empty (`"zzzz" OR "nonexistent" OR "9f3a2"` → **0** rows). `recall_rrf` still embeds, Qdrant + `search_with_decay` fill a vector list, `STREAM_TOP_RESCUE` keeps the top 5, and `apply_rerank` **writes the negative scores through** then `truncate(limit)`. Honest-empty is a refuse-to-ask-the-embedder problem, not a better embedding.

## What actually runs

[`vault.rs` `recall_rrf`](../gzmo-core/src/memory/vault.rs): FTS / evidence-FTS / graph / keyword lists first, then **unconditionally** `embedder.embed(q)` when the embedder is up (lines 1022–1048). Empty FTS does not skip that block.

`search_with_decay` scores **every** latest honeypot with an embedding blob — there is no “no lexical overlap → skip” gate.

`apply_rerank` ([`vault.rs:1670`](../gzmo-core/src/memory/vault.rs)) replaces RRF scores with reranker logits and truncates. It does not drop `score <= 0`. The abstention sitting’s −6.8 is the reranker saying “not this,” shipped as a hit anyway.

`extract_entity_tokens` ([`recall_rrf.rs`](../gzmo-core/src/memory/recall_rrf.rs)) does **not** share the FTS stop list (`use` / `token`). Graph fallback can still emit `token` as a hint if the query tokenizes that way. Neo4j hints, if any, are a third lexical-ish path.

`take_assertable_prefetch` only drops superseded ids, not weak vectors.

## Why the geometry will not abstain

Qwen3-Embedding on “zzzz-nonexistent-token-9f3a2” is still a point in the same space as “registration token” / “token consumption.” k-NN has no “none of the above.” LoCoMo abstention assumes the *system* can return empty; a pure vector top-K cannot.

MemRL’s two-phase retrieval assumes a relevance pool. No lexical pool + no vocab overlap ⇒ there is no phase A. Phase B (`apply_utility_boost`) cannot save this; Q only reorders.

## How to tackle it (lazy first)

1. **Skip vector when every lexical list is empty.** One `if` around the embed block in `recall_rrf`. Pair with sharing FTS `STOP` into `extract_entity_tokens` so `token` / `use` do not keep graph/keyword alive. This Keep: FTS leftover already 0; if graph/kw are also empty, search returns []. Doctrine queries stay lexical (`Felt`, `MemRL`) so vectors still run when they should.

2. **Drop rerank score ≤ 0 after `apply_rerank`.** Safety net when vector still runs (typos, short queries). Tonight’s abstention would vanish; `Felt Use` after #230 was +0.11…+0.19 and MemRL +3.x, so they survive. Calibrate on a few real weak-positive queries before treating 0.0 as gospel — bge logits are not a probability.

3. **Vocab membership (stronger, still local).** If no query token (post-stop, post-hyphen split) exists in `honeypot_fts` / `fts5vocab`, do not embed. Airgap-honest: never seen this word → do not invent. Cost: one vocab lookup. Misses: typo-rescue via vectors (probably acceptable here).

Do **not**: train a better embedder, gym-search the nonce, or add a cloud judge. Do **not** floor cosine without a Keep-local histogram — that is a tuning project.

## Tomorrow’s one ticket

If this becomes a ticket: **“Skip vector (and drop non-positive rerank) when lexical streams are empty.”** Prove: `zzzz-nonexistent-token-9f3a2` hits=0; `Felt Use` / `Prometheus PromQL` still rank. Same script, one extra abstention assert. No gym.
