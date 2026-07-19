# Unpark roadmap (post A+C GREEN)

**Status:** Active (2026-07-19)  
**Prerequisite:** `bash scripts/production-readiness-gate.sh` → A+C GREEN (0 HOLD)  
**Doctrine:** [SPINE_FOCUS.md](./SPINE_FOCUS.md) · [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md)

## What changed

A+C production readiness unlocked the former **Park freeze**. Satellites move into a sequenced **Unpark queue**. Keep pillars A+C remain co-primary brand; satellites are useful, not the stranger install.

## Hard boundaries (never reverse)

| Boundary | Why |
|----------|-----|
| ADR-0003 one overnight writer | CT101 sole living brain |
| A never requires C sidecars | Stranger `~/.gzmo` stays Redis/Qdrant/Neo4j off |
| Never point `gzmo-memory` at living vault | Product ≠ living attach |
| Arena / IpW / Forge outside `gzmo-daemon` by default | [OBOLUS_ARENA_BOUNDARY.md](./OBOLUS_ARENA_BOUNDARY.md) |
| Cognis / escape-loop / ZPD never production brain | Lab/research only; not GREEN overnight gate |
| Pi optional glass, not primary UX | [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md) |

## Waves

| Wave | Focus | Exit |
|------|-------|------|
| **0** | Doctrine flip (this doc + spine/map) | Docs agree Unpark open; production gate GREEN |
| **1** | Operator surfaces: herdr · Pi glass · tinyFolder · AOS poll | Script/gate artifacts; A+C 0 FAIL |
| **2** | Ritual/theater: pantheon feat skills · discovery theater · HSP emit | Skills installable; faithfulness + takeaway-recall PASS |
| **3** | Arena economics lab (sibling-first) | Arena overnight without changing daemon job set |
| **4** | Later packaging: AOS CE · marketplace · wiki mind · portable-core RFC | Separate install docs; A still sidecar-free |

## Wave detail

### Wave 1 — Operator surfaces

1. `herdr-metabolism-check.sh` — demable link/status/ritual probe  
2. `pi-glass-check.sh` — path/attach hygiene (CLI canonical)  
3. `tinyfolder-check.sh` — inbox → ingest spike readiness  
4. `aos-poll-check.sh` — read-only living status poll (no Arena required)

### Wave 2 — Ritual / theater

1. Pantheon ritual front door + feat-stack inventory (no ghost `DICE_MASTER_*`)  
2. Mutual-discovery theater scout≠KPI  
3. HSP Synapse emit hooks (not on GREEN overnight gate)

### Wave 3 — Arena lab

1. Arena / RAPL / €/night observability wrappers  
2. IpW router demable advice  
3. Forge recommend path — never auto-block distill

### Wave 4 — Later

1. AOS Customer Edition sketch (on top of C only)  
2. OKCP marketplace + multi-node forge notes  
3. Wiki / Observatory demable mind  
4. Portable GZMO core RFC (inventory-first; no big-bang rewrite)

## Never-as-brain / infra-parked

| Item | Treatment |
|------|-----------|
| Cognis dialect | Lab stub |
| Escape-loop / attractor brand | Research kit |
| ZPD tutor | Lab; never GREEN overnight |
| Sovereign `:8010` / VM200 `:8080` | Infra-parked (broken/retired) |
| Pedagogy deferred backlog | Wave 2b after pantheon ritual |

## Guardrails

```bash
bash scripts/production-readiness-gate.sh   # after each wave merge
bash scripts/unpark-wave-check.sh           # wave artifact presence
# → data-next/unpark-waves/latest.{json,md}
```

Wave 4 docs: [AOS_CUSTOMER_EDITION.md](./AOS_CUSTOMER_EDITION.md) · [OKCP_MARKETPLACE.md](./OKCP_MARKETPLACE.md) · [WIKI_OBSERVATORY_MIND.md](./WIKI_OBSERVATORY_MIND.md) · [PORTABLE_GZMO_CORE_RFC.md](./PORTABLE_GZMO_CORE_RFC.md).

Tag `v*` when tip exceeds release-freshness window. One wave focus per PR batch.
