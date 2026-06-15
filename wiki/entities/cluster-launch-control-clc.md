---
type: entity
title: Cluster Launch Control (CLC)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Cluster Launch Control (CLC)

Type: SYSTEM

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- Hardware-supported implementation of dynamic persistent tile scheduling on Blackwell.
- Driven by clusterlaunchcontrol PTX instructions.
- CUTLASS implements the PipelineCLCFetchAsync class to manage CLC.
- Blackwell Cluster Launch Control
- Dynamic persistent tile scheduling with Cluster Launch Control
