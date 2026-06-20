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
- [Redundant Zero Remapping (RaZeR)](/entities/redundant-zero-remapping-razer.md) (CONCEPT)
- [Custom K=64 Tile Templates](/entities/custom-k-64-tile-templates.md) (CONCEPT)
- [Blackwell Native Microscaling (NV Formats)](/entities/blackwell-native-microscaling-nv-formats.md) (CONCEPT)
- [Linux stack](/entities/linux-stack.md) (SYSTEM)
- [generate_kernels.py](/entities/generate-kernels-py.md) (TOOL)
- [PyPI nvidia-cuda-runtime-cu13 packages](/entities/pypi-nvidia-cuda-runtime-cu13-packages.md) (TOOL)
- [SM120 streaming multiprocessors](/entities/sm120-streaming-multiprocessors.md) (SYSTEM)
- [CUTLASS](/entities/cutlass.md) (TOOL)
- [libcudart.so](/entities/libcudart-so.md) (SYSTEM)
- [OCP-compliant MX formats](/entities/ocp-compliant-mx-formats.md) (CONCEPT)
- [sm120_blockscaled_mma_builder.inl](/entities/sm120-blockscaled-mma-builder-inl.md) (TOOL)
- [FP4 representations](/entities/fp4-representations.md) (CONCEPT)
- [float_ue8m0_t](/entities/float-ue8m0-t.md) (CONCEPT)
- [FP8 block scaling factors](/entities/fp8-block-scaling-factors.md) (CONCEPT)
- [Multi-Precision Split Decomposition (MSD)](/entities/multi-precision-split-decomposition-msd.md) (CONCEPT)
- [BF16](/entities/bf16.md) (CONCEPT)
- [cuTile DSL](/entities/cutile-dsl.md) (TOOL)
- [float_ue4m3_t](/entities/float-ue4m3-t.md) (CONCEPT)
- [TileIR](/entities/tileir.md) (CONCEPT)
- [CUDA 13.x nightly pip wheels](/entities/cuda-13-x-nightly-pip-wheels.md) (TOOL)
- [GCC](/entities/gcc.md) (TOOL)

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
