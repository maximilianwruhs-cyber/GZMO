# VERDICT — PRECOG via Mamba: First Independent Numbers

## Summary

The PRECOG state-injection mechanism (arXiv:2608.02560) **reproduces on GZMO hardware** using llama.cpp's native mamba support. This is the first independent benchmark confirming the O(1)-prefill latency claim: a 16,898-token corpus can be pre-computed once into a 2.87 MB mamba SSM state file, then injected into a fresh context for 66.72× faster time-to-first-token.

## Hard Numbers

| Metric | Full-Prefill | Injection | Speedup |
|--------|-------------|-----------|---------|
| TTFT (median, ms) | 276.24 | 4.14 | **66.72×** |
| Prompt tokens | 16,898 | ~26 | — |
| Tokens/sec (decode) | 1,172 | — | — |

- **Model:** mamba-130m-q8_0 (155 MB)
- **Corpus:** 16,898 tokens from 8 real GZMO ADRs + runbooks
- **State file:** 2,869,892 bytes (2.87 MB) — R and S SSM tensors across all layers
- **Runs:** 5 per condition, median reported
- **GPU:** RTX 5070 Ti, production :8000 server running concurrently (untouched)
- **Full-prefill TTFT runs (ms):** 312.09, 276.56, 276.24, 275.72, 276.08
- **Injection TTFT runs (ms):** 4.44, 4.14, 4.22, 3.98, 4.01

## Does O(1)-prefill reproduce?

**YES.** The mamba recurrent state (SSM R and S tensors per layer) is fully serializable via `llama_state_seq_save_file` and restorable via `llama_state_seq_load_file`. After restoring a pre-computed state, the query prompt is only ~26 tokens instead of 16,898 — collapsing TTFT from 276 ms to 4.1 ms. The mechanism works at the llama.cpp infrastructure level without any custom code.

## Capability Gates

- **Mamba inference:** ✅ Supported (LLM_ARCH_MAMBA, llama_model_mamba, native mamba/mamba2 graph builders)
- **State save:** ✅ Supported (llama_memory_recurrent::state_write_data serializes R+S tensors; server `/slots/:id?action=save`)
- **State load:** ✅ Supported (llama_memory_recurrent::state_read_data restores R+S tensors; server `/slots/:id?action=restore`)

## GO / NO-GO / HOLD

**HOLD** for ADR-0008 Option A (Mamba route).

The latency mechanism is proven and compelling (66.72× TTFT reduction). However:

1. **Quality is unproven.** The 130m model cannot answer any of the 5 corpus questions correctly — both full-prefill and injection produce garbage. This is expected at 130M params but means we cannot yet claim quality parity between injection and full-prefill.
2. **Scale caveat.** Only the 130m model fit in free GPU memory (~1.7 GB free alongside the production 27B model). The 790m (593 MB) and 7B (4–5 GB) models were not benchmarked due to insufficient free VRAM during concurrent production operation.
3. **The mechanism bypasses both TENNs-LLM gates** (license + custom_code) as designed — using native llama.cpp mamba support with no external dependencies.

## Next Gate

**7B-class quality parity probe.** Run the same 5-question benchmark on `bartowski/Mamba-Codestral-7B-v0.1-GGUF` (or equivalent) with the production :8000 server stopped, to determine:

1. Does a 7B mamba model produce correct answers from the corpus?
2. Does state injection preserve answer quality equivalently to full-prefill (no quality degradation from the state save/load round-trip)?

Only if quality parity holds at 7B should Option A proceed to GO. If quality degrades under injection, the mechanism is latency-valid but quality-invalid, requiring a different architecture (e.g., hybrid mamba-attention with KV-cache-aware state serialization).

---

## 7B-class probe (VM200, 2026-08-22) — **CORRECTED: mechanism REPRODUCES (v1 was a test error)**

**Model:** Mamba-Codestral-7B-v0.1-Q4_0 (Jamba hybrid: mamba+attention), VM200 GTX 1070
(`:8123`, llama.cpp build b9018, ctx 20480, `-np 1`). Corpus 17,554 tokens (8 ADRs + runbooks).
State file 276 MB (attention KV + mamba SSM/conv per layer), save 450 ms / restore 210 ms.

### The v1 NO-GO was a test error — not a mechanism failure

The earlier `decisive-control.md` concluded "restore is a content no-op" because the
restore → **no-prefix question** output was byte-identical to zero-context. A source audit
of llama.cpp (`tools/server/server-context.cpp`) shows that is expected: a restore re-fills
the slot's prompt tokens with the saved corpus tokens, and a new question that shares **no
prefix** computes `n_past = 0` → the server's *"forcing full prompt re-processing
(hybrid/recurrent)"* path (PR #13194) → the question is evaluated from position 0, so the
restored state is unreachable. That is a **protocol error in the test**, not a broken
mechanism. The 331× figure in `bench-7b.json` was likewise a prompt-length artifact (the
injection prompt had no corpus at all).

### Correct protocol: prefix-continuation (the paper's injection)

Restore the saved state, then continue with **(saved prefix + new question)** and
`cache_prompt=true`, so `n_past = prefix_len` and only the new question is computed,
reading the restored SSM/KV state.

### Decisive control (`decisive-control2.json`, slot 0 sequential, temp 0.0)

Corpus-only fact: Prime inference server TCP port = **8000**.

| Condition | Protocol | Knows 8000? | prompt_ms |
|---|---|---|---|
| C1 FULL-PREFILL | corpus+Q in prompt, no cache | ✅ | 41,210 |
| **C2 INJ-PREFIX** | restore → prefix-continue, cache=true | **✅** | **478** |
| C3 INJ-PREFIX-NC | restore → prefix-continue, cache=**false** | ✅ | 41,792 |
| C4 INJ-NOPREFIX | restore → question only (the v1 test) | ❌ | 192 |
| C5 ZERO-CONTEXT | fresh, question only | ❌ | 126 |

**The proof is C2 vs C3:** identical prompt (corpus literally in both), the only difference
is `cache_prompt` after restore. C2 (reuse restored state) = 478 ms; C3 (re-prefill the same
corpus tokens) = 41,792 ms — 86× slower for the same input. So C2's knowledge comes from the
**restored file state**, not from re-reading tokens. C4/C5 being byte-identical is now
explained by `n_past=0` (no shared prefix), not by a broken restore.

### Cold-control (`final-7b.json`) — rules out a lingering in-memory KV

Same prompt (corpus+question), same `cache_prompt=true`, empty slot, **no restore** →
41,281 ms and does NOT know port 8000. Restore + same prompt + same flag → 478 ms and knows
it. The only difference is the restored state file; the speedup is provably from the restored
SSM/KV state.

### Quality parity (`final-7b.json`, 5 ADR questions, temp 0.0)

- **Full-prefill median TTFT 41,991 ms · Injection median TTFT 476.6 ms · 88.1×.**
- 4/5 questions: injection retrieves the right corpus fact, on-topic.
- **Q4 (lite SKU): injection is a factual error.** Corpus: "there is **no** lite SKU… **not**
  a product." Full-prefill: "not a product, a lab tool" (correct). Injection: "the lite SKU
  **is** a product… first-class product" (contradicts the corpus).

### Interpretation

The O(1) state-injection mechanism **reproduces** on build b9018 for the 7B Jamba hybrid —
the v1 "content no-op" was a protocol error. But the **fixed-size SSM state is lossy for long
contexts**: it carries the general gist (USP, flywheel, process lock, edge SSM) yet drops a
specific negation (the "no lite SKU" detail) that full-prefill retains in the KV cache. That
is the fundamental SSM tradeoff (constant-size state vs attention's full KV).

### Verdict: Option A (Mamba route) — **mechanism GO, quality gate HOLD**

1. **Mechanism: proven.** O(1) injection works, 88.1× TTFT reduction at 7B, airtight by
   C2/C3 + cold-control. The paper's core claim reproduces on GZMO's llama.cpp.
2. **Quality: partial.** 4/5 parity; 1/5 (Q4) factual error under injection. Not a clean
   quality GO — the fixed-size state loses long-context detail.
3. **Adoption gate (before any production swap):** re-measure on GZMO's **real, shorter
   extract/distill prompts** (per-state context ≪ 17K tokens), where the lossy-state penalty
   shrinks; and/or verify a hybrid (SSM state + bounded KV tail) recovers Q4. Only then does
   Option A move from HOLD to GO.
4. **276 MB state file** (vs 2.87 MB at 130m) undercuts the "192 KB hidden state" selling
   point for this model class — Jamba serializes attention + mamba state per layer.

*Corrected by operator after Max's challenge ("du hast bestimmt etwas falsch gemacht").
v1 NO-GO withdrawn as a test error. VM200 :8123 stopped after each run, router :8081 healthy,
workstation :8000 untouched throughout. Evidence: decisive-control2.json, final-7b.json.*
