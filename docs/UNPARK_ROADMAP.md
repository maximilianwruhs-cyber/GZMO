# Unpark roadmap (post keep-quality soak)

**Status:** Active (2026-07-20) — USP = airgap living ([ADR-0004](./ADR-0004-airgap-living-usp.md))  
**Prerequisite:** `bash scripts/keep-quality-soak.sh --summary` → `soak_ready_unpark_ok` (default 3 trailing GREEN)  
**Also useful:** `bash scripts/production-readiness-gate.sh` (lite + living ops)  
**Doctrine:** [SPINE_FOCUS.md](./SPINE_FOCUS.md) · [KEEP_QUALITY.md](./KEEP_QUALITY.md) · [BRAIN_FEED.md](./BRAIN_FEED.md) · [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md)

## What changed

Keep-quality on the living box unlocked sequenced **Unpark**. Living is first-class USP; lite is bootstrap only. Satellites are local MCP clients — not a second metabolism.

**Active strengthen lane:** [BRAIN_FEED.md](./BRAIN_FEED.md) (`brain-feed-check.sh`) — takeaway, tinyFolder→living, Felt Use, serendipity promote, calibration/Arena **human** pin. Theater waves below stay demable but demoted.

## Hard boundaries (never reverse)

| Boundary | Why |
|----------|-----|
| ADR-0003 one overnight writer | One living brain per vault |
| Lite never requires living sidecars | Bootstrap `~/.gzmo` stays Redis/Qdrant/Neo4j off |
| Never point `gzmo-memory` at living vault as stranger default | Lite ≠ living attach labels |
| No public MCP webserver SKU | ADR-0004 |
| Arena / IpW / Forge outside `gzmo-daemon` by default | [OBOLUS_ARENA_BOUNDARY.md](./OBOLUS_ARENA_BOUNDARY.md) |
| Cognis / escape-loop / ZPD never production brain | Lab/research only; not GREEN overnight gate |
| Pi optional glass, not primary UX | [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md) |

## Waves

| Wave | Focus | Exit |
|------|-------|------|
| **0** | Doctrine flip (ADR-0004 + spine + keep-quality) | Docs agree; first keep-quality GREEN |
| **BF** | **Brain Feed** (active): herdr/takeaway · tinyFolder→living · Felt Use · serendipity promote · intel human-pin | `brain-feed-check.sh` GREEN |
| **1** | Operator surfaces (theater demoted): Pi glass · AOS poll · herdr polish | Script/gate artifacts |
| **2** | Ritual/theater: pantheon feat skills · discovery theater · HSP emit | Skills installable; demable only |
| **3** | Arena economics lab — **promote-only** feeds Brain Feed P1 | Arena overnight without changing daemon job set |
| **4** | Later packaging: AOS CE · marketplace · wiki mind · portable-core RFC | Separate install docs; lite still sidecar-free |

## Wave detail

### Wave 1 — Operator surfaces

1. `herdr-metabolism-demo.sh` / `herdr-metabolism-check.sh` — plugin contract + close-ritual (`now_flag=false`)  
2. `pi-glass-fix.sh` / `pi-glass-check.sh` — doctrine phrases + `surface.json` (CLI canonical)  
3. `tinyfolder-ingest-demo.sh` / `tinyfolder-check.sh` — require `demo.json` sample + dry-run log  
4. `aos-poll-dashboard.sh` / `aos-poll-check.sh` — require `dashboard.json` (Arena not required)

### Wave 2 — Ritual / theater

1. `pantheon-ritual-demo.sh` — demo inventory; C.1 + daemon `dice_loop` fire deferred (no ghost `DICE_MASTER_*`)  
2. `discovery-theater-demo.sh` — session prep + Socratic LINK dry-run score (≠ living KPI)  
3. `hsp-emit-demo.sh` — motif schema + non-empty MIDI/WAV (not on GREEN overnight gate)

### Wave 3 — Arena lab

1. `arena-lab-demo.sh` — RAPL/€ demo chain; `daemon_jobs_touched=false`  
2. `ipw-route-demo.sh` — chat vs heavy_bench route matrix (must diverge; never auto-block distill)  
3. `forge-lab-demo.sh` — recommend.json schema + `blocks_distill=false` (human promote only)

### Wave 4 — Later

1. `aos-ce-smoke.sh` + [AOS_CUSTOMER_EDITION.md](./AOS_CUSTOMER_EDITION.md) — `golden-path.json` pin; never overwrites `~/.gzmo`  
2. `marketplace-check.sh` + [OKCP_MARKETPLACE.md](./OKCP_MARKETPLACE.md) — read-only `data/okcp/concept-bundle.fixture.json`  
3. `wiki-mind-check.sh` + [WIKI_OBSERVATORY_MIND.md](./WIKI_OBSERVATORY_MIND.md) — seeded search must hit  
4. `portable-core-inventory.sh` + [PORTABLE_GZMO_CORE_RFC.md](./PORTABLE_GZMO_CORE_RFC.md) — `lib.rs` seam table, `hold_rewrite`

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
