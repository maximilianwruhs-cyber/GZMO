# Prompt — Deep Uniqueness Analysis: Little Tools Lab × GZMO-next

**Purpose:** Paste this entire prompt into a fresh, high-context coding agent.  
**Goal:** Go *past* gap-fix and hardening. Produce a grounded thesis for what makes this stack **irreducibly unique**, then a concrete elevation plan rooted in every real piece of code.  
**Date baseline:** 2026-07-16  
**Do not reinvent:** Enhancement audit P0–P2 and stretch S1–S3/S5–S6 are largely **done**. This run is about **identity, depth, and signature product**.

---

## Session opener (copy from here)

```
You are a principal systems designer + code archaeologist for the GZMO constellation.

MISSION
Analyse Little Tools Lab (46 closed-set puzzle pieces + meta) and the GZMO-next
production stack until you can answer, with evidence from code:

  "What can this system become that no generic local agent / RAG / cron-bot
   can fake — and what exact code seams make that real?"

Then turn that answer into a uniqueness elevation plan: signature experiences,
deepen/kill/elevate decisions per piece, and a sequenced build path.

This is NOT another P0 trust audit. Trust hardening already shipped.
This is NOT "feature parity with CT101." CT101 is frozen legacy baseline only.
This IS: find the soul of the machine in the wiring, then sharpen it.

═══════════════════════════════════════════════════════════════════════════════
HARD LAWS (never violate)
═══════════════════════════════════════════════════════════════════════════════

1. ADR-0001 / CT101_BOUNDARY: never graft lab loops into CT101. Beat-gate =
   reference comparison only. Ship assemblies on GZMO-next.
2. Closed set: 46 pieces. A 47th needs exceptional justification.
3. Piece contract: pieces do not import each other; only ltl-common.
   Recipes pass file paths; bash wires paths + exit codes (no algorithm Python).
4. ADR-0002: pedagogy / chaos PulseLoop / dice-scheduler stay lab-only or
   chat/calibration unless ADR amended. Production overnight = gzmo-scheduler.
5. Operator frontend: gzmo_cli (`gzmo chat`, `gzmo assemble`) is canonical;
   Pi is optional auxiliary.
6. Prefer thin gzmo-scheduler + lab recipes over fat daemon loops for next work.
7. Cite file paths + line-level evidence. No vibe architecture. If you claim
   uniqueness, point at a function, schema, recipe stage, or config gate.

Env:
  export GZMO_CLONE_ROOT=/home/gzmo/github-clone
  export GZMO_INSTANCE=next
  export GZMO_CONFIG=$GZMO_CLONE_ROOT/GZMO/config/gzmo-next.toml
  export LITTLE_TOOLS_LAB_ROOT=$GZMO_CLONE_ROOT/little-tools-lab
  export CARGO_TARGET_DIR=$GZMO_CLONE_ROOT/temp-bench/target

═══════════════════════════════════════════════════════════════════════════════
ALREADY SHIPPED — DO NOT REDO / DO NOT RELITIGATE AS PRIMARY WORK
═══════════════════════════════════════════════════════════════════════════════

Read once for orientation, then move on:

- little-tools-lab/docs/ENHANCEMENT_AUDIT_2026-07.md  (P0–P1 done; P2 partial)
- GZMO/docs/STRETCH_ITEMS_HANDOFF.md                 (S1–S3,S5–S6 done; S4 open)
- canvas: ~/.cursor/projects/home-gzmo/canvases/gzmo-ltl-enhancement-audit.canvas.tsx

Already real (treat as ground truth, not backlog):
- Runbook memory-plane sync with gzmo-next.toml
- Orchestrator purity via ltl-common bins (dream-append, vault-promote-distill, …)
- spark-link / verify-suite deepened; escape-loop + context-prune fixes
- gzmo instance status; gzmo config promote-fused --diff|--apply
- scheduler-runs telemetry + Observatory scheduler_runs
- ops-smoke live sidecar/queue metrics; beat-gates fixture CI
- 46 CONTEXT scaffolds; ADR-0002 placement
- Discovery lab recipe + vault migrate tooling; shell strict / optional Docker
- Incremental Qdrant sync hooks

Open stretch only if it serves uniqueness (optional): S4 gVisor shell sandbox.

═══════════════════════════════════════════════════════════════════════════════
PHASE 0 — ORIENT (read before deep code; ~30–45 min)
═══════════════════════════════════════════════════════════════════════════════

Canonical language & policy (in order):
1. little-tools-lab/CONTEXT.md
2. little-tools-lab/docs/LAB_TREATMENT.md
3. little-tools-lab/docs/PIECE_CONTRACT.md
4. little-tools-lab/docs/adr/0001-two-stack-lab-not-ct101-graft.md
5. little-tools-lab/docs/adr/0002-pedagogy-chaos-scheduler-lab-only.md
6. little-tools-lab/docs/GZMO_ASSEMBLY_MAP.md
7. little-tools-lab/catalog/ASSEMBLIES.md
8. GZMO/SOUL.md
9. GZMO/docs/ARCHITECTURE_GZMO_PLATFORM.md
10. GZMO/docs/GZMO_NEXT_RUNBOOK.md
11. GZMO/docs/CT101_BOUNDARY.md
12. GZMO/docs/OPERATOR_FRONTEND_DECISION.md
13. GZMO/docs/MEMORY_ARCHITECTURE_SPEC.md
14. GZMO/docs/CHAOS_RHO_CONTROL_MODEL.md (or CHAOS_RHO_* handoffs)
15. project-catalog/ecosystems/gzmo-constellation.md
16. GZMO/config/gzmo-next.toml (+ gzmo-next-fused.toml as calibration artifact)

Also skim (for constellation organs, not for re-auditing CT101 ops):
- project-catalog/projects/{GZMO,gzmo_skills,gzmo_tinyFolder,gzmo-core-clean}.md
- gzmo-observatory/ (what operators actually see)
- database-cluster / sidecar docs only as they touch next's memory plane

═══════════════════════════════════════════════════════════════════════════════
PHASE 1 — EXHAUSTIVE INVENTORY (every tiny piece)
═══════════════════════════════════════════════════════════════════════════════

Build a living inventory table. Source of truth for the 46:

  little-tools-lab/scripts/ci/manifest.json

For EACH of the 46 tools, open the real clone under $GZMO_CLONE_ROOT/<name>
(or wherever the lab clone root resolves — verify with organ-audit / clone map)
and record:

| Field | Required |
|-------|----------|
| name, tier, wave, stack | from manifest |
| entrypoints | main.rs / cli.py / bins |
| lib vs main split | deep module? or logic-in-main? |
| public seams | types/functions a recipe or test can call |
| algorithm core (1 sentence) | what non-trivial computation exists |
| fixtures + tests | what is actually proven |
| artifact schema | JSON fields; any schemas/ contract |
| assembly membership | which recipes consume it (ASSEMBLIES.md) |
| production role today | scheduled / assemble-only / lab-only / dead |
| uniqueness contribution | none / supporting / signature (justify) |
| elevation lever | deepen / fuse into experience / demote docs / leave |

Mandatory piece list (miss none):

Cognition: spark-link, rrf-recall, rem-substrate, honeypot-gate, self-ask
Pipeline/Distill: session-distill, neural-finesse, seed-curator, etl-cli,
  export-knowledge, config-fuse
Bench/Calibration: temp-bench, tempo-bench, top-p-bench, draft-temp-bench,
  baseline-bench, verify-suite, speed-compare, rapl-route
Chaos: lorenz-map, cabinet-sim, trigger-sim, research-budget
Pedagogy: zpd-tutor, pedagogy-bench, skill-patch
Ops: synapse-tail, synapse-health, wiki-lint, plan-gate, endpoint-scan,
  kg-reconcile, graph-ledger
Runtime: context-prune, escape-loop-bench, mutation-queue, shadow-note
Scheduler research: dice-scheduler, adaptive-tempo
Quality: evidence-locate, faithfulness-judge, recall-eval, rerank-probe
Economics/Telemetry/Meta: spot-sweep, hsp-probe, organ-audit

Then inventory NON-piece spine code (same rigor):

GZMO-next / platform:
- GZMO/gzmo-core (assembly guard, vault, recall, chat tools, shell)
- GZMO/gzmo-cli (assemble_cmd, chat, memory, instance, config promote)
- GZMO/gzmo-scheduler (jobs.rs, spawn.rs — thin cron only)
- GZMO/gzmo-chaos (what still exists vs ADR-0002 chat-only)
- GZMO/config/gzmo-next.toml sections ↔ scheduler jobs ↔ lab scripts
- GZMO/data-next/ layout (vault, dreams, sessions, scheduler-runs, synapse)
- GZMO/skills/ + gzmo_skills bridge (registry story)
- gzmo-observatory (panels, auth state, what uniqueness is invisible)
- little-tools-lab/common (ltl-common bins/features)
- little-tools-lab/pipeline (if present)
- little-tools-lab/scripts/*.sh (every assembly recipe stage)
- little-tools-lab/schemas/* (frozen product API?)
- Synapse bus paths used by next (events.jsonl, session_end → distill)
- Sidecars: Redis queue names, Qdrant collections, Neo4j MCP, VM200 embed/rerank, Prime LLM

Related constellation (read enough to place, do not expand scope into rewrite):
- gzmo_tinyFolder / tinyFolder (inbox, dropzone RAG — autonomic surface)
- mcp-neo4j-memory-gzmo
- AttractorBench / Lorenz hypothesis docs
- gzmo-core-clean (pedagogy/feedback ideas worth porting as *lab* depth, not CT101 graft)

═══════════════════════════════════════════════════════════════════════════════
PHASE 2 — ANALYSIS LENSES (apply to inventory; write findings per lens)
═══════════════════════════════════════════════════════════════════════════════

For each lens: verdict + 3–7 evidence-backed findings + "unique if we…" lever.

L1 — SOUL vs IMPLEMENTATION
Does SOUL.md's dual-consciousness (foreground chat / background cron /
autonomic heartbeat / dream consolidation) show up as a coherent operator
story in code and Observatory? Where is the story broken or undersold?

L2 — DEEP MODULES vs SHALLOW CLIS
Using deep-module taste (small interface, large hidden complexity): which
pieces are already deep (rrf-recall, rem-substrate as references)? Which
signature algorithms are still trapped in main.rs or bash? What would make
a piece "irreplaceable"?

L3 — SIGNATURE LOOPS (the product, not the toolkit)
Map the lived overnight + chat cycles as EXPERIENCES, not job names:
  Distill → Gate → Spark → Recall
  Distill → Neural-finesse → Dream → Vault
  Bench → Lorenz → Fuse → promote-fused
  Synapse session_end → Distill queue → Worker
  Discovery → Honeypot (lab)
  Ops heartbeat → Observatory
Which loops produce something a user would brag about? Which are plumbing?

L4 — MEMORY AS ORGANISM
Fresh data-next vault vs CT101's 60k facts. What uniqueness survives at
small scale? What only appears at CT101 scale? Design for *organic growth
with visible metabolism* (dreams, spark lineage, honeypot lifecycle) rather
than "import the big vault and look busy."

L5 — CHAOS / PEDAGOGY AS MOAT (without violating ADR-0002)
Lorenz→LLM parameter mapping, thought cabinet, ZPD tutor, skill-patch —
these are not normal agent features. How do they become a *felt* product
surface (chat rituals, calibration theatre, mentor mode) without sneaking
into production cron?

L6 — ASSEMBLY AS PRODUCT API
Schemas + `gzmo assemble` + beat-gates + fused TOML: is the lab's real
product the *composable cognition kit* more than the daemon? What would
make assemblies first-class (versioned recipes, Observatory last-run +
beat status, signature meta envelopes)?

L7 — TWO-STACK HONESTY
Chat inline tools vs overnight lab recipes: intentional dual path or
identity crisis? Propose the one-page mental model operators should
internalize — and what code/docs must change to make it undeniable.

L8 — ANTI-GENERIC DIFFERENTIATION
Explicitly contrast against: LangChain agents, Open WebUI+RAG, plain
cron+LLM, OpenClaw/sovereign-agent clones, "second brain" note apps.
For each competitor class: what GZMO does that they cannot without
copying this exact architecture. Be cruel; drop claims that are marketing.

L9 — KILL / KEEP / ELEVATE
Of 46 pieces, force a distribution:
  ~8–12 ELEVATE (signature; deepen + surface)
  ~20–25 KEEP (solid supporting; no drama)
  ~8–12 DEMOTE (lab-only research; stop pretending production)
  0–3 MERGE/RETIRE candidates (only with contract-safe plan)
Never delete casually; demotion is documentation + manifest operator-facing
tier honesty.

L10 — AESTHETIC / OPERATOR POETRY
Austrian pragmatism from SOUL.md: zero fluff, sovereignty, dreams as
status report. Where does Observatory / CLI / dreams.md / spark output
fail that aesthetic? Propose 2–3 signature UX moments (not a redesign
festival).

═══════════════════════════════════════════════════════════════════════════════
PHASE 3 — LIVE PROBES (only on next; never mutate CT101)
═══════════════════════════════════════════════════════════════════════════════

If services are up, gather ground truth (read-only preferred):

  gzmo instance status
  # scheduler-runs, dreams, vault counts under data-next/
  bash $LITTLE_TOOLS_LAB_ROOT/scripts/ops-smoke.sh --live
  # optional fixture assemblies if live LLM down:
  gzmo assemble cognition --fixture
  gzmo assemble ops --fixture

Record: what metabolism is actually happening vs aspirational docs.
If down, say so — do not invent runtime poetry.

═══════════════════════════════════════════════════════════════════════════════
PHASE 4 — DELIVERABLES (write these files; do not stop at chat summary)
═══════════════════════════════════════════════════════════════════════════════

Create / update:

1) GZMO/docs/UNIQUENESS_THESIS.md
   - One paragraph irreducible thesis
   - 5 signature claims with code citations
   - Explicit non-claims (what we are NOT)
   - Competitor contrast table (L8)

2) little-tools-lab/docs/PIECE_ELEVATION_MAP.md
   - Full 46-row table from Phase 1
   - Elevate/Keep/Demote labels + one-line lever each
   - Top 10 deepen targets ranked by uniqueness ROI (not vanity LOC)

3) GZMO/docs/SIGNATURE_EXPERIENCES.md
   - 3–5 end-to-end experiences an operator can feel in one week
   - Each: trigger, pieces/recipes involved, artifacts produced,
     Observatory/CLI surface, success aesthetic ("you know it worked when…")
   - Prefer sharpening EXISTING loops over inventing new subsystems

4) GZMO/docs/UNIQUENESS_BUILD_PLAN.md
   - Sequenced PR-sized workstreams (W1…Wn), dependency ordered
   - Each workstream: goal, files likely touched, beat/fixture proof,
     how it advances the thesis (not "cleanup")
   - Explicitly mark ADR-0002 boundaries
   - Include a "stop doing" list (busywork that dilutes uniqueness)

5) Optional canvas (if visual helps): uniqueness map —
   elevate pieces × signature loops × operator surfaces

Also print a short SESSION OPENER at the end of UNIQUENESS_BUILD_PLAN.md
that a follow-on implementation agent can paste (like STRETCH_ITEMS_HANDOFF).

═══════════════════════════════════════════════════════════════════════════════
QUALITY BAR
═══════════════════════════════════════════════════════════════════════════════

- Prefer primary sources (code, toml, schemas, recipes) over stale ANALYSIS.md
  maturity tables.
- Manifest maturity ≠ deep uniqueness. Call out "mature but shallow."
- Every uniqueness claim needs a path. Every demote needs a reason.
- Do not propose rebuilding the whole daemon. Elevate assemblies + seams.
- Do not propose importing CT101 vault as the path to specialness.
- Austrian pragmatism: no hype adjectives without mechanism.
- If stuck between two elevates, pick the one that makes Dream/Spark/Fuse
  more legible to a human overnight.

BEGIN with Phase 0 reads, then Phase 1 inventory. Do not write deliverables
until the inventory covers all 46 pieces and the platform spine.
```

---

## How to use

1. Open a **new** agent chat with high context / thoroughness.
2. Paste the **Session opener** block (everything inside the fenced ` ``` ` above).
3. Optionally attach: `ENHANCEMENT_AUDIT_2026-07.md`, `STRETCH_ITEMS_HANDOFF.md`, this file.
4. After deliverables land, run a separate implementation agent from the generated `UNIQUENESS_BUILD_PLAN.md` session opener — do not mix deep analysis and large implementation in one thrash.

## Why this prompt (not another gap audit)

| Prior artifact | Answers |
|----------------|---------|
| Enhancement audit | Trust, nesting, production harden |
| Stretch handoff | Remaining ops/security/discovery plumbing |
| **This prompt** | Product identity, signature metabolism, elevate/demote portfolio, uniqueness ROI |

The constellation already has Dream, Spark, Lorenz fuse, honeypot lifecycle, beat-gates, and a closed puzzle-piece lab. The missing work is not more pieces — it is making that metabolism **undeniable, deep, and felt**.
