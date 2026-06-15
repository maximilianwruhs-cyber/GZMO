---
type: entity
title: GGUF
created: 2026-06-08
updated: 2026-06-10
sources: 15
tags: []
status: draft
gzmo_synthetic: true
---

















# GGUF

Type: CONCEPT

## From [[drive-research-du-hast-gesagt-part1|drive-research-du-hast-gesagt-part1]] (2026-06-08)
- A model format that can be downloaded.
- Used by LM Studio.

## From [[drive-research-frankenmoe-blueprint-analysis|drive-research-frankenmoe-blueprint-analysis]] (2026-06-08)
- A quantization format used with llama.cpp.
- Extreme quantization formats can severely degrade gating precision.
- Various bit per weight (bpw) options are available.

## From [[drive-research-hidden-mode-technical-analysis-and-configurati|drive-research-hidden-mode-technical-analysis-and-configurati]] (2026-06-08)
- A quantization format offering various levels like Q8_0, Q6_K, Q5_K_M, Q4_K_M, Q3_K_M, Q2_K.
- Used in conjunction with llama.cpp for CPU-GPU hybrid execution.
- Strikes a balance between memory reduction and logic/reasoning accuracy at certain levels (e.g., Q4_K_M, Q5_K_M).

## From [[drive-research-hidden-mode-technical-analysis-and-configuration|drive-research-hidden-mode-technical-analysis-and-configuration]] (2026-06-08)
- Quantization format used for model deployment.
- Various quantization levels (e.g., Q8_0, Q6_K, Q4_K_M) are discussed.
- Extreme quantization can degrade routing performance.

## From [[architectures-for-agentic-memory-virtual-context-micro02|architectures-for-agentic-memory-virtual-context-micro02]] (2026-06-09)
- GPT-Generated Unified Format
- Used for highly optimized, quantized environments
- Models in the 1 billion to 10 billion parameter range predominantly use GGUF

## From [[architectures-for-agentic-memory-virtual-context-micro03|architectures-for-agentic-memory-virtual-context-micro03]] (2026-06-09)
- Quantized environments.
- Used for operating smaller parameter models (7B to 8B).
- Reliance on prompt engineering alone is inadequate for production systems.

## From [[drive-research-32gb-vram-ai-reasoning-models-micro01|drive-research-32gb-vram-ai-reasoning-models-micro01]] (2026-06-09)
- An executable file format for quantized models
- Used by llama.cpp backend, Ollama, and LM Studio
- Typically uses Q4_K_M quantization

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro01|drive-research-llama-bench-performance-benchmarking-tool-micro01]] (2026-06-09)
- Model file format whose execution efficiency depends on low-level backends and compiler optimizations.
- llama-bench can identify bottlenecks in GGUF model files.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro06|resilient-rust-based-mcp-client-and-llm-orchestrat-micro06]] (2026-06-09)
- Metadata for Gemma 4 lacked proper End-Of-Generation definitions.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro07|resilient-rust-based-mcp-client-and-llm-orchestrat-micro07]] (2026-06-09)
- Metadata format for models.
- Server's stability tied to quality of GGUF metadata.
- Models downloaded from repositories like HuggingFace are frequently quantized and uploaded in this format.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro01|the-architecture-of-speculative-decoding-and-infer-part1-micro01]] (2026-06-09)
- A file format for neural network model weights.
- Used to specify the path to the draft model in llama.cpp command-line interfaces.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro04|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro04]] (2026-06-09)
- Model file format used by LM Studio for local inference.
- Mergekit exports resulting models as GGUF files.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro01|optimizing-nvidia-blackwell-sm120-part3-micro01]] (2026-06-10)
- Model file format containing metadata headers.
- Can be inspected and manipulated by llama-gguf.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro05|resilient-rust-based-mcp-client-and-llm-orchestrat-micro05]] (2026-06-10)
- A model file format containing metadata like tokenizer.chat_template.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro03|the-architecture-of-speculative-decoding-and-infer-part1-micro03]] (2026-06-10)
- Format natively supported by llama.cpp.
