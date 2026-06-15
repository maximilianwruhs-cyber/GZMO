---
type: entity
title: nvidia-smi
created: 2026-06-08
updated: 2026-06-10
sources: 7
tags: []
status: draft
gzmo_synthetic: true
---







# nvidia-smi

Type: TOOL

## From [[drive-research-enhancing-local-ai-hypervisor-architecture|drive-research-enhancing-local-ai-hypervisor-architecture]] (2026-06-08)
- Confirms successful host-level driver compilation and execution.
- Exposes the running driver version, CUDA compilation profile, and physical state metrics of the GPU.
- Can be run inside an LXC console to verify GPU mapping.

## From [[drive-research-ok-so-designing-a-guide-around-llamabench-would-b|drive-research-ok-so-designing-a-guide-around-llamabench-would-b]] (2026-06-08)
- Used to query power draw via nvidia-smi.
- Queries power draw in a tight loop.

## From [[drive-research-to-product-engineering-leadership|drive-research-to-product-engineering-leadership]] (2026-06-08)
- Example of a command that creates suspicious child-process anomalies if spawned by the agent.

## From [[phantom-drive-autonomous-llm-deployment-architect-micro01|phantom-drive-autonomous-llm-deployment-architect-micro01]] (2026-06-09)
- Used to query GPU memory.
- Wrapped within a timeout to prevent driver hangs.
- Queries for memory.total and memory.free.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05]] (2026-06-09)
- Facilitates real-time performance monitoring of GPU.

## From [[drive-research-research-process-steps-micro03|drive-research-research-process-steps-micro03]] (2026-06-10)
- A CLI tool used to log real-time power metrics for NVIDIA GPUs.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro02|optimizing-nvidia-blackwell-sm120-part3-micro02]] (2026-06-10)
- Used to log real-time power metrics on NVIDIA GPUs
