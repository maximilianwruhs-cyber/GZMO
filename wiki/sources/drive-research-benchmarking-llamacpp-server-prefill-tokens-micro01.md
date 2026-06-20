---
type: source
title: drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01

Ingested source summary (2026-06-09).

## Entities
- [llama-benchy](/entities/llama-benchy.md) (TOOL)
- [/metrics Endpoint](/entities/metrics-endpoint.md) (TOOL)
- [Grafana k6](/entities/grafana-k6.md) (TOOL)
- [Technical Characterization of Prompt Prefill Throughput and HTTP Benchmarking Methodologies in llama.cpp Servers](/entities/technical-characterization-of-prompt-prefill-throughput-and-http-benchmarking-methodologies-in-llama-cpp-servers.md) (BOOK)
- [JSON APIs](/entities/json-apis.md) (TOOL)
- [MLC Engine](/entities/mlc-engine.md) (SYSTEM)
- [Apple Silicon](/entities/apple-silicon.md) (SYSTEM)
- [Blackwell platforms](/entities/blackwell-platforms.md) (SYSTEM)
- [NVIDIA Grace Hopper](/entities/nvidia-grace-hopper.md) (SYSTEM)
- [Flash Attention](/entities/flash-attention.md) (CONCEPT)
- [llama-batched-bench](/entities/llama-batched-bench.md) (TOOL)
- [PCIe](/entities/pcie.md) (SYSTEM)
- [KV cache](/entities/kv-cache.md) (CONCEPT)
- [/completion Endpoint](/entities/completion-endpoint.md) (TOOL)
- [Prometheus](/entities/prometheus.md) (SYSTEM)
- [Project Gutenberg](/entities/project-gutenberg.md) (ORGANIZATION)
- [OpenAI-compatible routes](/entities/openai-compatible-routes.md) (CONCEPT)
- [llama.cpp Server](/entities/llama-cpp-server.md) (SYSTEM)
- [Server-Sent Events (SSE)](/entities/server-sent-events-sse.md) (CONCEPT)

## Relations
- Technical Characterization of Prompt Prefill Throughput and HTTP Benchmarking Methodologies in llama.cpp Servers → USES → llama.cpp Server
- Technical Characterization of Prompt Prefill Throughput and HTTP Benchmarking Methodologies in llama.cpp Servers → PART_OF → KV cache
- llama.cpp Server → RELATED_TO → MLC Engine
- llama.cpp Server → USES → Flash Attention
- JSON APIs → USES → llama.cpp Server
- /completion Endpoint → PART_OF → JSON APIs
- llama.cpp Server → RELATED_TO → OpenAI-compatible routes
- Server-Sent Events (SSE) → USES → llama.cpp Server
- Prometheus → USES → llama.cpp Server
- /metrics Endpoint → PART_OF → Prometheus
- llama-benchy → USES → Project Gutenberg
- llama-batched-bench → USES → llama.cpp Server
- Grafana k6 → USES → llama.cpp Server
