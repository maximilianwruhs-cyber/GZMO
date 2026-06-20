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
- [HashMap<AgentId, usize>](/entities/hashmap-agentid-usize.md) (TOOL)
- [swap_remove](/entities/swap-remove.md) (TOOL)
- [cudaDeviceReset()](/entities/cudadevicereset.md) (TOOL)
- [llama.cpp](/entities/llama-cpp.md) (TOOL)
- [Paged KV Cache](/entities/paged-kv-cache.md) (CONCEPT)
- [Structure of Arrays (SoA)](/entities/structure-of-arrays-soa.md) (CONCEPT)
- [CUDA driver](/entities/cuda-driver.md) (SYSTEM)
- [NVIDIA System Management Interface (nvidia-smi)](/entities/nvidia-system-management-interface-nvidia-smi.md) (TOOL)
- [soa_derive Crate](/entities/soa-derive-crate.md) (TOOL)
- [EXCLUSIVE_PROCESS Mode](/entities/exclusive-process-mode.md) (CONCEPT)
- [Graph Reuse (-gr)](/entities/graph-reuse-gr.md) (CONCEPT)
- [Array of Structures (AoS)](/entities/array-of-structures-aos.md) (CONCEPT)
- [NUMA Balancing Script (disable-numa-balancing.sh)](/entities/numa-balancing-script-disable-numa-balancing-sh.md) (TOOL)
- [Unified KV Cache](/entities/unified-kv-cache.md) (CONCEPT)
- [RTX 5070 Ti](/entities/rtx-5070-ti.md) (SYSTEM)
- [NVIDIA Collective Communications Library (NCCL)](/entities/nvidia-collective-communications-library-nccl.md) (TOOL)
- [soa-rs Crate](/entities/soa-rs-crate.md) (TOOL)
- [cudaErrorDeviceUnavailable](/entities/cudaerrordeviceunavailable.md) (CONCEPT)
- [Deferred Deletion (Command Buffer)](/entities/deferred-deletion-command-buffer.md) (CONCEPT)
- [Tensor Parallelism](/entities/tensor-parallelism.md) (CONCEPT)
- [IntelligenceHypervisor](/entities/intelligencehypervisor.md) (SYSTEM)
- [Split Mode Graph (-sm graph)](/entities/split-mode-graph-sm-graph.md) (CONCEPT)
- [cudaFree(0)](/entities/cudafree-0.md) (TOOL)
- [Rust](/entities/rust.md) (TOOL)
- [Pipeline (Layer) Parallelism](/entities/pipeline-layer-parallelism.md) (CONCEPT)

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
