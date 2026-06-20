---
type: source
title: drive-research-llamacpp-gpu-memory-reporting-bug
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-llamacpp-gpu-memory-reporting-bug

Ingested source summary (2026-06-08).

## Entities
- [Unified Memory Architecture (UMA)](/entities/unified-memory-architecture-uma.md) (CONCEPT)
- [PR #20472](/entities/pr-20472.md) (PROJECT)
- [Windows](/entities/windows.md) (SYSTEM)
- [GGML_CUDA_DISABLE_GRAPHS](/entities/ggml-cuda-disable-graphs.md) (SYSTEM)
- [Linux](/entities/linux.md) (SYSTEM)
- [PR #20313](/entities/pr-20313.md) (PROJECT)
- [MSVC toolchain](/entities/msvc-toolchain.md) (TOOL)
- [PR #22133](/entities/pr-22133.md) (PROJECT)
- [common/arg.cpp](/entities/common-arg-cpp.md) (SYSTEM)
- [NVIDIA RTX 5070 Ti](/entities/nvidia-rtx-5070-ti.md) (SYSTEM)
- [ROCm](/entities/rocm.md) (SYSTEM)
- [amdgpu.gttsize](/entities/amdgpu-gttsize.md) (CONCEPT)
- [Apple Silicon](/entities/apple-silicon.md) (SYSTEM)
- [HIP](/entities/hip.md) (SYSTEM)
- [llama_params_fit](/entities/llama-params-fit.md) (TOOL)
- [GGML_SCHED_MAX_SPLIT_INPUTS](/entities/ggml-sched-max-split-inputs.md) (CONCEPT)
- [Multi-Token Prediction (MTP)](/entities/multi-token-prediction-mtp.md) (CONCEPT)
- [Vulkan](/entities/vulkan.md) (SYSTEM)
- [ggml_backend_sched_reserve](/entities/ggml-backend-sched-reserve.md) (TOOL)
- [llama-server](/entities/llama-server.md) (SYSTEM)
- [LLAMA_CONTEXT_TYPE_MTP](/entities/llama-context-type-mtp.md) (CONCEPT)
- [AMD RDNA hardware](/entities/amd-rdna-hardware.md) (SYSTEM)
- [NVIDIA RTX 4090](/entities/nvidia-rtx-4090.md) (SYSTEM)
- [Qwen](/entities/qwen.md) (SYSTEM)
- [GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp](/entities/gpu-memory-over-allocation-parameter-fitting-and-architectural-core-faults-in-llama-cpp.md) (SYSTEM)
- [macOS](/entities/macos.md) (SYSTEM)
- [automated parameter-fitting subsystem](/entities/automated-parameter-fitting-subsystem.md) (CONCEPT)
- [std::stoull](/entities/std-stoull.md) (TOOL)
- [kernel_flash_attn_ext_vec](/entities/kernel-flash-attn-ext-vec.md) (TOOL)
- [Mixture of Experts (MoE) models](/entities/mixture-of-experts-moe-models.md) (CONCEPT)
- [AMD APUs](/entities/amd-apus.md) (SYSTEM)
- [PR #17368](/entities/pr-17368.md) (PROJECT)
- [PR #18679](/entities/pr-18679.md) (PROJECT)
- [mmproj.gguf](/entities/mmproj-gguf.md) (SYSTEM)
- [LLP64 data model](/entities/llp64-data-model.md) (CONCEPT)
- [Metal backend](/entities/metal-backend.md) (SYSTEM)
- [LP64 data model](/entities/lp64-data-model.md) (CONCEPT)

## Relations
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → automated parameter-fitting subsystem
- automated parameter-fitting subsystem → RELATED_TO → llama_params_fit
- llama_params_fit → USES → HIP
- llama_params_fit → USES → Vulkan
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → Windows
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → Linux
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → macOS
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → MSVC toolchain
- Windows → USES → LLP64 data model
- Linux → USES → LP64 data model
- macOS → USES → LP64 data model
- common/arg.cpp → USES → Windows
- PR #20313 → USES → std::stoull
- Multi-Token Prediction (MTP) → RELATED_TO → Qwen
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → Multi-Token Prediction (MTP)
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → mmproj.gguf
- mmproj.gguf → USES → ggml_backend_sched_reserve
- llama-server → USES → GGML_CUDA_DISABLE_GRAPHS
- Unified Memory Architecture (UMA) → RELATED_TO → Apple Silicon
- Unified Memory Architecture (UMA) → RELATED_TO → AMD APUs
- AMD APUs → USES → amdgpu.gttsize
- macOS → USES → Metal backend
- kernel_flash_attn_ext_vec → USES → AMD RDNA hardware
- Mixture of Experts (MoE) models → RELATED_TO → GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp
- ROCm → RELATED_TO → HIP
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → GGML_CUDA_DISABLE_GRAPHS
- GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp → USES → GGML_SCHED_MAX_SPLIT_INPUTS
- PR #18679 → RELATED_TO → GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp
- PR #20313 → RELATED_TO → GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp
- PR #17368 → RELATED_TO → GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp
- PR #20472 → RELATED_TO → GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp
- PR #22133 → RELATED_TO → GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp
