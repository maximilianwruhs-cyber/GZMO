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
- [[multi-token-prediction|Multi-Token Prediction]] (CONCEPT)
- [[metal|Metal]] (SYSTEM)
- [[llama-params-fit|llama_params_fit]] (SYSTEM)
- [[qwen-3|Qwen 3]] (SYSTEM)
- [[flashattention|FlashAttention]] (SYSTEM)
- [[vulkan|Vulkan]] (SYSTEM)
- [[llama-cpp|llama.cpp]] (SYSTEM)
- [[fit-target|--fit-target]] (TOOL)
- [[hip|HIP]] (SYSTEM)
- [[fit-ctx|--fit-ctx]] (TOOL)
- [[msvc|MSVC]] (TOOL)
- [[windows|Windows]] (SYSTEM)
- [[apple-silicon|Apple Silicon]] (SYSTEM)
- [[cuda|CUDA]] (SYSTEM)
- [[macos|macOS]] (SYSTEM)
- [[override-tensor|--override-tensor]] (TOOL)
- [[amd-apu|AMD APU]] (SYSTEM)

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
