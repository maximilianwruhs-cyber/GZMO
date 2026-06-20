---
type: entity
title: llama-bench
created: 2026-06-08
updated: 2026-06-10
sources: 10
tags: []
status: draft
gzmo_synthetic: true
---










# llama-bench

Type: TOOL

## From [drive-research-llamabench](/entities/drive-research-llamabench.md) (2026-06-08)
- Native CLI performance benchmarking tool
- Bundled with llama.cpp
- Provides raw, hardware-level performance metrics for GGUF models

## From [drive-research-ok-so-designing-a-guide-around-llamabench-would-b](/entities/drive-research-ok-so-designing-a-guide-around-llamabench-would-b.md) (2026-06-08)
- Used to isolate hardware limits.
- Used to test compilation scaling.
- Used to map energy efficiency.
- Can be used to calculate how close a setup gets to theoretical hardware limits.
- Can be used to find the exact point where performance collapses.
- Can be used to measure how modern KV quantization styles protect throughput.
- Used as a regression and validation checker for compiling llama.cpp locally.

## From [drive-research-llama-bench-performance-benchmarking-tool-micro01](/entities/drive-research-llama-bench-performance-benchmarking-tool-micro01.md) (2026-06-09)
- Native command-line interface (CLI) performance benchmarking tool.
- Bundled directly with the core llama.cpp repository.
- Bypasses tokenization, sampling, and network dispatch pipeline.
- Executes GGML computational graph directly against target hardware backend APIs.
- Used for identifying hardware bottlenecks, tuning thread distribution, and evaluating quantization efficiency.
- Supports multi-parameter grid searches.
- Can output in Markdown, JSON, JSONL, CSV, and SQL formats.

## From [drive-research-llama-bench-performance-benchmarking-tool-micro02](/entities/drive-research-llama-bench-performance-benchmarking-tool-micro02.md) (2026-06-09)
- Profiles the C++ engine directly.
- Used to contrast C++ prefill and decode speeds.
- Has associated tools/scripts like `llama-bench/README.md`.

## From [optimizing-nvidia-blackwell-sm120-part3-micro06](/entities/optimizing-nvidia-blackwell-sm120-part3-micro06.md) (2026-06-09)
- It is a native CLI performance benchmarking tool bundled with llama.cpp.
- It provides raw, hardware-level performance metrics for GGUF models.
- It bypasses tokenization, sampling, or networking overhead.
- It is used for isolating hardware bottlenecks, dialing in optimal GPU offloading, tuning CPU thread counts, or profiling context-length degradation.
- It is used to profile context degradation.
- It is used to sweep thread counts.
- It is used to sweep VRAM layer offload.

## From [drive-research-research-process-steps-micro02](/entities/drive-research-research-process-steps-micro02.md) (2026-06-10)
- Modern replacement for 'llama-bench'.
- Core engine-level micro-benchmarking tool.
- Executes tensor operations directly inside the libllama C++ library.

## From [optimizing-nvidia-blackwell-sm120-part2-micro04](/entities/optimizing-nvidia-blackwell-sm120-part2-micro04.md) (2026-06-10)
- Native command-line interface (CLI) performance benchmarking tool.
- Bundled directly with the core llama.cpp repository.
- Bypasses tokenization, sampling, and network dispatch pipelines.

## From [optimizing-nvidia-blackwell-sm120-part2-micro05](/entities/optimizing-nvidia-blackwell-sm120-part2-micro05.md) (2026-06-10)
- Identifies bottlenecks in high-level software wrappers
- Profiles the C++ engine directly
- Used to measure C++ prefill and decode speeds

## From [optimizing-nvidia-blackwell-sm120-part3-micro01](/entities/optimizing-nvidia-blackwell-sm120-part3-micro01.md) (2026-06-10)
- Core engine-level micro-benchmarking tool.
- Executes tensor operations directly inside the libllama C++ library.
- Measures raw prompt processing (pp) and token generation (tg) speed.

## From [optimizing-nvidia-blackwell-sm120-part3-micro02](/entities/optimizing-nvidia-blackwell-sm120-part3-micro02.md) (2026-06-10)
- Used for benchmarking LLM backends
