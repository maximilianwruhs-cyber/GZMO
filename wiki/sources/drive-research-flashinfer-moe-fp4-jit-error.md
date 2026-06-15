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
- [[cuda-13-1|CUDA 13.1]] (SYSTEM)
- [[nvidia-rtx-pro-6000-gpus|NVIDIA RTX Pro 6000 GPUs]] (SYSTEM)
- [[sm89-ada-lovelace|Sm89 (Ada Lovelace)]] (SYSTEM)
- [[flashinfer|FlashInfer]] (TOOL)
- [[cuda-13-0|CUDA 13.0]] (SYSTEM)
- [[marlin-w4a16|Marlin W4A16]] (TOOL)
- [[gemmgrouped-maximum-active-blocks|GemmGrouped::maximum_active_blocks()]] (SYSTEM)
- [[wsl2|WSL2]] (SYSTEM)
- [[dxgkrnl-driver|dxgkrnl driver]] (SYSTEM)
- [[rtx-5080|RTX 5080]] (SYSTEM)
- [[dgx-spark-sm121|DGX Spark (SM121)]] (SYSTEM)
- [[pcie|PCIe]] (SYSTEM)
- [[hbm3e|HBM3e]] (SYSTEM)
- [[sm80|SM80]] (SYSTEM)
- [[fp16|FP16]] (CONCEPT)
- [[workstation-blackwell-sm120|Workstation Blackwell (SM120)]] (SYSTEM)
- [[cublaslt-h|cublasLt.h]] (SYSTEM)
- [[cuda-runtime-package|CUDA Runtime package]] (SYSTEM)
- [[nvcccompiler|NVCCCompiler]] (SYSTEM)
- [[uniform-batch-cuda-graph-support|UNIFORM_BATCH CUDA graph support]] (CONCEPT)
- [[bf16|BF16]] (CONCEPT)
- [[gemma-4-nvfp4|Gemma 4 NVFP4]] (PROJECT)
- [[vllm|vLLM]] (TOOL)
- [[grid-dependency-control-gdc|Grid Dependency Control (GDC)]] (CONCEPT)
- [[cuda-toolkit-development-package|CUDA Toolkit Development package]] (SYSTEM)
- [[moe|MoE]] (CONCEPT)
- [[nvidia-qwen3-5-397b-a17b-nvfp4|nvidia/Qwen3.5-397B-A17B-NVFP4]] (PROJECT)
- [[nemotron-3-nano-30b-a3b-nvfp4|Nemotron-3-Nano-30B-A3B-NVFP4]] (PROJECT)
- [[cutlass-detail-helper-macros-hpp|cutlass/detail/helper_macros.hpp]] (SYSTEM)
- [[tensor-memory-access-tma|Tensor Memory Access (TMA)]] (CONCEPT)
- [[compute-120a|compute_120a]] (CONCEPT)
- [[fp8|FP8]] (CONCEPT)
- [[nvlink|NVLink]] (SYSTEM)
- [[nvrtccompiler|NVRTCCompiler]] (SYSTEM)
- [[unified-system-bus|Unified System Bus]] (SYSTEM)
- [[compute-120f|compute_120f]] (CONCEPT)
- [[unified-lpddr5x|Unified LPDDR5x]] (SYSTEM)
- [[cuda-12-8|CUDA 12.8]] (SYSTEM)
- [[rtx-5090|RTX 5090]] (SYSTEM)
- [[datacenter-blackwell-sm100|Datacenter Blackwell (SM100)]] (SYSTEM)
- [[tma-warp-specialized-ws-grouped-gemm-kernels|TMA Warp-Specialized (WS) Grouped GEMM kernels]] (SYSTEM)
- [[minimax-m2-1-nvfp4|MiniMax-M2.1-NVFP4]] (PROJECT)
- [[deepseek-v4-flash|DeepSeek-V4-Flash]] (PROJECT)
- [[gpt-oss-120b-mxfp4|GPT-OSS-120B MXFP4]] (PROJECT)
- [[gddr7|GDDR7]] (SYSTEM)
- [[nvidia-blackwell-generation|NVIDIA Blackwell generation]] (SYSTEM)

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
