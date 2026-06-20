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
- [sm120_blockscaled_mma_builder.inl](/entities/sm120-blockscaled-mma-builder-inl.md) (SYSTEM)
- [CUTLASS](/entities/cutlass.md) (TOOL)
- [MSD](/entities/msd.md) (CONCEPT)
- [FlashInfer](/entities/flashinfer.md) (TOOL)
- [RaZeR](/entities/razer.md) (CONCEPT)
- [BF16](/entities/bf16.md) (CONCEPT)
- [generate_kernels.py](/entities/generate-kernels-py.md) (TOOL)
- [Tensor Core](/entities/tensor-core.md) (SYSTEM)
- [FP4](/entities/fp4.md) (CONCEPT)
- [Marlin](/entities/marlin.md) (SYSTEM)
- [vLLM](/entities/vllm.md) (SYSTEM)
- [CUDA](/entities/cuda.md) (SYSTEM)
- [MTP](/entities/mtp.md) (CONCEPT)

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
