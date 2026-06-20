---
type: entity
title: RadixAttention
created: 2026-06-08
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---






# RadixAttention

Type: CONCEPT

## From [drive-research-enhancing-local-ai-hypervisor-architecture](/entities/drive-research-enhancing-local-ai-hypervisor-architecture.md) (2026-06-08)
- Caches Key-Value (KV) states as a dynamic radix tree.
- Reuses cached KV states for matched prefixes.
- Manages memory allocations in page-aligned structures.
- Mentioned in the SGLang Learning Series.
- Part of SGLang's concepts.

## From [drive-research-32gb-vram-ai-reasoning-models-micro02](/entities/drive-research-32gb-vram-ai-reasoning-models-micro02.md) (2026-06-09)
- Utilized by SGLang.

## From [drive-research-llm-inference-engine-audit-2026-micro02](/entities/drive-research-llm-inference-engine-audit-2026-micro02.md) (2026-06-09)
- Mechanism used by SGLang.
- Effortlessly navigates complex context-sharing required by agentic workflows and retrieval-augmented generation pipelines.

## From [drive-research-llm-inference-engine-audit-2026-micro01](/entities/drive-research-llm-inference-engine-audit-2026-micro01.md) (2026-06-10)
- Organizes KV cache using a radix tree structure
- Enables automatic token-level prefix caching

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro03](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro03.md) (2026-06-10)
- Used by SGLang to provide throughput gains for shared prompt prefixes.
