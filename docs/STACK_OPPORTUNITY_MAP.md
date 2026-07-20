# Stack opportunity map

**Status:** Unpark active (2026-07-19). A+C GREEN; satellites sequenced in waves.  
**Audience:** Operator / product direction for the maximilianwruhs-cyber stack  
**Scope:** Projects and products that can be created from lived GZMO + adjacent repos — not a generic idea dump.  
**Doctrine:** [SPINE_FOCUS.md](SPINE_FOCUS.md) · [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md)

## North star

The singular asset is a **living overnight memory metabolism** with a **product memory MCP** attach surface. Living production vault owner is **CT101** (`/opt/gzmo/`); workstation `data-next/` is lab scratch only. Arena / energy / surfaces are satellites — useful, not the brand.

## Keep / Unpark / Later (spine)

Operator lock: co-primary **A** (product MCP) + **C** (living Redis/Qdrant/Neo4j appliance). See [SPINE_FOCUS.md](SPINE_FOCUS.md), [research/CT101_STACK_FUTURE_2026-07.md](research/CT101_STACK_FUTURE_2026-07.md).

| Lane | What | Action |
|------|------|--------|
| **Keep (A)** | Product Memory MCP · stranger install · release freshness · faithfulness fixture | `product-readiness-gate.sh` |
| **Keep (C)** | Living metabolism · Redis/Qdrant/Neo4j appliance compose · takeaway/watchdog · living faithfulness | `living-readiness-gate.sh` + **in-repo compose pin** |
| **Unpark queue** | Arena/€/RAPL · HSP · AOS poll · herdr/Pi polish · OKCP marketplace · IpW/Forge · pantheon ritual · tinyFolder | Sequenced waves — [UNPARK_ROADMAP.md](UNPARK_ROADMAP.md) |
| **Later (Wave 4)** | AOS CE golden path (Prime+OKForge on top of C) · public marketplace · multi-node forge · portable-core RFC | After Waves 1–3 stay demable |
| **Never-as-brain** | Cognis · escape-loop · ZPD on GREEN overnight | Lab/research only |

## Recommended next three

1. **Keep A+C GREEN** — `bash scripts/production-readiness-gate.sh`
2. **Scaffold archaeology exec picks** — build-spec: [tool-dev-amp-forget-verify-token-2026-07-20.md](../research/tool-dev-amp-forget-verify-token-2026-07-20.md) (`forget-lint` → `verify-gates` → `token-economy`; lab vault only)
3. **Wave gate** — `bash scripts/unpark-wave-check.sh` · tag `v*` when tip ahead of release ≥5

Unpark open after A+C GREEN (2026-07-19). Pantheon: [PANTHEON_THEATER_PACKAGING_PARK.md](PANTHEON_THEATER_PACKAGING_PARK.md). Doctrine: [SPINE_FOCUS.md](SPINE_FOCUS.md).

## Composition spine

```
KEEP pillars
  chat / session close --takeaway
    → distill · promote · embed · dream/spark  (CT101 living; data-next lab)
    → vault + honeypot + Qdrant · GREEN · watchdog
    → gzmo memory mcp  (product appliance)

UNPARK queue (sequenced — see UNPARK_ROADMAP.md)
  W1: herdr · Pi glass · tinyFolder · AOS poll
  W2: pantheon ritual · discovery theater · HSP emit
  W3: Arena · €/night · RAPL · IpW · Forge (outside living daemon)
  W4: AOS CE · marketplace · wiki mind · portable-core RFC
```
## Stack atlas

| Layer | What you already have |
|-------|------------------------|
| Lived runtime | GZMO serve · memory MCP · status GREEN gate · data-next vault |
| Knowledge plane | OKForge · OKCP · Observatory · gzmo-next-memory wiki push |
| Energy / routing | Obolus · RAPL · temp-bench · awattar · AOS hot-swap story |
| Organs (46) | little-tools-lab · assemblies · honeypot-gate · spark-link · verify-suite |
| Surfaces | gzmo chat/CLI · herdr · HSP · AOS dashboard · tinyFolder |
| Research adjacency | Cognis notes · agentic-loop synthesis · pedagogy / chaos lab |

## Legend

| Field | Values |
|-------|--------|
| **Horizon** | `now` — protect/prove · `near` — spike→ship · `later` — after proof |
| **Uniqueness** | `singular` — hard to copy · `differentiated` — strong niche · `commodity` — useful plumbing |
| **Status** | `exists` · `spike` · `product` · `research` |

---

## Opportunities

### Memory / metabolism

#### Felt-recall product — `now` · singular · exists

Package overnight metabolism as the product: chat today, search tomorrow, prove facts stuck.

- **Builds on:** GZMO serve · honeypot · rrf-recall · status
- **Why special:** Almost nobody ships a living overnight memory compiler you can feel.
- **Ship shape:** 2 burst metabolism+recall cycles (≥80% seeded hits); calendar soak optional when machine is parked. Proof log: `data-next/recall-proof.md` (2026-07-18: 9/9 HIT).

#### Missed-run watchdog — `now` · commodity · exists

Lightweight autonomy: alert if distill/dream missed >26h using `latest-*.json`.

- **Builds on:** serve scheduler-runs · status · `GZMO_METABOLISM_STALE_SECS` override for burst tests
- **Why special:** Boring reliability that protects the singular product.
- **Ship shape:** Soft-fail in serve poll + `gzmo metabolism watchdog` → `latest-watchdog.json` + YELLOW; never flips core GREEN math to RED. Lab: `scripts/watchdog-lab.sh` → `data-next/watchdog-lab/`.

#### Memory MCP appliance — `near` · differentiated · exists

Ship `gzmo memory mcp` as a drop-in MCP server other agents attach to for durable recall.

- **Builds on:** `gzmo init` · `scripts/install-gzmo.sh` · `scripts/verify-product-mcp.sh` · PRODUCT_MCP.md
- **Why special:** MCP memories are usually toys; yours is metabolized overnight.
- **Ship shape:** Product path `~/.gzmo` + `gzmo memory mcp` / `mcp-serve`; verify with `./scripts/verify-product-mcp.sh`. Living `data-next` stays operator-only.

#### Session takeaway ritual — `near` · differentiated · exists

End-of-session UX that forces durable takeaways into distill queue (CLI + herdr hook).

- **Builds on:** session-distill · scratch distill queue · SessionManager
- **Why special:** Closes the human loop that makes metabolism work.
- **Ship shape:** `gzmo session close [id] --takeaway "…" [--now]` appends `[TAKEAWAY]` turns and enqueues distill (or runs immediately with `--now`). Lab proof: `scripts/takeaway-ritual-lab.sh` → `data-next/takeaway-ritual/` (enqueue only; no workstation `--now`). Herdr pane exit hook later.

#### Dream compaction service — `near` · commodity · exists

Weekly GC that merges/compacts DREAMS.md and archives cold sessions without losing honeypot.

- **Builds on:** `dreams_md::compact_dreams_md` · serve soft-fail · sessions-archive
- **Why special:** Necessary plumbing; not the brand.
- **Ship shape:** `gzmo dream compact [--max-chars N] [--archive-sessions-days 30] [--dry-run]`; lab: `scripts/dream-compact-lab.sh` → `data-next/dream-compact/`; serve soft-fail Sunday 03:00 UTC; never on GREEN gate.

### Energy / Arena

#### Obolus Arena — `near` · singular · spike

Models compete for the right to run overnight jobs; scored on joules × faithfulness × recall.

- **Builds on:** Obolus evolve · verify-suite · faithfulness-judge · RAPL · serve jobs
- **Why special:** Ground truth is your wall meter + your living memory — not LMSYS vibes.
- **Ship shape:** `scripts/arena-night.sh` → `data-next/arena/latest.json` + sibling `champion-suggestion.toml` (human promote only). Nightburst spike 2026-07-18: quality=1.0 z≈0.94. RAPL probe (`scripts/rapl-probe.sh`) shows `energy_uj` root-only here → Arena stays `energy_source=estimate` until caps/ACL.

#### €/night dashboard — `near` · differentiated · spike (aggregate)

Publish last-night cost: Awattar price × RAPL joules × job durations.

- **Builds on:** Arena history · `scripts/euro-night-aggregate.sh` · scoreboard · Awattar
- **Why special:** European energy reality as a first-class agent metric.
- **Ship shape:** Each Arena run archives to `data-next/arena/history/`; aggregate → `euro-night.json` (Arena € sum + metabolism duration×W estimate); scoreboard pills show €/night.

#### Price-aware overnight shift — `later` · differentiated · spike (soft)

Slide distill/dream windows into cheapest Awattar hours when backlog allows.

- **Builds on:** price-window-suggest · `scripts/price-shift-soft.sh` · serve soft advice
- **Why special:** Agents that respect the grid — rare and storyable.
- **Ship shape:** Suggestions + `latest-price-shift.json`; serve logs “would shift”; `GZMO_PRICE_SHIFT=1` soft-delays distill/dream until suggested UTC (cron not overwritten).

#### Intelligence-per-Watt router — `later` · singular · spike (policy)

Live route chat vs overnight vs cloud failover using Obolus z-scores + task class.

- **Builds on:** `config/ipw-router.policy.toml` · `scripts/ipw-route.sh` · Arena/AOS signals
- **Why special:** AOS thesis made real on the living instance.
- **Ship shape:** Policy + `data-next/ipw-router/latest.json` route advice; cloud only if `GZMO_IPW_CLOUD=1`; metabolism never blocked.

#### Obolus Forge mutations — `later` · differentiated · spike (sibling)

Losers get mutated (prompt + size) until efficiency converges; winners pinned.

- **Builds on:** Arena history · `scripts/obolus-forge-mutate.sh` · champion-suggestion pattern
- **Why special:** Evolutionary pressure tied to hardware, not leaderboard chasing.
- **Ship shape:** Pin winners / propose mutations under `data-next/arena/forge/` (JSON + TOML sibling); human promote only.

### OKForge / knowledge

#### Living wiki appliance — `near` · singular · exists

Productize OKForge + nightly concept push as “agent-writable Wikipedia for one mind”.

- **Builds on:** okforge · OKCP · wiki push · Observatory
- **Why special:** Git forge + agent REST + observatory is a rare bundle.
- **Ship shape:** Compose install + sample OKCP client + one demo repo.

#### Concept PR review bot — `near` · differentiated · spike (webhook stub)

Inbound OKCP writes open PRs; honeypot-gate + wiki-lint decide merge vs hold.

- **Builds on:** concept-review-gate · `wiki-push-gated.sh` · `concept-gate-webhook.sh`
- **Why special:** Knowledge CI — missing from most agent wikis.
- **Ship shape:** Gate PASS/HOLD; serve soft-hold on HOLD; `bash scripts/concept-gate-webhook.sh [--serve :8766]` returns merge advice JSON; full OKForge PR merge later.

#### Observatory as public mind — `near` · differentiated · spike (enriched)

Read-only public mind showing metabolism pulse + wiki commits (no secrets).

- **Builds on:** nightburst-scoreboard · concept-gate · HSP motif · €/night · OKForge `/observatory`
- **Why special:** Makes the system demable without giving shell access.
- **Ship shape:** Sanitized HTML/JSON with metabolism, Arena, gate, faithfulness, HSP, €/night; Observatory stays agent-discovery.

#### OKCP memory marketplace — `later` · singular · spike (bundle)

Other agents pull/push concept bundles; GZMO is slow compiler, forge is the API.

- **Builds on:** `scripts/okcp-marketplace.sh` · concept-gate · wiki-push · OKCP scopes
- **Why special:** Turns private metabolism into a multi-agent knowledge bus.
- **Ship shape:** `data-next/okcp-marketplace/` export bundle + `--intent write` gate; no public auth server yet.

### Organs / Little Tools Lab

#### Living tool zoo — `near` · singular · spike

Dashboard of which of the 46 pieces fired overnight and what they changed in the vault.

- **Builds on:** scheduler-runs · organ-trace · nightburst scoreboard
- **Why special:** Catalogs are common; proof of overnight organ use is not.
- **Ship shape:** `scripts/organ-trace.sh` → `data-next/organ-trace/latest.{json,md}`; scoreboard “Living tool zoo” section.

#### Calibration pack SaaS-less — `near` · differentiated · exists

One command: temp/top-p/RAPL/verify → fused toml recommendations for any local stack.

- **Builds on:** bench-to-fuse · config-fuse · verify-suite
- **Why special:** Hardware-specific inference tuning as a productized recipe.
- **Ship shape:** Standalone `obolus calibrate` / LTL release tarball.

#### Serendipity engine — `near` · differentiated · spike

Productize spark-link: stale anchors + recent facts → verified surprising links.

- **Builds on:** spark reports · DREAMS spark sections · wiki push
- **Why special:** Most RAG is similarity; this is deliberate serendipity.
- **Ship shape:** `scripts/serendipity-digest.sh` → `data-next/serendipity/digest-YYYY-MM-DD.md` (+ `latest.md`); promote manually.

#### Beat-gate open eval kit — `later` · differentiated · spike (kit)

Open-source the fixture→meta→gate ladder as a framework for agent organ promotion.

- **Builds on:** `scripts/beat-gate-kit.sh` · little-tools-lab `beat-gate.sh` · beat-meta schema
- **Why special:** Promotion science for agent tools, not just unit tests.
- **Ship shape:** Fixture loops → `data-next/beat-gate/{contract,latest}.json`; human S0→S3 only.

#### Cognition pack — `later` · differentiated · spike (contract)

Distill → honeypot → spark-link → recall as a portable assembly for other agents.

- **Builds on:** `scripts/cognition-pack.sh` · scheduler-runs · memory search smoke
- **Why special:** The memory loop without the full GZMO binary.
- **Ship shape:** `data-next/cognition-pack/{contract,latest}.json` stage map + living status; `--smoke` recall probe.

### Surfaces / UX

#### herdr + metabolism — `near` · singular · spike (plugin)

Terminal agent mux that remembers across panes via memory MCP.

- **Builds on:** `integrations/herdr-gzmo-metabolism` · `gzmo memory mcp` · `gzmo session close`
- **Why special:** Muxers forget; yours metabolizes.
- **Ship shape:** `bash scripts/herdr-metabolism-link.sh` → actions `ensure-mcp` / `session-close` + popup close-ritual; `pane.closed` soft-logs missed ritual (no auto-distill).

#### HSP metabolism sonification — `near` · singular · spike (motif)

Hear distill/dream/embed as MIDI motifs; idle vs metabolism night as music.

- **Builds on:** `scripts/hsp-metabolism-sonify.sh` · scheduler-runs · organ-trace · Arena · HSP ping optional
- **Why special:** Unforgettable demo; zero competitors take this seriously.
- **Ship shape:** Artifact → MIDI/WAV motif under `data-next/hsp-metabolism/`; `--play` uses aplay/hsp ping; live HSP event-bus later.

#### AOS Intelligence Dashboard v2 — `later` · differentiated · spike (poll)

Editor sidebar: live energy, Arena champion, last-night €, recall health.

- **Builds on:** `scripts/aos-gzmo-poll.sh` · aos-status-feed · AOS-Intelligence-Dashboard GzmoStatusPoller
- **Why special:** Brings the stack into the coding surface without another web app.
- **Ship shape:** File/HTTP poll of `data-next/aos-status/latest.json` (or `:8765/telemetry.json`); status bar shows GZMO z/gate when AOS gateway is down.

#### tinyFolder daemon product — `later` · commodity · spike (drop)

Filesystem-driven inbox → ingest → metabolism for non-CLI users.

- **Builds on:** `scripts/tinyfolder-drop.sh` · `data-next/inbox` · distill-queue advice
- **Why special:** Accessible on-ramp; less unique alone.
- **Ship shape:** Drop/demo → pending markdown in inbox + queue JSONL; watcher stays off by default.

#### Pi / operator split polish — `later` · commodity · spike (glass)

Keep CLI canonical; Pi as optional glass for status + Arena + wiki.

- **Builds on:** `scripts/pi-operator-glass.sh` · aos-status · Arena · concept-gate · PI_* docs
- **Why special:** Coding-surface glass without another web app; CLI stays canonical.
- **Ship shape:** `data-next/pi-glass/{latest.json,latest.md}`; deepen after Arena + felt recall stay solid.

### Platform / AOS

#### AOS Customer Edition spike — `later` · differentiated · spike (pin)

One-curl Ubuntu appliance: Prime + GZMO serve + OKForge + Obolus recommend.

- **Builds on:** `scripts/aos-ce-pin.sh` · AOS-Customer-Edition · Obolus · GZMO
- **Why special:** Sovereign stack install is the go-to-market for everything else.
- **Ship shape:** `data-next/aos-ce/` golden-path pin from living SHAs; human promotes into CE bootstrap.

#### Edge fleet with shared forge — `later` · differentiated · spike (sketch)

Multiple edge-nodes; one OKForge hub; local metabolism stays on-box.

- **Builds on:** `scripts/edge-fleet-sketch.sh` · edge-node · aos-ce-pin · wiki-push
- **Why special:** Privacy-preserving multi-node knowledge sync.
- **Ship shape:** Topology sketch → `data-next/edge-fleet/`; hold until AOS CE single-node is boring.

#### Portable GZMO core — `later` · commodity · spike (inventory)

gzmo-core-clean / sovereign-agent as extractable brain without theatrical CT101 baggage.

- **Builds on:** `scripts/portable-core-inventory.sh` · gzmo-core-clean · ADR-0003
- **Why special:** Useful cleanup; risk of rewrite distraction.
- **Ship shape:** Inventory → `data-next/portable-core/`; default advice `hold_rewrite`.

### Research brands

#### Faithfulness CI for agents — `near` · differentiated · exists

CI action: claim set vs session/vault evidence → pass/fail for PRs and wiki concepts.

- **Builds on:** `gzmo memory search` · `scripts/fixtures/faithfulness-claims.json`
- **Why special:** Agent output CI is still rare in the wild.
- **Ship shape:** `scripts/faithfulness-ci.sh` (vault mode locally); CI job `faithfulness-fixture` offline; report `data-next/faithfulness/latest.json`.

#### ZPD tutor product — `later` · differentiated · spike (lab)

Pedagogy assembly as a personal tutor that writes skill patches from real work.

- **Builds on:** `scripts/zpd-tutor-lab.sh` · zpd-tutor dry-run · vault topic hint
- **Why special:** Tutor grounded in your vault, not generic courses.
- **Ship shape:** Soft-fail lab → `data-next/zpd-tutor/`; never on GREEN overnight gate.

#### Attractor / escape-loop brand — `later` · singular · spike (kit)

Research brand: dynamical-systems view of agent loops (lorenz-map, escape-loop-bench).

- **Builds on:** `scripts/escape-loop-kit.sh` · escape-loop-bench dry-run · AttractorBench
- **Why special:** Intellectual moat; not a day-job product.
- **Ship shape:** Soft kit → `data-next/escape-loop/`; never on GREEN; chaos/lorenz stay off serve.

#### Cognis dialect experiment — `later` · singular · spike (stub)

Probabilistic typed prompts / confidence gates as a small language over GZMO tools.

- **Builds on:** `scripts/cognis-dialect-stub.sh` · plan-gate fixtures · confidence forms
- **Why special:** Could define a new interface layer — high risk, high novelty.
- **Ship shape:** Soft stub → `data-next/cognis-dialect/`; never production brain / never on GREEN.

---

## What not to spawn yet

- Parallel cron daemon beside `gzmo serve`
- Pedagogy / cabinet on the GREEN overnight gate
- Cognis as production brain
- Neo4j live reconcile overnight
- Prometheus / Grafana as a prerequisite
- Full `AUTONOMOUS_CRON_IMPLEMENTATION.md` 12-job zoo
- Big-bang gzmo-core rewrite while nights are still proving

Those become interesting only after Arena + felt recall make the stack demable to a stranger in five minutes.

## Related docs

- [ADR-0003-one-instance-metabolism.md](ADR-0003-one-instance-metabolism.md) — living-instance doctrine
- [GZMO_NEXT_RUNBOOK.md](GZMO_NEXT_RUNBOOK.md) — serve + soft-fail satellites
- [OKFORGE_PRODUCTION.md](OKFORGE_PRODUCTION.md) — wiki / Observatory ops
- [CEILING_ROADMAP.md](CEILING_ROADMAP.md) — long-term ceiling
- [AUTONOMOUS_CRON_IMPLEMENTATION.md](AUTONOMOUS_CRON_IMPLEMENTATION.md) — backlog menu (superseded core path)

## Index (quick scan)

| ID | Idea | Horizon | Uniqueness | Status |
|----|------|---------|------------|--------|
| m1 | Felt-recall product | now | singular | exists |
| r5 | Missed-run watchdog | now | commodity | exists |
| m2 | Memory MCP appliance | near | differentiated | exists |
| m3 | Session takeaway ritual | near | differentiated | exists |
| m4 | Dream compaction service | near | commodity | exists |
| e1 | Obolus Arena | near | singular | spike (nightburst done) |
| e2 | €/night dashboard | near | differentiated | spike (aggregate) |
| f1 | Living wiki appliance | near | singular | exists |
| f3 | Concept PR review bot | near | differentiated | spike (webhook stub) |
| f4 | Observatory as public mind | near | differentiated | spike (enriched) |
| o1 | Living tool zoo | near | singular | spike (organ-trace) |
| o3 | Calibration pack | near | differentiated | exists |
| o5 | Serendipity engine | near | differentiated | spike (digest script) |
| s1 | herdr + metabolism | near | singular | spike (plugin) |
| s2 | HSP metabolism sonification | near | singular | spike (motif) |
| r4 | Faithfulness CI | near | differentiated | exists |
| e3 | Price-aware overnight shift | later | differentiated | spike (soft) |
| e4 | Intelligence-per-Watt router | later | singular | spike (policy) |
| e5 | Obolus Forge mutations | later | differentiated | spike (sibling) |
| f2 | OKCP memory marketplace | later | singular | spike (bundle) |
| o2 | Beat-gate open eval kit | later | differentiated | spike (kit) |
| o4 | Cognition pack | later | differentiated | spike (contract) |
| s3 | AOS Intelligence Dashboard v2 | later | differentiated | spike (poll) |
| s4 | tinyFolder daemon | later | commodity | spike (drop) |
| s5 | Pi / operator split polish | later | commodity | spike (glass) |
| p1 | AOS Customer Edition | later | differentiated | spike (pin) |
| p2 | Edge fleet + shared forge | later | differentiated | spike (sketch) |
| p3 | Portable GZMO core | later | commodity | spike (inventory) |
| r1 | ZPD tutor | later | differentiated | spike (lab) |
| r2 | Attractor / escape-loop brand | later | singular | spike (kit) |
| r3 | Cognis dialect experiment | later | singular | spike (stub) |
