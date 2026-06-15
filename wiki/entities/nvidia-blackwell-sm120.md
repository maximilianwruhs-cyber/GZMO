---
type: entity
title: NVIDIA Blackwell SM120
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# NVIDIA Blackwell SM120

Type: SYSTEM

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- A GPU architecture.
- Establishes a division between datacenter-class compute engines and local workstation/consumer accelerators.
- Requires abandoning development patterns designed for datacenter accelerators (sm_100 capability).
- RTX Blackwell
- DGX Spark
- microarchitecture
- B200 Datasheet
- Desktop Blackwell
- Hopper
- Introduces distinct compute capabilities, execution models, and memory systems.
- Datacenter-class accelerators are designated under compute capability 10.0 (SM100).
- Workstation and consumer-grade GPUs are referred to as SM120 and SM121.
- NVIDIA DGX Spark
- NVIDIA Developer Forums
- NVIDIA CUTLASS Documentation
- NVIDIA On-Demand
- NVIDIA Blackwell
- NVIDIA GPU architecture
- NVIDIA/cutlass
- Consumer-tier architecture.
- Relies on an extended flavor of mma.sync instructions.
- Register allocation is a primary performance bottleneck.
- compute_120f (CUDA 13.0)
- SM100 GEMMs
- SM120 FP8 GEMM

## From [[optimizing-nvidia-blackwell-sm120-part3-micro06|optimizing-nvidia-blackwell-sm120-part3-micro06]] (2026-06-09)
- It is a system related to optimizing performance.
- The document is part 3 of optimizing this system.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro01|optimizing-nvidia-blackwell-sm120-part2-micro01]] (2026-06-10)
- Subject of optimization research
