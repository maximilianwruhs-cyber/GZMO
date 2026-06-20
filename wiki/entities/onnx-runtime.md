---
type: entity
title: ONNX Runtime
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# ONNX Runtime

Type: SYSTEM

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- Local inference engine.
- Can be integrated into the agent for LLM execution.
- Its backend provider can be configured to utilize CPU threads based on passive probes.

## From [drive-research-enhancing-local-ai-hypervisor-architecture](/entities/drive-research-enhancing-local-ai-hypervisor-architecture.md) (2026-06-08)
- Can be used with FastEmbed.
- Supports CUDAExecutionProvider.
- Bypasses host CPU thread queues.
- Is used with FastEmbed for hardware-accelerated embeddings.
- Supports CUDAExecutionProvider for FastEmbed.
- Handles tokenization and float array generation.

## From [drive-research-license-and-native-binding-analysis](/entities/drive-research-license-and-native-binding-analysis.md) (2026-06-08)
- Utilized by transformers.js.
- Compiled to WebAssembly.
- Enables machine learning model execution.

## From [drive-research-architecting-zero-configuration-portable-agents-s-micro03](/entities/drive-research-architecting-zero-configuration-portable-agents-s-micro03.md) (2026-06-09)
- An example of a local inference engine.
- Agent's internal logic configures the runtime backend provider to utilize CPU threads.
