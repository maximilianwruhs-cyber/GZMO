---
type: entity
title: PCIe
created: 2026-06-08
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# PCIe

Type: SYSTEM

## From [drive-research-flashinfer-moe-fp4-jit-error](/entities/drive-research-flashinfer-moe-fp4-jit-error.md) (2026-06-08)
- Host interconnect for Workstation Blackwell (SM120)

## From [drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01](/entities/drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01.md) (2026-06-09)
- System interface that streams model weights from system RAM to accelerator compute cores.
- Dynamic downgrading of PCIe lane width can reduce weight transfer bandwidth.
- Bottlenecking can drop prefill speeds significantly.

## From [drive-research-cuda-graph-capture-failure-workarounds-micro02](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro02.md) (2026-06-09)
- Asymmetric PCIe lanes can disable direct Peer-to-Peer (P2P) memory access.
- Cross-GPU key-value cache transfers must be host-staged through system memory when P2P is disabled.

## From [drive-research-llamacpp-optimization-blueprint-micro02](/entities/drive-research-llamacpp-optimization-blueprint-micro02.md) (2026-06-09)
- Traversing the PCIe bus for layer-to-layer activations incurs severe latency penalties.
- Row Mode requires exceptionally high-speed interconnects—such as NVLink—to rapidly synchronize partial results over standard PCIe topologies without peer-to-peer (P2P) enablement.

## From [drive-research-ubuntu-extreme-hardware-tuning-micro01](/entities/drive-research-ubuntu-extreme-hardware-tuning-micro01.md) (2026-06-09)
- Interface for connecting components like GPUs and SSDs.
- Gen 5 x16 interface can be bifurcated.
- Bandwidth affects data transfer times.
