---
type: source
title: drive-research-what-else-can-directly-be-aligned-with-our-common
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-what-else-can-directly-be-aligned-with-our-common

Ingested source summary (2026-06-08).

## Entities
- [[redundant-zero-remapping-razer|Redundant Zero Remapping (RaZeR)]] (CONCEPT)
- [[custom-k-64-tile-templates|Custom K=64 Tile Templates]] (CONCEPT)
- [[blackwell-native-microscaling-nv-formats|Blackwell Native Microscaling (NV Formats)]] (CONCEPT)
- [[linux-stack|Linux stack]] (SYSTEM)
- [[generate-kernels-py|generate_kernels.py]] (TOOL)
- [[pypi-nvidia-cuda-runtime-cu13-packages|PyPI nvidia-cuda-runtime-cu13 packages]] (TOOL)
- [[sm120-streaming-multiprocessors|SM120 streaming multiprocessors]] (SYSTEM)
- [[cutlass|CUTLASS]] (TOOL)
- [[libcudart-so|libcudart.so]] (SYSTEM)
- [[ocp-compliant-mx-formats|OCP-compliant MX formats]] (CONCEPT)
- [[sm120-blockscaled-mma-builder-inl|sm120_blockscaled_mma_builder.inl]] (TOOL)
- [[fp4-representations|FP4 representations]] (CONCEPT)
- [[float-ue8m0-t|float_ue8m0_t]] (CONCEPT)
- [[fp8-block-scaling-factors|FP8 block scaling factors]] (CONCEPT)
- [[multi-precision-split-decomposition-msd|Multi-Precision Split Decomposition (MSD)]] (CONCEPT)
- [[bf16|BF16]] (CONCEPT)
- [[cutile-dsl|cuTile DSL]] (TOOL)
- [[float-ue4m3-t|float_ue4m3_t]] (CONCEPT)
- [[tileir|TileIR]] (CONCEPT)
- [[cuda-13-x-nightly-pip-wheels|CUDA 13.x nightly pip wheels]] (TOOL)
- [[gcc|GCC]] (TOOL)

## Relations
- Blackwell Native Microscaling (NV Formats) → RELATED_TO → OCP-compliant MX formats
- Blackwell Native Microscaling (NV Formats) → RELATED_TO → float_ue4m3_t
- OCP-compliant MX formats → RELATED_TO → float_ue8m0_t
- Custom K=64 Tile Templates → RELATED_TO → CUTLASS
- Custom K=64 Tile Templates → RELATED_TO → sm120_blockscaled_mma_builder.inl
- Custom K=64 Tile Templates → RELATED_TO → generate_kernels.py
- Redundant Zero Remapping (RaZeR) → RELATED_TO → FP4 representations
- Redundant Zero Remapping (RaZeR) → RELATED_TO → FP8 block scaling factors
- Multi-Precision Split Decomposition (MSD) → RELATED_TO → SM120 streaming multiprocessors
- Multi-Precision Split Decomposition (MSD) → RELATED_TO → BF16
- cuTile DSL → RELATED_TO → TileIR
- CUDA 13.x nightly pip wheels → USES → Linux stack
- PyPI nvidia-cuda-runtime-cu13 packages → RELATED_TO → GCC
- GCC → USES → libcudart.so
