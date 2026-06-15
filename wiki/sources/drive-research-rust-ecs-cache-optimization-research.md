---
type: source
title: drive-research-rust-ecs-cache-optimization-research
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-rust-ecs-cache-optimization-research

Ingested source summary (2026-06-08).

## Entities
- [[hashmap-agentid-usize|HashMap<AgentId, usize>]] (TOOL)
- [[swap-remove|swap_remove]] (TOOL)
- [[cudadevicereset|cudaDeviceReset()]] (TOOL)
- [[llama-cpp|llama.cpp]] (TOOL)
- [[paged-kv-cache|Paged KV Cache]] (CONCEPT)
- [[structure-of-arrays-soa|Structure of Arrays (SoA)]] (CONCEPT)
- [[cuda-driver|CUDA driver]] (SYSTEM)
- [[nvidia-system-management-interface-nvidia-smi|NVIDIA System Management Interface (nvidia-smi)]] (TOOL)
- [[soa-derive-crate|soa_derive Crate]] (TOOL)
- [[exclusive-process-mode|EXCLUSIVE_PROCESS Mode]] (CONCEPT)
- [[graph-reuse-gr|Graph Reuse (-gr)]] (CONCEPT)
- [[array-of-structures-aos|Array of Structures (AoS)]] (CONCEPT)
- [[numa-balancing-script-disable-numa-balancing-sh|NUMA Balancing Script (disable-numa-balancing.sh)]] (TOOL)
- [[unified-kv-cache|Unified KV Cache]] (CONCEPT)
- [[rtx-5070-ti|RTX 5070 Ti]] (SYSTEM)
- [[nvidia-collective-communications-library-nccl|NVIDIA Collective Communications Library (NCCL)]] (TOOL)
- [[soa-rs-crate|soa-rs Crate]] (TOOL)
- [[cudaerrordeviceunavailable|cudaErrorDeviceUnavailable]] (CONCEPT)
- [[deferred-deletion-command-buffer|Deferred Deletion (Command Buffer)]] (CONCEPT)
- [[tensor-parallelism|Tensor Parallelism]] (CONCEPT)
- [[intelligencehypervisor|IntelligenceHypervisor]] (SYSTEM)
- [[split-mode-graph-sm-graph|Split Mode Graph (-sm graph)]] (CONCEPT)
- [[cudafree-0|cudaFree(0)]] (TOOL)
- [[rust|Rust]] (TOOL)
- [[pipeline-layer-parallelism|Pipeline (Layer) Parallelism]] (CONCEPT)

## Relations
- IntelligenceHypervisor → USES → Structure of Arrays (SoA)
- IntelligenceHypervisor → RELATED_TO → Array of Structures (AoS)
- IntelligenceHypervisor → USES → HashMap<AgentId, usize>
- IntelligenceHypervisor → USES → Deferred Deletion (Command Buffer)
- IntelligenceHypervisor → USES → NVIDIA System Management Interface (nvidia-smi)
- IntelligenceHypervisor → USES → llama.cpp
- IntelligenceHypervisor → USES → Paged KV Cache
- soa_derive Crate → RELATED_TO → Structure of Arrays (SoA)
- soa-rs Crate → RELATED_TO → Structure of Arrays (SoA)
- NVIDIA System Management Interface (nvidia-smi) → USES → EXCLUSIVE_PROCESS Mode
- NVIDIA System Management Interface (nvidia-smi) → USES → RTX 5070 Ti
- EXCLUSIVE_PROCESS Mode → RELATED_TO → CUDA driver
- cudaFree(0) → USES → IntelligenceHypervisor
- cudaFree(0) → RELATED_TO → EXCLUSIVE_PROCESS Mode
- cudaDeviceReset() → USES → IntelligenceHypervisor
- llama.cpp → USES → Tensor Parallelism
- llama.cpp → RELATED_TO → Pipeline (Layer) Parallelism
- llama.cpp → USES → NVIDIA Collective Communications Library (NCCL)
- llama.cpp → USES → Split Mode Graph (-sm graph)
- llama.cpp → USES → Graph Reuse (-gr)
- llama.cpp → USES → NUMA Balancing Script (disable-numa-balancing.sh)
- llama.cpp → RELATED_TO → Unified KV Cache
- llama.cpp → USES → Paged KV Cache
- NVIDIA Collective Communications Library (NCCL) → RELATED_TO → Tensor Parallelism
- NVIDIA Collective Communications Library (NCCL) → RELATED_TO → RTX 5070 Ti
- Tensor Parallelism → RELATED_TO → Split Mode Graph (-sm graph)
- Paged KV Cache → RELATED_TO → Unified KV Cache
- Rust → RELATED_TO → Structure of Arrays (SoA)
- Rust → RELATED_TO → Array of Structures (AoS)
- Rust → RELATED_TO → HashMap<AgentId, usize>
- swap_remove → USES → IntelligenceHypervisor
- swap_remove → RELATED_TO → HashMap<AgentId, usize>
- Deferred Deletion (Command Buffer) → RELATED_TO → swap_remove
