---
type: source
title: optimizing-nvidia-blackwell-sm120-part3-micro03
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# optimizing-nvidia-blackwell-sm120-part3-micro03

Ingested source summary (2026-06-10).

## Entities
- [Multi-Token Prediction](/entities/multi-token-prediction.md) (CONCEPT)
- [Metal](/entities/metal.md) (SYSTEM)
- [llama_params_fit](/entities/llama-params-fit.md) (SYSTEM)
- [Qwen 3](/entities/qwen-3.md) (SYSTEM)
- [FlashAttention](/entities/flashattention.md) (SYSTEM)
- [Vulkan](/entities/vulkan.md) (SYSTEM)
- [llama.cpp](/entities/llama-cpp.md) (SYSTEM)
- [--fit-target](/entities/fit-target.md) (TOOL)
- [HIP](/entities/hip.md) (SYSTEM)
- [--fit-ctx](/entities/fit-ctx.md) (TOOL)
- [MSVC](/entities/msvc.md) (TOOL)
- [Windows](/entities/windows.md) (SYSTEM)
- [Apple Silicon](/entities/apple-silicon.md) (SYSTEM)
- [CUDA](/entities/cuda.md) (SYSTEM)
- [macOS](/entities/macos.md) (SYSTEM)
- [--override-tensor](/entities/override-tensor.md) (TOOL)
- [AMD APU](/entities/amd-apu.md) (SYSTEM)

## Relations
- llama_params_fit → PART_OF → llama.cpp
- llama.cpp → USES → CUDA
- llama.cpp → USES → HIP
- llama.cpp → USES → Vulkan
- llama.cpp → USES → Metal
- llama.cpp → USES → llama_params_fit
- llama.cpp → USES → --fit-target
- llama.cpp → USES → --fit-ctx
- llama.cpp → USES → --override-tensor
- Qwen 3 → USES → Multi-Token Prediction
