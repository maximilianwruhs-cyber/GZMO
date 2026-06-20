---
type: entity
title: Speculative Decoding
created: 2026-06-08
updated: 2026-06-09
sources: 17
tags: []
status: draft
gzmo_synthetic: true
---





















# Speculative Decoding

Type: CONCEPT

## From [drive-research-advanced-local-ai-features-guide](/entities/drive-research-advanced-local-ai-features-guide.md) (2026-06-08)
- Pairs a massive "Target Mode" with a tiny, lightning-fast "Draft Model."
- Can yield a 2x to 4x inference speedup with zero loss in generation quality.
- Works best when the drafted output is predictable (low temperature).

## From [drive-research-du-hast-gesagt-part1](/entities/drive-research-du-hast-gesagt-part1.md) (2026-06-08)
- Dramatically enhances speed and performance on local hardware.
- Pairs a draft model (junior dev) with a target model (senior dev).
- LM Studio has native support for it.
- Used in LM Studio.
- Involves a 'Draft' model and a 'Senior' reasoning model.
- Allows faster AI typing.
- Requires VRAM.
- Can be enabled in LM Studio.

## From [drive-research-enhancing-local-ai-hypervisor-architecture](/entities/drive-research-enhancing-local-ai-hypervisor-architecture.md) (2026-06-08)
- Leverages a two-model collaborative generation scheme.
- Bypasses memory-bandwidth bottlenecks in LLM serving.
- Can be configured via vLLM.

## From [drive-research-imagine-creating-sm120-according-to-our-progress](/entities/drive-research-imagine-creating-sm120-according-to-our-progress.md) (2026-06-08)
- Enabling multi-token prediction (MTP) or speculative decoding alongside the Marlin fallback path introduces a critical precision mismatch.
- Speculative draft heads are trained to operate on native FP4 activation distributions.
- Speculative decoding must be disabled when running under the emulated Marlin path.
- The constant computational overhead of rolling back key-value (KV) caches and re-evaluating the target model creates a net performance regression of up to -22%.

## From [architectures-and-optimizations-for-speculative-de-micro04](/entities/architectures-and-optimizations-for-speculative-de-micro04.md) (2026-06-09)
- Mitigates autoregressive generation bottleneck by introducing intra-request parallelism.
- Trades surplus computational power for memory bandwidth efficiency.
- Splits generation into distinct phases using a smaller mechanism to bypass the primary model.
- Accepts multiple tokens in a single execution cycle when draft predictions align with the target model.

## From [architectures-and-optimizations-for-speculative-de-micro06](/entities/architectures-and-optimizations-for-speculative-de-micro06.md) (2026-06-09)
- Takes on a new hardware dimension with MoE models.
- Involves predicting the sequence of experts.
- Enables prefetching of expert weights.

## From [building-a-private-local-ai-development-environmen-micro02](/entities/building-a-private-local-ai-development-environmen-micro02.md) (2026-06-09)
- A technique to speed up AI generation.
- Involves a small 'Draft-model' guessing the next tokens quickly, and a larger 'Target-model' verifying them in parallel.
- Requires both models to use the same tokenizer.

## From [building-a-private-local-ai-development-environmen-micro03](/entities/building-a-private-local-ai-development-environmen-micro03.md) (2026-06-09)
- Verfahren zur Beschleunigung lokaler KI-Inferenz
- nutzt ein 'Junior-Modell' (Draft Model) und ein 'Senior-Modell' (Target Model)
- beschleunigt die Generierung bei repetitivem Code um das 2- bis 4-fache

## From [building-a-private-local-ai-development-environmen-micro06](/entities/building-a-private-local-ai-development-environmen-micro06.md) (2026-06-09)
- Also known as Draft-and-Verify, Assisted Generation, or Multi-Token Prediction.
- Dramatically enhances speed and performance on local hardware.
- Pairs two models: a Draft Model (Junior Dev) and a Target Model (Senior Dev).
- The Draft Model is ultra-fast and microscopic.
- The Target Model is larger and smarter, verifying the Draft Model's output.
- Achieves faster generation by verifying multiple tokens at once.
- LM Studio has native support for it.
- Requires the draft model to share the exact same vocabulary/tokenizer as the main model.
- Can be enabled in LM Studio's configuration.

## From [drive-research-llamacpp-optimization-blueprint-micro03](/entities/drive-research-llamacpp-optimization-blueprint-micro03.md) (2026-06-09)
- Exploits spare compute capacity to amplify token generation throughput.
- Frequently results in 1.5x to 2x speedups with no loss in output quality.
- Leverages a smaller 'draft' model to predict next K tokens.

## From [drive-research-llamacpp-optimization-blueprint-micro04](/entities/drive-research-llamacpp-optimization-blueprint-micro04.md) (2026-06-09)
- Can significantly increase tokens/sec in llama.cpp
- Has potential for running big LLMs on consumer grade GPUs
- Provides speed improvements in llama.cpp server

## From [optimizing-nvidia-blackwell-sm120-part1-micro02](/entities/optimizing-nvidia-blackwell-sm120-part1-micro02.md) (2026-06-09)
- Exploits spare compute capacity to radically amplify token generation throughput.
- Frequently results in 1.5x to 2x speedups with no loss in output quality.
- Leverages a smaller "draft" model to predict the next K tokens, which the larger target model then verifies in a single pass.
- Circumvents the sequential bottleneck of traditional autoregressive decoding.
- Requires a smaller draft model of the identical architecture family.
- Benchmarking demonstrates staggering results.
- Applying a 0.5B draft model to a 27B parameter target architecture on a dedicated RTX 3090 improved throughput from 38 t/s to 65 t/s—an increase of over 70%.

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Enabling it with Marlin dequantization on SM120 architectures results in a performance regression of up to -22%.
- Unquantized speculative draft heads (e.g., in MTP drafter) do not support the Marlin quantization format.
- Typically yields substantial end-to-end speedups, but contradicts expectations when used with Marlin on SM120.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro05](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro05.md) (2026-06-09)
- An inference optimization technique that accelerates LLMs by predicting and verifying multiple tokens simultaneously.
- Employs a 'draft-then-verify' architecture.
- Breaks the sequential bottleneck of autoregressive token generation.

## From [the-architecture-of-speculative-decoding-and-infer-part2-micro03](/entities/the-architecture-of-speculative-decoding-and-infer-part2-micro03.md) (2026-06-09)
- Requires draft and target models to ideally belong to the same architectural family and share the exact same tokenizer.
- High acceptance rates are crucial for acceleration.

## From [ultimate-local-ai-development-stack-for-vscodium-micro02](/entities/ultimate-local-ai-development-stack-for-vscodium-micro02.md) (2026-06-09)
- A technique to enhance AI speed and performance on local hardware.
- Pairs a fast, microscopic predicting model with a larger, smarter model.
- Also known as Draft-and-Verify or Assisted Generation.

## From [ultimate-local-ai-development-stack-for-vscodium-micro03](/entities/ultimate-local-ai-development-stack-for-vscodium-micro03.md) (2026-06-09)
- Can be turned on in LM Studio.
- Can be swapped out for a 1.58-bit model.
