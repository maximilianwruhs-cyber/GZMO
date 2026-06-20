---
type: entity
title: Remote Procedure Call (RPC)
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Remote Procedure Call (RPC)

Type: CONCEPT

## From [drive-research-hermes-session-storage-migration-analysis](/entities/drive-research-hermes-session-storage-migration-analysis.md) (2026-06-08)
- Used for communication between the primary orchestrator and sub-agents in Hermes.
- Implemented via Python scripts.

## From [drive-research-cuda-graph-capture-failure-workarounds-micro02](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro02.md) (2026-06-09)
- Distributed execution models utilizing an RPC interface frequently experience memory exhaustion.
- Client-side graph verification has a fundamental mismatch with server-side context management.
- Server-side RPC compute interface instantiates a fresh ggml_context for each execution call.

## From [optimizing-nvidia-blackwell-sm120-part3-micro05](/entities/optimizing-nvidia-blackwell-sm120-part3-micro05.md) (2026-06-09)
- Distributed execution models utilizing an RPC interface experience memory exhaustion.
- Caching mechanism for CUDA graphs breaks down under RPC.
- Server-side RPC compute interface instantiates fresh ggml_context.
