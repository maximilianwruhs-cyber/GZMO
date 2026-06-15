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
- [[unified-memory-architecture-uma|Unified Memory Architecture (UMA)]] (CONCEPT)
- [[pr-20472|PR #20472]] (PROJECT)
- [[windows|Windows]] (SYSTEM)
- [[ggml-cuda-disable-graphs|GGML_CUDA_DISABLE_GRAPHS]] (SYSTEM)
- [[linux|Linux]] (SYSTEM)
- [[pr-20313|PR #20313]] (PROJECT)
- [[msvc-toolchain|MSVC toolchain]] (TOOL)
- [[pr-22133|PR #22133]] (PROJECT)
- [[common-arg-cpp|common/arg.cpp]] (SYSTEM)
- [[nvidia-rtx-5070-ti|NVIDIA RTX 5070 Ti]] (SYSTEM)
- [[rocm|ROCm]] (SYSTEM)
- [[amdgpu-gttsize|amdgpu.gttsize]] (CONCEPT)
- [[apple-silicon|Apple Silicon]] (SYSTEM)
- [[hip|HIP]] (SYSTEM)
- [[llama-params-fit|llama_params_fit]] (TOOL)
- [[ggml-sched-max-split-inputs|GGML_SCHED_MAX_SPLIT_INPUTS]] (CONCEPT)
- [[multi-token-prediction-mtp|Multi-Token Prediction (MTP)]] (CONCEPT)
- [[vulkan|Vulkan]] (SYSTEM)
- [[ggml-backend-sched-reserve|ggml_backend_sched_reserve]] (TOOL)
- [[llama-server|llama-server]] (SYSTEM)
- [[llama-context-type-mtp|LLAMA_CONTEXT_TYPE_MTP]] (CONCEPT)
- [[amd-rdna-hardware|AMD RDNA hardware]] (SYSTEM)
- [[nvidia-rtx-4090|NVIDIA RTX 4090]] (SYSTEM)
- [[qwen|Qwen]] (SYSTEM)
- [[gpu-memory-over-allocation-parameter-fitting-and-architectural-core-faults-in-llama-cpp|GPU Memory Over-Allocation, Parameter Fitting, and Architectural Core Faults in llama.cpp]] (SYSTEM)
- [[macos|macOS]] (SYSTEM)
- [[automated-parameter-fitting-subsystem|automated parameter-fitting subsystem]] (CONCEPT)
- [[std-stoull|std::stoull]] (TOOL)
- [[kernel-flash-attn-ext-vec|kernel_flash_attn_ext_vec]] (TOOL)
- [[mixture-of-experts-moe-models|Mixture of Experts (MoE) models]] (CONCEPT)
- [[amd-apus|AMD APUs]] (SYSTEM)
- [[pr-17368|PR #17368]] (PROJECT)
- [[pr-18679|PR #18679]] (PROJECT)
- [[mmproj-gguf|mmproj.gguf]] (SYSTEM)
- [[llp64-data-model|LLP64 data model]] (CONCEPT)
- [[metal-backend|Metal backend]] (SYSTEM)
- [[lp64-data-model|LP64 data model]] (CONCEPT)

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
