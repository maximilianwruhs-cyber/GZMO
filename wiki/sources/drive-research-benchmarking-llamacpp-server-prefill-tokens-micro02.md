---
type: source
title: drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-benchmarking-llamacpp-server-prefill-tokens-micro02

Ingested source summary (2026-06-09).

## Entities
- [micro-batch sizing](/entities/micro-batch-sizing.md) (CONCEPT)
- [GPU-accelerated sampling](/entities/gpu-accelerated-sampling.md) (CONCEPT)
- [CPU-bound sampling bottleneck](/entities/cpu-bound-sampling-bottleneck.md) (CONCEPT)
- [gguf weights](/entities/gguf-weights.md) (CONCEPT)
- [llama-benchy CLI](/entities/llama-benchy-cli.md) (TOOL)
- [HTTP server overhead](/entities/http-server-overhead.md) (CONCEPT)
- [backend sampling](/entities/backend-sampling.md) (CONCEPT)
- [prefix cache immutability](/entities/prefix-cache-immutability.md) (CONCEPT)
- [PCIe bandwidth bottlenecks](/entities/pcie-bandwidth-bottlenecks.md) (CONCEPT)
- [Real-Time progress SSE](/entities/real-time-progress-sse.md) (METHODOLOGY)
- [Prometheus Exporter](/entities/prometheus-exporter.md) (TOOL)
- [llama-server](/entities/llama-server.md) (SYSTEM)
- [API Timing Extraction](/entities/api-timing-extraction.md) (METHODOLOGY)
- [k6 Load Generator](/entities/k6-load-generator.md) (TOOL)
- [host CPU scheduling bottlenecks](/entities/host-cpu-scheduling-bottlenecks.md) (CONCEPT)
- [llama-batched-bench](/entities/llama-batched-bench.md) (TOOL)
- [memory thrashing](/entities/memory-thrashing.md) (CONCEPT)
- [system memory fragmentation](/entities/system-memory-fragmentation.md) (CONCEPT)

## Relations
- llama-server → RELATED_TO → CPU-bound sampling bottleneck
- llama-server → USES → GPU-accelerated sampling
- llama-batched-bench → USES → gguf weights
- llama-batched-bench → RELATED_TO → HTTP server overhead
- llama-batched-bench → RELATED_TO → CPU-bound sampling bottleneck
- CPU-bound sampling bottleneck → RELATED_TO → GPU-accelerated sampling
- GPU-accelerated sampling → RELATED_TO → host CPU scheduling bottlenecks
- micro-batch sizing → RELATED_TO → PCIe bandwidth bottlenecks
- backend sampling → RELATED_TO → CPU-bound sampling bottleneck
- micro-batch sizing → RELATED_TO → memory thrashing
- micro-batch sizing → RELATED_TO → system memory fragmentation
