---
type: entity
title: Mixture of Experts (MoE)
created: 2026-06-08
updated: 2026-06-09
sources: 13
tags: []
status: draft
gzmo_synthetic: true
---













# Mixture of Experts (MoE)

Type: CONCEPT

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- A technique for consolidating multiple independently trained or fine-tuned dense models.
- Results in a unified sparse architecture.
- Does not require backpropagation or additional GPU training time.

## From [drive-research-blackwell-sm120-gemm-optimization-guide](/entities/drive-research-blackwell-sm120-gemm-optimization-guide.md) (2026-06-08)
- NVFP4 MoE
- FP4 MoE Kernel Engineering
- A layer implemented in deep learning pipelines.
- Standard modular implementation suffers from memory bandwidth overhead.
- High-efficiency kernel fusion can optimize MoE passes.

## From [drive-research-erbandbreite-und-latenzengpässe](/entities/drive-research-erbandbreite-und-latenzengp-sse.md) (2026-06-08)
- Achieves efficiency through sparse activation, selecting a fractional subset of experts per token.
- Introduces complications that clash with speculative decoding mechanics.
- Speculative decoding systematically breaks the sparsity paradigm in MoE.

## From [drive-research-frankenmoe-blueprint-analysis](/entities/drive-research-frankenmoe-blueprint-analysis.md) (2026-06-08)
- A unified sparse architecture.
- Consolidates multiple independently fine-tuned dense models.
- An architecture where multiple independently fine-tuned dense models are consolidated.
- A sparse architecture.
- Runtime execution is governed by a dynamic routing network.

## From [drive-research-frankenmoe-merging-ai-models](/entities/drive-research-frankenmoe-merging-ai-models.md) (2026-06-08)
- A sparse MoE architecture.
- The gating network determines the routing probabilities for tokens.
- Operational efficiency depends heavily on the routing decisions made by the gating network.
- Consolidates multiple independently trained or fine-tuned dense models.
- Requires a gating network or router to regulate data routing.

## From [drive-research-mergekit-moe-model-creation-guide](/entities/drive-research-mergekit-moe-model-creation-guide.md) (2026-06-08)
- A fusion paradigm that synthesizes a sparse MoE architecture from pre-trained dense models.
- Extracts self-attention and layer normalization weights from a base model.
- Pairs them with parallel, routing-gated feed-forward networks (FFNs) derived from specialized expert models.
- Enables zero-shot multitask ensembling of highly divergent domain specialists.
- Fusion paradigm that synthesizes a sparse MoE architecture from pre-trained dense models.
- Often referred to as 'frankenMoE' creation.
- Requires training from scratch historically, but libraries like MergeKit allow cold-start construction.

## From [drive-research-optimizing-qwen36-on-blackwell-gpus](/entities/drive-research-optimizing-qwen36-on-blackwell-gpus.md) (2026-06-08)
- A highly sparse MoE layer operating at a 1:50 activation ratio.
- Activates 8 routed experts and 1 shared expert out of a pool of 256 experts.
- Expert Parallelism (EP) shards MoE expert weights across GPUs.

## From [drive-research-32gb-vram-ai-reasoning-models-micro02](/entities/drive-research-32gb-vram-ai-reasoning-models-micro02.md) (2026-06-09)
- Architecture used by DeepSeek-R1.
- Architecture used by Llama 4 Scout.
- Architecture used by Kimi K2.6.

## From [drive-research-cuda-graph-capture-failure-workarounds-micro02](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro02.md) (2026-06-09)
- High-capacity MoE architectures expose critical defects in context checkpoint allocation.
- CPU-offloaded experts contribute to complex memory footprints.
- Can be deployed on hardware configurations with constrained physical memory.

## From [drive-research-linux-gaming-and-ai-build-guide-micro05](/entities/drive-research-linux-gaming-and-ai-build-guide-micro05.md) (2026-06-09)
- Architecture used by Llama 4 family.
- Inferences rapidly as only a small subset of parameters is active per token.
- Requires the entire parameter set to be loaded into VRAM simultaneously.

## From [drive-research-llamacpp-optimization-blueprint-micro02](/entities/drive-research-llamacpp-optimization-blueprint-micro02.md) (2026-06-09)
- For architectures utilizing MoE, tensor placement requires prioritization.
- The MoE automation logic in llama.cpp inherently prioritizes dense (non-sparse) tensors for VRAM allocation over the sparse MoE expert tensors.
- The --cpu-moe flag explicitly forces all MoE weights onto the CPU.

## From [optimizing-nvidia-blackwell-sm120-part3-micro05](/entities/optimizing-nvidia-blackwell-sm120-part3-micro05.md) (2026-06-09)
- High-capacity MoE architectures expose defects in context checkpoint allocation.
- Expert offloading parameters distribute weights across host and device.
- Context checkpoints accumulate in VRAM, leading to fragmentation.

## From [the-architecture-of-speculative-decoding-and-infer-part2-micro02](/entities/the-architecture-of-speculative-decoding-and-infer-part2-micro02.md) (2026-06-09)
- Architectures that achieve extreme efficiency through sparse activation.
- A routing mechanism selects only a fractional subset of expert neural networks to process data for any given token.
- Industry-wide transition toward these architectures.
