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
