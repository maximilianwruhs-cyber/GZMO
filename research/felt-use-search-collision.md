# Why Felt Use search returned Glances / Prometheus / PEP8 / L.I.N.C.

**Date:** 2026-08-30  
**Status:** Research only (no product code)  
**Ticket:** [#223](https://github.com/maximilianwruhs-cyber/GZMO/issues/223) (map [#221](https://github.com/maximilianwruhs-cyber/GZMO/issues/221))  
**Question:** On 2026-08-30, living MCP search for Felt Use / MemRL / `utility_score` returned Glances, Prometheus, PEP8, L.I.N.C. — and the vault has **zero** matching doctrine rows. Trace `search_recall` / FTS / RRF / `apply_utility_boost`. What tokenized, what pool formed, why utility could not save it.

**Living query (read-only):** `sqlite3 -readonly /home/gzmo/.gzmo-living/data/vault.db`  
**Code tip:** `origin/main` @ `8fc07a2` (this note’s branch base)

---

## Verdict (short)

The query never had a doctrine row to retrieve. `fts_match_query` turns `Felt Use / MemRL / utility_score` into an **OR of four quoted tokens**. Porter FTS stems `"Use"` to `us`, which matches `use` / `used` / `using` / `user` / `useful` / `usage`. That single stem is the entire lexical pool: **407** latest `obolus` honeypot rows. `"Felt"`, `"MemRL"`, and `"utility_score"` contribute **zero** FTS hits. Glances / PEP8 / Prometheus sit in that pool because their sentences say “uses” / “used”. `apply_utility_boost` only re-ranks inside an already-formed pool and **does not add hits**, so MemRL Q cannot invent a missing Felt Use fact. L.I.N.C. is **outside** the lexical pool; if it appeared in the MCP top-N, that was a later stream (vector) or a mixed sitting, and Q would have **promoted** it, not filtered it out.

---

## 1. Call chain (MCP → pool → Q-select)

Default living MCP search (`limit` unset → **5**, `write_scratch` unset → **true**):

| Step | Function | File:line |
|------|----------|-----------|
| MCP tool | `gzmo_memory_search` | [`gzmo-core/src/mcp/serve.rs:170`](../gzmo-core/src/mcp/serve.rs) |
| Platform | `PlatformMemory::memory_search` → `memory_search_scoped` | [`gzmo-core/src/platform_memory.rs:205`](../gzmo-core/src/platform_memory.rs) |
| Cross-search | `platform_cross_search` | [`gzmo-core/src/platform_search.rs:20`](../gzmo-core/src/platform_search.rs) |
| Vault text | `memory_search_core` → `vault.search_recall` | [`gzmo-core/src/platform_memory.rs:338`](../gzmo-core/src/platform_memory.rs) |
| Alias | `search_recall` → `recall_rrf(..., "obolus")` | [`gzmo-core/src/memory/vault.rs:972`](../gzmo-core/src/memory/vault.rs) |
| Hybrid | `recall_rrf` | [`gzmo-core/src/memory/vault.rs:981`](../gzmo-core/src/memory/vault.rs) |
| Felt Use touch | `felt_use::touch_hits` (Cited if scratch written, else Glance) | [`gzmo-core/src/platform_memory.rs:256`](../gzmo-core/src/platform_memory.rs) |

`include_knowledge_collection = false` on this Keep ([`~/.gzmo-living/gzmo.toml`](../../.gzmo-living/gzmo.toml) `[platform_search]`), so `platform_cross_search` returns honeypot-only after `memory_search_core` ([`platform_search.rs:36`](../gzmo-core/src/platform_search.rs)). Pi knowledge is not in this collision.

`cognition_from_honeypot` is true when latest honeypot count > 0 ([`vault.rs:785`](../gzmo-core/src/memory/vault.rs)). Living census: **3005** latest / **3005** `honeypot_fts` / all `container_tag='obolus'` / schema `user_version=10`. So `search_recall_legacy` is not taken.

---

## 2. What tokenized

### 2.1 Product MATCH string

`honeypot_fts_stream` tries `fts_match_query` first, then `fts_match_query_broad` only if the narrow list is empty ([`vault.rs:1222`](../gzmo-core/src/memory/vault.rs)).

Both builders call `fts_match_query_mode` ([`recall_rrf.rs:203`](../gzmo-core/src/memory/recall_rrf.rs)):

1. `split_whitespace`
2. drop tokens with `len < 2` (the two `/` separators die here)
3. wrap each survivor in FTS5 quotes
4. join with ` OR ` — **both** the “narrow” and “broad” branches do this ([`recall_rrf.rs:221`](../gzmo-core/src/memory/recall_rrf.rs)). There is no AND / required-term path.

For `Felt Use / MemRL / utility_score` the MATCH string is therefore:

```text
"Felt" OR "Use" OR "MemRL" OR "utility_score"
```

That is the living query used below.

### 2.2 Porter stem (index + query)

Living `honeypot_fts` / `evidence_fts` are created with `tokenize='porter'` ([`vault.rs:197`](../gzmo-core/src/memory/vault.rs), [`vault.rs:231`](../gzmo-core/src/memory/vault.rs)). Confirmed on the Keep: `sqlite_master.sql` for `honeypot_fts` contains `tokenize='porter'`.

SQLite FTS5 porter is a wrapper around unicode61 that applies the Porter stemmer to every token ([FTS5 §4.3.3](https://www.sqlite.org/fts5.html#tokenizers)). In-memory check on this host (`CREATE VIRTUAL TABLE t USING fts5(x, tokenize='porter')`):

| Quoted query token | Stem that MATCH uses | Living latest hits |
|--------------------|----------------------|--------------------|
| `"Use"` | `us` | **407** |
| `"Felt"` | `felt` | **0** |
| `"MemRL"` | `memrl` | **0** |
| `"utility_score"` | phrase after unicode61 split on `_` → `util` + `score` adjacent | **0** |

Live `fts5vocab(main, honeypot_fts, row)`: term `us` is in **407** docs (**818** occurrences). `feel` exists on 2 docs but is a different stem than `felt`. `util` exists on 16 docs; the **quoted phrase** `"utility_score"` still matches 0 (underscore splits; those 16 are not `utility`+`score` as a phrase).

`"Use"` MATCH hits the in-memory sentences “Glances used to monitor”, “Prometheus is used for”, “PEP8 use 4 spaces”, “L.I.N.C. use of”. That is the collision mechanism. The living L.I.N.C. rows do **not** contain the substring `use` (see §4).

### 2.3 Graph / keyword tokens (second lexical stream)

`extract_entity_tokens` keeps words with `len >= 4` and drops a small stop list ([`recall_rrf.rs:158`](../gzmo-core/src/memory/recall_rrf.rs)). For this query that is `Felt`, `MemRL`, `utility_score` — **`Use` is dropped** (length 3).

`honeypot_graph_stream` ([`vault.rs:1380`](../gzmo-core/src/memory/vault.rs)) maps those hints with `LIKE %needle%` ([`vault.rs:1443`](../gzmo-core/src/memory/vault.rs)). Living counts on latest honeypot:

| LIKE needle | n |
|-------------|--:|
| `%Felt%` | 0 |
| `%MemRL%` | 0 |
| `%utility_score%` | 0 |

If Neo4j hints are empty (script missing or bolt miss), graph is empty and `recall_rrf` falls through to `honeypot_keyword_stream` → `keyword_search` ([`vault.rs:1007`](../gzmo-core/src/memory/vault.rs), [`vault.rs:2261`](../gzmo-core/src/memory/vault.rs)). Keyword scoring is **substring** `content_lower.contains(word)` over `split_whitespace`, so the 3-letter word `use` matches the same “uses/used” mass as FTS. It cannot add a doctrine row that is not there.

---

## 3. What pool formed

### 3.1 Doctrine census (ticket claim, re-measured)

```sql
-- all zero on this Keep, 2026-08-30
SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND (
  content LIKE '%Felt Use%' OR content LIKE '%felt use%' OR content LIKE '%felt_use%'
  OR content LIKE '%MemRL%' OR content LIKE '%memrl%'
  OR content LIKE '%utility_score%');
-- same pattern on semantic_vault: 0
```

| Table | Felt Use / felt_use | MemRL | utility_score |
|-------|--------------------:|------:|--------------:|
| `honeypot` latest | 0 | 0 | 0 |
| `semantic_vault` | 0 | 0 | 0 |

`mcp` `gzmo_memory_status` this sitting: `vault_path=/home/gzmo/.gzmo-living/data/vault.db`, `honeypot_latest=3005`, `vault_facts=4711`. Same path as `[memory] vault_db` in living `gzmo.toml`.

### 3.2 FTS OR pool = `"Use"` pool

```sql
SELECT COUNT(*) FROM honeypot h
JOIN honeypot_fts fts ON h.rowid = fts.rowid
WHERE honeypot_fts MATCH '"Felt" OR "Use" OR "MemRL" OR "utility_score"'
  AND h.is_latest=1 AND h.container_tag='obolus';
-- 407
```

| MATCH | latest n |
|-------|---------:|
| full OR (product string) | 407 |
| `"Use"` only | 407 |
| `"Felt"` / `"MemRL"` / `"utility_score"` | 0 / 0 / 0 |
| evidence FTS, same OR, distinct `fact_id` | 301 |

`PREFETCH_K = 50` ([`recall_rrf.rs:10`](../gzmo-core/src/memory/recall_rrf.rs)). `honeypot_fts_stream_query` `ORDER BY rank LIMIT 50` ([`vault.rs:1247`](../gzmo-core/src/memory/vault.rs)). The FTS rank list is the **best-BM25 50 of those 407 Use-stem rows**, not a Felt Use result set.

Top of that list (same `ORDER BY rank`) is generic “uses/used” ingest: Lightpanda “using 24 megabytes”, Speculative Decoding “uses a small draft model”, Obolus Validator “Uses three evaluation criteria”, GPU Boost “Uses a V/F curve”, … — all `utility_score = 0`.

### 3.3 Named collision rows inside the Use pool

| Reported hit | Why it matched `"Use"` → `us` | FTS position in the 407 | Q / recall |
|--------------|-------------------------------|------------------------:|------------|
| `[PROJECT:Glances] Uses PEP8 compatible code` | “Uses” | 14 | 0 / 0 |
| `[TOOL:Prometheus] Used for metrics collection and monitoring` | “Used” | 42 | 0 / 3 |
| `[CONCEPT:PEP8] A code style standard that Glances uses` | “uses” | 112 | **3.0 / 3** |
| `[SYSTEM:Prometheus] Recommended for use with PromQL…` | “use” | 378 | 0 / 0 |

Evidence FTS `ORDER BY rank` also surfaces `[PROJECT:Glances] Uses PEP8 compatible code` in the first handful. RRF therefore sees Glances/PEP8 on **two** lexical lists ([`recall_rrf`](../gzmo-core/src/memory/vault.rs) pushes FTS, then evidence FTS, then graph-or-keyword — [`vault.rs:1012`](../gzmo-core/src/memory/vault.rs)). `STREAM_TOP_RESCUE = 0.025` ([`vault.rs:1072`](../gzmo-core/src/memory/vault.rs)) further protects each stream’s top-5.

A separate operator query `Glance` (map #221 note) is even tighter: MATCH `"Glance"` → **6** latest rows, all `share-doc-glances-CONTRIBUTING.md` (porter `glanc`). That query does not need `"Use"` at all. It still cannot find Felt Use doctrine (zero rows).

### 3.4 L.I.N.C. is not in the lexical pool

| Id | Content | Q / recall | In Use FTS? | `INSTR(lower(content),'use')` |
|----|---------|------------|-------------|--------------------------------|
| `17d0564b-…` | `[CONCEPT:L.I.N.C.] Works on edge candidates before honeypot promotion` | **9.0 / 10** | no | 0 |
| `4a2d633c-…` | `[CONCEPT:L.I.N.C.] A four-gate neurosymbolic validator` | 0 / 0 | no | 0 |

Evidence FTS OR does not hit those `fact_id`s either. Graph LIKE needles are empty (§2.3). Keyword `contains("use")` misses them.

So L.I.N.C. **cannot** be explained by FTS/RRF lexical fusion for this query. The only product stream that can admit it is the optional vector list inside `recall_rrf` (`embedder.embed` → `search_with_decay` + Qdrant, [`vault.rs:1022`](../gzmo-core/src/memory/vault.rs)). Living snapshot: **3005/3005** latest honeypot rows have embeddings (`length(embedding) > 16`); living `gzmo.toml` now has `[embeddings] enabled = true`. This note does **not** replay that embed (would mutate Felt Use via MCP `touch_hits`). If the 2026-08-30 sitting had the embedder attached, cosine against “honeypot promotion / neurosymbolic validator” can put L.I.N.C. into the RRF lists; if embed failed or was off, L.I.N.C. was not in this query’s pool and the operator mixed sittings.

---

## 4. Why utility could not save it

Phase B is explicit:

```118:156:gzmo-core/src/memory/recall_rrf.rs
/// Pool-relative utility weight. RRF adjacent ranks differ by ~3e-4; 0.05 lets
/// max-Q in the pool outrank min-Q without inventing hits. …
pub const UTILITY_POOL_LAMBDA: f64 = 0.05;

/// MemRL phase B: boost relevance scores by in-pool `utility_score` (Q), then
/// re-sort. Does not add hits. Equal utility leaves relative relevance order.
pub fn apply_utility_boost(...)
```

Call site: after RRF + optional rerank, `apply_utility_select` loads `honeypot.utility_score` for **ids already in `scored`**, then `apply_utility_boost`, then `truncate(limit)` ([`vault.rs:1094`](../gzmo-core/src/memory/vault.rs), [`vault.rs:1103`](../gzmo-core/src/memory/vault.rs)).

| Fact | Why Q cannot rescue this search |
|------|----------------------------------|
| No doctrine id in any stream | Boost has nothing to promote. The test that Q-select works (`search_recall_orders_by_utility_inside_fts_pool`, [`vault.rs:3176`](../gzmo-core/src/memory/vault.rs)) inserts **two FTS-matching** fixtures first. |
| Pool is 407 Use-stem collisions | 396/407 have `utility_score = 0`. Max Q **inside** the pool is **5.0** (unrelated “Uses scoring / Ollama / RAPL” rows). Mean Q ≈ **0.12**. |
| λ = 0.05 | Even max-Q vs min-Q only adds 0.05 to the relevance score. It reorders near-ties; it does not replace “uses” BM25 with a missing Felt Use sentence. |
| Rerank still sees the same docs | `apply_rerank` ([`vault.rs:1670`](../gzmo-core/src/memory/vault.rs)) reorders `scored`; it does not retrieve. Living `[rerank] enabled = true` can shuffle Glances vs Lightpanda; it cannot mint doctrine. |
| L.I.N.C. Q=9 is the wrong direction | Global 5th-highest Q on the Keep (`17d0564b`). If a vector stream admitted it, phase B would **prefer** it over Q=0 Use-stem rows. Utility would make the reported collision worse, not better. |
| Glance cannot mint Q | `FeltUseKind::Glance` `utility_weight = 0` ([`felt_use.rs:34`](../gzmo-core/src/memory/felt_use.rs)). MCP default `write_scratch=true` is **Cited** (+3 Q) on whatever ranked ([`platform_memory.rs:256`](../gzmo-core/src/platform_memory.rs)). The PEP8/Glances rows already at Q=3 / recall=3 are consistent with a prior Cited search — gym-shaped reinforcement of the collision, not of doctrine. |

`reinforce_felt` only `UPDATE`s the id it is given ([`vault.rs:472`](../gzmo-core/src/memory/vault.rs)). There is no “search failed → boost something else” path.

---

## 5. Honest-empty vs this collision (input to #225)

`memory_search_core` already has an empty-result string: `No relevant memories found for query: '{query}'` when `search_recall` returns `[]` ([`platform_memory.rs:346`](../gzmo-core/src/platform_memory.rs)). This sitting never reached it. FTS returned 407 rows, so the product path is “confident wrong hits”, not honest-empty.

Search-success for Felt Use doctrine on this Keep, given this trace:

- **Cannot** be “any top-5 hit” — top-5 is Use-stem / Glance-stem furniture.
- **Can** be: a ranked honeypot row whose `content` actually states Felt Use / MemRL / `utility_score`, **or** honest-empty when no such row exists.
- Utility / RRF / rerank are not the missing piece. **Ingest of a doctrine row** is. Until that row exists, the only honest product behavior for this query family is empty (or a fail-closed “no doctrine” signal), not a 5-hit Glances card.

---

## 6. Sources

| Claim | Source |
|-------|--------|
| MCP default limit 5, scratch on | [`serve.rs:174`](../gzmo-core/src/mcp/serve.rs) |
| `search_recall` → `recall_rrf` | [`vault.rs:972`](../gzmo-core/src/memory/vault.rs) |
| MATCH builder is OR of quoted tokens ≥2 chars | [`recall_rrf.rs:212`](../gzmo-core/src/memory/recall_rrf.rs) |
| Porter FTS tables | [`vault.rs:197`](../gzmo-core/src/memory/vault.rs); [SQLite FTS5 porter](https://www.sqlite.org/fts5.html#tokenizers) |
| Q-select does not invent hits | [`recall_rrf.rs:123`](../gzmo-core/src/memory/recall_rrf.rs) |
| Glance Q=0 / Cited Q=+3 | [`felt_use.rs:34`](../gzmo-core/src/memory/felt_use.rs) |
| Living census + MATCH counts | `sqlite3 -readonly ~/.gzmo-living/data/vault.db` (this sitting) |
| Vault attach | `gzmo_memory_status` → `vault_path=/home/gzmo/.gzmo-living/data/vault.db`, `honeypot_latest=3005` |

No INSERT/UPDATE was issued against the living vault. MCP search was **not** re-run (would `touch_hits`).
