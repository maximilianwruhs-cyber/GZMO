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

## 7B-class probe (VM200, 2026-08-22) — **NEGATIVE: restore is a content no-op**

**Model:** Mamba-Codestral-7B-v0.1-Q4_0 (Jamba hybrid) on VM200 GTX 1070 (`:8123`,
llama.cpp build b9018 rebuilt from pre-refactor commit c84e6d6db). Corpus 17,554 tokens.

### Latency numbers (bench-7b.json)

| Metric | Full-Prefill | Injection | "Speedup" |
|---|---|---|---|
| TTFT (median of 25 runs, ms) | 42,042.6 | 126.9 | 331.35× |
| State file | — | 276,371,596 B (276 MB) | save 450 ms / restore 210 ms |

### But `restore_verified: false` — the decisive control test

Question answerable ONLY from the corpus (Prime port = **8000**):

| Condition | Answer (start) |
|---|---|
| Full-prefill | `8000 (`gzmo serve`)…` ✅ corpus fact retrieved |
| Injection (save→restore→query) | `8080 B: 8081 C: 8082 D: 8083 — correct answer B: 80…` ❌ |
| Zero-context | **byte-identical to injection** ❌ |

Save/restore report byte-level success (n_saved = n_restored = 17563), but the restored
state does **not** contribute to inference on this build. The 331× figure is a
prompt-length artifact (prefill 17,554 tokens vs prefill ~20 tokens with NO corpus),
NOT working O(1) state injection. See `decisive-control.md`.

### Correction to the earlier "mechanism proven" claim (Run C, 130m)

The 66.7× TTFT gap from the 130m run was the same prompt-length artifact. With garbage
answers in both conditions, no content test was possible at 130m — attributing "mechanism
proven" to that run was premature. Withdrawn.

### Verdict: **NO-GO** for ADR-0008 Option A (Mamba route) on the current llama.cpp build

1. Slot save/restore is a content no-op on build b9018 for the Jamba hybrid — the
   state-injection mechanism does NOT reproduce on GZMO's llama.cpp.
2. Quality at 7B: full-prefill answers were superficially on-topic but shallow
   (see bench-7b.json answers); no parity claim is possible while restore is a no-op.
3. The 276 MB state file (vs 2.87 MB at 130m) also undercuts the "192 KB hidden state"
   selling point for this model class — Jamba serializes attention + mamba state per layer.

### Re-open conditions (any of)
- A llama.cpp build where `/slots/:id?action=restore` demonstrably changes subsequent
  inference (verify with a corpus-only-fact question, as in `decisive-control.md`),
  OR a dedicated state-injection API that wires restored state into the active slot.
- A non-hybrid SSM (pure mamba-780m-class) where state save/load is the ONLY recurrent
  memory — re-run the decisive control there before any latency claim.

*Probe by operator (bench-7b.py executed after delegated worker timed out at 90 min;
worker had produced BASELINE-EMBED.md + scripts). VM200 :8123 stopped, router :8081 healthy,
workstation :8000 untouched throughout.*
