# PRECOG Spike — VERDICT

**Date:** 2026-08-22
**ADR:** [ADR-0008](../../docs/adr/ADR-0008-edge-ssm-memory.md) — Option A
**Paper:** arXiv:2608.02560 (PRECOG — Pre-Computed Context Injection)

---

## What works

- **TENNs-LLM weights exist on HuggingFace.** Three repos found: `BrainChip-AI/tenns-llm-1b`, `BrainChipInc/tenns-llm-1b`, `NickMarkovsky/tenns-llm-1b`. All tagged `tenns_llm`, `ssm`, `recurrent`, `causal-lm`, `custom-architecture`.
- **llama.cpp has native SSM support** for the Mamba family (Mamba, Mamba2) and RWKV family (RWKV6, RWKV7). These are true SSMs with fixed-size recurrent state — the architectural prerequisite for PRECOG's O(1) state injection.
- **PRECOG's latency claim is architecturally sound.** Collapsing prefill from O(L_context) to O(1) per query is a property unique to SSMs (position-agnostic fixed-size hidden state). The reported ~27 s → <6 ms (4500×) speedup on edge hardware is plausible for the SSM architecture class.

## What is blocked

- **License: `cc-by-nc-4.0` (non-commercial).** TENNs-LLM weights are released under Creative Commons Attribution-NonCommercial 4.0. GZMO is a sovereign product — this license likely prohibits production use. **Weight status = GATED.** This is the primary blocker.
- **Architecture: `custom_code`, not in llama.cpp.** TENNs-LLM uses `model_type: tenns_llm` with the `custom_code` tag, meaning it requires custom Python (transformers) inference code. It is NOT in llama.cpp's architecture registry (which supports Mamba/Mamba2/RWKV6/RWKV7 but not `tenns_llm`). **SSM inference status = PARTIAL.** PRECOG's state-injection mechanism cannot run through llama.cpp as-is.
- **No GGUF conversion path.** Without a standard architecture, there is no convert.py → GGUF → llama.cpp path for TENNs-LLM. State injection would require a custom inference server (transformers + custom_code on GPU), which violates the airgap simplicity goal and adds a non-llama.cpp dependency.
- **Quality parity untested.** TENNs-LLM is 1.2B parameters — 29× smaller than Qwen3.6-35B-MTP. Even if inference were possible, quality on GZMO's real extract/distill prompts is unmeasured.

## Bench result

`bench.json`: `{"skipped": true, "reason": "weights gate failed (status=gated)"}`

The latency benchmark was not run because the weights gate failed (CC-BY-NC-4.0 license) and the SSM inference gate is partial (TENNs-LLM not in llama.cpp architecture registry).

## Recommendation: **NO-GO** (for TENNs-LLM specifically)

The PRECOG mechanism is architecturally sound and the latency claim is plausible. However, the specific TENNs-LLM model is blocked on two hard gates:

1. **License (CC-BY-NC-4.0)** — likely prohibits production use in a sovereign product.
2. **Inference (custom_code, not in llama.cpp)** — no local GGUF/llama.cpp path; requires custom Python inference.

### HOLD conditions for re-evaluation

If any of the following change, re-open this spike:
- TENNs-LLM weights re-released under a permissive license (MIT/Apache-2.0/CC-BY).
- A GGUF conversion path for `tenns_llm` architecture lands in llama.cpp.
- An alternative gated-SSM model (Mamba-class, already supported by llama.cpp) demonstrates PRECOG-style state injection with comparable hidden-state size and quality.

### Alternative path

PRECOG's core idea (pre-compute SSM hidden states offline, inject at query time) could be explored with a Mamba-class model already supported by llama.cpp (e.g., `state-spaces/mamba-130m`, `mistralai/Mamba-Codestral-7B-v0.1`). This would bypass both the license and inference gates, but would require verifying that Mamba's hidden state is large enough to serve as a useful corpus summary (192 KB in TENNs-LLM vs. typically smaller in Mamba variants).

---

*Verdict: GZMO operator surface (OpenClaw) · 2026-08-22 · no runtime code changed*
