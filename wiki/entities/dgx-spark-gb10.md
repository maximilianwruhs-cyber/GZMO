---
type: entity
title: DGX Spark GB10
created: 2026-06-08
updated: 2026-06-10
sources: 7
tags: []
status: draft
gzmo_synthetic: true
---







# DGX Spark GB10

Type: SYSTEM

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- A platform with sm_120 and sm_121 compute capabilities.
- A consumer-grade Blackwell GPU.
- Is DGX Spark Actually Blackwell?
- DGX Spark / GB10

## From [[aether-grid-micro02|aether-grid-micro02]] (2026-06-09)
- real and available since 2025
- Specs: 20-Core Arm-CPU (10 Cortex-X925 + 10 Cortex-A725), Blackwell-GPU, 128 GB LPDDR5X unified memory (273 GB/s), 1 PFLOP FP4 AI-Performance, 4 TB NVMe, compact (150x150x50.5 mm), 240 W TDP, price from 4,699 USD
- 2026-Updates: 2.5x Performance through NVFP4-Quantization and Eagle3-Decoding
- Integration into Jetson AGX Orin via TensorRT-LLM v1.2
- Best Practice: BCM 11 for Provisioning, combined with TensorRT-LLM for local inference up to 200B models
- Edge-Hardware
- Asymmetric 20-Core Arm-CPU (10x Cortex-X925 + 10x Cortex-A725) combined with a dedicated Blackwell-GPU
- 128 GB LPDDR5X Unified Memory (273 GB/s bandwidth), 4 TB NVMe local storage, 240 W TDP at extremely compact dimensions (150x150x50.5 mm)
- Cost point: from approx. 4,699 USD per Node
- Delivers up to 1 PFLOP FP4 AI-Performance
- Current status: TensorRT-LLM (v1.2) enables a 2.5x performance gain compared to 2025 through NVFP4-Quantization and Eagle3-Speculative-Decoding
- Seamless ecosystem integration with existing Jetson AGX Orin fleets is now possible
- Best Practice (Fleet-management): Mandatory use of NVIDIA Base Command Manager (BCM 11) for zero-touch provisioning of edge nodes to prevent OS image drift in fleets >100 devices

## From [[aether-grid-micro03|aether-grid-micro03]] (2026-06-09)
- Cost is $4,699 per unit.
- Has a TDP of 240W.
- Can be cloned with 3 seconds of audio material using 2026-tools.
- Has highly sensitive microphone arrays.
- Will exist alongside older Jetson AGX Orin in long-term scaling.
- Edge nodes in Phase 4-6.
- Experiences thermal throttling at 35°C ambient temperature.
- Captures ultrasound attacks with sensitive microphone arrays.
- Nodes in Phase 13 detect critical events.
- Nodes form a P2P mesh network.
- Edge hardware to be procured, budget starting from $4,699/Node.

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- Deep benchmarking on the Blackwell-class DGX Spark GB10 demonstrated architectural savings with KV cache quantization.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Is a workstation-class Blackwell GPU.
- Represents the SM121 variant.

## From [[drive-research-marlin-baseline-for-early-deployments-micro01|drive-research-marlin-baseline-for-early-deployments-micro01]] (2026-06-10)
- Workstation-class Blackwell GPU.
- Part of the SM121 architecture.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro02|optimizing-nvidia-blackwell-sm120-part2-micro02]] (2026-06-10)
- Hardware platform utilizing sm_121 compute capability.
