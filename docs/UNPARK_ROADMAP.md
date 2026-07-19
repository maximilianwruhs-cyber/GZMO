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

1. `herdr-metabolism-demo.sh` / `herdr-metabolism-check.sh`  
2. `pi-glass-fix.sh` / `pi-glass-check.sh`  
3. `tinyfolder-ingest-demo.sh` / `tinyfolder-check.sh`  
4. `aos-poll-dashboard.sh` / `aos-poll-check.sh`

### Wave 2 — Ritual / theater

1. `pantheon-ritual-demo.sh` — thin skills + re-land checklist (no ghost `DICE_MASTER_*`)  
2. `discovery-theater-demo.sh` — scout≠KPI session prep  
3. `hsp-emit-demo.sh` — motif event files + `hsp-metabolism-sonify.sh` MIDI/WAV (not on GREEN overnight gate)

### Wave 3 — Arena lab

1. `arena-lab-demo.sh` — RAPL probe + €/night aggregate observability; sibling Arena  
2. `ipw-route-demo.sh` — advice for chat/heavy_bench  
3. `forge-lab-demo.sh` — recommend.json stub (never auto-block distill)

### Wave 4 — Later

1. `aos-ce-smoke.sh` + [AOS_CUSTOMER_EDITION.md](./AOS_CUSTOMER_EDITION.md)  
2. `marketplace-check.sh` + [OKCP_MARKETPLACE.md](./OKCP_MARKETPLACE.md)  
3. `wiki-mind-check.sh` + [WIKI_OBSERVATORY_MIND.md](./WIKI_OBSERVATORY_MIND.md)  
4. `portable-core-inventory.sh` + [PORTABLE_GZMO_CORE_RFC.md](./PORTABLE_GZMO_CORE_RFC.md)

## Never-as-brain / infra-parked

| Item | Treatment |
|------|-----------|
| Cognis dialect | Lab stub |
| Escape-loop / attractor brand | Research kit |
| ZPD tutor | Lab; never GREEN overnight |
| Sovereign `:8010` / VM200 `:8080` | Infra-parked (broken/retired) |
| Pedagogy deferred backlog | Wave 2b chat + Wave 2b.1 TUI `maybe_teach` landed. Chaos Slice C.1 oscillator stays lab-only — never daemon PulseLoop / living overnight ([PANTHEON_FEAT_RELAND.md](./PANTHEON_FEAT_RELAND.md); `bash scripts/verify-mentor.sh`) |

## Guardrails

```bash
bash scripts/production-readiness-gate.sh   # after each wave merge
bash scripts/unpark-wave-check.sh           # wave artifact presence
# → data-next/unpark-waves/latest.{json,md}
```

Wave 4 docs: [AOS_CUSTOMER_EDITION.md](./AOS_CUSTOMER_EDITION.md) · [OKCP_MARKETPLACE.md](./OKCP_MARKETPLACE.md) · [WIKI_OBSERVATORY_MIND.md](./WIKI_OBSERVATORY_MIND.md) · [PORTABLE_GZMO_CORE_RFC.md](./PORTABLE_GZMO_CORE_RFC.md).

Tag `v*` when tip exceeds release-freshness window. One wave focus per PR batch.
