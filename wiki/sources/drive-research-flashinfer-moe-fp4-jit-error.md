---
type: source
title: drive-research-flashinfer-moe-fp4-jit-error
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-flashinfer-moe-fp4-jit-error

Ingested source summary (2026-06-08).

## Entities
- [CUDA 13.1](/entities/cuda-13-1.md) (SYSTEM)
- [NVIDIA RTX Pro 6000 GPUs](/entities/nvidia-rtx-pro-6000-gpus.md) (SYSTEM)
- [Sm89 (Ada Lovelace)](/entities/sm89-ada-lovelace.md) (SYSTEM)
- [FlashInfer](/entities/flashinfer.md) (TOOL)
- [CUDA 13.0](/entities/cuda-13-0.md) (SYSTEM)
- [Marlin W4A16](/entities/marlin-w4a16.md) (TOOL)
- [GemmGrouped::maximum_active_blocks()](/entities/gemmgrouped-maximum-active-blocks.md) (SYSTEM)
- [WSL2](/entities/wsl2.md) (SYSTEM)
- [dxgkrnl driver](/entities/dxgkrnl-driver.md) (SYSTEM)
- [RTX 5080](/entities/rtx-5080.md) (SYSTEM)
- [DGX Spark (SM121)](/entities/dgx-spark-sm121.md) (SYSTEM)
- [PCIe](/entities/pcie.md) (SYSTEM)
- [HBM3e](/entities/hbm3e.md) (SYSTEM)
- [SM80](/entities/sm80.md) (SYSTEM)
- [FP16](/entities/fp16.md) (CONCEPT)
- [Workstation Blackwell (SM120)](/entities/workstation-blackwell-sm120.md) (SYSTEM)
- [cublasLt.h](/entities/cublaslt-h.md) (SYSTEM)
- [CUDA Runtime package](/entities/cuda-runtime-package.md) (SYSTEM)
- [NVCCCompiler](/entities/nvcccompiler.md) (SYSTEM)
- [UNIFORM_BATCH CUDA graph support](/entities/uniform-batch-cuda-graph-support.md) (CONCEPT)
- [BF16](/entities/bf16.md) (CONCEPT)
- [Gemma 4 NVFP4](/entities/gemma-4-nvfp4.md) (PROJECT)
- [vLLM](/entities/vllm.md) (TOOL)
- [Grid Dependency Control (GDC)](/entities/grid-dependency-control-gdc.md) (CONCEPT)
- [CUDA Toolkit Development package](/entities/cuda-toolkit-development-package.md) (SYSTEM)
- [MoE](/entities/moe.md) (CONCEPT)
- [nvidia/Qwen3.5-397B-A17B-NVFP4](/entities/nvidia-qwen3-5-397b-a17b-nvfp4.md) (PROJECT)
- [Nemotron-3-Nano-30B-A3B-NVFP4](/entities/nemotron-3-nano-30b-a3b-nvfp4.md) (PROJECT)
- [cutlass/detail/helper_macros.hpp](/entities/cutlass-detail-helper-macros-hpp.md) (SYSTEM)
- [Tensor Memory Access (TMA)](/entities/tensor-memory-access-tma.md) (CONCEPT)
- [compute_120a](/entities/compute-120a.md) (CONCEPT)
- [FP8](/entities/fp8.md) (CONCEPT)
- [NVLink](/entities/nvlink.md) (SYSTEM)
- [NVRTCCompiler](/entities/nvrtccompiler.md) (SYSTEM)
- [Unified System Bus](/entities/unified-system-bus.md) (SYSTEM)
- [compute_120f](/entities/compute-120f.md) (CONCEPT)
- [Unified LPDDR5x](/entities/unified-lpddr5x.md) (SYSTEM)
- [CUDA 12.8](/entities/cuda-12-8.md) (SYSTEM)
- [RTX 5090](/entities/rtx-5090.md) (SYSTEM)
- [Datacenter Blackwell (SM100)](/entities/datacenter-blackwell-sm100.md) (SYSTEM)
- [TMA Warp-Specialized (WS) Grouped GEMM kernels](/entities/tma-warp-specialized-ws-grouped-gemm-kernels.md) (SYSTEM)
- [MiniMax-M2.1-NVFP4](/entities/minimax-m2-1-nvfp4.md) (PROJECT)
- [DeepSeek-V4-Flash](/entities/deepseek-v4-flash.md) (PROJECT)
- [GPT-OSS-120B MXFP4](/entities/gpt-oss-120b-mxfp4.md) (PROJECT)
- [GDDR7](/entities/gddr7.md) (SYSTEM)
- [NVIDIA Blackwell generation](/entities/nvidia-blackwell-generation.md) (SYSTEM)

## Relations
- FlashInfer → USES → MoE
- vLLM → USES → MoE
- Gemma 4 NVFP4 → RELATED_TO → MoE
- Workstation Blackwell (SM120) → RELATED_TO → Gemma 4 NVFP4
- FlashInfer → RELATED_TO → Workstation Blackwell (SM120)
- vLLM → RELATED_TO → Workstation Blackwell (SM120)
- cutlass/detail/helper_macros.hpp → RELATED_TO → FlashInfer
- Datacenter Blackwell (SM100) → RELATED_TO → Workstation Blackwell (SM120)
- NVIDIA RTX Pro 6000 GPUs → PART_OF → Workstation Blackwell (SM120)
- RTX 5090 → PART_OF → Workstation Blackwell (SM120)
- RTX 5080 → PART_OF → Workstation Blackwell (SM120)
- DGX Spark (SM121) → PART_OF → Workstation Blackwell (SM120)
- Datacenter Blackwell (SM100) → PART_OF → Workstation Blackwell (SM120)
- FlashInfer → USES → CUDA 13.1
- vLLM → USES → CUDA 13.1
- NVCCCompiler → USES → cutlass/detail/helper_macros.hpp
- NVRTCCompiler → USES → cutlass/detail/helper_macros.hpp
- TMA Warp-Specialized (WS) Grouped GEMM kernels → USES → Grid Dependency Control (GDC)
- TMA Warp-Specialized (WS) Grouped GEMM kernels → RELATED_TO → Workstation Blackwell (SM120)
- compute_120f → RELATED_TO → CUDA 13.0
- compute_120f → RELATED_TO → Workstation Blackwell (SM120)
- Marlin W4A16 → USES → FP16
- Marlin W4A16 → RELATED_TO → Gemma 4 NVFP4
- nvidia/Qwen3.5-397B-A17B-NVFP4 → RELATED_TO → Gemma 4 NVFP4
- WSL2 → USES → dxgkrnl driver
- UNIFORM_BATCH CUDA graph support → RELATED_TO → Workstation Blackwell (SM120)
- Workstation Blackwell (SM120) → RELATED_TO → SM80
- Datacenter Blackwell (SM100) → USES → HBM3e
- Workstation Blackwell (SM120) → USES → GDDR7
- Datacenter Blackwell (SM100) → USES → NVLink
- Workstation Blackwell (SM120) → USES → PCIe
- FlashInfer → RELATED_TO → cublasLt.h
- NVCCCompiler → RELATED_TO → cutlass/detail/helper_macros.hpp
- MiniMax-M2.1-NVFP4 → RELATED_TO → Workstation Blackwell (SM120)
- Nemotron-3-Nano-30B-A3B-NVFP4 → RELATED_TO → Workstation Blackwell (SM120)
- Tensor Memory Access (TMA) → USES → Grid Dependency Control (GDC)
- compute_120a → RELATED_TO → Workstation Blackwell (SM120)
- FP8 → RELATED_TO → MoE
- FP16 → USES → Marlin W4A16
- BF16 → RELATED_TO → Gemma 4 NVFP4
- Gemma 4 NVFP4 → USES → vLLM
