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
- [[rlimit-memlock|RLIMIT_MEMLOCK]] (CONCEPT)
- [[vm-swappiness-10|vm.swappiness=10]] (CONCEPT)
- [[cudahostregister|cudaHostRegister]] (TOOL)
- [[developer|developer]] (PERSON)
- [[page-cache-starvation|page cache starvation]] (CONCEPT)
- [[etc-sysctl-d-99-swappiness-conf|/etc/sysctl.d/99-swappiness.conf]] (TOOL)
- [[oom-killer|OOM killer]] (SYSTEM)
- [[ulimit-l|ulimit -l]] (TOOL)
- [[context-window-length|context window length]] (CONCEPT)
- [[pam-limits-so-module|pam_limits.so module]] (TOOL)
- [[gguf-model-scales|GGUF model scales]] (CONCEPT)
- [[pageable-caches|pageable caches]] (CONCEPT)
- [[openai-compatible-api-server|OpenAI-compatible API server]] (TOOL)
- [[filesystem-cache-buffering|filesystem cache buffering]] (CONCEPT)
- [[ulimit-hl|ulimit -Hl]] (TOOL)
- [[cudahostregisterportable|cudaHostRegisterPortable]] (CONCEPT)
- [[enomem|ENOMEM]] (CONCEPT)
- [[gguf-model-weights|GGUF model weights]] (CONCEPT)
- [[network-socket-queues|network socket queues]] (CONCEPT)
- [[ggml-backend-cuda-register-host-buffer|ggml_backend_cuda_register_host_buffer]] (TOOL)
- [[kv-cache|KV cache]] (CONCEPT)
- [[nvidia|NVIDIA]] (ORGANIZATION)
- [[ai-workloads|AI workloads]] (CONCEPT)
- [[sigkill-signal-9|SIGKILL (Signal 9)]] (CONCEPT)
- [[llama-cpp|llama.cpp]] (TOOL)
- [[docker|Docker]] (TOOL)
- [[display-operations|display operations]] (CONCEPT)
- [[quantization-scheme|quantization scheme]] (CONCEPT)
- [[limits-conf|limits.conf]] (TOOL)
- [[web-browser|web browser]] (TOOL)
- [[gcc|GCC]] (TOOL)
- [[pcie-bus|PCIe bus]] (CONCEPT)
- [[cuda-runtime-api|CUDA Runtime API]] (SYSTEM)
- [[llm-inference-engine|LLM inference engine]] (TOOL)
- [[machine-learning-workloads|machine learning workloads]] (CONCEPT)
- [[limitmemlock-infinity|LimitMEMLOCK=infinity]] (CONCEPT)
- [[capabilityboundingset-cap-ipc-lock|CapabilityBoundingSet=CAP_IPC_LOCK]] (CONCEPT)
- [[page-locked-memory|page-locked memory]] (CONCEPT)
- [[compilation|compilation]] (CONCEPT)
- [[anonymous-memory-pages|anonymous memory pages]] (CONCEPT)
- [[direct-memory-access-dma|Direct Memory Access (DMA)]] (CONCEPT)
- [[cudahostregisterreadonly|cudaHostRegisterReadOnly]] (CONCEPT)
- [[systemd|systemd]] (SYSTEM)
- [[vllm|vLLM]] (TOOL)
- [[virtual-memory|Virtual Memory]] (CONCEPT)
- [[oom-score|oom_score]] (CONCEPT)
- [[page-fault|page fault]] (CONCEPT)
- [[google-takeout|Google Takeout]] (TOOL)
- [[cudaerrormemoryallocation|cudaErrorMemoryAllocation]] (CONCEPT)
- [[vm-swappiness-1|vm.swappiness=1]] (CONCEPT)
- [[qwen-2-5|Qwen 2.5]] (BOOK)
- [[device-driver-state-allocations|device driver state allocations]] (CONCEPT)
- [[driver-and-heap-overhead-multiplier|driver and heap overhead multiplier]] (CONCEPT)
- [[ai-engineers|ai-engineers]] (ORGANIZATION)
- [[podman|Podman]] (TOOL)
- [[llama-3|Llama 3]] (BOOK)
- [[physical-ram|physical RAM]] (CONCEPT)
- [[llama-2|Llama 2]] (BOOK)
- [[linux|Linux]] (SYSTEM)

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
