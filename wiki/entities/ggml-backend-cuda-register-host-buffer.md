---
type: entity
title: ggml_backend_cuda_register_host_buffer
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# ggml_backend_cuda_register_host_buffer

Type: TOOL

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- Used for optimizing execution of large language models (LLMs) on local Linux workstations.
- In llama.cpp and GGML backends, memory registration behaves as a highly configurable pipeline.
- When llama.cpp loads a model, host memory registration is controlled by the environment variable GGML_CUDA_REGISTER_HOST and command-line execution flags such as --mlock and --no-mmap.
- The GGML backend catches failed registration and outputs a debugging or warning message.
- The GGML backend then falls back to non-pinned, slow pageable transfer paths, or the application crashes entirely.
- An environment variable that controls host memory registration in llama.cpp and GGML backends.
- If GGML_CUDA_REGISTER_HOST is defined and mmap is deactivated, the backend routes allocated host buffers through a specialized registration function.
- A specialized registration function invoked when GGML_CUDA_REGISTER_HOST is defined and mmap is deactivated.
- This function invokes cudaHostRegister using the flags cudaHostRegisterPortable and cudaHostRegisterReadOnly.
