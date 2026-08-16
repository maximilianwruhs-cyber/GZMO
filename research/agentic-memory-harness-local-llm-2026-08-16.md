# Agentic memory, harnesses, and local-LLM performance

**Date:** 2026-08-16  
**Host:** telescope (`/home/mw/gzmo_full`). Living writer is CT101 — this note does not start `gzmo serve`.  
**Product:** honeypot + verify + promote on one airgapped box ([MACHINE.md](../MACHINE.md), [ADR-0004](../docs/ADR-0004-airgap-living-usp.md), [ADR-0007](../docs/ADR-0007-one-product-living.md)).  
**Active ship (unchanged):** [`felt-use-mass-growth`](opportunities/felt-use-mass-growth.md). This research does not flip that bet.  
**Sister notes:** [lineage-watch](lineage-watch/README.md) (memory harvest) · [this sitting’s kickoff](../docs/templates/AGENTIC_MEMORY_HARNESS_RESEARCH_PROMPT.md).

---

## Thesis

A frozen local model gets better from **three stacked planes**, not from overnight weight updates:

| Plane | Job | GZMO organ (already exists) | Field name |
|-------|-----|-----------------------------|------------|
| **1. Memory metabolism** | Decide what is true, useful, stale, or refuse | Distill → gate → vault/honeypot → dream/spark/immune | Agentic memory |
| **2. Harness** | Bound the loop the model runs in: tools, context prune, skills, attach | `agent_loop`, `context`, workflow `SKILL.md`, MCP, subagents, overnight daemon | Harness / context engineering |
| **3. Frozen-model lift** | Improve the *inputs* the model sees without touching weights | Recall Q-select, VM200 rerank, Arena/calibration **human pin**, workflow playbooks | ACE / DSPy-class context adaptation |

GZMO’s uniqueness is plane 1 **as overnight metabolism on hardware you own**. Planes 2 and 3 are how a weaker local LLM *uses* that Keep. Steal algorithms onto existing organs. Reject SaaS SKUs, multi-tenant HTTP, and overnight LoRA.

The 2026 field is loud about “memory is the bottleneck.” That is true for chatbots with a vector attachment. It is already GZMO’s product. The remaining work is **felt-use mass on CT101**, then one graft at a time: retrieve failures, protect skill context from prune, incremental playbook deltas under the gate.

---

## What this machine already is (do not re-found)

Verified 2026-08-16 against Proxmox + `gzmo-core` (not a brochure):

| Piece | Role |
|-------|------|
| **CT101** LXC `.202` | Living writer. `gzmo-daemon`, Litestream, Redis/Qdrant/Neo4j, vault `/opt/gzmo/data/vault.db`. Mutex claim `ct101`. |
| **VM200** `ollama-gpu` | Retrieval GPU (embed / rerank). Satellite, not a second brain. |
| **CT100** | Samba only. Not on the hot path. |
| **Telescope** | Prime LLM `:8000` (extract / verify / dream think) + Cursor. Must not `gzmo serve` while CT101 holds the mutex. |
| **Binary organs** | Session distill, honeypot gate, RRF + `utility_score` Q-select, felt-use (Glance Q=0; Outcome=8), region rewrite, `gate_event`, `failure_cases` **write**, bi-temporal `as_of`, immune plan, spark verify, workflow skills, MCP stdio, subagent isolation, context prune→archive→distill. |

Schema on living path: `PRAGMA user_version=10` (utility, `gate_event`, `failure_cases`, valid_from/to) — in-tree as of harvest PRs **#166 / #167**. Lineage-watch “queued grafts 1, 2, 4” are **code-complete**; remaining is living mass + retrieve-side use of failures + named night labels.

---

## Plane 1 — Agentic memory

**Claim to beat:** RAG dumps similar text. Agentic memory *mutates* a store: extract, link, supersede, forget, retrieve by **utility**.

### Field (steal the rule)

| Work | Rule | Non-steal |
|------|------|-----------|
| **MemRL** ([2601.03192](https://arxiv.org/abs/2601.03192)) | Frozen LLM; Intent–Experience–Utility; semantic filter then Q-select; Q from *outcomes* | Gym reward (ALFWorld / BigCodeBench) as living KPI |
| **Memento** ([2508.16153](https://arxiv.org/abs/2508.16153)) | Case bank including **failures**; retrieve-by-value | Cloud executor as product path |
| **A-Mem** ([2502.12110](https://arxiv.org/html/2502.12110), NeurIPS 2025) | Zettelkasten notes; link + evolve neighbors on write | LLM rewrite of the whole graph every ingest |
| **MemGPT / Letta** | Core vs archival; paging as an *action* | OS-SKU; agent-chosen page-in as the only retrieve |
| **HippoRAG** | Associative retrieve over a graph (PPR cousin) | Replace RRF+Neo4j with a new graph product |
| **Mem0 / MemoryOS / Zep** | Extract → consolidate → retrieve | Cloud-default graph SKU |
| **Auto-Dreamer** ([2605.20616](https://arxiv.org/html/2605.20616)) | Offline **region rewrite**: replacement set supersedes a region | Second-model RL consolidator gym |
| **Memory as Metabolism** ([2604.12034](https://arxiv.org/html/2604.12034v1)) | Named night: TRIAGE → CONSOLIDATE → AUDIT | Companion-wiki / Observatory glass |
| **SleepGate** ([2603.14517](https://arxiv.org/abs/2603.14517)) | Conflict-aware supersession; when-to-sleep | KV-cache surgery on a toy transformer |
| **SuperLocalMemory 4.0** | Confirm → mutate → blind-verify → quarantine; rollback; bi-temporal | Multi-tenant RBAC / provider-assisted default |
| **MemPalace** (2026 local-first leaderboard noise) | Verbatim local recall as a *bench* | Treat R@5 theater as soak GREEN |
| **Agent Memory characterization** ([2606.06448](https://arxiv.org/html/2606.06448v1)) | Write-path mutation cost is the real systems tax | Shopping their benchmark as USP |

### GZMO delta (honest)

| Already in-tree | Still open (not a new crate) |
|-----------------|------------------------------|
| Distill → verify → promote → honeypot | **Retrieve** `failure_cases` into recall (table is write-only except tests) |
| `utility_score` + Q-select after RRF ([#166](https://github.com/maximilianwruhs-cyber/GZMO/pull/166)) | Living census: rising `utility_positive` / `recall≥3` from **real** sessions |
| Outcome Q when a later takeaway cites a recalled entity (`reinforce_outcome_from_new_truths`) | Do not mint Q from glance/search |
| `maybe_region_rewrite` + `gate_event` (`promote` / `supersede` / `region_rewrite`) | Named night labels on scheduler (docs only) |
| `failure_cases` on `verify_fail` / `gate_refuse` | Immune **apply** on living still lab |
| `honeypot_as_of` bi-temporal | MCP attach HOLD (`gzmo-living` on the box) |
| Spark: stale × importance × cosine, then verify | — |

**A-Mem vs Spark:** they evolve *links on write*. We verify *links before promote*. Keep Pasteur. Do not let an LLM rewrite neighbor attributes without the gate.

**Letta vs overnight:** they page inside the chat loop. We metabolize **between** sessions (daemon). Both are memory. Ours is the Keep; paging inside `agent_loop` is already context prune + scratch archive + distill — do not import a second MemGPT runtime.

---

## Plane 2 — Harnesses

**Claim to beat:** a bigger context window. Harness engineering says: isolate, prune, disclose progressively, and put durable state **outside** the window.

### Field

| Work | Rule | Non-steal |
|------|------|-----------|
| **HumanLayer — harness engineering** ([skill-issue](https://www.humanlayer.dev/blog/skill-issue-harness-engineering-for-coding-agents)) | AGENTS.md / CLAUDE.md always on; skills progressive; MCP tool *descriptions* are prompt; **subagents isolate context**; protect skill text from compaction | Treat Cursor/Claude as the product |
| **Agent Skills** ([agentskills.io](https://agentskills.io/skill.md)) | `SKILL.md` name+description cheap; body on trigger; scripts/references on demand | Mass-install 60k marketplace skills |
| **Pocock / Superpowers** | Grill → TDD → handoff as *discipline*, not a second overnight writer | Two overlapping orchestrators |
| **madebywild/agent-harness** | One source of truth → emit Cursor/Claude/Codex configs | Multi-provider generator as a GZMO SKU |
| **SWE-agent / OpenHands / Aider** | Tool loop + tests as back-pressure | Clone their cloud sandbox as living |

### GZMO harness (code, not costume)

| Surface | File / path | What it already does |
|---------|-------------|----------------------|
| Agentic loop | `gzmo-core/src/agent_loop.rs` | Prompt → stream → tools → inject; max 40 iterations |
| Context prune | `gzmo-core/src/context.rs` | Keep system + recent + tool-chain integrity; archive overflow |
| Archive → distill | `scratch` + `DistillJob` | Pruned history becomes a distill source (memory, not a lost window) |
| Workflow skills | `gzmo-core/src/workflow_skills/` + `skills/workflows/*/SKILL.md` | Grill, TDD, review, diagnose, handoff, living-attach — progressive disclosure |
| Chaos skills | `gzmo-core/src/skills/` | Pantheon slash skills — **theater**, not Brain Feed |
| Tools | `gzmo-core/src/tools/` | Memory, fs, shell, jail, delegate, web (profile-gated) |
| Subagents | `gzmo-core/src/subagent.rs` | Isolated scratch + budget; findings flow back condensed |
| MCP attach | `gzmo mcp-serve` stdio | Living vault tools; **no public HTTP SKU** ([MCP_LOCAL_ATTACH.md](../docs/MCP_LOCAL_ATTACH.md)) |
| Overnight | `gzmo-daemon` on CT101 | Distill / promote / embed / dream / spark — the writer |
| Telescope harness | `AGENTS.md`, Cursor skills, herdr takeaway | Dev loop. Side-effect takeaway feeds the Keep. Not a second brain. |

### Steal next (harness, still not a new organ)

1. **Protect workflow skill bodies from prune** — HumanLayer: skill instructions dying in compaction silently degrades the local model. Flag injected `SKILL.md` as pinned in `context.rs` (system-adjacent), not as ordinary user turns.
2. **Failure-case retrieve in the loop** — when the agent repeats a verify_fail pattern, inject Memento cases *after* Q-select, not as a dump.
3. **Do not** grow a LangGraph / multi-provider harness product. Cursor already is the telescope IDE. GZMO’s harness is the Rust loop + MCP + daemon.

Telescope skills (grill, tdd, handoff) make **you** a better operator. They do not nourish the vault unless a session close `--takeaway` fires. That is the Brain Feed contract.

---

## Plane 3 — Making the local LLM better without training it

**Parked horizon:** [`local-intel-32gb-128k`](opportunities/local-intel-32gb-128k.md) — do not chase 256k-on-32GB as an active ship.

**What actually lifts a frozen Prime / local chat model today:**

| Lever | Mechanism | GZMO landing | Status |
|-------|-----------|--------------|--------|
| **Better retrieve** | Two-phase: semantic then utility; rerank | RRF + `apply_utility_boost` + VM200 | In-tree; needs living mass |
| **Better evidence** | Verify-on-merged; refuse; quarantine | Honeypot gate | In-tree |
| **Better instructions** | Evolving playbook, incremental deltas (not rewrite) | Workflow `SKILL.md`, `SOUL.md`, living toml | Partial — human pin only for engine swap |
| **Better routing** | Right model for extract vs chat vs embed | Prime `:8000` think; VM200 retrieve; Arena **suggest** | P1 nutrient; no auto swap |
| **Better loop** | Subagents, prune, tool jail | `subagent` + `context` + path jail | In-tree |
| **Serving ops** | Quantization, speculative decode, vLLM | Private ops (`vllm-blackwell-backend`) | Ops, not USP |
| **Weight updates** | LoRA / PEFT overnight | — | **Forbidden** as living path |

### ACE — Agentic Context Engineering ([2510.04618](https://arxiv.org/abs/2510.04618), ICLR 2026)

Stanford / SambaNova / Berkeley: context as an **evolving playbook**. Generator → Reflector → Curator. Incremental `ADD` / `UPDATE` / `REMOVE`. Deterministic merge (no LLM full rewrite). Avoids **brevity bias** and **context collapse**. Claims +10.6% on agents vs ICL/GEPA/Dynamic Cheatsheet; smaller open model matching a production agent on AppWorld.

| Steal | Non-steal |
|-------|-----------|
| Incremental deltas, not monolithic prompt rewrite | Unsupervised overnight rewrite of living `SOUL.md` / engine toml |
| Reflector separate from Curator (critique ≠ mutate) | Three extra local models as required runtime |
| Grow-and-refine with helpful/harmful counters | Gym rollouts as soak GREEN |
| Natural execution feedback (test fail, gate refuse) | Labeled AppWorld as the Keep’s eval |

**Map onto GZMO:** Curator deltas belong in (a) workflow skill bodies after a **human pin**, or (b) honeypot supersession (already a gated mutate). Distill already *extracts*; ACE’s warning is: **do not compact playbooks into slogans**. Dream compaction that drops provenance is context collapse. Region rewrite with origin kept is ACE-legal. Observatory summaries are ACE-illegal (brevity bias as product).

DSPy / MIPRO / GEPA are the same family (optimize prompts/programs). Steal the *eval-then-delta* idea. Do not add a Python optimizer crate to `gzmo-core`.

---

## Three-plane map (one page)

```text
TELESCOPE                         LIVING (CT101)                    RETRIEVAL (VM200)
─────────                         ──────────────                    ────────────────
Cursor + AGENTS.md                gzmo-daemon (one writer)          embed / rerank
Prime :8000 (think)               vault + honeypot                  Qdrant satellite
takeaway on close ──────────────► distill → gate → promote
workflow SKILL.md (dev)           MCP stdio attach ◄── Cursor/Pi
                                  dream / spark / immune
                                  ACE deltas only via gate/pin
```

If a paper improves **retrieve / forget / supersede / consolidate / verify / time** — plane 1.  
If it improves **loop, tools, prune, skills, subagents** — plane 2.  
If it improves **playbook / pin / serving** without weights — plane 3.  
If it is a company shape — park.

---

## Harvest table (this sitting)

Status: **steal** = take the rule onto an existing organ. **in-tree** = do not re-ship. **open** = remaining loop. **park** = SKU / theater / horizon.

| ID | Steal | Source | Organ | Status |
|----|-------|--------|-------|--------|
| M1 | Two-phase retrieve + Q | MemRL | `rrf-recall` / `utility_score` | **in-tree** (#166) |
| M2 | Outcome Q from later cite | MemRL | `felt_use::Outcome` | **in-tree** (#167) |
| M3 | Store failures | Memento | `failure_cases` insert | **in-tree write**; **open retrieve** |
| M4 | Region rewrite supersession | Auto-Dreamer | `maybe_region_rewrite` | **in-tree** |
| M5 | Typed gate event | SleepGate / SLM | `gate_event` | **in-tree** |
| M6 | Bi-temporal as-of | SuperLocalMemory | `honeypot_as_of` | **in-tree** |
| M7 | Named night TRIAGE/CONSOLIDATE/AUDIT | Memory as Metabolism | scheduler docs | **open** (labels only) |
| M8 | Zettelkasten neighbor evolve | A-Mem | spark-link | **park as write-mutate**; keep verify-then-promote |
| M9 | Graph associative retrieve | HippoRAG | Neo4j + RRF | **cousin in-tree**; no new graph SKU |
| H1 | Pin skill text against prune | HumanLayer | `context.rs` | **open** |
| H2 | Progressive SKILL.md | Agent Skills | `workflow_skills` | **in-tree** |
| H3 | Subagent context isolation | HumanLayer / SWE-agent | `subagent.rs` | **in-tree** |
| H4 | MCP descriptions as prompt | MCP spec | `gzmo mcp-serve` | **in-tree**; no public HTTP |
| L1 | Incremental playbook deltas | ACE | skills / SOUL / toml | **open** — human pin + gate; no auto rewrite |
| L2 | Champion model pin | Arena | living toml | **P1 nutrient**; suggest only |
| L3 | 128k–256k local strong model | horizon | Prime | **parked** (`local-intel-32gb-128k`) |
| L4 | Overnight LoRA | — | — | **park forever** (living path) |
| P1 | Observatory / OKForge glass | — | wiki emit | **theater**; not Brain Feed |
| P2 | Mem0/Zep/Letta SKU clone | — | — | **park** |
| P3 | `eml-core` as organ | — | — | **park** (R&D calculator) |

---

## Ranked next (after this note)

`felt-use-mass-growth` stays **active**. These are **candidates** only. One loop per PR. Living census first (`bash scripts/felt-use-depth.sh`).

| Rank | Loop | Why | Filter |
|------|------|-----|--------|
| 0 | **Soak nights 2–3** + felt-use mass on CT101 | Done-when of the active bet | No memory gym |
| 1 | **Retrieve `failure_cases`** in recall (bounded, Q-gated) | M3 is write-only; Memento’s actual steal | No dump into every prompt |
| 2 | **Pin workflow skill bodies** in `context.rs` | Local models forget the harness first | Do not pin pantheon theater skills |
| 3 | **Named night labels** | Cheap vocabulary; zero new crates | Docs/scheduler only |
| 4 | **ACE curator deltas** on one workflow skill, human-pinned | Frozen-model lift without LoRA | Reflector may use Prime; Curator merge is deterministic; living SOUL still gated |
| 5 | Immune apply on living | Forget as signal | Still lab until soak |

Do not start 1–5 until rank 0 is moving or blocked with `INCONCLUSIVE` (SSH), never a synthetic GREEN.

---

## Eval honesty

| Allowed | Forbidden |
|---------|-----------|
| CT101 felt-use depth, keep-quality soak (3 honest nights, ≥18h) | LoCoMo / LongMemEval as living GREEN |
| `organism-memory-bench-spike.sh` as **borrow-eval** | MemPalace R@5 screenshots |
| Gate refuse + test fail as ACE execution signal | LLM-as-judge of the vault |
| Fail-closed: no SSH → RED / INCONCLUSIVE | Hybrid recall without embeddings |

---

## Explicit park list (this research)

- Observatory glass, OKForge wiki push, HSP, pantheon, AOS CE, stitcher-as-OS  
- Public forge / herdr / okforge (private R&D mirrors — do not publicize)  
- Second overnight writer; `gzmo serve` on the telescope  
- Overnight LoRA / PEFT; energy routing as USP  
- Multi-tenant memory HTTP; provider-assisted default  
- Re-shipping #166/#167 as new work  
- Ecosystem tour of forty memory startups as a sitting

---

## Brutal test

If Prime’s next extract/verify/dream, or Cursor’s next attach to `gzmo-living`, would see **the same honeypot and the same pinned skill**, you wrote a manifesto. Stop.

The Keep is well-designed. This note exists so the next sitting **grafts one loop**, not so we collect another field.

---

## Sources fetched this sitting

- MemRL 2601.03192 · A-Mem 2502.12110 · Agent Memory systems 2606.06448  
- ACE 2510.04618 (ICLR 2026) · Auto-Dreamer 2605.20616 · Memory as Metabolism 2604.12034 · SleepGate 2603.14517 · SuperLocalMemory 2608.08253  
- [agentskills.io/skill.md](https://agentskills.io/skill.md) · [HumanLayer harness engineering](https://www.humanlayer.dev/blog/skill-issue-harness-engineering-for-coding-agents)  
- In-repo: `agent_loop.rs`, `context.rs`, `felt_use.rs`, `workflow_skills/`, `vault.rs` (v10), lineage-watch 2026-08-15, AGENT_SKILLS_LANDSCAPE_2026-07  
- Live PVE: CT101 / VM200 / CT100 (2026-08-16)

Refresh: on or before **2026-09-16**, or when a paper in planes 1–3 lands a measurable steal. File a lineage-watch `sota-YYYY-MM-DD.md`; do not duplicate this atlas.
