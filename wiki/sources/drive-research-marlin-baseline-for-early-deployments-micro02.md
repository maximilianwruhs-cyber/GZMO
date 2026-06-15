---
type: source
title: drive-research-marlin-baseline-for-early-deployments-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-marlin-baseline-for-early-deployments-micro02

Ingested source summary (2026-06-09).

## Entities
- [[sm120-blockscaled-mma-builder-inl|sm120_blockscaled_mma_builder.inl]] (SYSTEM)
- [[cutlass|CUTLASS]] (TOOL)
- [[msd|MSD]] (CONCEPT)
- [[flashinfer|FlashInfer]] (TOOL)
- [[razer|RaZeR]] (CONCEPT)
- [[bf16|BF16]] (CONCEPT)
- [[generate-kernels-py|generate_kernels.py]] (TOOL)
- [[tensor-core|Tensor Core]] (SYSTEM)
- [[fp4|FP4]] (CONCEPT)
- [[marlin|Marlin]] (SYSTEM)
- [[vllm|vLLM]] (SYSTEM)
- [[cuda|CUDA]] (SYSTEM)
- [[mtp|MTP]] (CONCEPT)

## Relations
- Marlin → RELATED_TO → FP4
- CUTLASS → RELATED_TO → sm120_blockscaled_mma_builder.inl
- generate_kernels.py → PART_OF → CUTLASS
- RaZeR → RELATED_TO → FP4
- MSD → RELATED_TO → BF16
- MSD → USES → Tensor Core
- FlashInfer → RELATED_TO → FP4
- vLLM → RELATED_TO → sm120_blockscaled_mma_builder.inl
- vLLM → USES → Marlin
- sm120_blockscaled_mma_builder.inl → PART_OF → CUTLASS
- Marlin → RELATED_TO → MTP
