# Uniqueness Thesis — Little Tools Lab × GZMO-next

**Date:** 2026-07-16  
**Scope:** Product identity for the closed-set cognition kit and the workstation production instance.  
**Not:** Another P0 trust audit or CT101 feature-parity list.

---

## Irreducible thesis

**GZMO-next is a closed-set cognition kit whose overnight metabolism is assembled from composable puzzle pieces (bash recipes + schema-gated artifacts), not a chatbot with RAG bolted on.** Uniqueness lives in the *wiring*: Distill → Honeypot/Spark/Dream → Vault → Bench→Lorenz→Fuse→promote, enforced by a two-stack assembly guard, and made legible as Austrian-pragmatist overnight status (`DREAMS.md`, `scheduler-runs/`, fused TOML). A generic LangChain agent or cron+LLM stack cannot fake that without copying this exact architecture — and today the organism is *alive but undersold*: 111 vault facts, live librarian distill, ops-smoke PASS, fused calibration pending — yet **zero honeypot-origin facts** and thin spark lineage on the operator surface.

---

## Five signature claims (with evidence)

### 1. Assembly is the product API; the daemon is not the product

`gzmo assemble` + `[assembly] = "lab"` + frozen schemas are the public contract. Overnight cognition is a thin cron that shells recipes; algorithms stay in piece repos.

- [`GZMO/gzmo-cli/src/assemble_cmd.rs`](../gzmo-cli/src/assemble_cmd.rs) — recipe → script map  
- [`GZMO/gzmo-core/src/assembly.rs`](../gzmo-core/src/assembly.rs) — `effective()` forces Lab→Inline unless `GZMO_INSTANCE=next`  
- [`GZMO/gzmo-scheduler/src/jobs.rs`](../gzmo-scheduler/src/jobs.rs) — dream/distill/spark/handoff/ops → lab scripts  
- [`little-tools-lab/schemas/`](../../little-tools-lab/schemas/) — `cognition-smoke-meta`, `fuse-meta`, `dream-stats`, …

### 2. Honeypot + verify + promote is the identity sentence — not “chat remembers”

Canonical MACHINE identity: **Honeypot + verify + promote = GZMO**; GZMO is a distillation pipeline.

- [`GZMO/MACHINE.md`](../MACHINE.md)  
- Hot path pieces: `session-distill` → `honeypot-gate` → `spark-link` → `rrf-recall` in [`cognition-smoke.sh`](../../little-tools-lab/scripts/cognition-smoke.sh)  
- Gate algorithm: [`honeypot-gate/src/qualify.rs`](../../honeypot-gate/src/qualify.rs), lifecycle in `lifecycle.rs`

**Live gap (2026-07-16):** `data-next/vault.db` has **111** facts (`librarian_extract` 69 + `session_distill` 42); `decay_class` all `SessionDistill`; **no honeypot-origin rows**. The pipeline skeleton runs; the signature lifecycle stage is not yet *felt*.

### 3. Spark is serendipity with a triangular score — not random recall

`spark-link` ranks stale anchors via `stale_sweetness` × importance × recent cosine, optional LLM hypothesize+verify.

- [`spark-link/src/scoring.rs`](../../spark-link/src/scoring.rs) — `stale_sweetness`, `score_spark_anchor`  
- Scheduled: `cognition-smoke.sh --spark-run` at 03:30/22:30 UTC ([`jobs.rs`](../gzmo-scheduler/src/jobs.rs))

### 4. Calibration theatre: Lorenz → LLM params → fuse → human promote

Chaos is not a production PulseLoop cron (ADR-0002). It *is* a signature when Lorenz trajectories map to temperature/top_p and feed `config-fuse`, then an operator promotes fused TOML deliberately.

- [`lorenz-map/src/parameter_mapper.rs`](../../lorenz-map/src/parameter_mapper.rs) — `map_state`  
- [`bench-to-fuse.sh`](../../little-tools-lab/scripts/bench-to-fuse.sh) → [`config-fuse`](../../config-fuse/config_fuse/fuse.py)  
- [`assembly.rs` `handoff_apply_target`](../gzmo-core/src/assembly.rs) — sibling `*-fused.toml`, never clobber live  
- Live: `gzmo-next-fused.toml` **present**; `gzmo instance status` shows promote pending

### 5. Two-stack honesty is runtime-enforced, not a slide

CT101 stays frozen; lab backends cannot activate without `GZMO_INSTANCE=next`. Beat-gate compares behavior; it does not authorize CT101 grafts.

- ADR-0001 / [`CT101_BOUNDARY.md`](CT101_BOUNDARY.md)  
- [`assembly.rs:62-71`](../gzmo-core/src/assembly.rs)  
- [`gzmo-scheduler` refuses non-next](../gzmo-scheduler/src/main.rs)

---

## Explicit non-claims

| We are NOT | Why |
|------------|-----|
| A Mem0/Zep/Supermemory clone | Identity is honeypot lifecycle + distill, not a hosted memory API |
| “Local Open WebUI + RAG” | Overnight is recipe assemblies with beat/schemas, not a chat UI with embeddings |
| Feature-parity with CT101’s 60k vault | Fresh `data-next/` by design; importing CT101 vault fakes uniqueness |
| A 47-piece shopping list | Closed set; deepen Dream/Spark/Fuse, don’t invent organs |
| Pedagogy/chaos as overnight cron | ADR-0002 — lab/chat/calibration only unless ADR amended |
| Mature ≡ deep | Enhancement audit: 46/46 mature; many pieces still shallow or unwired |

---

## Competitor contrast (L8 — cruel)

| Class | What they have | What GZMO has that they cannot fake without this architecture |
|-------|----------------|---------------------------------------------------------------|
| **LangChain / agent frameworks** | Tool graphs, memory abstractions | Closed 46-piece contract; file-path recipes; instance-gated assembly; beat-gate vs frozen incumbent |
| **Open WebUI + RAG** | Chat + vector store | Distill→honeypot→spark overnight cron; fused Lorenz calibration with human promote; Synapse session_end → distill queue |
| **Plain cron + LLM** | Shell scripts calling an API | Schema-validated meta envelopes; `ltl-common` bins (no algorithm Python in bash); `gzmo assemble` product API; Observatory `scheduler_runs` |
| **OpenClaw / sovereign-agent clones** | Local LLM + tools + notes | MACHINE pipeline (verify/promote/honeypot); spark `stale_sweetness`; RAPL/IPW routing (`rapl-route`); organ-audit constellation discipline |
| **Second-brain note apps** | Wikilinks, daily notes | Operator surface is metabolism status (`DREAMS.md`, vault origins, fused TOML), not a journaling app with an LLM |

Drop any marketing claim that reduces to “we run a local model” or “we have Redis+Qdrant” — those are commodity. Keep claims that require this exact puzzle-piece + assembly + two-stack guard.

---

## Live metabolism snapshot (Phase 3, 2026-07-16)

| Probe | Result |
|-------|--------|
| `gzmo instance status` | `next`; all five assembly backends `lab→lab`; fused TOML present |
| `gzmo-scheduler` / `llama-prime` | systemd **active** |
| Prime LLM / Qdrant | HTTP 200 |
| Vault | next vault growing; origins include librarian/session_distill/**honeypot** (honeypot-origin facts live as of 2026-07-16) |
| `DREAMS.md` | 2026-07-16 librarian_live; 4 sessions, 11 facts promoted this run |
| `ops-smoke.sh --live` | **PASS**; redis/qdrant/neo4j true; synapse healthy=false (stale fixture sessions); queue_depth=4 |
| `scheduler-runs/latest.json` | `ops_health` ok (2026-07-15) |

**Verdict:** Plumbing and calibration path are real. Signature *organism* (honeypot ripen, spark lineage brag, graph-ledger drift) is the elevation target — not more infra.

---

## One-page operator mental model (L7)

```text
Foreground:  gzmo chat  → inline tools + /status  (fixture assemble slash = rehearsal)
Overnight:   gzmo-scheduler → lab recipes → data-next artifacts
Operator API: gzmo assemble <recipe> [--live] [--apply]
Promotion:   review gzmo-next-fused.toml → gzmo config promote-fused --diff|--apply
Never:       graft lab loops into CT101; treat dice-scheduler as production cron
```

---

## References

- [`UNIQUENESS_DEEP_ANALYSIS_PROMPT.md`](UNIQUENESS_DEEP_ANALYSIS_PROMPT.md)  
- [`little-tools-lab/docs/PIECE_ELEVATION_MAP.md`](../../little-tools-lab/docs/PIECE_ELEVATION_MAP.md)  
- [`SIGNATURE_EXPERIENCES.md`](SIGNATURE_EXPERIENCES.md)  
- [`UNIQUENESS_BUILD_PLAN.md`](UNIQUENESS_BUILD_PLAN.md)  
- [`config/SOUL-next.md`](../config/SOUL-next.md)  
- ADR-0001 / ADR-0002 under `little-tools-lab/docs/adr/`
