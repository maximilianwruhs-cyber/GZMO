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
- [[micro-batch-sizing|micro-batch sizing]] (CONCEPT)
- [[gpu-accelerated-sampling|GPU-accelerated sampling]] (CONCEPT)
- [[cpu-bound-sampling-bottleneck|CPU-bound sampling bottleneck]] (CONCEPT)
- [[gguf-weights|gguf weights]] (CONCEPT)
- [[llama-benchy-cli|llama-benchy CLI]] (TOOL)
- [[http-server-overhead|HTTP server overhead]] (CONCEPT)
- [[backend-sampling|backend sampling]] (CONCEPT)
- [[prefix-cache-immutability|prefix cache immutability]] (CONCEPT)
- [[pcie-bandwidth-bottlenecks|PCIe bandwidth bottlenecks]] (CONCEPT)
- [[real-time-progress-sse|Real-Time progress SSE]] (METHODOLOGY)
- [[prometheus-exporter|Prometheus Exporter]] (TOOL)
- [[llama-server|llama-server]] (SYSTEM)
- [[api-timing-extraction|API Timing Extraction]] (METHODOLOGY)
- [[k6-load-generator|k6 Load Generator]] (TOOL)
- [[host-cpu-scheduling-bottlenecks|host CPU scheduling bottlenecks]] (CONCEPT)
- [[llama-batched-bench|llama-batched-bench]] (TOOL)
- [[memory-thrashing|memory thrashing]] (CONCEPT)
- [[system-memory-fragmentation|system memory fragmentation]] (CONCEPT)

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
