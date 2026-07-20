# Living metabolism organs — implementation note (2026-07-20)

Implements the four Keep organs from the living-output-quality scorecard ([living-output-quality-2026-07-20.md](living-output-quality-2026-07-20.md)).

| Organ | Module | Seam | Artifact |
|-------|--------|------|----------|
| **Felt Use** | `memory/felt_use.rs` | `PlatformMemory::memory_search`, tool search, spark Bonded on promote | honeypot `recall_count` / `last_recalled_at` |
| **Refractory Field** | `spark_field.rs` | `SparkEngine::select_phase` score × refractory + soft-pick | `{data}/spark/refractory.json`, `last-spark-report.json` |
| **Immune Patrol** | `immune.rs` | end of `DreamEngine::consolidate` | `{data}/immune/plan-{night}.json` (**dry_run only**) |
| **Night Lymph** | `night_lymph.rs` | dream + spark complete | `{data}/night-lymph/latest.json` (status surface) |

## Config (`[spark]`)

```toml
refractory_slots = 48
refractory_half_life_hours = 72.0
refractory_strength = 0.85
soft_pick_top_k = 8
soft_pick_temperature = 0.35
```

## Acceptance (after deploy to CT101)

1. Run `gzmo memory search "…"` (living) → census `recall_count > 0` on hit ids.
2. Two spark cycles → `spark/refractory.json` grows; last-N unique anchors should diverge from monoculture.
3. After dream → `immune/latest.json` lists plan-only candidates (no vault mutate).
4. `gzmo status` / ecosystem shows **Night lymph** section.

## Explicit non-actions

- Immune does **not** apply tombstones on living.
- No vault import into `data-next`.
- Product MCP still must not point at living vault.
