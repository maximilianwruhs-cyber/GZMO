# Evidence — Quote Localization & Recall Stream

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Source:** `gzmo-core/src/memory/evidence_localize.rs`

---

## Capability

The evidence subsystem localizes verifier quotes into source documents, expands to ±1 sentence windows, stores rows in the `evidence` table with FTS5 mirror, and feeds a dedicated **evidence FTS + vector stream** inside honeypot RRF recall. This grounds derived cognition (dream/spark/distill) before honeypot promotion.

---

## How it works

### Core localization

Normalized substring match with LCS fallback (≥12 chars), then sentence-window expansion:

```5:78:gzmo-core/src/memory/evidence_localize.rs
pub fn localize_evidence(body: &str, verifier_quote: &str) -> EvidenceSpan {
    let (norm_body, body_map) = normalize_with_map(body);
    let norm_quote = normalize_only(verifier_quote);
    let char_range = if let Some(byte_start) = norm_body.find(&norm_quote) {
        // char_start/end via body_map
    } else {
        longest_common_substring_chars(&body_chars, &quote_chars)
    };
    // segment_sentences → window ±1 sentence
```

### Per-observation path (ingest)

Prefers observation text over shared entity quote when multiple observations exist:

```82:111:gzmo-core/src/memory/evidence_localize.rs
pub fn localize_observation_evidence(
    body: &str,
    observation: &str,
    entity_evidence: &str,
    observation_count: usize,
) -> Option<EvidenceSpan> {
    // obs ≥8 chars first; entity quote only if observation_count == 1
```

Vault recall uses `honeypot_evidence_fts_stream` and `honeypot_evidence_vector_stream` (see `vault.rs` RRF) as additional rank lists fused via `recall_rrf`.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Table | `evidence` — FK `fact_id` → `honeypot(id)` |
| FTS | `evidence_fts` (Porter) |
| Writes | `honeypot::upsert_evidence_row` during promote |
| Gate | `lifecycle::is_unverified_derived` — derived facts need evidence or ≥0.92 confidence |
| KG | `kg_promotion::MIN_EVIDENCE_CHARS` (12) for Neo4j promotion |

---

## THINKING nodes

> **THINKING — evidence_localize.rs:localize_evidence**
> - *Reviewed:* normalize_with_map + LCS fallback + sentence segmentation.
> - *Insight:* Designed for German/English ingest with whitespace-tolerant matching.
> - *Risk / limitation:* Fallback stores verifier quote verbatim without char offsets — weaker cite-back.
> - *Enhancement:* Log localization method in evidence row metadata [CT101-safe].

> **THINKING — evidence_localize.rs:localize_observation_evidence**
> - *Reviewed:* Multi-observation docs avoid sharing one entity-level quote across facts.
> - *Insight:* Prevents duplicate evidence spans polluting RRF evidence stream.
> - *Risk / limitation:* Short observations (<8 chars) may skip localization entirely.
> - *Enhancement:* Lower threshold for structured bullet observations [GZMO-next].

> **THINKING — vault RRF evidence streams**
> - *Reviewed:* evidence FTS + vector lists merged in `recall_rrf`.
> - *Insight:* Evidence tier makes recall cite-backed, not just fact-text matching.
> - *Risk / limitation:* Evidence without embeddings skips vector stream silently.
> - *Enhancement:* Embed evidence_text on promote [CT101-safe].

---

## Advancement

- **CT101:** Backfill evidence embeddings for top-recalled honeypot rows.
- **GZMO-next:** Char-offset UI in Observatory for operator quote verification.

---

## Enhancement backlog

1. **[CT101-safe]** Metric: % honeypot rows with localized evidence vs fallback-only.
2. **[CT101-safe]** Batch evidence embed backfill script tied to VM200.
3. **[CT101-safe]** Stricter `quote_verifier` length audit in promote path.
4. **[GZMO-next]** PDF/HTML offset maps for non-markdown document layer.
5. **[GZMO-next]** Evidence contradiction detection against superseded honeypot rows.
