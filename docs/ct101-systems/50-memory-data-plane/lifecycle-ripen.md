# Lifecycle, Ripen & KG Promotion

**System:** [50-memory-data-plane](./SYSTEM.md)  
**Sources:** `gzmo-core/src/memory/lifecycle.rs`, `gzmo-core/src/memory/ripen.rs`, `gzmo-core/src/memory/kg_promotion.rs`

---

## Capability

**Lifecycle** classifies new truths against existing honeypot rows (duplicate, extends, contradicts, derives) and drives supersession. **Ripen** (M5) groups honeypot by entity, resolves contradictions, synthesizes concept cards, exports to `knowledge_core` — orchestrator cron **honeypot_ripen** midnight UTC. **KG promotion** sanitizes relation types and entity names for Neo4j writes via MCP (**63k** graph nodes live).

---

## How it works

### Lifecycle classification

```50:67:gzmo-core/src/memory/lifecycle.rs
pub fn classify_truth_pair(old_content: &str, new_content: &str) -> LifecycleKind {
    let old_n = normalize_truth_content(old_content);
    if old_n == new_n { return LifecycleKind::Duplicate; }
    if contradicts_heuristic(old, new) { return LifecycleKind::Contradicts; }
    if is_extension(&old_n, &new_n) { return LifecycleKind::Extends; }
    LifecycleKind::Unrelated
}
```

Derived cognition gate:

```182:200:gzmo-core/src/memory/lifecycle.rs
pub fn is_unverified_derived(truth: &ExtractedTruth, origin: &str) -> bool {
    // dream/spark/session_distill origins need evidence or confidence ≥ 0.92
```

### Ripen pipeline

```82:105:gzmo-core/src/memory/ripen.rs
pub fn ripen_honeypot(vault: &SqliteVault, config: &RipenConfig) -> Result<Vec<ConceptCard>> {
    let groups = group_by_entity(&conn, config)?;
    let resolved = resolve_contradictions(&conn, groups, config)?;
    if config.export { export_cards(&conn, &resolved)?; }
```

Default: ≥5 entries per entity, confidence ≥0.85, max 50 cards/run.

### KG promotion helpers

```14:47:gzmo-core/src/memory/kg_promotion.rs
pub fn sanitize_relation_type(raw: &str) -> String { /* ASCII upper, underscores */ }
pub fn canonicalize_relation_type(raw: &str) -> String {
    // AUTHOR/WROTE → AUTHORED_BY; HYPOTHESIZED_LINK → empty (blocked)
}
```

Used by dream deep-phase and ingest Neo4j batch (`KG_BATCH_SIZE = 20`).

---

## Interfaces

| Kind | Detail |
|------|--------|
| Orchestrator | `[orchestration.jobs.honeypot_ripen]` daily midnight UTC |
| Table | `knowledge_core` — ripen export target |
| Neo4j | MCP `mcp__memory__create_entities` / `create_relations` |
| Vault API | `supersede_honeypot`, `find_latest_honeypot_by_entity` during promote |

---

## THINKING nodes

> **THINKING — lifecycle.rs:classify_truth_pair**
> - *Reviewed:* Rule-based, no LLM; entity tags required for contradicts heuristic.
> - *Insight:* Keeps CT101 promote path deterministic and cheap.
> - *Risk / limitation:* Subtle contradictions without negation keywords slip through as Unrelated.
> - *Enhancement:* Optional LLM adjudicate queue for borderline pairs [GZMO-next].

> **THINKING — lifecycle.rs:is_unverified_derived**
> - *Reviewed:* Covers renamed origins (`verified_dream`, `session_distill`).
> - *Insight:* Closes bypass if origin strings change without updating gate.
> - *Risk / limitation:* 0.91 confidence derived without evidence still blocked — may frustrate spark.
> - *Enhancement:* Configurable derived confidence floor per engine [CT101-safe].

> **THINKING — ripen.rs:group_by_entity**
> - *Reviewed:* Parses `[TYPE:Name]` prefix; filters groups < min_entries_for_card.
> - *Insight:* M5 mature DB export without LLM synthesis — template summary from top facts.
> - *Risk / limitation:* Entities with <5 facts never ripen despite high value.
> - *Enhancement:* Lower threshold for Structural decay_class entities [CT101-safe].

> **THINKING — kg_promotion.rs:canonicalize_relation_type**
> - *Reviewed:* Collapses author synonyms; blocks HYPOTHESIZED_LINK from ingest.
> - *Insight:* Protects 63k-node graph from relation explosion.
> - *Risk / limitation:* Aggressive RELATED_TO collapse may lose semantic precision.
> - *Enhancement:* Relation whitelist per doc_class [GZMO-next].

---

## Advancement

- **CT101:** Run ripen after large ingest waves; compare `knowledge_core` row count in health.
- **GZMO-next:** LLM concept card synthesis per MEMORY_ARCHITECTURE_SPEC M5 vision.

---

## Enhancement backlog

1. **[CT101-safe]** Ripen run summary in Synapse (`distill_complete`-class event with card count).
2. **[CT101-safe]** `[derived_min_confidence]` config for lifecycle gate.
3. **[CT101-safe]** Entity-level ripen dashboard in `gzmo health`.
4. **[GZMO-next]** Human review queue before knowledge_core export.
5. **[GZMO-next]** Graph reconcile metrics tied to ripen supersession ids.
