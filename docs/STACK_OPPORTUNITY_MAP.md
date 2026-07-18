# Stack opportunity map

**Status:** Working map (promoted from Cursor canvas, 2026-07-16; checked into repo 2026-07-18; compressed proof executed 2026-07-18)  
**Audience:** Operator / product direction for the maximilianwruhs-cyber stack  
**Scope:** Projects and products that can be created from lived GZMO + adjacent repos — not a generic idea dump.

## North star

The singular asset is a **living overnight memory metabolism** on one workstation. Everything below either protects that loop, scores models against it (Obolus Arena), or publishes its knowledge (OKForge). Rewrite labs and cron zoos are distractions until nights stay boring.

## Recommended next three

1. **Compressed felt-recall** (2 burst metabolism+recall cycles, ≥80% seeded hits) + **missed-run watchdog** — *done 2026-07-18* (`data-next/recall-proof.md`)
2. **Obolus Arena** nightburst spike — *done 2026-07-18* (`scripts/arena-night.sh` → `data-next/arena/latest.json`)
3. **Nightburst scoreboard** (sanitized metabolism + wiki + Arena) — *done 2026-07-18* (`scripts/nightburst-scoreboard.sh` → `data-next/arena/scoreboard.html`; OKForge `/observatory` stays agent-discovery)

Calendar soak when the machine is parked is an optional bonus, not a gate.

## Composition spine

```
chat / herdr
  → sessions
  → GZMO serve (distill · promote · embed · dream/spark)
  → vault + honeypot + Qdrant · status GREEN · optional GC / watchdog
  → Obolus Arena ← RAPL / Awattar / faithfulness / recall from real jobs
  → champion engine map · €/night · optional price-aware shift
  → OKForge / OKCP ← wiki concepts · Observatory · optional PR gates
  → public mind · marketplace later · AOS curl-install packaging
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
- **Ship shape:** Soft-fail in `gzmo status` / serve poll → `latest-watchdog.json` + YELLOW; never flips core GREEN math to RED.

#### Memory MCP appliance — `near` · differentiated · spike

Ship `gzmo memory mcp` as a drop-in MCP server other agents attach to for durable recall.

- **Builds on:** gzmo memory mcp · vault · Qdrant
- **Why special:** MCP memories are usually toys; yours is metabolized overnight.
- **Ship shape:** Docker/one-binary + sample Cursor/Claude Desktop config.

#### Session takeaway ritual — `near` · differentiated · product

End-of-session UX that forces durable takeaways into distill queue (CLI + herdr hook).

- **Builds on:** session-distill · synapse handoff · herdr
- **Why special:** Closes the human loop that makes metabolism work.
- **Ship shape:** `gzmo session close` + herdr pane exit hook.

#### Dream compaction service — `near` · commodity · product

Weekly GC that merges/compacts DREAMS.md and archives cold sessions without losing honeypot.

- **Builds on:** serve · garbage_collect idea · vault
- **Why special:** Necessary plumbing; not the brand.
- **Ship shape:** Soft-fail Sunday job on serve; never on GREEN gate day-one.

### Energy / Arena

#### Obolus Arena — `near` · singular · spike

Models compete for the right to run overnight jobs; scored on joules × faithfulness × recall.

- **Builds on:** Obolus evolve · verify-suite · faithfulness-judge · RAPL · serve jobs
- **Why special:** Ground truth is your wall meter + your living memory — not LMSYS vibes.
- **Ship shape:** `scripts/arena-night.sh` → `data-next/arena/latest.json` + sibling `champion-suggestion.toml` (human promote only). Nightburst spike 2026-07-18: quality=1.0 z≈0.94 (duration proxy until RAPL wired).

#### €/night dashboard — `near` · differentiated · product

Publish last-night cost: Awattar price × RAPL joules × job durations.

- **Builds on:** Obolus awattar · serve scheduler-runs · Observatory
- **Why special:** European energy reality as a first-class agent metric.
- **Ship shape:** One Observatory panel + `gzmo status` line.

#### Price-aware overnight shift — `later` · differentiated · research

Slide distill/dream windows into cheapest Awattar hours when backlog allows.

- **Builds on:** Obolus · dice/adaptive-tempo · serve cron
- **Why special:** Agents that respect the grid — rare and storyable.
- **Ship shape:** Soft window ±2h around configured cron; metabolism still wins.

#### Intelligence-per-Watt router — `later` · singular · research

Live route chat vs overnight vs cloud failover using Obolus z-scores + task class.

- **Builds on:** AOS · Obolus · rapl-route · GZMO engine map
- **Why special:** AOS thesis made real on the living instance.
- **Ship shape:** Policy file + serve/chat share router; cloud only on ceiling breach.

#### Obolus Forge mutations — `later` · differentiated · spike

Losers get mutated (prompt + size) until efficiency converges; winners pinned.

- **Builds on:** Obolus evolve · temp-bench · config-fuse
- **Why special:** Evolutionary pressure tied to hardware, not leaderboard chasing.
- **Ship shape:** Arena mode after static tournaments are trusted.

### OKForge / knowledge

#### Living wiki appliance — `near` · singular · exists

Productize OKForge + nightly concept push as “agent-writable Wikipedia for one mind”.

- **Builds on:** okforge · OKCP · wiki push · Observatory
- **Why special:** Git forge + agent REST + observatory is a rare bundle.
- **Ship shape:** Compose install + sample OKCP client + one demo repo.

#### Concept PR review bot — `near` · differentiated · product

Inbound OKCP writes open PRs; honeypot-gate + wiki-lint decide merge vs hold.

- **Builds on:** okforge · honeypot-gate · wiki-lint · faithfulness-judge
- **Why special:** Knowledge CI — missing from most agent wikis.
- **Ship shape:** Webhook → gate → merge; soft-fail alerts.

#### Observatory as public mind — `near` · differentiated · spike

Read-only public mind showing metabolism pulse + wiki commits (no secrets).

- **Builds on:** scheduler-runs · wiki meta · Arena JSON · OKForge `/observatory` (agent discovery)
- **Why special:** Makes the system demable without giving shell access.
- **Ship shape:** `scripts/nightburst-scoreboard.sh` → sanitized `data-next/arena/scoreboard.{json,html}`; open HTML locally. OKForge Observatory remains agent-discovery, not metabolism board.

#### OKCP memory marketplace — `later` · singular · product

Other agents pull/push concept bundles; GZMO is slow compiler, forge is the API.

- **Builds on:** OKCP · OKF bundles · wiki-lint · export-knowledge
- **Why special:** Turns private metabolism into a multi-agent knowledge bus.
- **Ship shape:** Auth scopes + PR/review for external writers.

### Organs / Little Tools Lab

#### Living tool zoo — `near` · singular · product

Dashboard of which of the 46 pieces fired overnight and what they changed in the vault.

- **Builds on:** little-tools-lab · organ-audit · serve runs · Observatory
- **Why special:** Catalogs are common; proof of overnight organ use is not.
- **Ship shape:** Per-job organ trace in scheduler-runs + UI.

#### Calibration pack SaaS-less — `near` · differentiated · exists

One command: temp/top-p/RAPL/verify → fused toml recommendations for any local stack.

- **Builds on:** bench-to-fuse · config-fuse · verify-suite
- **Why special:** Hardware-specific inference tuning as a productized recipe.
- **Ship shape:** Standalone `obolus calibrate` / LTL release tarball.

#### Serendipity engine — `near` · differentiated · spike

Productize spark-link: stale anchors + recent facts → verified surprising links.

- **Builds on:** spark-link · vault · dream
- **Why special:** Most RAG is similarity; this is deliberate serendipity.
- **Ship shape:** Weekly digest markdown + forge concepts.

#### Beat-gate open eval kit — `later` · differentiated · product

Open-source the fixture→meta→gate ladder as a framework for agent organ promotion.

- **Builds on:** LTL schemas · assemblies · CI templates
- **Why special:** Promotion science for agent tools, not just unit tests.
- **Ship shape:** `ltl` CLI publish + docs; GZMO as reference assembly.

#### Cognition pack — `later` · differentiated · spike

Distill → honeypot → spark-link → recall as a portable assembly for other agents.

- **Builds on:** cognition-smoke · session-distill · spark-link
- **Why special:** The memory loop without the full GZMO binary.
- **Ship shape:** Rust crates or container with JSON contract.

### Surfaces / UX

#### herdr + metabolism — `near` · singular · spike

Terminal agent mux that remembers across panes via memory MCP.

- **Builds on:** herdr · gzmo memory mcp · session close
- **Why special:** Muxers forget; yours metabolizes.
- **Ship shape:** herdr plugin/MCP attach + close ritual.

#### HSP metabolism sonification — `near` · singular · spike

Hear distill/dream/embed as MIDI motifs; idle vs metabolism night as music.

- **Builds on:** HSP · serve job events · RAPL
- **Why special:** Unforgettable demo; zero competitors take this seriously.
- **Ship shape:** Event bus → HSP MIDI map; 60s demo video.

#### AOS Intelligence Dashboard v2 — `later` · differentiated · product

Editor sidebar: live energy, Arena champion, last-night €, recall health.

- **Builds on:** AOS-Intelligence-Dashboard · Obolus · status JSON
- **Why special:** Brings the stack into the coding surface without another web app.
- **Ship shape:** VSCodium extension reading local status endpoints.

#### tinyFolder daemon product — `later` · commodity · spike

Filesystem-driven inbox → ingest → metabolism for non-CLI users.

- **Builds on:** tinyFolder · ingest · GZMO
- **Why special:** Accessible on-ramp; less unique alone.
- **Ship shape:** Drop folder → overnight facts.

#### Pi / operator split polish — `later` · commodity · research

Keep CLI canonical; Pi as optional glass for status + Arena + wiki.

- **Builds on:** PI_* docs · Observatory
- **Why special:** UX polish, not a new product category.
- **Ship shape:** Only after Arena + felt recall are solid.

### Platform / AOS

#### AOS Customer Edition spike — `later` · differentiated · product

One-curl Ubuntu appliance: Prime + GZMO serve + OKForge + Obolus recommend.

- **Builds on:** AOS · AOS-Customer-Edition · okforge · GZMO
- **Why special:** Sovereign stack install is the go-to-market for everything else.
- **Ship shape:** Golden path on clean VM; pin versions from living workstation.

#### Edge fleet with shared forge — `later` · differentiated · research

Multiple edge-nodes; one OKForge hub; local metabolism stays on-box.

- **Builds on:** edge-node · okforge · GZMO portable
- **Why special:** Privacy-preserving multi-node knowledge sync.
- **Ship shape:** After single-node AOS is boring.

#### Portable GZMO core — `later` · commodity · research

gzmo-core-clean / sovereign-agent as extractable brain without theatrical CT101 baggage.

- **Builds on:** gzmo-core-clean · sovereign-agent · ADR-0003
- **Why special:** Useful cleanup; risk of rewrite distraction.
- **Ship shape:** Only if living binary blocks packaging.

### Research brands

#### Faithfulness CI for agents — `near` · differentiated · product

CI action: claim set vs session/vault evidence → pass/fail for PRs and wiki concepts.

- **Builds on:** faithfulness-judge · evidence-locate · verify-suite
- **Why special:** Agent output CI is still rare in the wild.
- **Ship shape:** GitHub Action + OKForge merge gate.

#### ZPD tutor product — `later` · differentiated · spike

Pedagogy assembly as a personal tutor that writes skill patches from real work.

- **Builds on:** zpd-tutor · pedagogy-bench · skill-patch
- **Why special:** Tutor grounded in your vault, not generic courses.
- **Ship shape:** Weekly lab job; never on GREEN overnight gate.

#### Attractor / escape-loop brand — `later` · singular · research

Research brand: dynamical-systems view of agent loops (lorenz-map, escape-loop-bench).

- **Builds on:** lorenz-map · escape-loop-bench · AttractorBench · chaos lab
- **Why special:** Intellectual moat; not a day-job product.
- **Ship shape:** Papers + benches; keep chaos off production path.

#### Cognis dialect experiment — `later` · singular · research

Probabilistic typed prompts / confidence gates as a small language over GZMO tools.

- **Builds on:** Schreibtisch Cognis notes · plan-gate · verify-suite
- **Why special:** Could define a new interface layer — high risk, high novelty.
- **Ship shape:** Weekend prototype only; no production coupling.

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
| m2 | Memory MCP appliance | near | differentiated | spike |
| m3 | Session takeaway ritual | near | differentiated | product |
| m4 | Dream compaction service | near | commodity | product |
| e1 | Obolus Arena | near | singular | spike (nightburst done) |
| e2 | €/night dashboard | near | differentiated | product |
| f1 | Living wiki appliance | near | singular | exists |
| f3 | Concept PR review bot | near | differentiated | product |
| f4 | Observatory as public mind | near | differentiated | spike (scoreboard HTML) |
| o1 | Living tool zoo | near | singular | product |
| o3 | Calibration pack | near | differentiated | exists |
| o5 | Serendipity engine | near | differentiated | spike |
| s1 | herdr + metabolism | near | singular | spike |
| s2 | HSP metabolism sonification | near | singular | spike |
| r4 | Faithfulness CI | near | differentiated | product |
| e3 | Price-aware overnight shift | later | differentiated | research |
| e4 | Intelligence-per-Watt router | later | singular | research |
| e5 | Obolus Forge mutations | later | differentiated | spike |
| f2 | OKCP memory marketplace | later | singular | product |
| o2 | Beat-gate open eval kit | later | differentiated | product |
| o4 | Cognition pack | later | differentiated | spike |
| s3 | AOS Intelligence Dashboard v2 | later | differentiated | product |
| s4 | tinyFolder daemon | later | commodity | spike |
| s5 | Pi / operator split polish | later | commodity | research |
| p1 | AOS Customer Edition | later | differentiated | product |
| p2 | Edge fleet + shared forge | later | differentiated | research |
| p3 | Portable GZMO core | later | commodity | research |
| r1 | ZPD tutor | later | differentiated | spike |
| r2 | Attractor / escape-loop brand | later | singular | research |
| r3 | Cognis dialect experiment | later | singular | research |
