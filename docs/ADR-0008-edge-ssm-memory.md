# ADR-0008 — Edge SSM backbone (PRECOG) + structured memory backend (MemoryLake)

**Status:** Proposed (2026-08-22)
**Related:** [ADR-0003](./ADR-0003-one-instance-metabolism.md), [ADR-0004](./ADR-0004-airgap-living-usp.md), [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md), [ADR-0007](./ADR-0007-one-product-living.md), [CT101_BOUNDARY.md](./CT101_BOUNDARY.md), [SOTA_FIXES_BACKLOG.md](./SOTA_FIXES_BACKLOG.md)
**Decision date / owner:** Max, after spike results (spikes in `spikes/pre-cog/` and `spikes/memoryarena-baseline/`)

---

## Context

The 2026-08-22 SOTA scan (`data-next/research-sota/latest.md`, run 20260822T041632Z) flagged two papers with `benefit=True`. Both concern memory on edge hardware — one at the backbone level, one at the backend level. This ADR proposes hard-gated investigations for both, without touching runtime code.

### Paper A — PRECOG (arXiv:2608.02560, 2026-08-03)

**Title:** *Structured Memory for Edge Language Models: Persistent Context and Corpus Retrieval via O(1) SSM State Injection*

**Authors:** Gopal, Pirbadian, Carlson, Lewis, Tapson

**Abstract (verified fetch 2026-08-22):**

> Retrieval-augmented generation (RAG) imposes a prefill cost proportional to retrieved context length, and -- with Transformer backbones -- a KV-cache that grows with each generated token. State-Space Models (SSMs) avoid the second cost by construction; we eliminate the first, collapsing prefill from $O(L_{context})$ to $O(1)$ per query. We introduce PRECOG (Pre-Computed Context Injection), a retrieval mechanism that exploits a property unique to SSMs: the fixed-size, position-agnostic recurrent hidden state is a complete summary of everything the model has read. PRECOG pre-encodes document corpora offline as SSM hidden states and injects the best-matching state directly at query time, bypassing in-context re-ingestion entirely. The same state-injection mechanism enables SMC (Structured Memory Consolidation): a hierarchical persistent memory with cognitive-domain clustering, an adjustable fidelity-vs-storage dial, and $O(1)$ session initialization, which consolidates short-term episodic states into long-term semantic memory and fuses both with retrieved corpus states at query time. We demonstrate the system on TENNs-LLM, a 1.2B-parameter gated-SSM language model with a 192 KB hidden state. PRECOG matches in-context RAG answer quality, reducing prefill latency from $\sim$27 s to $<$6 ms on edge hardware -- a $\sim$4500$\times$ speedup that crosses the threshold from unusable to interactive. The mechanism is architecturally impossible for Transformer KV-caches, which are position-entangled and grow linearly with context length.

**SOTA report excerpt (TRL 5, benefit=True):**

> *Integration-hebel:* Replace the local LLM backbone in the 'extract lane' or 'Brain Feed' with an SSM-based model to enable O(1) state injection for persistent context, drastically reducing energy (RAPL) and memory footprint on edge hardware.
>
> *Why:* GZMO operates on constrained edge hardware (CT101); SSMs offer a critical efficiency lever for local inference by eliminating the quadratic cost of attention and large KV caches, aligning with AOS energy routing goals.

### Paper B — MemoryLake (arXiv:2608.13883, 2026-08-14)

**Title:** *MemoryLake on MemoryArena: A Matched Study of Agent Memory Backends*

**Authors:** Zhan, Zhou, Li, Huang, Wang

**Abstract (verified fetch 2026-08-22):**

> Most agent-memory benchmarks test post-hoc recall, whereas MemoryArena evaluates whether memory supports interdependent, multi-session task completion. We compare MemoryLake, a structured multi-track memory backend, with Mem0, text-embedding-3-small vector RAG, and a long-context control across all five MemoryArena domains. The systems share the same agent framework, requested gpt-5-mini model alias, task samples, and scoring code; the memory integration is the intentionally changed component. Because each backend bundles write, retrieval, consolidation, budgeting, and prompt-assembly choices, the study is a matched system-level comparison, not a representation-only ablation or a cost-matched experiment. On the shared evaluation sets, MemoryLake has the highest observed success rate (SR) in mathematics (9/40), physics (12/20), and progressive retrieval (4/20). Every system has zero SR in travel planning, and web shopping yields a single bundle-level success (long context, 1/150); MemoryLake ranks third on both the travel soft process score and shopping step match. Following MemoryArena's suite-level convention, a post-hoc equal-weight average over the five SRs is 20.5% for MemoryLake versus 13.6% for the best comparator. These are point estimates: sample sizes are modest, confidence intervals overlap, and we do not report paired significance tests. A separate MemoryLake-only run over all 221 progressive queries yields a failure-counted SR of 26.7% (59/221) and is not a baseline comparison. The results support a workload-dependent view of memory backends and an observed lead among the four evaluated systems on the shared sets; they do not establish benchmark-wide state of the art or a causal advantage of representation structure.

**SOTA report excerpt (TRL 4, benefit=True):**

> *Integration-hebel:* Adopt MemoryLake's structured multi-track backend for the 'living vault' on CT101 to ensure overnight metabolism consolidates interdependent task states rather than just isolated facts.
>
> *Why:* GZMO requires memory that supports complex, multi-step sovereign operations; standard RAG recall is insufficient for maintaining coherent long-term agent state in an airgapped environment.

---

## Decision (proposed — no runtime code changes)

This ADR proposes two gated investigations. Neither changes any runtime code, scripts, systemd, cron, CT101, or daemon behavior. Both require explicit GO from Max after spike results.

### Option A — PRECOG: SSM backbone in the extract lane / Brain Feed

**What it would mean:** Replace the current local LLM backbone (llama.cpp `qwen3.6-35b-mtp` / `qwen3.8-27b` on `:8000`) in the extract/distill lane with TENNs-LLM (1.2B gated-SSM, 192 KB hidden state). PRECOG pre-encodes document corpora offline as SSM hidden states and injects the best-matching state at query time — collapsing prefill from ~27 s to <6 ms on edge hardware. The SMC (Structured Memory Consolidation) component provides hierarchical persistent memory with O(1) session initialization.

This would:
- Eliminate Transformer KV-cache memory overhead (which grows linearly with each generated token).
- Collapse prefill cost from O(L_context) to O(1) per query.
- Align with AOS energy routing goals (RAPL reduction on edge hardware / CT101).
- Potentially enable persistent context without in-context re-ingestion — useful for Brain Feed nutrient processing and the honeypot extract lane.

**Hard gates before GO:**

1. **TENNs-LLM weights license + availability offline.** The model is on HuggingFace (`BrainChip-AI/tenns-llm-1b`) with `license:cc-by-nc-4.0` (non-commercial). GZMO is a sovereign product — a non-commercial license may block production use. Must verify: does the license permit sovereign personal use? Can weights be downloaded and cached for airgap operation?

2. **llama.cpp (or alternative) SSM inference support.** llama.cpp supports Mamba/Mamba2 and RWKV6/RWKV7 architectures natively, but TENNs-LLM uses a `custom_code` transformers architecture (`model_type: tenns_llm`) that is NOT in llama.cpp's architecture registry. Must verify: can TENNs-LLM be converted to GGUF, or does it require a custom inference path (transformers + custom_code)? Can PRECOG's state-injection mechanism work through any existing local inference server?

3. **Quality parity with Qwen3.6-35B-MTP on GZMO's actual extract/distill prompts.** TENNs-LLM is 1.2B parameters — 29× smaller than Qwen3.6-35B. Must run A/B on GZMO's real `librarian_extract` / `SessionDistill` / `CuratedVault` prompts, not generic QA. Quality parity means: distill output passes the existing quality gate, not just "looks plausible."

4. **Energy (RAPL) comparison.** Measure joules per extract/distill cycle on the actual edge target. PRECOG's 4500× prefill speedup is on edge hardware — verify it holds on CT101-class hardware or workstation GPU, and measure actual RAPL delta vs current backbone.

### Option B — MemoryLake: structured multi-track backend for the living vault

**What it would mean:** Adopt MemoryLake's structured multi-track memory backend pattern for the living vault on CT101. MemoryLake bundles write, retrieval, consolidation, budgeting, and prompt-assembly choices — it is a system-level backend, not a representation-only change. The current system (Qdrant vector RAG + SQLite vault + Neo4j KG, single-writer overnight metabolism) stores isolated promoted facts. MemoryLake's multi-track design could consolidate interdependent task states across sessions — closer to the MemoryArena evaluation paradigm (multi-session interdependent tasks) than post-hoc recall.

This would:
- Move the vault from isolated-fact storage toward interdependent-task-state consolidation.
- Require evaluation against the current system's baseline (deliverable 3).
- Need a migration path that preserves ADR-0003 (single-writer) + ADR-0004 (airgap) invariants.

**Hard gates before GO:**

1. **MemoryLake code availability + license.** The MemoryArena benchmark code is on GitHub (`memorylake-ai/memorylake-memoryarena-benchmark`, Apache-2.0). MemoryLake itself is on PyPI (`memorylake`, Powerdrill). Must verify: is the MemoryLake backend library itself open-source and Apache-2.0, or is it a hosted SaaS? Can it run fully airgapped?

2. **Baseline from deliverable 3 showing current system's weaknesses on multi-session interdependent tasks.** The `spikes/memoryarena-baseline/` harness must demonstrate that the current Qdrant+SQLite system fails on multi-session interdependent queries (e.g., "I decided X last week, then Y; what is the current state?"). A low score is a valid result — it justifies Option B.

3. **Migration path that keeps ADR-0003 single-writer + airgap intact.** Any MemoryLake adoption must be behind the living-host mutex, not a second writer. Must not require public net (airgap honesty). Must not break the overnight metabolism pipeline (distill → verify → promote → vault → honeypot).

---

## NO-GO criteria (explicit)

Any of the following kills the respective option:

### Option A NO-GO if:
- TENNs-LLM `cc-by-nc-4.0` license prohibits production use → NO-GO (license block).
- TENNs-LLM cannot run locally without a hosted/cloud dependency → NO-GO (airgap violation, ADR-0004).
- No local inference path (llama.cpp or transformers) can serve TENNs-LLM with state injection → NO-GO (inference block).
- Quality parity fails on GZMO's real extract/distill prompts (measured A/B, not claimed) → NO-GO (quality block).
- Energy delta is not measurable or negative → HOLD (not necessarily NO-GO, but blocks GO).

### Option B NO-GO if:
- MemoryLake backend is a hosted SaaS, not self-hostable → NO-GO (airgap violation).
- Baseline (deliverable 3) shows the current system handles multi-session interdependent tasks adequately → NO-GO (no demonstrated need).
- Migration path requires breaking single-writer or airgap invariants → NO-GO (doctrine conflict, ADR-0003/0004).

---

## Spike results (2026-08-22, updated after challenge)

Spikes in `spikes/pre-cog/`, `spikes/pre-cog-mamba/`, `spikes/memoryarena-baseline/`. **No runtime code changed.**

### Option A — PRECOG / Mamba (updated 2026-08-22)

- **TENNs-LLM (paper's own model):** `spikes/pre-cog/availability.json` — weights `gated`, license `cc-by-nc-4.0`, SSM inference `partial` (custom_code not in llama.cpp registry). **Gate 1 (license) and Gate 2 (inference) remain BLOCKED for TENNs-LLM.**
- **7B Mamba-class alternative (Mamba-Codestral-7B Jamba, llama.cpp b9018):** `spikes/pre-cog-mamba/` — a protocol-correct decisive control **reproduces the O(1) state-injection mechanism** (88.1× TTFT reduction, 17.5K-token corpus, airtight by C2/C3 + cold-control). The earlier "restore is a content no-op" NO-GO was a **test error** (no-prefix query → `n_past=0` → re-process path), not a mechanism failure. See `spikes/pre-cog-mamba/VERDICT.md`.
- **Quality gate: HOLD.** 4/5 parity; 1/5 (lite-SKU) factual error under injection — the fixed-size SSM state is lossy for long-context detail that full-prefill retains in KV. **Gate 3 (quality parity) is not yet a clean pass** and must be re-measured on GZMO's real, shorter extract/distill prompts before Option A moves to GO.
- **Net:** mechanism GO, TENNs-LLM license/inference still blocked, quality gate HOLD. Option A stays **HOLD** (not NO-GO) — re-open on real-prompt quality A/B and/or a hybrid SSM+KV-tail that recovers the lost detail.

### Option B — MemoryLake (updated 2026-08-22)

- **Baseline:** `spikes/memoryarena-baseline/` — keyword-only baseline 3/12; **real embed path** (router :8081 + Qdrant) **8/12 top-1, 9/12 top-5** on 12 MemoryArena-style questions. The current system is stronger than a naive baseline on these — **Gate 2 (demonstrated weakness) not yet met**; a harder multi-session interdependent set is needed before Option B is justified.
- **License/airgap (Gate 1):** still to verify (MemoryLake backend on PyPI/Powerdrill — self-hostable? airgap?).
- **Net:** Option B stays **HOLD**.

---

## Consequences (if Proposed → Accepted after spike GO)

- Option A would be a promote-by-loop candidate (ADR-0005): beat-gate the extract loop with the SSM backbone, then narrow promote after operator ack.
- Option B would be a vault-schema migration: atomar step with rollback, behind living-host mutex, not a silent overnight cutover.
- Neither changes any runtime code until spikes pass and Max says GO.

## Non-goals

- Blind backbone swap without quality A/B on real GZMO prompts.
- MemoryLake adoption without a demonstrated baseline weakness.
- Breaking single-writer, airgap, or no-public-MCP-SKU invariants.
- Changing any runtime code, scripts, systemd, cron, CT101, or daemon behavior in this ADR.

---

*Proposed: GZMO operator surface (OpenClaw) · 2026-08-22 · no runtime code changed · spikes in `spikes/pre-cog/`, `spikes/pre-cog-mamba/`, `spikes/memoryarena-baseline/` · 7B Mamba spike corrected 2026-08-22 after operator challenge (v1 NO-GO was a test error — mechanism reproduces; quality gate HOLD)*
