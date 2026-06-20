---
type: source
title: drive-research-cuda-memory-locking-limits-configuration
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-cuda-memory-locking-limits-configuration

Ingested source summary (2026-06-08).

## Entities
- [RLIMIT_MEMLOCK](/entities/rlimit-memlock.md) (CONCEPT)
- [vm.swappiness=10](/entities/vm-swappiness-10.md) (CONCEPT)
- [cudaHostRegister](/entities/cudahostregister.md) (TOOL)
- [developer](/entities/developer.md) (PERSON)
- [page cache starvation](/entities/page-cache-starvation.md) (CONCEPT)
- [/etc/sysctl.d/99-swappiness.conf](/entities/etc-sysctl-d-99-swappiness-conf.md) (TOOL)
- [OOM killer](/entities/oom-killer.md) (SYSTEM)
- [ulimit -l](/entities/ulimit-l.md) (TOOL)
- [context window length](/entities/context-window-length.md) (CONCEPT)
- [pam_limits.so module](/entities/pam-limits-so-module.md) (TOOL)
- [GGUF model scales](/entities/gguf-model-scales.md) (CONCEPT)
- [pageable caches](/entities/pageable-caches.md) (CONCEPT)
- [OpenAI-compatible API server](/entities/openai-compatible-api-server.md) (TOOL)
- [filesystem cache buffering](/entities/filesystem-cache-buffering.md) (CONCEPT)
- [ulimit -Hl](/entities/ulimit-hl.md) (TOOL)
- [cudaHostRegisterPortable](/entities/cudahostregisterportable.md) (CONCEPT)
- [ENOMEM](/entities/enomem.md) (CONCEPT)
- [GGUF model weights](/entities/gguf-model-weights.md) (CONCEPT)
- [network socket queues](/entities/network-socket-queues.md) (CONCEPT)
- [ggml_backend_cuda_register_host_buffer](/entities/ggml-backend-cuda-register-host-buffer.md) (TOOL)
- [KV cache](/entities/kv-cache.md) (CONCEPT)
- [NVIDIA](/entities/nvidia.md) (ORGANIZATION)
- [AI workloads](/entities/ai-workloads.md) (CONCEPT)
- [SIGKILL (Signal 9)](/entities/sigkill-signal-9.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (TOOL)
- [Docker](/entities/docker.md) (TOOL)
- [display operations](/entities/display-operations.md) (CONCEPT)
- [quantization scheme](/entities/quantization-scheme.md) (CONCEPT)
- [limits.conf](/entities/limits-conf.md) (TOOL)
- [web browser](/entities/web-browser.md) (TOOL)
- [GCC](/entities/gcc.md) (TOOL)
- [PCIe bus](/entities/pcie-bus.md) (CONCEPT)
- [CUDA Runtime API](/entities/cuda-runtime-api.md) (SYSTEM)
- [LLM inference engine](/entities/llm-inference-engine.md) (TOOL)
- [machine learning workloads](/entities/machine-learning-workloads.md) (CONCEPT)
- [LimitMEMLOCK=infinity](/entities/limitmemlock-infinity.md) (CONCEPT)
- [CapabilityBoundingSet=CAP_IPC_LOCK](/entities/capabilityboundingset-cap-ipc-lock.md) (CONCEPT)
- [page-locked memory](/entities/page-locked-memory.md) (CONCEPT)
- [compilation](/entities/compilation.md) (CONCEPT)
- [anonymous memory pages](/entities/anonymous-memory-pages.md) (CONCEPT)
- [Direct Memory Access (DMA)](/entities/direct-memory-access-dma.md) (CONCEPT)
- [cudaHostRegisterReadOnly](/entities/cudahostregisterreadonly.md) (CONCEPT)
- [systemd](/entities/systemd.md) (SYSTEM)
- [vLLM](/entities/vllm.md) (TOOL)
- [Virtual Memory](/entities/virtual-memory.md) (CONCEPT)
- [oom_score](/entities/oom-score.md) (CONCEPT)
- [page fault](/entities/page-fault.md) (CONCEPT)
- [Google Takeout](/entities/google-takeout.md) (TOOL)
- [cudaErrorMemoryAllocation](/entities/cudaerrormemoryallocation.md) (CONCEPT)
- [vm.swappiness=1](/entities/vm-swappiness-1.md) (CONCEPT)
- [Qwen 2.5](/entities/qwen-2-5.md) (BOOK)
- [device driver state allocations](/entities/device-driver-state-allocations.md) (CONCEPT)
- [driver and heap overhead multiplier](/entities/driver-and-heap-overhead-multiplier.md) (CONCEPT)
- [ai-engineers](/entities/ai-engineers.md) (ORGANIZATION)
- [Podman](/entities/podman.md) (TOOL)
- [Llama 3](/entities/llama-3.md) (BOOK)
- [physical RAM](/entities/physical-ram.md) (CONCEPT)
- [Llama 2](/entities/llama-2.md) (BOOK)
- [Linux](/entities/linux.md) (SYSTEM)

## Relations
- llama.cpp → RELATED_TO → ggml_backend_cuda_register_host_buffer
- llama.cpp → USES → Linux
- Virtual Memory → PART_OF → Linux
- cudaHostRegister → USES → CUDA Runtime API
- cudaHostRegister → USES → Linux
- RLIMIT_MEMLOCK → PART_OF → Linux
- CapabilityBoundingSet=CAP_IPC_LOCK → PART_OF → Linux
- systemd → USES → Linux
- limits.conf → USES → Linux
- vm.swappiness=10 → PART_OF → Linux
- GGUF model scales → RELATED_TO → Virtual Memory
- page fault → RELATED_TO → Virtual Memory
- page-locked memory → RELATED_TO → CUDA Runtime API
- page-locked memory → RELATED_TO → Direct Memory Access (DMA)
- Direct Memory Access (DMA) → USES → PCIe bus
- OOM killer → PART_OF → Linux
- pam_limits.so module → USES → Linux
- Docker → USES → Linux
- Podman → USES → Linux
- KV cache → RELATED_TO → GGUF model scales
- NVIDIA → RELATED_TO → CUDA Runtime API
- ggml_backend_cuda_register_host_buffer → USES → cudaHostRegister
- cudaHostRegisterPortable → RELATED_TO → ggml_backend_cuda_register_host_buffer
- cudaHostRegisterReadOnly → RELATED_TO → ggml_backend_cuda_register_host_buffer
- ENOMEM → RELATED_TO → Linux
- cudaErrorMemoryAllocation → RELATED_TO → CUDA Runtime API
- SIGKILL (Signal 9) → RELATED_TO → OOM killer
- vm.swappiness=10 → RELATED_TO → Linux
- vm.swappiness=1 → RELATED_TO → Linux
- quantization scheme → RELATED_TO → GGUF model scales
- context window length → RELATED_TO → KV cache
- LLM inference engine → USES → systemd
- OpenAI-compatible API server → RELATED_TO → LLM inference engine
- LimitMEMLOCK=infinity → RELATED_TO → systemd
- CapabilityBoundingSet=CAP_IPC_LOCK → RELATED_TO → systemd
- GGUF model weights → RELATED_TO → GGUF model scales
- driver and heap overhead multiplier → RELATED_TO → GGUF model scales
- Qwen 2.5 → RELATED_TO → GGUF model scales
- physical RAM → RELATED_TO → Linux
- filesystem cache buffering → RELATED_TO → Linux
- network socket queues → RELATED_TO → Linux
- device driver state allocations → RELATED_TO → Linux
- page cache starvation → RELATED_TO → physical RAM
- oom_score → RELATED_TO → OOM killer
- /etc/sysctl.d/99-swappiness.conf → USES → Linux
- anonymous memory pages → RELATED_TO → Linux
- pageable caches → RELATED_TO → Linux
- AI workloads → RELATED_TO → Linux
- machine learning workloads → RELATED_TO → Linux
- developer → USES → Linux
- ulimit -l → USES → Linux
- ulimit -Hl → USES → Linux
- web browser → USES → Linux
- compilation → RELATED_TO → Linux
- display operations → RELATED_TO → Linux
- GCC → USES → compilation
