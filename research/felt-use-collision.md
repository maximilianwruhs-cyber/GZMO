# Why Felt Use search returned Glances, Prometheus, and Φ Brain

Primary sources: `gzmo-core/src/memory/recall_rrf.rs` (`fts_match_query_mode`, `apply_utility_boost`), `vault.rs` (`recall_rrf`). Living vault `~/.gzmo-living/data/vault.db` queried 2026-08-30. Live MCP `gzmo_memory_search` same day.

## Mechanism

`fts_match_query` splits on whitespace, keeps tokens length ≥ 2, and joins them with **OR** (`recall_rrf.rs`). `"Felt Use"` becomes `"Felt" OR "Use"`. Graph tokens skip words shorter than 4 characters, so `Use` is FTS-only.

`apply_utility_boost` re-ranks **inside** the relevance pool (`UTILITY_POOL_LAMBDA = 0.05`). It cannot add a fact that FTS/RRF never retrieved. Glance does not mint utility.

## This Keep

| Check | Result |
|---|---|
| Latest honeypot with Felt Use / MemRL / utility_score | **0** |
| `semantic_vault` with those strings | **0** |
| FTS `"Felt" OR "Use"` | **407** rows |
| Latest honeypot | 3005 |
| `utility_score > 0` | 78 |

Rows that match because they contain the English word *use* include Prometheus (`Recommended for use with PromQL…`, `Used for metrics…`) and Glances via PEP8 (`A code style standard that Glances uses`).

A live MCP search for `Felt Use MemRL utility_score Brain Feed` (limit 5, no scratch) returned Φ Brain / Forge / Consciousness Score — the token **Brain** from "Brain Feed", not Felt Use doctrine.

## Conclusion

The collision is lexical OR on common words against a vault that has **no doctrine row**. Utility Q cannot help until a row that actually mentions Felt Use / MemRL enters the pool.
