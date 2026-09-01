# 07 — Opportunity-finder claims assessment (AI- & Agentic-Opportunities 2026)

**Research date:** 2026-09-01
**Scope:** External verification of ten claim families from a user German “AI- & Agentic-Opportunities 2026” synthesis, mapped to the **approved** GZMO North Star (`docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`, commit `385b4d9`, tag `north-star-design-approved-2026-08-31`) and research notes `01`–`06`.
**Non-goals:** Edit the North Star, ADRs, or prior research; implementation; purchase; multi-node product topology.

**Label key**

| Field | Values |
| --- | --- |
| **Evidence** | `Supported` · `Conditional` · `Unverified` · `Misleading` |
| **North-Star map** | `Already covered` · `North-Star successor candidate` · `Strategic only` · `Reject/out of scope` |

**Method:** Primary papers, official project docs/repos, vendor/engineering first-party notes, or original forecasting orgs only. Workload-specific numbers are bound to stated method/hardware. Missing primary support → `Unverified`, not false.

---

## Executive verdict

Most of the synthesis mixes **three different kinds of statements**: (1) durable engineering principles already assumed by GZMO, (2) **workload- and hardware-specific** speedups that must never become product law, and (3) **market/productivity forecasts and coined control terms** with no primary anchor found in this pass.

**Keep (as engineering lanes inside existing authority):** decode-phase memory-bandwidth pressure; optional native/llama.cpp serving; **lossless** speculative decoding as a qualified accelerator (not a new cognitive authority); NUMA/affinity as Forge/x86 qualification knobs; episodic→semantic consolidation already embodied as metabolism; sleep-time / overnight compute as the existing Dream/Spark/metabolism window; context economy (semantic cache, DyCP-class pruning) as **Redis-class ephemeral** acceleration under one-writer PostgreSQL authority; MCP as the already-chosen agent surface.

**Do not promote into the North Star:** multi-node EXO / heterogeneous VRAM pooling as product topology; any claim that multi-box “100B+” is required for Living; fixed “+30% C++”, “+53% NUMA”, “cache up to 100%”, “adoption <5%→40%”, “SDD +50–126%”, or “junior jobs −13–30%” as floors or roadmap targets; terms **Grain / Flexion / Anchory** (no primary definition found); procedural-memory *product* redesign that bypasses evidence→verify→promote.

**Successor-design concerns that *are* evidence-backed and missing as explicit North-Star *lanes* (not baseline edits):** (P0) resource-loop circuit breakers for agent tool thrash / unbounded metabolism; (P1) optional draft-model speculative decoding under role qualification; (P1) context-economy projections (semantic response cache + dialogue pruning) with faithfulness revalidation; (P1) procedural *skill/recipe* memory as **derived, signed, non-authoritative** artifacts; (P2) NUMA/affinity qualification only on multi-socket Forge nodes.

**Constitution preserved:** one physical node; runtime airgap; role qualification; constitutional authority (no self-issued promotion); no-overwrite of approved baseline.

---

## Claim matrix

| # | Claim family | Evidence | North-Star map | Primary anchors | Caveats |
| --- | --- | --- | --- | --- | --- |
| 1 | Inference is memory-bandwidth-bound | **Supported** (decode, small batch); **Conditional** if stated as all inference | **Already covered** (bandwidth/unified memory dominate TOPS; HIR/CM resource floors) | Yuan et al., *LLM Inference Unveiled* [arXiv:2402.16363](https://arxiv.org/abs/2402.16363) (decode layers memory-bound on A6000; prefill often compute-bound); Leviathan et al. [arXiv:2211.17192](https://arxiv.org/abs/2211.17192) (memory/comms often bottleneck → room for speculative concurrency); GZMO `01` Pareto on unified memory bandwidth | Prefill, large batch, and high arithmetic-intensity ops can be compute-bound. Never encode “always memory-bound” as a hard floor. |
| 2 | Native C++ engines ~30% over Python | **Conditional** (native/serving stacks often beat naive HF/Python decode); **Unverified** for “~30%” as a universal delta | **Already covered** (llama.cpp/GGUF default spine in `03` + North Star §8) | llama.cpp project spine in `03`; community/HF discussions of C++/GGML vs PyTorch paths (not a controlled universal bench); no first-party paper found pinning **exactly ~30%** across models/devices | PyTorch core is already C++/CUDA; gap is stack overhead, kernels, quant formats, batching—not “C++ magic.” Do not put 30% into signed policy. |
| 3 | Speculative decoding is lossless + quantization caveat | **Supported** (classic speculative sampling preserves target distribution); **Conditional** on quantization/joint low-bit drafts | **Already covered** as optional draft accelerator (`03` §4 item 5); **successor candidate** for explicit catalog+qualification of draft models | Leviathan, Kalman, Matias [arXiv:2211.17192](https://arxiv.org/abs/2211.17192) (“without any changes to the outputs” / same distribution); llama.cpp [speculative.md](https://github.com/ggml-org/llama.cpp/blob/master/docs/speculative.md); QSpec [arXiv:2410.11305](https://arxiv.org/abs/2410.11305); Zhang et al. [arXiv:2505.22179](https://arxiv.org/abs/2505.22179) (spec + quant interaction can erase expected memory wins) | Lossless only under correct verify/rejection sampling. Synthetic acceptance modes are **not** valid model output (llama.cpp docs). Quantized draft/target pairs need separate quality qualification; draft is never a cognitive authority. |
| 4 | EXO heterogeneous VRAM pooling / 100B+ vs one-node doctrine | **Supported** that EXO *is* multi-device cluster inference for models larger than one device; **Misleading** if proposed as GZMO Living topology; **Conditional** for “100B+” as vendor/demo scale not GZMO requirement | **Reject/out of scope** for product/runtime topology | [exo-explore/exo](https://github.com/exo-explore/exo) README (cluster devices; models larger than one device; TP speedups; DeepSeek-class multi-Mac demos); heterogeneous scheduling issues e.g. [exo#2180](https://github.com/exo-explore/exo/issues/2180); North Star invariant §4.1 “One physical node” | EXO is real and useful *elsewhere*. Dual-Spark / multi-Mac is explicitly multi-box. Living claims must fit one node (Orin/Thor/Strix/Forge dGPU). 100B+ is optional catalog curiosity, not a Living floor. |
| 5 | NUMA pinning up to +53% | **Supported** as a **specific** multi-NUMA Neoverse llama.cpp PoC result; **Conditional** if generalized | **Strategic only** → optional Forge qualification knob | Arm Community, Bolt Liu, 2026-01-28: up to **55%** S_TG and **53.2%** S t/s on ZhuFeng Neoverse with PoC patch ([Arm blog](https://developer.arm.com/community/arm-community-blogs/b/ai-blog/posts/introduce-the-cross-numa-problem-and-optimization-in-llama-cpp-with-llama3-model-running-in-neoverse-n2)); patch discussed vs llama.cpp (#14232 lineage); llama.cpp multi-NUMA discussion | Gains require multi-socket/multi-NUMA, thread counts past one node, and NUMA-aware layout—not Jetson single-package UMA. PoC not a universal product default. |
| 6 | Enterprise agentic-memory adoption &lt;5% → 40% | **Unverified** | **Strategic only** (market narrative; not architecture) | No primary forecast org series found in this pass pinning **&lt;5%→40%**. Related *technical* enterprise memory substrate reports exist (e.g. Oracle Agent Memory tech report [arXiv:2607.13157](https://arxiv.org/abs/2607.13157)) but are not adoption statistics | Do not drive roadmap capacity from this number. GZMO differentiator is air-gapped evidence-grounded Keep, not enterprise TAM %. |
| 7 | Episodic/semantic/procedural hierarchy; A-MEM; sleep-time compute; AgeMem; MCP | **Supported** as research families and protocol; **Conditional** on forcing a three-store product ontology | **Already covered** (episodic logs, semantic vault/honeypot, dream consolidation, MCP gateway); **successor candidate** for explicit procedural *skill* lane + sleep-time as named metabolism budget | Cognitive hierarchy is classical (not re-proven here). **A-Mem** [arXiv:2502.12110](https://arxiv.org/abs/2502.12110); **MemGPT** hierarchical OS memory [arXiv:2310.08560](https://arxiv.org/abs/2310.08560); **AgeMem** unified LTM/STM tools [arXiv:2601.01885](https://arxiv.org/abs/2601.01885); **sleep-time compute** [arXiv:2504.13171](https://arxiv.org/abs/2504.13171) (~5× less test-time for same accuracy on Stateful GSM/AIME; amortize ~2.5× multi-query); **MCP** [modelcontextprotocol.io](https://modelcontextprotocol.io/) | A-Mem/AgeMem assume cloud-scale LLM managers and often rewrite memory without GZMO’s evidence→verify→promote. Sleep-time maps to overnight metabolism, not a new authority tier. Procedural knowledge in GZMO must remain recipes/candidates under operator signature if they change code/schema/capability. |
| 8 | SDD/vibe coding productivity +50–126%; junior job decline 13–30% | **Supported** that “vibe coding” is a studied practice; **Unverified** for those exact %; **Misleading** if treated as guaranteed GZMO delivery velocity | **Strategic only** (operator workflow culture); not product architecture | Sarkar & Drosos, vibe coding empirical study [arXiv:2506.23253](https://arxiv.org/abs/2506.23253); multivocal review [arXiv:2607.21652](https://arxiv.org/abs/2607.21652); formal-methods critique [arXiv:2511.00202](https://arxiv.org/abs/2511.00202). No BLS/primary labor series found here for **13–30% junior decline** or **+50–126%** SDD productivity | Expertise redistributes (prompting, review, verification)—does not vanish. GZMO candidate plane already assumes AI-generated code is untrusted until signed promotion. |
| 9 | Semantic caching up to 100%; DyCP; OptiLLM | **Supported** as techniques; **Conditional/Misleading** for “up to 100%” as expected savings | **Successor candidate** (ephemeral context economy under Redis/process cache); not durable authority | GPTCache docs claim cost/speed marketing (“10x / 100x”) for **exact/similar hit** paths ([GPTCache](https://gptcache.readthedocs.io/en/latest/)); **DyCP** dynamic dialogue pruning [arXiv:2601.07994](https://arxiv.org/abs/2601.07994) (selective context, latency example ~3× first-token in one figure—not universal 100%); **OptiLLM** inference *accuracy* proxy via extra compute ([optillm](https://github.com/algorithmicsuperintelligence/optillm))—orthogonal to semantic cache | 100% only if every request is a perfect cache hit—impossible as a planning assumption. Semantic cache can return **wrong** answers on near-duplicate unsafe matches; GZMO must revalidate against authority and never cache past faithfulness gates. OptiLLM multiplies tokens—conflicts with airgap energy floors unless envelope-bounded. |
| 10 | Agent inversion / loop drift; terms Grain / Flexion / Anchory | **Conditional** on the *problem class* (unbounded agent loops, goal drift, excessive agency); **Unverified** for those three terms | **Successor candidate** for circuit-breakers / resource floors (problem); **Reject** of the terminology as North-Star vocabulary | OWASP LLM06 Excessive Agency; North Star §12–13 thermal/resource stop-candidate-first; `04`/`06` envelopes and corrigibility. **No primary paper, standard, or vendor doc found defining Grain / Flexion / Anchory** as agent-control primitives | Rename to existing vocabulary: capability envelopes, resource floors, soak/rollback, stop-evolve, audit. Do not mint undefined terms into constitution. |

---

## Family-by-family notes (principle vs number)

### 1. Memory-bandwidth-bound inference

- **Principle:** Autoregressive **decode** at batch≈1 is typically memory-bound (weight/KV traffic per tiny matmul).
- **Number/context:** Roofline tables in [arXiv:2402.16363](https://arxiv.org/abs/2402.16363) show decode projections at intensity ~1 OPs/byte on A6000; prefill projections often compute-bound.
- **GZMO:** Already the reason `01` ranks unified memory capacity/bandwidth over TOPS and `03` budgets VRAM/RAM/context in CM.

### 2. Native C++ vs Python

- **Principle:** Dedicated serving engines (llama.cpp, etc.) remove Python orchestration overhead and use tight quantized kernels—often faster than stock HF generate loops.
- **Number:** “~30%” **Unverified** as a general constant.
- **GZMO:** Default spine is already llama.cpp/GGUF; optional accelerator lanes only after CM proof.

### 3. Speculative decoding

- **Principle:** Draft-then-verify can be **distribution-identical** to the target ([arXiv:2211.17192](https://arxiv.org/abs/2211.17192)).
- **Caveat:** Quantization changes draft acceptance and can change the efficiency story ([arXiv:2505.22179](https://arxiv.org/abs/2505.22179), [arXiv:2410.11305](https://arxiv.org/abs/2410.11305)); synthetic accept rates are benchmarks only (llama.cpp).
- **GZMO:** Draft model = accelerator for an already-qualified target role; separate RAM/VRAM in fit math; operator-signed catalog entry.

### 4. EXO / multi-node pooling

- **Principle:** Model-parallel clusters can host weights that do not fit one machine.
- **Conflict:** North Star constitutional invariant **one physical node**; research `01` already flags dual-Spark networking as multi-box out of topology.
- **GZMO:** **Reject** EXO-style clustering for Living. Capacity strategy = smaller/quantized roles, honest degrade, Forge headroom—not a second host.

### 5. NUMA +53%

- **Principle:** Cross-NUMA atomics and remote tensor traffic hurt CPU inference; locality helps.
- **Number:** **+53.2% / +55%** on a **specific** Neoverse dual-node PoC with unmerged/low-interest server patch ([Arm blog](https://developer.arm.com/community/arm-community-blogs/b/ai-blog/posts/introduce-the-cross-numa-problem-and-optimization-in-llama-cpp-with-llama3-model-running-in-neoverse-n2)).
- **GZMO:** Relevant only if Forge/x86 multi-socket appears in HIR; never a Jetson UMA default.

### 6. Enterprise adoption curve

- **Unverified** quantitative forecast.
- **GZMO:** Ignore for architecture. Track competitor *capabilities*, not uncited TAM ramps.

### 7. Memory hierarchy stack

| External idea | GZMO analogue today | Gap |
| --- | --- | --- |
| Episodic | `memory/YYYY-MM-DD.md`, session logs | Already present |
| Semantic / verified facts | vault/honeypot + evidence + bi-temporal | Already present; authority moves to PostgreSQL per North Star |
| Procedural | scripts/skills/candidates; not a first-class verified memory class | **Successor:** procedural cards as derived, non-SoT, signed if they change behavior |
| A-Mem dynamic notes/links | spark/dream linking, wiki emit | Useful algorithms; must not auto-promote without verify |
| Sleep-time compute | overnight metabolism / dream / spark | Name the **budget** explicitly in envelopes (joules/tokens/wall) |
| AgeMem tool-memory RL | tool-using agent over MCP | Training AgeMem-style policies is out of airgap default; tool allowlists already required |
| MCP | `McpGateway` sole agent interface | Already constitutional |

### 8. SDD / vibe coding labor claims

- Treat as **operator process** research, not appliance KPIs.
- Candidate plane + immutable evaluator already encode “AI writes, constitution judges.”

### 9. Semantic cache / DyCP / OptiLLM

- **Semantic cache:** ephemeral projection; key by content hash + model pin + policy version; never bypass PostgreSQL revalidation for memory answers.
- **DyCP:** dialogue **input** pruning—cousin to context budgets already in CM; good for chat latency.
- **OptiLLM:** multiplies reasoning tokens—use only inside signed energy/latency envelopes, primarily Forge eval—not Living overnight writer default.

### 10. Loop drift vocabulary

- Real failure mode: unbounded tool loops, self-reinforcing wrong memory, evaluator gaming.
- **Reject** Grain/Flexion/Anchory as undefined jargon.
- Encode as: max tool iterations, token/joule caps, refractory periods (already spark-field shaped), stop-evolve, dual-writer refuse, candidate isolation.

---

## GZMO implications (ranked)

### P0 — Preserve / enforce now (no North Star edit required)

1. **Keep one-node airgap doctrine.** Any opportunity-finder item that needs EXO/multi-host VRAM pooling is auto-`Reject/out of scope` for Living.
2. **Resource-loop circuit breakers** for agent/MCP and metabolism: max steps, max tokens, max joules, thermal stop (candidate work first)—close the real “loop drift” class without new mythology.
3. **Faithfulness before speedups:** speculative decode, caches, and sleep-time precompute must not weaken extract_verify gates or evidence binding.
4. **Do not encode external %** (30 / 53 / 100 / 5→40 / 50–126 / 13–30) into signed profile policy or fitness floors.

### P1 — North-Star *successor* design candidates (future tickets; baseline immutable)

1. **Inference optimization lane (single node):** optional draft speculative decoding + quant policy in Model Catalog; qualify draft+target pairs; fit math includes both.
2. **Context-economy projections:** Redis/process semantic cache for *immutable* completion fragments; DyCP-like pruning for chat context; always fall back to authority path.
3. **Sleep-time budget named in capability envelopes:** overnight metabolism as first-class scheduled compute with joules/token caps (maps sleep-time compute paper to Dream/Spark without new tier).
4. **Procedural memory lane:** versioned skill/recipe objects derived from verified outcomes; **not** durable truth; code-changing procedures remain Candidates (tier C) requiring operator signature.
5. **A-Mem-like link evolution** only behind verify/honeypot and one-writer transactions—never free-form LLM rewrite of SoT.

### P2 — Forge / strategic only

1. **NUMA/affinity probes** in HIR for multi-socket x86 Forge; document measured deltas; no copy-paste of +53% into Orin/Thor docs.
2. **OptiLLM-class extra reasoning** only in isolated evaluator/Forge, envelope-bounded.
3. **Market narratives** (enterprise adoption, junior labor) may inform operator staffing, not appliance architecture.
4. **Vibe/SDD workflow guides** for humans working *on* GZMO—outside constitutional runtime.

---

## Must NOT enter the North Star

Explicit exclusion list (even if the opportunity synthesis repeats them):

1. **Multi-node EXO / RDMA Thunderbolt clusters / heterogeneous multi-host VRAM pooling** as Living or Scout product topology.
2. **“100B+ local” as a Living requirement** or marketing claim without single-node qualification.
3. **Cloud or second-machine inference** “just for big models.”
4. **Numeric folklore as floors:** universal +30% C++ advantage; universal +53% NUMA; semantic cache “up to 100%”; adoption &lt;5%→40%; SDD +50–126%; junior employment −13–30%.
5. **Grain / Flexion / Anchory** as constitutional modules or domain language.
6. **AgeMem/A-Mem autonomous memory rewrite** without evidence→verify→promote and operator authority tiers.
7. **OptiLLM unbounded test-time compute** on the Living overnight writer path.
8. **Procedural memory as override of code/schema/security** without PromotionKernel signatures.
9. **Any weakening of:** one physical node; one authoritative writer; runtime airgap; role qualification; no self-issued authority; reversible production changes.
10. **Edits to** `docs/superpowers/specs/2026-08-31-self-developing-living-database-design.md`, ADRs, or research `01`–`06` on the basis of this synthesis alone.

---

## Coverage vs research `01`–`06`

| Theme | Prior note | This assessment |
| --- | --- | --- |
| Bandwidth & one-node hardware | `01` | Confirms; rejects multi-node EXO overlay |
| Boot/trust | `02` | Unchanged; no claim family alters boot |
| Runtime / llama.cpp / draft mention | `03` | Deepens speculative+quant caveats; NUMA Forge-only |
| Evolution authority | `04` | Loop drift → envelopes/circuit breakers; reject new jargon |
| Durable memory plane | `05` | Caches/DyCP = ephemeral; procedural ≠ SoT |
| Threat model | `06` | Caches and sleep-time content remain untrusted until gates |

---

## Sources (primary, non-exhaustive)

- Yuan et al., *LLM Inference Unveiled: Survey and Roofline Model Insights*, [arXiv:2402.16363](https://arxiv.org/abs/2402.16363)
- Leviathan, Kalman, Matias, *Fast Inference from Transformers via Speculative Decoding*, [arXiv:2211.17192](https://arxiv.org/abs/2211.17192)
- llama.cpp speculative decoding docs, [github.com/ggml-org/llama.cpp/blob/master/docs/speculative.md](https://github.com/ggml-org/llama.cpp/blob/master/docs/speculative.md)
- Zhao et al., *QSpec*, [arXiv:2410.11305](https://arxiv.org/abs/2410.11305); Zhang et al., *Speculative Decoding Meets Quantization*, [arXiv:2505.22179](https://arxiv.org/abs/2505.22179)
- exo labs, [github.com/exo-explore/exo](https://github.com/exo-explore/exo)
- Arm Community, *Scaling llama.cpp on Neoverse N2…*, 2026-01-28, [developer.arm.com/.../introduce-the-cross-numa-problem-and-optimization-in-llama-cpp...](https://developer.arm.com/community/arm-community-blogs/b/ai-blog/posts/introduce-the-cross-numa-problem-and-optimization-in-llama-cpp-with-llama3-model-running-in-neoverse-n2)
- Xu et al., *A-Mem*, [arXiv:2502.12110](https://arxiv.org/abs/2502.12110)
- Packer et al., *MemGPT*, [arXiv:2310.08560](https://arxiv.org/abs/2310.08560)
- Yu et al., *AgeMem*, [arXiv:2601.01885](https://arxiv.org/abs/2601.01885)
- Lin et al., *Sleep-time Compute*, [arXiv:2504.13171](https://arxiv.org/abs/2504.13171)
- Choi, Zhang, Choi, *DyCP*, [arXiv:2601.07994](https://arxiv.org/abs/2601.07994)
- GPTCache documentation, [gptcache.readthedocs.io](https://gptcache.readthedocs.io/en/latest/)
- OptiLLM, [github.com/algorithmicsuperintelligence/optillm](https://github.com/algorithmicsuperintelligence/optillm)
- Model Context Protocol, [modelcontextprotocol.io](https://modelcontextprotocol.io/)
- Sarkar & Drosos, *Vibe coding…*, [arXiv:2506.23253](https://arxiv.org/abs/2506.23253)
- GZMO North Star design (approved 2026-08-31) and `research/north-star/01`–`06`

---

*Additive assessment only. Approved North Star baseline remains immutable. Research current as of 2026-09-01; numbers without primary method context stay Unverified.*
